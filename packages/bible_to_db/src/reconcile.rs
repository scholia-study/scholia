//! Reconciling re-import for a Bible translation: update an existing translation
//! in place from the asset JSON, preserving sentence UUIDs (and the quotations
//! anchored to them) across text corrections and verse re-segmentation.
//!
//! Identity is anchored to the **verse**: `book:chapter:verse` is a stable,
//! externally-meaningful coordinate, so a split/merge inside a verse never
//! ripples to other translations. We reconcile one verse at a time with the
//! shared aligner. Heading sentences ("Genesis") carry no verse marker and are
//! left untouched. A changed chapter set or a verse added/removed within a
//! chapter is a versification change and aborts to `db:reset`.
//!
//! Reconcile is incremental via content hashes: each chapter node stores a hash
//! and the book stores the root. An unchanged root short-circuits the run in one
//! query; only chapters whose hash differs are loaded and applied. A chapter
//! whose verse counts are unchanged writes only the sentences whose text differs,
//! while one whose counts changed is re-laid-out and triggers a set-based global
//! renumber of both `sentence_number` and verse-marker `sort_order`.
//! `--full-rewrite` bypasses the hash checks and rewrites everything. See
//! docs/architecture/reconcile-incremental-hashing.md.

use std::collections::{HashMap, HashSet};
use std::fs;
use std::path::PathBuf;

use reconcile::{
    BlockContent, Existing, MarkerContent, NodeContent, SentenceContent, extend_anchors_to,
    migrate_dependents, node_hash, plan_block, root_hash, sentence_has_dependents,
};
use sqlx::{Postgres, Transaction};
use uuid::Uuid;

use crate::{BIBLE_BOOKS, Chapter, TranslationMeta, clean_verse, html_escape, segment_sentences};

// Temp `sentence_number` base for rows inserted during apply — non-null, unique,
// and out of the dense [1, N] range so the set-based renumber can reassign it.
const TEMP_SENTENCE_NUMBER_BASE: i32 = 8_000_000;

#[derive(Default)]
pub struct ReconcileReport {
    pub unchanged: bool,
    pub chapters_changed: u32,
    pub updated: u32,
    pub split: u32,
    pub merged: u32,
    pub inserted: u32,
    pub deleted: u32,
    pub deps_repointed: u32,
    pub markers_rebuilt: u32,
    pub renumbered: bool,
}

impl ReconcileReport {
    pub fn print(&self, dry_run: bool) {
        eprintln!();
        eprintln!(
            "=== Reconcile {}===",
            if dry_run {
                "(dry-run, nothing committed) "
            } else {
                ""
            }
        );
        if self.unchanged {
            eprintln!("  no changes (root hash matched) — nothing written");
            return;
        }
        eprintln!("  chapters changed:    {}", self.chapters_changed);
        eprintln!("  sentences updated:   {}", self.updated);
        eprintln!("  splits:              {}", self.split);
        eprintln!("  merges:              {}", self.merged);
        eprintln!("  sentences inserted:  {}", self.inserted);
        eprintln!("  sentences deleted:   {}", self.deleted);
        eprintln!("  dependents repointed:{}", self.deps_repointed);
        eprintln!("  page markers rebuilt:{}", self.markers_rebuilt);
        eprintln!(
            "  global renumber:     {}",
            if self.renumbered { "yes" } else { "skipped" }
        );
    }
}

struct DesiredVerse {
    verse: u32,
    ref_value: String,                // "chapter:verse", e.g. "1:1"
    full_text: String,                // cleaned whole-verse text (for block paragraph)
    sentences: Vec<(String, String)>, // (text, html) after segmentation
}

struct DesiredChapter {
    source_ref: String, // "{book}:{chapter}", e.g. "genesis:1"
    chapter_num: u32,
    node_id: Uuid,
    block_id: Uuid,
    verses: Vec<DesiredVerse>,
}

// --- Content hashing (tier-2 incremental reconcile) ------------------------
// A Bible chapter node has exactly one paragraph block; its content identity is
// the ordered verse sentences plus each sentence's verse marker. Built the same
// way on the insert and reconcile paths so the stored hashes match.

