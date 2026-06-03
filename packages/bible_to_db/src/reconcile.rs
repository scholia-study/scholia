//! Reconciling re-import for a Bible translation: update an existing translation
//! in place from the asset JSON, preserving sentence UUIDs (and the quotations
//! anchored to them) across text corrections and verse re-segmentation.
//!
//! Identity is anchored to the **verse**: `book:chapter:verse` is a stable,
//! externally-meaningful coordinate, and cross-translation alignment is at verse
//! granularity, so a sentence split/merge inside a verse never ripples to other
//! translations. We reconcile one verse at a time with the shared aligner.
//!
//! Simpler than the Kant reconcile: no footnotes, no sentence-level translation
//! links, no `segment`/`original_text`. Book-node heading sentences ("Genesis")
//! are immutable and carry no verse marker, so they're left untouched. A changed
//! chapter set or a verse added/removed within a chapter is a versification
//! change and aborts to `db:reset`.

use std::collections::{HashMap, HashSet};
use std::fs;
use std::path::PathBuf;

use reconcile::{
    Existing, extend_anchors_to, migrate_dependents, plan_block, sentence_has_dependents,
};
use sqlx::{Postgres, Transaction};
use uuid::Uuid;

use crate::{BIBLE_BOOKS, Chapter, TranslationMeta, clean_verse, html_escape, segment_sentences};

#[derive(Default)]
pub struct ReconcileReport {
    pub updated: u32,
    pub split: u32,
    pub merged: u32,
    pub inserted: u32,
    pub deleted: u32,
    pub deps_repointed: u32,
    pub markers_rebuilt: u32,
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
        eprintln!("  sentences updated:   {}", self.updated);
        eprintln!("  splits:              {}", self.split);
        eprintln!("  merges:              {}", self.merged);
        eprintln!("  sentences inserted:  {}", self.inserted);
        eprintln!("  sentences deleted:   {}", self.deleted);
        eprintln!("  dependents repointed:{}", self.deps_repointed);
        eprintln!("  page markers rebuilt:{}", self.markers_rebuilt);
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
    node_id: Uuid,
    block_id: Uuid,
    verses: Vec<DesiredVerse>,
}

