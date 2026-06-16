use std::collections::HashMap;
use std::fs;

use sqlx::PgPool;
use sqlx::postgres::PgConnectOptions;
use uuid::Uuid;

use shakespeare1_md_to_struct::model::Output;

const ABOUT_TEXT: &str = "Shakespeare's Sonnets, first printed in the 1609 Quarto. \
The modern-spelling reading text is drawn from public-domain sources; the \
original-spelling layer reproduces the 1609 Quarto via EEBO-TCP (released CC0). \
The digital edition on Scholia is a community-driven project; corrections are welcome.";

/// Build sqlx connect options. CLI `--database-url` wins; otherwise prefer
/// discrete `POSTGRES_*` env vars (k8s Secret pattern); fall back to
/// `DATABASE_URL` for laptop `.env`. Mirrors `kant1_struct_to_db`.
fn pg_connect_options(
    cli_url: Option<String>,
) -> Result<PgConnectOptions, Box<dyn std::error::Error>> {
    if let Some(url) = cli_url {
        return Ok(url.parse()?);
    }
    if let Ok(user) = std::env::var("POSTGRES_USER") {
        let password = std::env::var("POSTGRES_PASSWORD")
            .map_err(|_| "POSTGRES_PASSWORD must be set when POSTGRES_USER is set")?;
        let database = std::env::var("POSTGRES_DB")
            .map_err(|_| "POSTGRES_DB must be set when POSTGRES_USER is set")?;
        let host = std::env::var("POSTGRES_HOST").unwrap_or_else(|_| "localhost".to_string());
        let port: u16 = std::env::var("POSTGRES_PORT")
            .ok()
            .and_then(|s| s.parse().ok())
            .unwrap_or(5432);
        return Ok(PgConnectOptions::new()
            .username(&user)
            .password(&password)
            .database(&database)
            .host(&host)
            .port(port));
    }
    let url = std::env::var("DATABASE_URL").map_err(
        |_| "Set POSTGRES_USER + POSTGRES_PASSWORD + POSTGRES_DB (preferred) or DATABASE_URL",
    )?;
    Ok(url.parse()?)
}

/// Auto-generate sort_name: "William Shakespeare" -> "Shakespeare, William".
fn sort_name(name: &str) -> Option<String> {
    let parts: Vec<&str> = name.split_whitespace().collect();
    if parts.len() >= 2 {
        Some(format!(
            "{}, {}",
            parts.last().unwrap(),
            parts[..parts.len() - 1].join(" ")
        ))
    } else {
        None
    }
}

