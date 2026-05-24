use std::collections::HashMap;
use std::fs;

use sqlx::PgPool;
use sqlx::postgres::PgConnectOptions;
use uuid::Uuid;

use kant1_md_to_struct::model::Output;

/// Build sqlx connect options. CLI `--database-url` wins; otherwise
/// prefer discrete `POSTGRES_*` env vars (k8s Secret pattern — avoids
/// the URL-special-char trap when `$(VAR)` substitution is literal);
/// fall back to `DATABASE_URL` for laptop `.env`. Same shape as
/// `apps/api/src/config.rs::pg_connect_options_from_env` — kept
/// inline rather than extracted to a shared crate because there are
/// only two ingest consumers today.
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

pub async fn run(
    input_file: &str,
    database_url: Option<String>,
    source_book_slug: Option<String>,
) -> Result<(), Box<dyn std::error::Error>> {
    dotenvy::dotenv().ok();

    let data = fs::read_to_string(input_file)?;
    let output: Output = serde_json::from_str(&data)?;

    let pool = PgPool::connect_with(pg_connect_options(database_url)?).await?;

    // Idempotency guard. `books.slug` is UNIQUE; a second run would
    // otherwise hit a constraint violation mid-transaction. Detect
    // upfront and no-op cleanly. Re-ingesting under a flag is a
    // separate (still-deferred) design — for now, schema/source-data
    // fixes go through `pnpm db:reset`.
    let existing_book: Option<Uuid> = sqlx::query_scalar("SELECT id FROM books WHERE slug = $1")
        .bind(&output.book.slug)
        .fetch_optional(&pool)
        .await?;
    if existing_book.is_some() {
        eprintln!("Book '{}' already imported. Skipping.", output.book.slug);
        return Ok(());
    }

    let mut tx = pool.begin().await?;

    // System user owns all seed-imported persons/sources; see db/001_schema.sql.
    let system_user_id: Uuid =
        Uuid::parse_str("00000000-0000-0000-0000-000000000001").expect("valid system user UUID");

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

    // 2. Reference systems — reuse source book's systems in translation mode, otherwise insert new
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

        // Store label_html only when it differs from plain label (i.e. has formatting)
        let label_html = if node.label_html != node.label {
            Some(&node.label_html)
        } else {
            None
        };

        let node_id: Uuid = sqlx::query_scalar(
            "INSERT INTO toc_nodes (book_id, parent_id, source_node_id, source_ref, slug, path, sort_order, depth, label, label_html)
             VALUES ($1, $2, $3, $4, $5, $6::ltree, $7, $8, $9, $10)
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

    // Drop stale listing entries so the new book shows up immediately
    // rather than waiting out the TTL. No-op if CACHE_PURGE_URL is
    // unset (local dev without a proxy).
    purge_cache(&["/api/library", "/api/books", "/books"]).await;

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

/// Send PURGE requests to the proxy's cluster-internal admin port for
/// each given path. `CACHE_PURGE_URL` looks like
/// `http://scholia-proxy.scholia.svc.cluster.local:8080`. If unset or
/// empty (local dev without a proxy), this is a no-op.
///
/// Synchronous on purpose — the ingest binary exits right after, so
/// fire-and-forget tasks would just be killed. We wait for each PURGE
/// and log the outcome. Failures are logged but don't error out the
/// import: the cache is best-effort.
async fn purge_cache(paths: &[&str]) {
    let Ok(base) = std::env::var("CACHE_PURGE_URL") else {
        return;
    };
    if base.is_empty() {
        return;
    }
    let base = base.trim_end_matches('/');
    let client = match reqwest::Client::builder()
        .timeout(std::time::Duration::from_secs(5))
        .build()
    {
        Ok(c) => c,
        Err(e) => {
            eprintln!("PURGE client init failed: {e} (skipping cache invalidation)");
            return;
        }
    };
    let method = reqwest::Method::from_bytes(b"PURGE").expect("PURGE is a valid method");
    for path in paths {
        let url = format!("{base}{path}");
        match client.request(method.clone(), &url).send().await {
            Ok(resp) => eprintln!("PURGE {} → {}", path, resp.status()),
            Err(e) => eprintln!("PURGE {} failed: {}", path, e),
        }
    }
}
