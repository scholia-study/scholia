use std::collections::{HashMap, HashSet};
use std::fs;

use sqlx::PgPool;
use uuid::Uuid;

use reconcile::{node_hash, root_hash};
use text_struct::model::Output;

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

/// A translation edition is locked 1:1 to its source book: every block (and
/// every footnote) must carry the same number of sentences as the source, or
/// quotation projection + side-by-side alignment break (the extra/missing
/// translation sentences end up with no `source_sentence_start_id`). The two
/// editions are built independently, so a split/merge made in only one would
/// otherwise pass silently. Check up front — before either the fresh-insert or
/// reconcile path — listing every offender. (Block *type* may legitimately
/// differ between editions; only the sentence count per (node, block position)
/// must match.)
fn validate_translation_parity(
    output: &Output,
    source_sentence_map: &HashMap<(String, i16, i16), Uuid>,
    source_fn_sentence_counts: &HashMap<i32, i16>,
) -> Result<(), Box<dyn std::error::Error>> {
    let mut src_block_counts: HashMap<(String, i16), usize> = HashMap::new();
    for (source_ref, block_pos, _sent_pos) in source_sentence_map.keys() {
        *src_block_counts
            .entry((source_ref.clone(), *block_pos))
            .or_insert(0) += 1;
    }

    let mut problems: Vec<String> = Vec::new();
    for node in &output.toc_nodes {
        for block in &node.content_blocks {
            let translated = block.sentences.len();
            let source = src_block_counts
                .get(&(node.source_ref.clone(), block.position))
                .copied()
                .unwrap_or(0);
            if translated != source {
                problems.push(format!(
                    "  node {} / block {}: translation has {translated} sentence(s), source has {source}",
                    node.source_ref, block.position
                ));
            }
            for footnote in block.sentences.iter().flat_map(|s| &s.footnotes) {
                let translated = footnote.sentences.len();
                match source_fn_sentence_counts.get(&footnote.number) {
                    Some(source) if translated != *source as usize => problems.push(format!(
                        "  footnote {}: translation has {translated} sentence(s), source has {source}",
                        footnote.number
                    )),
                    None => problems.push(format!(
                        "  footnote {}: present in translation but missing from source",
                        footnote.number
                    )),
                    _ => {}
                }
            }
        }
    }
    let translated_keys: HashSet<(String, i16)> = output
        .toc_nodes
        .iter()
        .flat_map(|n| {
            n.content_blocks
                .iter()
                .map(move |b| (n.source_ref.clone(), b.position))
        })
        .collect();
    for (source_ref, block_pos) in src_block_counts.keys() {
        if !translated_keys.contains(&(source_ref.clone(), *block_pos)) {
            problems.push(format!(
                "  node {source_ref} / block {block_pos}: present in source but missing from translation"
            ));
        }
    }

    if !problems.is_empty() {
        return Err(format!(
            "translation is out of sync with the source edition ({} mismatch(es)) — fix the \
             markdown so sentence/footnote splits match, then re-run (reconcile the source book \
             first if you edited it too):\n{}",
            problems.len(),
            problems.join("\n")
        )
        .into());
    }
    Ok(())
}

