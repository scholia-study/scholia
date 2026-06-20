use std::collections::HashMap;
use std::fs;

use sqlx::PgPool;
use uuid::Uuid;

use poetry_md_to_struct::model::Output;
use reconcile::{node_hash, root_hash};

use crate::reconcile_input::node_content;

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
    force: bool,
    full_rewrite: bool,
) -> Result<(), Box<dyn std::error::Error>> {
    dotenvy::dotenv().ok();

    let data = fs::read_to_string(input_file)?;
    let output: Output = serde_json::from_str(&data)?;

    let pool = PgPool::connect_with(dataduct::db::pg_connect_options(database_url)?).await?;
    let mut tx = pool.begin().await?;

    let system_user_id = dataduct::seed::SYSTEM_USER_ID;

    // A book already in the DB is reconciled in place — matching the freshly
    // parsed struct against existing rows and carrying sentence UUIDs (and the
    // quotations/resources anchored to them) forward across edits. `--replace`
    // is the destructive escape hatch (cascading delete + fresh insert).
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
            // Reconcile-by-default: load the existing reference systems, then
            // update the book in place.
            eprintln!("Reconciling existing book {:?} ({})", output.book.slug, id);
            let mut system_ids: HashMap<String, Uuid> = HashMap::new();
            let rows: Vec<(Uuid, String)> =
                sqlx::query_as("SELECT id, slug FROM reference_systems WHERE book_id = $1")
                    .bind(id)
                    .fetch_all(&mut *tx)
                    .await?;
            for (sys_id, slug) in rows {
                system_ids.insert(slug, sys_id);
            }

            // The compilation source + author, needed when an added node carries
            // its own per-work `source` anchor (a Bible-shape sub-work). The
            // fresh-insert path below creates them; a reconcile only inherits the
            // source, so fetch it and re-upsert the author here. Existing nodes'
            // source links are never touched by the reconcile.
            let bib_source_id: Uuid =
                sqlx::query_scalar("SELECT source_id FROM books WHERE id = $1")
                    .bind(id)
                    .fetch_one(&mut *tx)
                    .await?;
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

            let (desired_node_hashes, desired_root) =
                crate::reconcile_input::compute_hashes(&output);
            let input =
                crate::reconcile_input::to_input(&output, bib_source_id, person_id, system_user_id);
            let report = reconcile::reconcile_book(
                &mut tx,
                id,
                &input,
                &desired_node_hashes,
                &desired_root,
                &system_ids,
                false,
                &HashMap::new(),
                &HashMap::new(),
                force,
                full_rewrite,
            )
            .await?;
            if dry_run {
                tx.rollback().await?;
            } else {
                tx.commit().await?;
                dataduct::cache::purge_blocking(&dataduct::cache::purge_paths(&output.book.slug))
                    .await;
            }
            report.print(dry_run);
            return Ok(());
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

    // NB: the author is linked to each WORK source (in the node loop), not to
    // this compilation source. An author-less compilation is what the library
    // classifies as a Bible-shape "self" group (pills + grand TOC); an authored
    // top-level source would group under the author instead (work cards, no pills).

    let book_id: Uuid = sqlx::query_scalar(
        "INSERT INTO books (slug, source_id, language, about_text)
         VALUES ($1, $2, $3, $4)
         RETURNING id",
    )
    .bind(&output.book.slug)
    .bind(bib_source_id)
    .bind(&output.book.language)
    .bind(&output.book.about_text)
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
    let mut source_count = 1u32; // the compilation source; sub-work sources add on
    // Node content hashes accumulate as we insert; the root is stored on `books`
    // after the loop, so the first *reconcile* is already in the fast state.
    let mut node_hashes: Vec<String> = Vec::new();

    for node in &output.toc_nodes {
        let content_hash = node_hash(&node_content(node));
        node_hashes.push(content_hash.clone());
        let parent_id: Option<Uuid> = node
            .parent_source_ref
            .as_ref()
            .and_then(|r| node_ids.get(r).copied());
        let label_html = (node.label_html != node.label).then_some(&node.label_html);

        // Source-anchored work node (e.g. "Sonnets"): create its sub-work source
        // (source_type 'chapter', parented to the book's compilation source) and
        // point the node's source_id at it — the Bible-shape work anchor.
        let node_source_id: Option<Uuid> = if let Some(src) = &node.source {
            source_count += 1;
            let sid: Uuid = sqlx::query_scalar(
                "INSERT INTO sources (source_type, title, publication_year, parent_source_id, protected, created_by)
                 VALUES ('chapter', $1, $2, $3, true, $4)
                 RETURNING id",
            )
            .bind(&src.title)
            .bind(src.publication_year)
            .bind(bib_source_id)
            .bind(system_user_id)
            .fetch_one(&mut *tx)
            .await?;
            // Author lives on the work source (keeps the compilation author-less).
            sqlx::query(
                "INSERT INTO source_persons (source_id, person_id, role, position)
                 VALUES ($1, $2, 'author', 0)
                 ON CONFLICT DO NOTHING",
            )
            .bind(sid)
            .bind(person_id)
            .execute(&mut *tx)
            .await?;
            Some(sid)
        } else {
            None
        };

        let node_id: Uuid = sqlx::query_scalar(
            "INSERT INTO toc_nodes (book_id, parent_id, source_id, source_ref, slug, path, sort_order, depth, label, label_html, content_hash)
             VALUES ($1, $2, $3, $4, $5, $6::ltree, $7, $8, $9, $10, $11)
             RETURNING id",
        )
        .bind(book_id)
        .bind(parent_id)
        .bind(node_source_id)
        .bind(&node.source_ref)
        .bind(&node.slug)
        .bind(&node.path)
        .bind(node.sort_order)
        .bind(node.depth)
        .bind(&node.label)
        .bind(label_html)
        .bind(&content_hash)
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
                    reconcile::natural_key(&node.source_ref, block.position, sent.position);

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

    // Store the book root hash so the first reconcile can short-circuit.
    sqlx::query("UPDATE books SET content_hash = $2 WHERE id = $1")
        .bind(book_id)
        .bind(root_hash(&node_hashes))
        .execute(&mut *tx)
        .await?;

    if dry_run {
        tx.rollback().await?;
        eprintln!("(dry-run: nothing committed)");
    } else {
        tx.commit().await?;

        // Drop stale listing entries so the new book shows up immediately
        // rather than waiting out the TTL. No-op if CACHE_PURGE_URL is
        // unset (local dev without a proxy).
        dataduct::cache::purge_blocking(&dataduct::cache::purge_paths(&output.book.slug)).await;
    }

    eprintln!();
    eprintln!("=== Import complete ===");
    eprintln!("  book:           1");
    eprintln!("  sources:        {source_count}");
    eprintln!("  ref_systems:    {}", system_ids.len());
    eprintln!("  toc_nodes:      {node_count}");
    eprintln!("  content_blocks: {block_count}");
    eprintln!("  sentences:      {sentence_count}");
    eprintln!("  page_markers:   {marker_count}");
    Ok(())
}
