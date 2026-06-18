use std::collections::HashMap;
use std::fs;

use sqlx::PgPool;
use uuid::Uuid;

use kant1_md_to_struct::model::Output;
use reconcile::{node_hash, root_hash};

use crate::reconcile_input::node_content;

/// A translation edition is locked 1:1 to its source book: every paragraph and
/// footnote must carry the same number of sentences as the German source, or
/// quotation projection and side-by-side alignment break (the extra/missing
/// translation sentences end up with no `source_sentence_start_id`). The MD →
/// struct pipeline builds the two editions independently, so a split/merge made
/// in only one edition would otherwise pass silently. Check up front — before
/// either the fresh-insert or reconcile path — and abort listing every offender.
fn validate_translation_parity(
    output: &Output,
    source_sentence_map: &HashMap<(String, i16, i16), Uuid>,
    source_fn_sentence_counts: &HashMap<i32, i16>,
) -> Result<(), Box<dyn std::error::Error>> {
    // German sentence count per (source_ref, block_position).
    let mut de_block_counts: HashMap<(String, i16), usize> = HashMap::new();
    for (source_ref, block_pos, _sent_pos) in source_sentence_map.keys() {
        *de_block_counts
            .entry((source_ref.clone(), *block_pos))
            .or_insert(0) += 1;
    }

    let mut problems: Vec<String> = Vec::new();

    for node in &output.toc_nodes {
        for block in &node.content_blocks {
            let en = block.sentences.len();
            let de = de_block_counts
                .get(&(node.source_ref.clone(), block.position))
                .copied()
                .unwrap_or(0);
            if en != de {
                problems.push(format!(
                    "  node {} / block {}: translation has {en} sentence(s), source has {de}",
                    node.source_ref, block.position
                ));
            }
            for footnote in block.sentences.iter().flat_map(|s| &s.footnotes) {
                let en = footnote.sentences.len();
                match source_fn_sentence_counts.get(&footnote.number) {
                    Some(de) if en != *de as usize => problems.push(format!(
                        "  footnote {}: translation has {en} sentence(s), source has {de}",
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

    // Symmetric direction: a source block carrying sentences that has no
    // counterpart in the translation (the forward loop only sees translation
    // blocks, so a wholly missing block would otherwise slip through).
    let en_block_keys: std::collections::HashSet<(String, i16)> = output
        .toc_nodes
        .iter()
        .flat_map(|n| {
            n.content_blocks
                .iter()
                .map(move |b| (n.source_ref.clone(), b.position))
        })
        .collect();
    for (source_ref, block_pos) in de_block_counts.keys() {
        if !en_block_keys.contains(&(source_ref.clone(), *block_pos)) {
            problems.push(format!(
                "  node {source_ref} / block {block_pos}: present in source but missing from translation"
            ));
        }
    }

    if !problems.is_empty() {
        return Err(format!(
            "translation is out of sync with the source edition ({} mismatch(es)) — \
             fix the markdown so sentence/footnote splits match, then re-run \
             (reconcile the source book first if you edited it too):\n{}",
            problems.len(),
            problems.join("\n")
        )
        .into());
    }
    Ok(())
}

pub async fn run(
    input_file: &str,
    database_url: Option<String>,
    source_book_slug: Option<String>,
    dry_run: bool,
    force: bool,
    full_rewrite: bool,
) -> Result<(), Box<dyn std::error::Error>> {
    dotenvy::dotenv().ok();

    let data = fs::read_to_string(input_file)?;
    let output: Output = serde_json::from_str(&data)?;

    let pool = PgPool::connect_with(dataduct::db::pg_connect_options(database_url)?).await?;

    // A book already in the DB is reconciled in place — matching the freshly
    // parsed struct against existing rows and carrying sentence UUIDs (and the
    // quotations/resources anchored to them) forward across edits. A new book
    // is inserted fresh below.
    let existing_book_id: Option<Uuid> = sqlx::query_scalar("SELECT id FROM books WHERE slug = $1")
        .bind(&output.book.slug)
        .fetch_optional(&pool)
        .await?;

    let mut tx = pool.begin().await?;

    let system_user_id = dataduct::seed::SYSTEM_USER_ID;

    let is_translation = source_book_slug.is_some();

    // 0. Look up source book (translation mode only)
    let source_book: Option<(Uuid, Uuid)> = if let Some(ref slug) = source_book_slug {
        let row: Option<(Uuid, Uuid)> =
            sqlx::query_as("SELECT id, source_id FROM books WHERE slug = $1")
                .bind(slug)
                .fetch_optional(&mut *tx)
                .await?;
        let (book_id, src_id) = row.ok_or_else(|| format!("Source book not found: {slug}"))?;
        eprintln!(
            "Source book {:?} (book={}, source={})",
            slug, book_id, src_id
        );
        Some((book_id, src_id))
    } else {
        None
    };
    let source_book_id: Option<Uuid> = source_book.map(|(id, _)| id);
    let translation_of_id: Option<Uuid> = source_book.map(|(_, id)| id);

    let book_id: Uuid = if let Some(id) = existing_book_id {
        eprintln!("Reconciling existing book {:?} ({})", output.book.slug, id);
        id
    } else {
        // 1a. Upsert person (author)
        let person_id: Uuid = sqlx::query_scalar(
            "INSERT INTO persons (name, sort_name, protected, created_by)
         VALUES ($1, $2, true, $3)
         ON CONFLICT (name) DO UPDATE SET name = EXCLUDED.name
         RETURNING id",
        )
        .bind(&output.book.author)
        .bind({
            // Auto-generate sort_name: "Immanuel Kant" → "Kant, Immanuel"
            let parts: Vec<&str> = output.book.author.split_whitespace().collect();
            if parts.len() >= 2 {
                Some(format!(
                    "{}, {}",
                    parts.last().unwrap(),
                    parts[..parts.len() - 1].join(" ")
                ))
            } else {
                None
            }
        })
        .bind(system_user_id)
        .fetch_one(&mut *tx)
        .await?;
        eprintln!("Person {:?} ({})", output.book.author, person_id);

        // 1b. Insert bibliographic source
        let publication_year: Option<i16> = output.book.source_date.parse::<i16>().ok();
        let bib_source_id: Uuid = sqlx::query_scalar(
        "INSERT INTO sources (source_type, title, publication_year, publisher, translation_of_id, protected, created_by)
         VALUES ('book', $1, $2, $3, $4, true, $5)
         RETURNING id",
    )
    .bind(&output.book.title)
    .bind(publication_year)
    .bind(&output.book.source)
    .bind(translation_of_id)
    .bind(system_user_id)
    .fetch_one(&mut *tx)
    .await?;
        eprintln!("Source {:?} ({})", output.book.title, bib_source_id);

        // 1c. Link person to source as author
        sqlx::query(
            "INSERT INTO source_persons (source_id, person_id, role, position)
         VALUES ($1, $2, 'author', 0)
         ON CONFLICT DO NOTHING",
        )
        .bind(bib_source_id)
        .bind(person_id)
        .execute(&mut *tx)
        .await?;

        // 1d. Insert book
        let about_text: &str = if source_book_id.is_some() {
            // Translation: English (or other) rendering of the German source.
            "This English translation of Kant's Kritik der reinen Vernunft is a Scholia community project. \
         It is prepared from the 1911 Akademie-Ausgabe (Band III) facsimile of the second edition (B), \
         which serves as the underlying German text on Scholia."
        } else {
            // Source-language book: the German B-edition.
            "This German edition reproduces the text of Kant's Kritik der reinen Vernunft as printed in the \
         1911 Akademie-Ausgabe (Band III) facsimile of the second edition (B, 1787). Margin markers \
         refer to AA page numbers; inline B-edition pagination is preserved within the text. \
         The text itself is in public domain. The digital edition on Scholia is a community-driven \
         project. Corrections and refinements are welcome."
        };

        let book_id: Uuid = sqlx::query_scalar(
            "INSERT INTO books (slug, source_id, language, about_text)
         VALUES ($1, $2, $3, $4)
         RETURNING id",
        )
        .bind(&output.book.slug)
        .bind(bib_source_id)
        .bind(&output.book.language)
        .bind(about_text)
        .fetch_one(&mut *tx)
        .await?;

        eprintln!("Inserted book {:?} ({})", output.book.slug, book_id);
        book_id
    };

    // 2. Reference systems — translation reuses the source book's systems;
    // a fresh source-language book inserts its own; a reconcile loads the ones
    // already in place.
    let mut system_ids: HashMap<String, Uuid> = HashMap::new();

    if let Some(source_id) = source_book_id {
        let rows: Vec<(Uuid, String)> =
            sqlx::query_as("SELECT id, slug FROM reference_systems WHERE book_id = $1")
                .bind(source_id)
                .fetch_all(&mut *tx)
                .await?;

        for (id, slug) in rows {
            system_ids.insert(slug, id);
        }
        eprintln!(
            "Reusing {} reference systems from source book",
            system_ids.len()
        );
    } else if existing_book_id.is_some() {
        let rows: Vec<(Uuid, String)> =
            sqlx::query_as("SELECT id, slug FROM reference_systems WHERE book_id = $1")
                .bind(book_id)
                .fetch_all(&mut *tx)
                .await?;
        for (id, slug) in rows {
            system_ids.insert(slug, id);
        }
        eprintln!("Loaded {} existing reference systems", system_ids.len());
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
        let node_rows: Vec<(Uuid, String)> =
            sqlx::query_as("SELECT id, source_ref FROM toc_nodes WHERE book_id = $1")
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

    // Translation must stay structurally locked to its source (covers both the
    // fresh-insert and reconcile paths below).
    if is_translation {
        validate_translation_parity(&output, &source_sentence_map, &source_fn_sentence_counts)?;
    }

    // Reconcile path: the book exists, so update it in place from `output`
    // rather than running the fresh-insert loop below. Kant computes its own
    // hashes (parity with the fresh-insert path) and maps the struct → the
    // shared IR; the generic orchestration takes it from there.
    if let Some(eid) = existing_book_id {
        let (desired_node_hashes, desired_root) = crate::reconcile_input::compute_hashes(&output);
        let input = crate::reconcile_input::to_input(&output, &source_node_map);
        let report = reconcile::reconcile_book(
            &mut tx,
            eid,
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
            dataduct::cache::purge_blocking(&dataduct::cache::purge_paths(&output.book.slug)).await;
        }
        report.print(dry_run);
        return Ok(());
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
    // Node content hashes accumulate as we insert; the root is stored on `books`
    // after the loop, so the first *reconcile* is already in the fast state.
    let mut node_hashes: Vec<String> = Vec::new();

    for node in &output.toc_nodes {
        let content_hash = node_hash(&node_content(node));
        node_hashes.push(content_hash.clone());
        let parent_id: Option<Uuid> = node
            .parent_source_ref
            .as_ref()
            .and_then(|ref_str| node_ids.get(ref_str).copied());

        let source_node_id: Option<Uuid> = if is_translation {
            source_node_map.get(&node.source_ref).copied()
        } else {
            None
        };

        // Store label_html only when it differs from plain label (i.e. has formatting)
        let label_html = if node.label_html != node.label {
            Some(&node.label_html)
        } else {
            None
        };

        let node_id: Uuid = sqlx::query_scalar(
            "INSERT INTO toc_nodes (book_id, parent_id, source_node_id, source_ref, slug, path, sort_order, depth, label, label_html, content_hash)
             VALUES ($1, $2, $3, $4, $5, $6::ltree, $7, $8, $9, $10, $11)
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
                // Resolve source sentence linkage (translation mode, block sentences only)
                let source_sentence_start_id: Option<Uuid> = if is_translation {
                    source_sentence_map
                        .get(&(node.source_ref.clone(), block.position, sent.position))
                        .copied()
                } else {
                    None
                };

                let natural_key =
                    reconcile::natural_key(&node.source_ref, block.position, sent.position);

                let sentence_id: Uuid = sqlx::query_scalar(
                    "INSERT INTO sentences (book_id, node_id, block_id, position, sentence_number, segment, source_sentence_start_id, text, html, original_text, original_html, natural_key)
                     VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9, $10, $11, $12)
                     RETURNING id",
                )
                .bind(book_id)
                .bind(node_id)
                .bind(block_id)
                .bind(sent.position)
                .bind(sent.sentence_number)
                .bind(sent.segment)
                .bind(source_sentence_start_id)
                .bind(&sent.text)
                .bind(&sent.html)
                .bind(&sent.original_text)
                .bind(&sent.original_html)
                .bind(&natural_key)
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

                    // Footnote/sentence parity with the source edition is
                    // validated up front by validate_translation_parity.

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
