use std::collections::HashMap;
use std::fs;

use sqlx::PgPool;
use uuid::Uuid;

use kant1_md_to_struct::model::Output;

pub async fn run(
    input_file: &str,
    database_url: Option<String>,
) -> Result<(), Box<dyn std::error::Error>> {
    dotenvy::dotenv().ok();

    let db_url = database_url
        .or_else(|| std::env::var("DATABASE_URL").ok())
        .ok_or("No database URL: pass --database-url or set DATABASE_URL")?;

    let data = fs::read_to_string(input_file)?;
    let output: Output = serde_json::from_str(&data)?;

    let pool = PgPool::connect(&db_url).await?;
    let mut tx = pool.begin().await?;

    // 1. Insert book
    let book_id: Uuid = sqlx::query_scalar(
        "INSERT INTO books (slug, title, author, language, source, source_date)
         VALUES ($1, $2, $3, $4, $5, $6)
         RETURNING id",
    )
    .bind(&output.book.slug)
    .bind(&output.book.title)
    .bind(&output.book.author)
    .bind(&output.book.language)
    .bind(&output.book.source)
    .bind(&output.book.source_date)
    .fetch_one(&mut *tx)
    .await?;

    eprintln!("Inserted book {:?} ({})", output.book.slug, book_id);

    // 2. Insert reference systems
    let mut system_ids: HashMap<String, Uuid> = HashMap::new();
    for sys in &output.reference_systems {
        let sys_id: Uuid = sqlx::query_scalar(
            "INSERT INTO reference_systems (book_id, slug, label, ref_type)
             VALUES ($1, $2, $3, $4)
             RETURNING id",
        )
        .bind(book_id)
        .bind(&sys.slug)
        .bind(&sys.label)
        .bind(&sys.ref_type)
        .fetch_one(&mut *tx)
        .await?;

        system_ids.insert(sys.slug.clone(), sys_id);
    }

    eprintln!("Inserted {} reference systems", system_ids.len());

    // 3. Insert toc_nodes — need to resolve parent_source_ref → parent_id
    let mut node_ids: HashMap<String, Uuid> = HashMap::new();
    let mut node_count = 0u32;
    let mut block_count = 0u32;
    let mut sentence_count = 0u32;
    let mut marker_count = 0u32;
    let mut footnote_count = 0u32;
    let mut footnote_sentence_count = 0u32;

    for node in &output.toc_nodes {
        let parent_id: Option<Uuid> = node
            .parent_source_ref
            .as_ref()
            .and_then(|ref_str| node_ids.get(ref_str).copied());

        let node_id: Uuid = sqlx::query_scalar(
            "INSERT INTO toc_nodes (book_id, parent_id, source_ref, slug, path, sort_order, depth, label)
             VALUES ($1, $2, $3, $4, $5::ltree, $6, $7, $8)
             RETURNING id",
        )
        .bind(book_id)
        .bind(parent_id)
        .bind(&node.source_ref)
        .bind(&node.slug)
        .bind(&node.path)
        .bind(node.sort_order)
        .bind(node.depth)
        .bind(&node.label)
        .fetch_one(&mut *tx)
        .await?;

        node_ids.insert(node.source_ref.clone(), node_id);
        node_count += 1;

        // 4. Insert content blocks, sentences, page markers, footnotes
        for block in &node.content_blocks {
            let block_id: Uuid = sqlx::query_scalar(
                "INSERT INTO content_blocks (book_id, node_id, position, block_type, paragraph_number, text, html, original_text, original_html)
                 VALUES ($1, $2, $3, $4::block_type, $5, $6, $7, $8, $9)
                 RETURNING id",
            )
            .bind(book_id)
            .bind(node_id)
            .bind(block.position)
            .bind(&block.block_type)
            .bind(block.paragraph_number)
            .bind(&block.text)
            .bind(&block.html)
            .bind(&block.original_text)
            .bind(&block.original_html)
            .fetch_one(&mut *tx)
            .await?;

            block_count += 1;

            for sent in &block.sentences {
                // Insert anchor sentence (block_id set, footnote_id NULL)
                let sentence_id: Uuid = sqlx::query_scalar(
                    "INSERT INTO sentences (book_id, node_id, block_id, position, sentence_number, text, html, original_text, original_html)
                     VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9)
                     RETURNING id",
                )
                .bind(book_id)
                .bind(node_id)
                .bind(block_id)
                .bind(sent.position)
                .bind(sent.sentence_number)
                .bind(&sent.text)
                .bind(&sent.html)
                .bind(&sent.original_text)
                .bind(&sent.original_html)
                .fetch_one(&mut *tx)
                .await?;

                sentence_count += 1;

                // Insert page markers
                for pm in &sent.page_markers {
                    let system_id = system_ids
                        .get(&pm.system)
                        .ok_or_else(|| format!("Unknown reference system: {}", pm.system))?;

                    sqlx::query(
                        "INSERT INTO page_markers (system_id, sentence_id, ref_value, sort_order, char_offset)
                         VALUES ($1, $2, $3, $4, $5)",
                    )
                    .bind(system_id)
                    .bind(sentence_id)
                    .bind(&pm.ref_value)
                    .bind(pm.sort_order)
                    .bind(pm.char_offset)
                    .execute(&mut *tx)
                    .await?;

                    marker_count += 1;
                }

                // Insert footnotes attached to this sentence
                for footnote in &sent.footnotes {
                    let footnote_id: Uuid = sqlx::query_scalar(
                        "INSERT INTO footnotes (book_id, number, anchor_sentence_id)
                         VALUES ($1, $2, $3)
                         RETURNING id",
                    )
                    .bind(book_id)
                    .bind(footnote.number)
                    .bind(sentence_id)
                    .fetch_one(&mut *tx)
                    .await?;

                    footnote_count += 1;

                    // Insert footnote sentences (block_id NULL, footnote_id set)
                    for fn_sent in &footnote.sentences {
                        sqlx::query(
                            "INSERT INTO sentences (book_id, node_id, footnote_id, position, text, html, original_text, original_html)
                             VALUES ($1, $2, $3, $4, $5, $6, $7, $8)",
                        )
                        .bind(book_id)
                        .bind(node_id)
                        .bind(footnote_id)
                        .bind(fn_sent.position)
                        .bind(&fn_sent.text)
                        .bind(&fn_sent.html)
                        .bind(&fn_sent.original_text)
                        .bind(&fn_sent.original_html)
                        .execute(&mut *tx)
                        .await?;

                        footnote_sentence_count += 1;
                        sentence_count += 1;
                    }
                }
            }
        }
    }

    tx.commit().await?;

    eprintln!();
    eprintln!("=== Import complete ===");
    eprintln!("  book:              1");
    eprintln!("  ref_systems:       {}", system_ids.len());
    eprintln!("  toc_nodes:         {}", node_count);
    eprintln!("  content_blocks:    {}", block_count);
    eprintln!("  sentences:         {}", sentence_count);
    eprintln!("  footnotes:         {}", footnote_count);
    eprintln!("  footnote_sentences:{}", footnote_sentence_count);
    eprintln!("  page_markers:      {}", marker_count);

    Ok(())
}