#[allow(clippy::too_many_arguments)]
pub async fn run(
    input_file: &str,
    database_url: Option<String>,
    source_book_slug: Option<String>,
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
    let is_translation = source_book_slug.is_some();

    // Translation mode: resolve the source book and build the natural-key →
    // source node / sentence maps that lock this edition 1:1 to it. These drive
    // `source_node_id` / `source_sentence_start_id` in both the fresh-insert and
    // reconcile paths.
    let mut source_book_id: Option<Uuid> = None;
    let mut translation_of_id: Option<Uuid> = None;
    let mut source_node_map: HashMap<String, Uuid> = HashMap::new();
    let mut source_sentence_map: HashMap<(String, i16, i16), Uuid> = HashMap::new();
    // (footnote_number, sentence_position) → source footnote sentence id, and
    // footnote_number → source footnote sentence count (parity check input).
    let mut source_fn_sentence_map: HashMap<(i32, i16), Uuid> = HashMap::new();
    let mut source_fn_sentence_counts: HashMap<i32, i16> = HashMap::new();
    if let Some(ref slug) = source_book_slug {
        let (sbid, src_id): (Uuid, Uuid) =
            sqlx::query_as("SELECT id, source_id FROM books WHERE slug = $1")
                .bind(slug)
                .fetch_optional(&mut *tx)
                .await?
                .ok_or_else(|| format!("Source book not found: {slug}"))?;
        source_book_id = Some(sbid);
        translation_of_id = Some(src_id);

        let node_rows: Vec<(Uuid, String)> =
            sqlx::query_as("SELECT id, source_ref FROM toc_nodes WHERE book_id = $1")
                .bind(sbid)
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
        .bind(sbid)
        .fetch_all(&mut *tx)
        .await?;
        for (id, source_ref, block_pos, sent_pos) in sent_rows {
            source_sentence_map.insert((source_ref, block_pos, sent_pos), id);
        }

        let fn_sent_rows: Vec<(Uuid, i32, i16)> = sqlx::query_as(
            "SELECT s.id, f.number, s.position
             FROM sentences s
             JOIN footnotes f ON s.footnote_id = f.id
             WHERE s.book_id = $1 AND s.footnote_id IS NOT NULL",
        )
        .bind(sbid)
        .fetch_all(&mut *tx)
        .await?;
        for (id, fn_number, sent_pos) in &fn_sent_rows {
            source_fn_sentence_map.insert((*fn_number, *sent_pos), *id);
            let count = source_fn_sentence_counts.entry(*fn_number).or_insert(0);
            *count = (*count).max(*sent_pos + 1);
        }

        eprintln!(
            "Source book {:?} ({}): {} nodes, {} sentences, {} footnote sentences",
            slug,
            sbid,
            source_node_map.len(),
            source_sentence_map.len(),
            fn_sent_rows.len()
        );
        validate_translation_parity(&output, &source_sentence_map, &source_fn_sentence_counts)?;
    }

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
            eprintln!(
                "Replacing book {:?} ({}) — deleting old rows…",
                output.book.slug, id
            );
            // Drop the book AND its bibliographic source — deleting the book
            // leaves the source row orphaned, and the fresh insert below would
            // then collide on the sources (title, source_type, publication_year)
            // unique key. (A translation edition's source carries
            // translation_of_id, ON DELETE SET NULL, so dropping the source
            // book's source just nulls a stale link the re-import resets.)
            let old_source_id: Option<Uuid> =
                sqlx::query_scalar("SELECT source_id FROM books WHERE id = $1")
                    .bind(id)
                    .fetch_optional(&mut *tx)
                    .await?;
            sqlx::query("DELETE FROM books WHERE id = $1")
                .bind(id)
                .execute(&mut *tx)
                .await?;
            if let Some(sid) = old_source_id {
                sqlx::query("DELETE FROM sources WHERE id = $1")
                    .bind(sid)
                    .execute(&mut *tx)
                    .await?;
            }
            eprintln!("Replaced existing book {:?} ({})", output.book.slug, id);
        } else {
            // Reconcile-by-default: load the reference systems (the source
            // book's, for a translation edition — it owns none of its own), then
            // update the book in place.
            eprintln!("Reconciling existing book {:?} ({})", output.book.slug, id);
            let systems_book_id = source_book_id.unwrap_or(id);
            let mut system_ids: HashMap<String, Uuid> = HashMap::new();
            let rows: Vec<(Uuid, String)> =
                sqlx::query_as("SELECT id, slug FROM reference_systems WHERE book_id = $1")
                    .bind(systems_book_id)
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
            let input = crate::reconcile_input::to_input(
                &output,
                bib_source_id,
                person_id,
                system_user_id,
                &source_node_map,
            );
            let report = reconcile::reconcile_book(
                &mut tx,
                id,
                &input,
                &desired_node_hashes,
                &desired_root,
                &system_ids,
                is_translation,
                &source_sentence_map,
                &source_fn_sentence_map,
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
    // A translation edition's source carries `translation_of_id` → the source
    // book's source, the link the side-by-side companion view resolves.
    let bib_source_id: Uuid = sqlx::query_scalar(
        "INSERT INTO sources (source_type, title, publication_year, publisher, translation_of_id, protected, created_by)
         VALUES ('book', $1, $2, $3, $4, true, $5)
         RETURNING id",
    )
    .bind(&output.book.title)
    .bind(publication_year)
    .bind(&output.book.publisher)
    .bind(translation_of_id)
    .bind(system_user_id)
    .fetch_one(&mut *tx)
    .await?;

    // Standalone authored work: link the author to the book's own source so the
    // library groups it under the author (work cards), like Kant — not a
    // Bible-shape "self" group. (The per-node WorkSource branch below is generic
    // infra and stays inert here, since no node carries its own source.)
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
        "INSERT INTO books (slug, source_id, language, about_text, nodes_per_page)
         VALUES ($1, $2, $3, $4, $5)
         RETURNING id",
    )
    .bind(&output.book.slug)
    .bind(bib_source_id)
    .bind(&output.book.language)
    .bind(&output.book.about_text)
    .bind(output.book.nodes_per_page)
    .fetch_one(&mut *tx)
    .await?;
    eprintln!("Inserted book {:?} ({})", output.book.slug, book_id);

    // 2. Reference systems — a translation edition reuses the source book's
    // systems (page markers map by slug); a standalone book inserts its own.
    let mut system_ids: HashMap<String, Uuid> = HashMap::new();
    if let Some(sbid) = source_book_id {
        let rows: Vec<(Uuid, String)> =
            sqlx::query_as("SELECT id, slug FROM reference_systems WHERE book_id = $1")
                .bind(sbid)
                .fetch_all(&mut *tx)
                .await?;
        for (sys_id, slug) in rows {
            system_ids.insert(slug, sys_id);
        }
        eprintln!(
            "Reusing {} reference systems from source book",
            system_ids.len()
        );
    } else {
        for sys in &output.reference_systems {
            let sys_id: Uuid = sqlx::query_scalar(
                "INSERT INTO reference_systems (book_id, slug, label, ref_type, cite_priority, cite_template)
                 VALUES ($1, $2, $3, $4, $5, $6)
                 RETURNING id",
            )
            .bind(book_id)
            .bind(&sys.slug)
            .bind(&sys.label)
            .bind(&sys.ref_type)
            .bind(sys.cite_priority)
            .bind(&sys.cite_template)
            .fetch_one(&mut *tx)
            .await?;
            system_ids.insert(sys.slug.clone(), sys_id);
        }
    }

    // 3. Nodes -> blocks -> sentences -> page markers -> footnotes.
    let mut node_ids: HashMap<String, Uuid> = HashMap::new();
    let (mut node_count, mut block_count, mut sentence_count, mut marker_count) =
        (0u32, 0u32, 0u32, 0u32);
    let (mut footnote_count, mut footnote_sentence_count) = (0u32, 0u32);
    // Footnote sentences are numbered in one book-global sequence, separate
    // from the block-sentence numbering.
    let mut footnote_sentence_number = 1i32;
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

        // Translation node → link to its source-book node (`source_node_id`).
        let source_node_id: Option<Uuid> = if is_translation {
            source_node_map.get(&node.source_ref).copied()
        } else {
            None
        };

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
            "INSERT INTO toc_nodes (book_id, parent_id, source_id, source_node_id, source_ref, slug, path, sort_order, depth, label, label_html, content_hash)
             VALUES ($1, $2, $3, $4, $5, $6, $7::ltree, $8, $9, $10, $11, $12)
             RETURNING id",
        )
        .bind(book_id)
        .bind(parent_id)
        .bind(node_source_id)
        .bind(source_node_id)
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

                // Translation sentence → its source-book counterpart by key.
                let source_sentence_start_id: Option<Uuid> = if is_translation {
                    source_sentence_map
                        .get(&(node.source_ref.clone(), block.position, sent.position))
                        .copied()
                } else {
                    None
                };

                let sentence_id: Uuid = sqlx::query_scalar(
                    "INSERT INTO sentences (book_id, node_id, block_id, position, sentence_number, segment, indent, source_sentence_start_id, text, html, original_text, original_html, natural_key)
                     VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9, $10, $11, $12, $13)
                     RETURNING id",
                )
                .bind(book_id)
                .bind(node_id)
                .bind(block_id)
                .bind(sent.position)
                .bind(sent.sentence_number)
                .bind(sent.segment)
                .bind(sent.indent)
                .bind(source_sentence_start_id)
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

                // Footnotes anchored to this sentence; their sentences live in
                // `sentences` with `block_id` NULL and `footnote_id` set.
                // Footnote/sentence parity with the source edition is validated
                // up front by validate_translation_parity.
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

                        let fn_natural_key = reconcile::footnote_natural_key(
                            &node.source_ref,
                            footnote.number,
                            fn_sent.position,
                        );

                        sqlx::query(
                            "INSERT INTO sentences (book_id, node_id, footnote_id, position, sentence_number, source_sentence_start_id, text, html, original_text, original_html, natural_key)
                             VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9, $10, $11)",
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
                        .bind(&fn_natural_key)
                        .execute(&mut *tx)
                        .await?;
                        footnote_sentence_count += 1;
                        sentence_count += 1;
                    }
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
    if is_translation {
        eprintln!("  mode:           translation");
    }
    eprintln!("  book:           1");
    eprintln!("  sources:        {source_count}");
    eprintln!("  ref_systems:    {}", system_ids.len());
    eprintln!("  toc_nodes:      {node_count}");
    eprintln!("  content_blocks: {block_count}");
    eprintln!("  sentences:      {sentence_count}");
    eprintln!("  footnotes:      {footnote_count}");
    eprintln!("  fn_sentences:   {footnote_sentence_count}");
    eprintln!("  page_markers:   {marker_count}");
    Ok(())
}