pub async fn reconcile_translation(
    tx: &mut Transaction<'_, Postgres>,
    book_id: Uuid,
    translation: &TranslationMeta,
    assets_dir: &str,
    force: bool,
) -> Result<ReconcileReport, Box<dyn std::error::Error>> {
    let mut report = ReconcileReport::default();

    // --- Load existing chapter structure ----------------------------------
    let node_rows: Vec<(String, Uuid)> =
        sqlx::query_as("SELECT source_ref, id FROM toc_nodes WHERE book_id = $1 AND depth = 1")
            .bind(book_id)
            .fetch_all(&mut **tx)
            .await?;
    let node_by_chapter: HashMap<String, Uuid> = node_rows.into_iter().collect();

    let block_rows: Vec<(String, Uuid)> = sqlx::query_as(
        "SELECT tn.source_ref, cb.id
         FROM content_blocks cb JOIN toc_nodes tn ON cb.node_id = tn.id
         WHERE cb.book_id = $1 AND tn.depth = 1",
    )
    .bind(book_id)
    .fetch_all(&mut **tx)
    .await?;
    let block_by_chapter: HashMap<String, Uuid> = block_rows.into_iter().collect();

    // Existing verse sentences grouped by (chapter source_ref, verse ref_value).
    // The verse-marker join excludes book-node heading sentences (no marker).
    let sent_rows: Vec<(String, String, Uuid, String)> = sqlx::query_as(
        "SELECT tn.source_ref, pm.ref_value, s.id, s.text
         FROM sentences s
         JOIN page_markers pm ON pm.sentence_id = s.id
         JOIN reference_systems rs ON rs.id = pm.system_id AND rs.slug = 'verse'
         JOIN toc_nodes tn ON tn.id = s.node_id
         WHERE s.book_id = $1
         ORDER BY tn.sort_order, s.position",
    )
    .bind(book_id)
    .fetch_all(&mut **tx)
    .await?;
    let mut existing_by_verse: HashMap<(String, String), Vec<Existing>> = HashMap::new();
    for (chapter_ref, verse_ref, id, text) in sent_rows {
        existing_by_verse
            .entry((chapter_ref, verse_ref))
            .or_default()
            .push(Existing { id, text });
    }

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

            let node_id = *node_by_chapter
                .get(&source_ref)
                .ok_or_else(|| format!("chapter node {source_ref} missing; use `pnpm db:reset`"))?;
            let block_id = *block_by_chapter.get(&source_ref).ok_or_else(|| {
                format!("chapter block {source_ref} missing; use `pnpm db:reset`")
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
                node_id,
                block_id,
                verses,
            });
        }
    }

    // --- Pre-flight: chapter + verse structure must be stable --------------
    let desired_chapters: HashSet<&str> = desired.iter().map(|c| c.source_ref.as_str()).collect();
    let existing_chapters: HashSet<&str> = node_by_chapter.keys().map(|s| s.as_str()).collect();
    if desired_chapters != existing_chapters {
        return Err(
            "chapter set changed; not reconcilable — use `pnpm db:reset` + re-import".into(),
        );
    }
    for chapter in &desired {
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
                "chapter {}: verses added/removed; not reconcilable — use `pnpm db:reset` + re-import",
                chapter.source_ref
            )
            .into());
        }
    }

    // --- Plan each verse (abort on ambiguity) ------------------------------
    let mut plans: HashMap<(String, String), reconcile::BlockPlan> = HashMap::new();
    let mut all_retired: Vec<(Uuid, Option<Uuid>)> = Vec::new();
    for chapter in &desired {
        for verse in &chapter.verses {
            let key = (chapter.source_ref.clone(), verse.ref_value.clone());
            let empty: Vec<Existing> = Vec::new();
            let old = existing_by_verse.get(&key).unwrap_or(&empty);
            let new: Vec<&str> = verse.sentences.iter().map(|(t, _)| t.as_str()).collect();
            let label = format!("{} v{}", chapter.source_ref, verse.verse);
            let plan = plan_block(&label, old, &new)?;
            all_retired.extend(plan.retired.iter().copied());
            plans.insert(key, plan);
        }
    }

    // Pure deletes (no survivor) that still carry user data are unsafe.
    for (retired_id, survivor) in &all_retired {
        if survivor.is_none() && !force && sentence_has_dependents(tx, *retired_id).await? {
            return Err(format!(
                "sentence {retired_id} would be deleted but has quotations/resources anchored to it; \
                 aborting (pass --force to delete anyway, or `pnpm db:reset`)"
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

    // 2. Offset verse sentences (depth-1 chapter nodes) out of the unique-index
    //    space; book-node heading sentences (depth 0) are left untouched.
    sqlx::query(
        "UPDATE sentences
         SET position = position + 10000,
             sentence_number = CASE WHEN sentence_number IS NOT NULL THEN sentence_number + 1000000 END,
             natural_key = NULL
         WHERE book_id = $1
           AND node_id IN (SELECT id FROM toc_nodes WHERE book_id = $1 AND depth = 1)",
    )
    .bind(book_id)
    .execute(&mut **tx)
    .await?;

    // 3. Apply keeps + inserts in document order, reassigning chapter-wide
    //    positions and the book-wide sentence_number (verses only).
    let mut resolved: Vec<(Uuid, String)> = Vec::new(); // (sentence_id, verse ref_value) for marker rebuild
    let mut sentence_number: i32 = 1;
    let mut split_new_ids: Vec<(Uuid, Uuid)> = Vec::new();

    for chapter in &desired {
        let mut block_position: i16 = 0;
        for verse in &chapter.verses {
            let key = (chapter.source_ref.clone(), verse.ref_value.clone());
            let plan = &plans[&key];
            let mut idx_uuid: Vec<Uuid> = Vec::with_capacity(verse.sentences.len());
            for (idx, (text, html)) in verse.sentences.iter().enumerate() {
                let natural_key = format!("{}:{}/s{}", chapter.source_ref, verse.verse, idx);
                let sid = match plan.assignment[idx] {
                    Some(existing_id) => {
                        sqlx::query(
                            "UPDATE sentences
                             SET position = $2, sentence_number = $3, text = $4, html = $5,
                                 natural_key = $6, updated_at = now()
                             WHERE id = $1",
                        )
                        .bind(existing_id)
                        .bind(block_position)
                        .bind(sentence_number)
                        .bind(text)
                        .bind(html)
                        .bind(&natural_key)
                        .execute(&mut **tx)
                        .await?;
                        report.updated += 1;
                        existing_id
                    }
                    None => {
                        let id: Uuid = sqlx::query_scalar(
                            "INSERT INTO sentences (book_id, node_id, block_id, position, sentence_number, text, html, natural_key)
                             VALUES ($1, $2, $3, $4, $5, $6, $7, $8)
                             RETURNING id",
                        )
                        .bind(book_id)
                        .bind(chapter.node_id)
                        .bind(chapter.block_id)
                        .bind(block_position)
                        .bind(sentence_number)
                        .bind(text)
                        .bind(html)
                        .bind(&natural_key)
                        .fetch_one(&mut **tx)
                        .await?;
                        report.inserted += 1;
                        id
                    }
                };
                idx_uuid.push(sid);
                resolved.push((sid, verse.ref_value.clone()));
                sentence_number += 1;
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

    // 6. Rebuild verse page markers (one per sentence) in document order.
    sqlx::query(
        "DELETE FROM page_markers WHERE sentence_id IN (SELECT id FROM sentences WHERE book_id = $1)",
    )
    .bind(book_id)
    .execute(&mut **tx)
    .await?;
    let mut marker_sort_order: i32 = 1;
    for (sid, ref_value) in &resolved {
        sqlx::query(
            "INSERT INTO page_markers (system_id, sentence_id, ref_value, sort_order, char_offset)
             VALUES ($1, $2, $3, $4, NULL)",
        )
        .bind(verse_system_id)
        .bind(sid)
        .bind(ref_value)
        .bind(marker_sort_order)
        .execute(&mut **tx)
        .await?;
        report.markers_rebuilt += 1;
        marker_sort_order += 1;
    }

    // 7. Regenerate each chapter's paragraph block text/html (reader fallback).
    for chapter in &desired {
        let para_text = chapter
            .verses
            .iter()
            .map(|v| v.full_text.clone())
            .collect::<Vec<_>>()
            .join(" ");
        let para_html = format!("<p>{}</p>", html_escape(&para_text));
        sqlx::query("UPDATE content_blocks SET text = $1, html = $2 WHERE id = $3")
            .bind(&para_text)
            .bind(&para_html)
            .bind(chapter.block_id)
            .execute(&mut **tx)
            .await?;
    }

    Ok(report)
}