/// Hash of one chapter node: label + the joined paragraph + the ordered
/// (text, html, verse_ref) sentences. `sentences` carries one verse marker each.
pub(crate) fn chapter_node_hash(
    chapter_num: u32,
    para_text: &str,
    sentences: &[(String, String, String)],
) -> String {
    let label = format!("Chapter {chapter_num}");
    let para_html = format!("<p>{}</p>", html_escape(para_text));
    let node = NodeContent {
        label: &label,
        label_html: &label,
        blocks: vec![BlockContent {
            block_type: "paragraph",
            text: para_text,
            html: &para_html,
            original_text: None,
            original_html: None,
            sentences: sentences
                .iter()
                .map(|(t, h, vref)| SentenceContent {
                    text: t,
                    html: h,
                    original_text: None,
                    original_html: None,
                    segment: None,
                    markers: vec![MarkerContent {
                        system: "verse",
                        ref_value: vref,
                        char_offset: None,
                    }],
                    footnotes: Vec::new(),
                })
                .collect(),
        }],
    };
    node_hash(&node)
}

/// Wrapper so the insert path (which lives in `main.rs`) can compute the book
/// root without naming the `reconcile` crate through the local module shadow.
pub(crate) fn root_of(node_hashes: &[String]) -> String {
    root_hash(node_hashes)
}

/// The chapter paragraph text the reader falls back to — the cleaned verses
/// joined by spaces. Shared so insert and reconcile (and the hash) agree.
fn chapter_para_text(full_texts: &[String]) -> String {
    full_texts.join(" ")
}

fn chapter_sentence_tuples(verses: &[DesiredVerse]) -> Vec<(String, String, String)> {
    verses
        .iter()
        .flat_map(|v| {
            v.sentences
                .iter()
                .map(move |(t, h)| (t.clone(), h.clone(), v.ref_value.clone()))
        })
        .collect()
}

fn desired_chapter_hash(chapter: &DesiredChapter) -> String {
    let para_text = chapter_para_text(
        &chapter
            .verses
            .iter()
            .map(|v| v.full_text.clone())
            .collect::<Vec<_>>(),
    );
    let sentences = chapter_sentence_tuples(&chapter.verses);
    chapter_node_hash(chapter.chapter_num, &para_text, &sentences)
}

