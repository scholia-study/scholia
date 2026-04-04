use std::collections::HashMap;
use std::fs;

use sqlx::PgPool;
use uuid::Uuid;

use kant1_md_to_struct::model::Output;

pub async fn run(
    input_file: &str,
    database_url: Option<String>,
    source_book_slug: Option<String>,
) -> Result<(), Box<dyn std::error::Error>> {
    dotenvy::dotenv().ok();

    let db_url = database_url
        .or_else(|| std::env::var("DATABASE_URL").ok())
        .ok_or("No database URL: pass --database-url or set DATABASE_URL")?;

    let data = fs::read_to_string(input_file)?;
    let output: Output = serde_json::from_str(&data)?;

    let pool = PgPool::connect(&db_url).await?;
    let mut tx = pool.begin().await?;

    let is_translation = source_book_slug.is_some();

    // 0. Look up source book (translation mode only)
    let source_book_id: Option<Uuid> = if let Some(ref slug) = source_book_slug {
        let id: Option<Uuid> =
            sqlx::query_scalar("SELECT id FROM books WHERE slug = $1")
                .bind(slug)
                .fetch_optional(&mut *tx)
                .await?;
        let id = id.ok_or_else(|| format!("Source book not found: {slug}"))?;
        eprintln!("Source book {:?} ({})", slug, id);
        Some(id)
    } else {
        None
    };

    // 1. Insert book
    let book_id: Uuid = sqlx::query_scalar(
        "INSERT INTO books (slug, title, author, language, source, source_date, source_book_id)
         VALUES ($1, $2, $3, $4, $5, $6, $7)
         RETURNING id",
    )
    .bind(&output.book.slug)
    .bind(&output.book.title)
    .bind(&output.book.author)
    .bind(&output.book.language)
    .bind(&output.book.source)
    .bind(&output.book.source_date)
    .bind(source_book_id)
    .fetch_one(&mut *tx)
    .await?;

    eprintln!("Inserted book {:?} ({})", output.book.slug, book_id);

    // 2. Reference systems — reuse source book's systems in translation mode, otherwise insert new
    let mut system_ids: HashMap<String, Uuid> = HashMap::new();

    if let Some(source_id) = source_book_id {
        let rows: Vec<(Uuid, String)> = sqlx::query_as(
            "SELECT id, slug FROM reference_systems WHERE book_id = $1",
        )
        .bind(source_id)
        .fetch_all(&mut *tx)
        .await?;

        for (id, slug) in rows {
            system_ids.insert(slug, id);
        }
        eprintln!("Reusing {} reference systems from source book", system_ids.len());
    } else {
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
    }

    // 3. Pre-load source lookups (translation mode only)
    // source_ref → source toc_node id
    let mut source_node_map: HashMap<String, Uuid> = HashMap::new();
    // (source_ref, block_position, sentence_position) → source sentence id
    let mut source_sentence_map: HashMap<(String, i16, i16), Uuid> = HashMap::new();
    // (footnote_number, sentence_position) → source footnote sentence id
    let mut source_fn_sentence_map: HashMap<(i32, i16), Uuid> = HashMap::new();
    // footnote_number → source footnote sentence count
    let mut source_fn_sentence_counts: HashMap<i32, i16> = HashMap::new();

    if let Some(source_id) = source_book_id {
        let node_rows: Vec<(Uuid, String)> = sqlx::query_as(
            "SELECT id, source_ref FROM toc_nodes WHERE book_id = $1",
        )
        .bind(source_id)
        .fetch_all(&mut *tx)
        .await?;

        for (id, source_ref) in node_rows {
            source_node_map.insert(source_ref, id);
        }

        let sent_rows: Vec<(Uuid, String, i16, i16)> = sqlx::query_as(
            "SELECT s.id, tn.source_ref, cb.position, s.position
             FROM sentences s
             JOIN content_blocks cb ON s.block_id = cb.id
             JOIN toc_nodes tn ON cb.node_id = tn.id
             WHERE s.book_id = $1 AND s.block_id IS NOT NULL",
        )
        .bind(source_id)
        .fetch_all(&mut *tx)
        .await?;

        for (id, source_ref, block_pos, sent_pos) in sent_rows {
            source_sentence_map.insert((source_ref, block_pos, sent_pos), id);
        }

        // Load source footnote sentences keyed by (footnote_number, position)
        let fn_sent_rows: Vec<(Uuid, i32, i16)> = sqlx::query_as(
            "SELECT s.id, f.number, s.position
             FROM sentences s
             JOIN footnotes f ON s.footnote_id = f.id
             WHERE s.book_id = $1 AND s.footnote_id IS NOT NULL",
        )
        .bind(source_id)
        .fetch_all(&mut *tx)
        .await?;

        for (id, fn_number, sent_pos) in &fn_sent_rows {
            source_fn_sentence_map.insert((*fn_number, *sent_pos), *id);
            let count = source_fn_sentence_counts.entry(*fn_number).or_insert(0);
            *count = (*count).max(*sent_pos + 1);
        }

        eprintln!(
            "Loaded {} source nodes, {} source block sentences, {} source footnote sentences",
            source_node_map.len(),
            source_sentence_map.len(),
            fn_sent_rows.len()
        );
    }

    // 4. Insert toc_nodes, content blocks, sentences, page markers, footnotes
    let mut node_ids: HashMap<String, Uuid> = HashMap::new();
    let mut node_count = 0u32;
    let mut block_count = 0u32;
    let mut sentence_count = 0u32;
    let mut marker_count = 0u32;
    let mut footnote_count = 0u32;
    let mut footnote_sentence_count = 0u32;
    let mut footnote_sentence_number = 1i32;

    for node in &output.toc_nodes {
        let parent_id: Option<Uuid> = node
            .parent_source_ref
            .as_ref()
            .and_then(|ref_str| node_ids.get(ref_str).copied());

        let source_node_id: Option<Uuid> = if is_translation {
            source_node_map.get(&node.source_ref).copied()
        } else {
            None
        };

        let node_id: Uuid = sqlx::query_scalar(
            "INSERT INTO toc_nodes (book_id, parent_id, source_node_id, source_ref, slug, path, sort_order, depth, label)
             VALUES ($1, $2, $3, $4, $5, $6::ltree, $7, $8, $9)
             RETURNING id",
        )
        .bind(book_id)
        .bind(parent_id)
        .bind(source_node_id)
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
                // Resolve source sentence linkage (translation mode, block sentences only)
                let source_sentence_start_id: Option<Uuid> = if is_translation {
                    source_sentence_map
                        .get(&(node.source_ref.clone(), block.position, sent.position))
                        .copied()
                } else {
                    None
                };

                let sentence_id: Uuid = sqlx::query_scalar(
                    "INSERT INTO sentences (book_id, node_id, block_id, position, sentence_number, source_sentence_start_id, text, html, original_text, original_html)
                     VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9, $10)
                     RETURNING id",
                )
                .bind(book_id)
                .bind(node_id)
                .bind(block_id)
                .bind(sent.position)
                .bind(sent.sentence_number)
                .bind(source_sentence_start_id)
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

                    // Verify footnote sentence parity in translation mode
                    if is_translation {
                        let en_count = footnote.sentences.len() as i16;
                        let de_count = source_fn_sentence_counts
                            .get(&footnote.number)
                            .copied()
                            .ok_or_else(|| {
                                format!("Source footnote #{} not found", footnote.number)
                            })?;
                        if en_count != de_count {
                            return Err(format!(
                                "Footnote #{} sentence count mismatch: source has {}, translation has {}",
                                footnote.number, de_count, en_count
                            )
                            .into());
                        }
                    }

                    // Insert footnote sentences (block_id NULL, footnote_id set)
                    for fn_sent in &footnote.sentences {
                        let source_fn_sentence_id: Option<Uuid> = if is_translation {
                            Some(
                                *source_fn_sentence_map
                                    .get(&(footnote.number, fn_sent.position))
                                    .ok_or_else(|| {
                                        format!(
                                            "Source footnote #{} sentence {} not found",
                                            footnote.number, fn_sent.position
                                        )
                                    })?,
                            )
                        } else {
                            None
                        };

                        let fn_sent_num = footnote_sentence_number;
                        footnote_sentence_number += 1;

                        sqlx::query(
                            "INSERT INTO sentences (book_id, node_id, footnote_id, position, sentence_number, source_sentence_start_id, text, html, original_text, original_html)
                             VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9, $10)",
                        )
                        .bind(book_id)
                        .bind(node_id)
                        .bind(footnote_id)
                        .bind(fn_sent.position)
                        .bind(fn_sent_num)
                        .bind(source_fn_sentence_id)
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
    if is_translation {
        eprintln!("  mode:              translation");
    }
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