pub async fn run(
    input_file: &str,
    database_url: Option<String>,
    replace: bool,
    dry_run: bool,
) -> Result<(), Box<dyn std::error::Error>> {
    dotenvy::dotenv().ok();

    let data = fs::read_to_string(input_file)?;
    let output: Output = serde_json::from_str(&data)?;

    let pool = PgPool::connect_with(pg_connect_options(database_url)?).await?;
    let mut tx = pool.begin().await?;

    // System user owns all seed-imported persons/sources.
    let system_user_id: Uuid =
        Uuid::parse_str("00000000-0000-0000-0000-000000000001").expect("valid system user UUID");

    // This importer is fresh-insert only (no reconcile). A pre-existing book is
    // either replaced (cascading delete) or a hard error.
    let existing: Option<Uuid> = sqlx::query_scalar("SELECT id FROM books WHERE slug = $1")
        .bind(&output.book.slug)
        .fetch_optional(&mut *tx)
        .await?;
    if let Some(id) = existing {
        if replace {
            sqlx::query("DELETE FROM books WHERE id = $1")
                .bind(id)
                .execute(&mut *tx)
                .await?;
            eprintln!("Replaced existing book {:?} ({})", output.book.slug, id);
        } else {
            return Err(format!(
                "book {:?} already exists ({id}); pass --replace to overwrite",
                output.book.slug
            )
            .into());
        }
    }

    // 1. Author, bibliographic source, book.
    let person_id: Uuid = sqlx::query_scalar(
        "INSERT INTO persons (name, sort_name, protected, created_by)
         VALUES ($1, $2, true, $3)
         ON CONFLICT (name) DO UPDATE SET name = EXCLUDED.name
         RETURNING id",
    )
    .bind(&output.book.author)
    .bind(sort_name(&output.book.author))
    .bind(system_user_id)
    .fetch_one(&mut *tx)
    .await?;

    let publication_year: Option<i16> = output.book.source_date.parse::<i16>().ok();
    let bib_source_id: Uuid = sqlx::query_scalar(
        "INSERT INTO sources (source_type, title, publication_year, protected, created_by)
         VALUES ('book', $1, $2, true, $3)
         RETURNING id",
    )
    .bind(&output.book.title)
    .bind(publication_year)
    .bind(system_user_id)
    .fetch_one(&mut *tx)
    .await?;

    sqlx::query(
        "INSERT INTO source_persons (source_id, person_id, role, position)
         VALUES ($1, $2, 'author', 0)
         ON CONFLICT DO NOTHING",
    )
    .bind(bib_source_id)
    .bind(person_id)
    .execute(&mut *tx)
    .await?;

    let book_id: Uuid = sqlx::query_scalar(
        "INSERT INTO books (slug, source_id, language, about_text)
         VALUES ($1, $2, $3, $4)
         RETURNING id",
    )
    .bind(&output.book.slug)
    .bind(bib_source_id)
    .bind(&output.book.language)
    .bind(ABOUT_TEXT)
    .fetch_one(&mut *tx)
    .await?;
    eprintln!("Inserted book {:?} ({})", output.book.slug, book_id);

    // 2. Reference systems.
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

    // 3. Nodes -> blocks -> sentences -> page markers.
    let mut node_ids: HashMap<String, Uuid> = HashMap::new();
    let (mut node_count, mut block_count, mut sentence_count, mut marker_count) =
        (0u32, 0u32, 0u32, 0u32);

    for node in &output.toc_nodes {
        let parent_id: Option<Uuid> = node
            .parent_source_ref
            .as_ref()
            .and_then(|r| node_ids.get(r).copied());
        let label_html = (node.label_html != node.label).then_some(&node.label_html);

        let node_id: Uuid = sqlx::query_scalar(
            "INSERT INTO toc_nodes (book_id, parent_id, source_ref, slug, path, sort_order, depth, label, label_html)
             VALUES ($1, $2, $3, $4, $5::ltree, $6, $7, $8, $9)
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
        .bind(label_html)
        .fetch_one(&mut *tx)
        .await?;
        node_ids.insert(node.source_ref.clone(), node_id);
        node_count += 1;

        for block in &node.content_blocks {
            let block_id: Uuid = sqlx::query_scalar(
                "INSERT INTO content_blocks (book_id, node_id, position, block_type, paragraph_number, figure_number, text, html, original_text, original_html)
                 VALUES ($1, $2, $3, $4::block_type, $5, $6, $7, $8, $9, $10)
                 RETURNING id",
            )
            .bind(book_id)
            .bind(node_id)
            .bind(block.position)
            .bind(&block.block_type)
            .bind(block.paragraph_number)
            .bind(block.figure_number)
            .bind(&block.text)
            .bind(&block.html)
            .bind(&block.original_text)
            .bind(&block.original_html)
            .fetch_one(&mut *tx)
            .await?;
            block_count += 1;

            for sent in &block.sentences {
                let natural_key =
                    format!("{}/b{}/s{}", node.source_ref, block.position, sent.position);

                let sentence_id: Uuid = sqlx::query_scalar(
                    "INSERT INTO sentences (book_id, node_id, block_id, position, sentence_number, segment, indent, text, html, original_text, original_html, natural_key)
                     VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9, $10, $11, $12)
                     RETURNING id",
                )
                .bind(book_id)
                .bind(node_id)
                .bind(block_id)
                .bind(sent.position)
                .bind(sent.sentence_number)
                .bind(sent.segment)
                .bind(sent.indent)
                .bind(&sent.text)
                .bind(&sent.html)
                .bind(&sent.original_text)
                .bind(&sent.original_html)
                .bind(&natural_key)
                .fetch_one(&mut *tx)
                .await?;
                sentence_count += 1;

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
            }
        }
    }

    if dry_run {
        tx.rollback().await?;
        eprintln!("(dry-run: nothing committed)");
    } else {
        tx.commit().await?;
    }

    eprintln!();
    eprintln!("=== Import complete ===");
    eprintln!("  book:           1");
    eprintln!("  ref_systems:    {}", system_ids.len());
    eprintln!("  toc_nodes:      {node_count}");
    eprintln!("  content_blocks: {block_count}");
    eprintln!("  sentences:      {sentence_count}");
    eprintln!("  page_markers:   {marker_count}");
    Ok(())
}