pub async fn reconcile_translation(
    tx: &mut Transaction<'_, Postgres>,
    book_id: Uuid,
    translation: &TranslationMeta,
    assets_dir: &str,
    force: bool,
    full_rewrite: bool,
) -> Result<ReconcileReport, Box<dyn std::error::Error>> {
    let mut report = ReconcileReport::default();

    // --- Load existing chapter structure (with stored hashes) --------------
    let node_rows: Vec<(String, Uuid, Option<String>)> = sqlx::query_as(
        "SELECT source_ref, id, content_hash FROM toc_nodes WHERE book_id = $1 AND depth = 1",
    )
    .bind(book_id)
    .fetch_all(&mut **tx)
    .await?;
    let node_by_chapter: HashMap<String, Uuid> = node_rows
        .iter()
        .map(|(r, id, _)| (r.clone(), *id))
        .collect();
    let stored_hash_by_chapter: HashMap<String, Option<String>> = node_rows
        .iter()
        .map(|(r, _, h)| (r.clone(), h.clone()))
        .collect();

    let block_rows: Vec<(String, Uuid)> = sqlx::query_as(
        "SELECT tn.source_ref, cb.id
         FROM content_blocks cb JOIN toc_nodes tn ON cb.node_id = tn.id
         WHERE cb.book_id = $1 AND tn.depth = 1",
    )
    .bind(book_id)
    .fetch_all(&mut **tx)
    .await?;
    let block_by_chapter: HashMap<String, Uuid> = block_rows.into_iter().collect();

    let verse_system_id: Uuid = sqlx::query_scalar(
        "SELECT id FROM reference_systems WHERE book_id = $1 AND slug = 'verse'",
    )
    .bind(book_id)
    .fetch_one(&mut **tx)
    .await?;

    // --- Build desired structure from the asset JSON -----------------------
    let mut desired: Vec<DesiredChapter> = Vec::new();
    for bb in BIBLE_BOOKS {
        for chapter_num in 1..=bb.chapters {
            let source_ref = format!("{}:{}", bb.slug, chapter_num);
            let path: PathBuf = [
                assets_dir,
                translation.slug,
                bb.slug,
                &format!("{chapter_num}.json"),
            ]
            .iter()
            .collect();
            let raw = fs::read_to_string(&path)
                .map_err(|e| format!("Failed to read {}: {}", path.display(), e))?;
            let chapter: Chapter = serde_json::from_str(&raw)?;

            let node_id = *node_by_chapter.get(&source_ref).ok_or_else(|| {
                format!("chapter node {source_ref} missing; use `just db-reload`")
            })?;
            let block_id = *block_by_chapter.get(&source_ref).ok_or_else(|| {
                format!("chapter block {source_ref} missing; use `just db-reload`")
            })?;

            let mut verses = Vec::new();
            for v in &chapter.verses {
                let full_text = clean_verse(&v.text);
                let sentences = segment_sentences(&full_text)
                    .iter()
                    .map(|t| (t.clone(), html_escape(t)))
                    .collect();
                verses.push(DesiredVerse {
                    verse: v.verse,
                    ref_value: format!("{}:{}", chapter_num, v.verse),
                    full_text,
                    sentences,
                });
            }
            desired.push(DesiredChapter {
                source_ref,
                chapter_num,
                node_id,
                block_id,
                verses,
            });
        }
    }

    // --- Desired hashes + root short-circuit -------------------------------
    let desired_chapter_hash: HashMap<String, String> = desired
        .iter()
        .map(|c| (c.source_ref.clone(), desired_chapter_hash(c)))
        .collect();
    let desired_root = root_hash(
        &desired
            .iter()
            .map(|c| desired_chapter_hash[&c.source_ref].clone())
            .collect::<Vec<_>>(),
    );

    let stored_root: Option<String> =
        sqlx::query_scalar("SELECT content_hash FROM books WHERE id = $1")
            .bind(book_id)
            .fetch_one(&mut **tx)
            .await?;
    if !full_rewrite && stored_root.as_deref() == Some(desired_root.as_str()) {
        report.unchanged = true;
        return Ok(report);
    }

    // --- Pre-flight: chapter + verse structure must be stable --------------
    let desired_chapters: HashSet<&str> = desired.iter().map(|c| c.source_ref.as_str()).collect();
    let existing_chapters: HashSet<&str> = node_by_chapter.keys().map(|s| s.as_str()).collect();
    if desired_chapters != existing_chapters {
        return Err("chapter set changed; not reconcilable — use `just db-reload`".into());
    }

    // --- Changed set (NULL stored hash ⇒ changed; `--full` ⇒ everything) ----
    let changed: HashSet<String> = desired
        .iter()
        .filter(|c| {
            full_rewrite
                || stored_hash_by_chapter
                    .get(&c.source_ref)
                    .cloned()
                    .flatten()
                    .as_deref()
                    != Some(desired_chapter_hash[&c.source_ref].as_str())
        })
        .map(|c| c.source_ref.clone())
        .collect();
    report.chapters_changed = changed.len() as u32;

    if changed.is_empty() {
        sqlx::query("UPDATE books SET content_hash = $2 WHERE id = $1")
            .bind(book_id)
            .bind(&desired_root)
            .execute(&mut **tx)
            .await?;
        return Ok(report);
    }
    let changed_vec: Vec<String> = changed.iter().cloned().collect();

    // --- Load existing verse sentences, scoped to changed chapters ---------
    // The verse-marker join excludes book-node heading sentences (no marker).
    let sent_rows: Vec<(String, String, Uuid, String, String)> = sqlx::query_as(
        "SELECT tn.source_ref, pm.ref_value, s.id, s.text, s.html
         FROM sentences s
         JOIN page_markers pm ON pm.sentence_id = s.id
         JOIN reference_systems rs ON rs.id = pm.system_id AND rs.slug = 'verse'
         JOIN toc_nodes tn ON tn.id = s.node_id
         WHERE s.book_id = $1 AND tn.source_ref = ANY($2)
         ORDER BY tn.sort_order, s.position",
    )
    .bind(book_id)
    .bind(&changed_vec)
    .fetch_all(&mut **tx)
    .await?;
    let mut existing_by_verse: HashMap<(String, String), Vec<Existing>> = HashMap::new();
    let mut existing_content: HashMap<Uuid, (String, String)> = HashMap::new();
    for (chapter_ref, verse_ref, id, text, html) in sent_rows {
        existing_by_verse
            .entry((chapter_ref, verse_ref))
            .or_default()
            .push(Existing {
                id,
                text: text.clone(),
            });
        existing_content.insert(id, (text, html));
    }

    // --- Pre-flight: verse sets stable within changed chapters -------------
    for chapter in desired.iter().filter(|c| changed.contains(&c.source_ref)) {
        let desired_verses: HashSet<&str> = chapter
            .verses
            .iter()
            .map(|v| v.ref_value.as_str())
            .collect();
        let existing_verses: HashSet<&str> = existing_by_verse
            .keys()
            .filter(|(cref, _)| cref == &chapter.source_ref)
            .map(|(_, vref)| vref.as_str())
            .collect();
        if desired_verses != existing_verses {
            return Err(format!(
                "chapter {}: verses added/removed; not reconcilable — use `just db-reload`",
                chapter.source_ref
            )
            .into());
        }
    }

    // --- Plan each verse in changed chapters (abort on ambiguity) ----------
    let mut plans: HashMap<(String, String), reconcile::BlockPlan> = HashMap::new();
    let mut all_retired: Vec<(Uuid, Option<Uuid>)> = Vec::new();
    // A chapter is laid out fresh (offset + reassign) only if some verse changed
    // its sentence count — that shifts chapter-wide block positions. Otherwise
    // its sentences are written selectively.
    let mut chapter_count_delta: HashMap<String, bool> = HashMap::new();
    for chapter in desired.iter().filter(|c| changed.contains(&c.source_ref)) {
        let mut delta = full_rewrite;
        for verse in &chapter.verses {
            let key = (chapter.source_ref.clone(), verse.ref_value.clone());
            let empty: Vec<Existing> = Vec::new();
            let old = existing_by_verse.get(&key).unwrap_or(&empty);
            let new: Vec<&str> = verse.sentences.iter().map(|(t, _)| t.as_str()).collect();
            let label = format!("{} v{}", chapter.source_ref, verse.verse);
            let plan = plan_block(&label, old, &new)?;
            all_retired.extend(plan.retired.iter().copied());
            if old.len() != new.len() {
                delta = true;
            }
            plans.insert(key, plan);
        }
        chapter_count_delta.insert(chapter.source_ref.clone(), delta);
    }
    let any_count_delta = chapter_count_delta.values().any(|d| *d);

    // Pure deletes (no survivor) that still carry user data are unsafe.
    for (retired_id, survivor) in &all_retired {
        if survivor.is_none() && !force && sentence_has_dependents(tx, *retired_id).await? {
            return Err(format!(
                "sentence {retired_id} would be deleted but has quotations/resources anchored to it; \
                 aborting (pass --force to delete anyway, or `just db-reload`)"
            )
            .into());
        }
    }

    // --- Apply -------------------------------------------------------------
    // 1. Migrate dependents of merged-away sentences onto their survivor.
    for (retired_id, survivor) in &all_retired {
        if let Some(survivor_id) = survivor {
            report.deps_repointed += migrate_dependents(tx, *retired_id, *survivor_id).await?;
            report.merged += 1;
        }
    }

    // 2. Offset only the count-changed chapters out of the position +
    //    natural_key index space. `sentence_number` is owned by the set-based
    //    renumber (step 6).
    let offset_node_ids: Vec<Uuid> = desired
        .iter()
        .filter(|c| chapter_count_delta.get(&c.source_ref) == Some(&true))
        .map(|c| c.node_id)
        .collect();
    if !offset_node_ids.is_empty() {
        sqlx::query(
            "UPDATE sentences SET position = position + 10000, natural_key = NULL
             WHERE book_id = $1 AND node_id = ANY($2)",
        )
        .bind(book_id)
        .bind(&offset_node_ids)
        .execute(&mut **tx)
        .await?;
    }

    // 3. Apply keeps + inserts. `resolved` (sentence_id, verse ref) is built for
    //    count-delta chapters so their markers can be rebuilt.
    let mut resolved: Vec<(Uuid, String, Uuid)> = Vec::new(); // (sid, verse_ref, chapter_node)
    let mut split_new_ids: Vec<(Uuid, Uuid)> = Vec::new();
    let mut next_temp: i32 = TEMP_SENTENCE_NUMBER_BASE;

    for chapter in desired.iter().filter(|c| changed.contains(&c.source_ref)) {
        let count_delta = chapter_count_delta[&chapter.source_ref];
        let mut block_position: i16 = 0;
        for verse in &chapter.verses {
            let key = (chapter.source_ref.clone(), verse.ref_value.clone());
            let plan = &plans[&key];
            let mut idx_uuid: Vec<Uuid> = Vec::with_capacity(verse.sentences.len());
            for (idx, (text, html)) in verse.sentences.iter().enumerate() {
                let sid = if count_delta {
                    let natural_key = format!("{}:{}/s{}", chapter.source_ref, verse.verse, idx);
                    match plan.assignment[idx] {
                        Some(existing_id) => {
                            // Full reassign: chapter-wide positions shifted.
                            sqlx::query(
                                "UPDATE sentences
                                 SET position = $2, text = $3, html = $4,
                                     natural_key = $5, updated_at = now()
                                 WHERE id = $1",
                            )
                            .bind(existing_id)
                            .bind(block_position)
                            .bind(text)
                            .bind(html)
                            .bind(&natural_key)
                            .execute(&mut **tx)
                            .await?;
                            report.updated += 1;
                            existing_id
                        }
                        None => {
                            let temp_number = next_temp;
                            next_temp += 1;
                            let id: Uuid = sqlx::query_scalar(
                                "INSERT INTO sentences (book_id, node_id, block_id, position, sentence_number, text, html, natural_key)
                                 VALUES ($1, $2, $3, $4, $5, $6, $7, $8)
                                 RETURNING id",
                            )
                            .bind(book_id)
                            .bind(chapter.node_id)
                            .bind(chapter.block_id)
                            .bind(block_position)
                            .bind(temp_number)
                            .bind(text)
                            .bind(html)
                            .bind(&natural_key)
                            .fetch_one(&mut **tx)
                            .await?;
                            report.inserted += 1;
                            id
                        }
                    }
                } else {
                    // Same-count chapter: positions/keys/number stable. Write
                    // only the sentences whose text differs.
                    let existing_id = plan.assignment[idx]
                        .expect("same-count verse: every desired sentence maps to an existing row");
                    let differs = match existing_content.get(&existing_id) {
                        Some((t, h)) => t != text || h != html,
                        None => true,
                    };
                    if differs {
                        sqlx::query(
                            "UPDATE sentences SET text = $2, html = $3, updated_at = now()
                             WHERE id = $1",
                        )
                        .bind(existing_id)
                        .bind(text)
                        .bind(html)
                        .execute(&mut **tx)
                        .await?;
                        report.updated += 1;
                    }
                    existing_id
                };
                idx_uuid.push(sid);
                if count_delta {
                    resolved.push((sid, verse.ref_value.clone(), chapter.node_id));
                }
                block_position += 1;
            }
            for (first_half_id, second_idx) in &plan.splits {
                split_new_ids.push((*first_half_id, idx_uuid[*second_idx]));
                report.split += 1;
            }
        }
    }

    // 4. Delete retired rows (dependents already migrated / verified safe).
    for (retired_id, _survivor) in &all_retired {
        sqlx::query("DELETE FROM sentences WHERE id = $1")
            .bind(retired_id)
            .execute(&mut **tx)
            .await?;
        report.deleted += 1;
    }

    // 5. Extend single-sentence anchors over each split's new second half.
    for (first_half_id, new_id) in &split_new_ids {
        report.deps_repointed += extend_anchors_to(tx, *first_half_id, *new_id).await?;
    }

    // 6. Rebuild verse markers for count-changed chapters only (same-count
    //    chapters keep their markers — verse identity and order are unchanged).
    //    sort_order is fixed globally in step 7 alongside the renumber.
    if !offset_node_ids.is_empty() {
        sqlx::query(
            "DELETE FROM page_markers
             WHERE sentence_id IN (SELECT id FROM sentences WHERE node_id = ANY($1))",
        )
        .bind(&offset_node_ids)
        .execute(&mut **tx)
        .await?;
        for (sid, ref_value, _node) in &resolved {
            sqlx::query(
                "INSERT INTO page_markers (system_id, sentence_id, ref_value, sort_order, char_offset)
                 VALUES ($1, $2, $3, 0, NULL)",
            )
            .bind(verse_system_id)
            .bind(sid)
            .bind(ref_value)
            .execute(&mut **tx)
            .await?;
            report.markers_rebuilt += 1;
        }
    }

    // 7. Set-based global renumber — only when some chapter changed its count.
    //    Two statements reassign sentence_number dense by document order; a third
    //    re-points every verse marker's sort_order at its sentence's new number
    //    (marker order == reading order, matching the fresh-insert invariant).
    if any_count_delta {
        sqlx::query(
            "UPDATE sentences SET sentence_number = sentence_number + 1000000
             WHERE book_id = $1 AND sentence_number IS NOT NULL AND block_id IS NOT NULL",
        )
        .bind(book_id)
        .execute(&mut **tx)
        .await?;
        sqlx::query(
            "WITH ordered AS (
                 SELECT s.id, ROW_NUMBER() OVER (
                     ORDER BY tn.sort_order, cb.position, s.position
                 ) AS rn
                 FROM sentences s
                 JOIN content_blocks cb ON s.block_id = cb.id
                 JOIN toc_nodes tn ON cb.node_id = tn.id
                 WHERE s.book_id = $1 AND s.sentence_number IS NOT NULL AND s.block_id IS NOT NULL
             )
             UPDATE sentences s SET sentence_number = o.rn FROM ordered o WHERE s.id = o.id",
        )
        .bind(book_id)
        .execute(&mut **tx)
        .await?;
        sqlx::query(
            "UPDATE page_markers pm SET sort_order = s.sentence_number
             FROM sentences s
             WHERE pm.sentence_id = s.id
               AND s.book_id = $1 AND s.sentence_number IS NOT NULL",
        )
        .bind(book_id)
        .execute(&mut **tx)
        .await?;
        report.renumbered = true;
    }

    // 8. Regenerate each changed chapter's paragraph block text/html.
    for chapter in desired.iter().filter(|c| changed.contains(&c.source_ref)) {
        let para_text = chapter_para_text(
            &chapter
                .verses
                .iter()
                .map(|v| v.full_text.clone())
                .collect::<Vec<_>>(),
        );
        let para_html = format!("<p>{}</p>", html_escape(&para_text));
        sqlx::query("UPDATE content_blocks SET text = $1, html = $2 WHERE id = $3")
            .bind(&para_text)
            .bind(&para_html)
            .bind(chapter.block_id)
            .execute(&mut **tx)
            .await?;
    }

    // 9. Write back hashes: changed chapters + the book root.
    for chapter in desired.iter().filter(|c| changed.contains(&c.source_ref)) {
        sqlx::query("UPDATE toc_nodes SET content_hash = $2 WHERE id = $1")
            .bind(chapter.node_id)
            .bind(&desired_chapter_hash[&chapter.source_ref])
            .execute(&mut **tx)
            .await?;
    }
    sqlx::query("UPDATE books SET content_hash = $2 WHERE id = $1")
        .bind(book_id)
        .bind(&desired_root)
        .execute(&mut **tx)
        .await?;

    Ok(report)
}
