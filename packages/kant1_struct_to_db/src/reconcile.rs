//! Reconciling re-import: update an already-imported book in place from the
//! freshly parsed struct, preserving the UUIDs of unchanged sentences (and the
//! quotations / resources / cross-references anchored to them).
//!
//! Identity is anchored to the block: a sentence split/merge leaves the block
//! count untouched and only reshuffles ordinals inside the one affected
//! paragraph. We therefore reconcile per block, aligning old rows against the
//! new struct by text — paragraphs and footnotes alike (a footnote's sentences
//! split/merge just like a paragraph's). Anything we cannot attribute
//! confidently — a changed TOC, an added/removed paragraph, a whole footnote
//! added/removed, or two structural edits piled into one paragraph — aborts the
//! run with guidance to `db:reset`. "For sweeping edits, re-import the whole
//! thing" is the deliberate fallback.

use std::collections::HashMap;

use common::textmatch::{concat_similarity, normalized_eq, similarity};
use kant1_md_to_struct::model::Output;
use sqlx::{Postgres, Transaction};
use uuid::Uuid;

/// Concatenation similarity required to accept a split/merge hypothesis.
const SPLIT_MERGE_MIN_SIM: f64 = 0.90;
/// Below this, a same-count sentence is too different to be a plausible
/// in-place edit — treat as a scrambled block and abort.
const SCRAMBLE_MIN_SIM: f64 = 0.25;

#[derive(Default)]
pub struct ReconcileReport {
    pub updated: u32,
    pub split: u32,
    pub merged: u32,
    pub inserted: u32,
    pub deleted: u32,
    pub deps_repointed: u32,
    pub markers_rebuilt: u32,
    pub footnote_sentences_updated: u32,
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
        eprintln!("  footnote sents upd.: {}", self.footnote_sentences_updated);
    }
}

struct Existing {
    id: Uuid,
    text: String,
}

/// Per-block alignment outcome.
struct BlockPlan {
    /// One entry per desired sentence (in order): `Some(id)` reuses an existing
    /// row, `None` inserts a fresh one.
    assignment: Vec<Option<Uuid>>,
    /// Existing rows to retire: `(retired_id, survivor)`. `Some(survivor)` is a
    /// merge (migrate dependents onto the survivor); `None` is a plain delete.
    retired: Vec<(Uuid, Option<Uuid>)>,
    /// `(first_half_id, desired_index_of_second_half)` for each split — used to
    /// extend single-sentence quotations onto the new second half.
    splits: Vec<(Uuid, usize)>,
}

fn trunc(s: &str) -> String {
    let t: String = s.chars().take(50).collect();
    if s.chars().count() > 50 {
        format!("{t}…")
    } else {
        t
    }
}

/// Align one block's existing rows against the desired sentence texts.
/// Returns an error (caller aborts the whole run) for any ambiguous change.
fn plan_block(label: &str, old: &[Existing], new: &[&str]) -> Result<BlockPlan, String> {
    let m = old.len();
    let n = new.len();
    let mut assignment: Vec<Option<Uuid>> = vec![None; n];
    let mut retired: Vec<(Uuid, Option<Uuid>)> = Vec::new();
    let mut splits: Vec<(Uuid, usize)> = Vec::new();

    // No count change → 1:1 by position. Each pair is an unchanged sentence or
    // an in-place edit; both keep the UUID. A pair too dissimilar to be an edit
    // signals a hidden reorder/replace we won't risk mis-anchoring.
    if m == n {
        for i in 0..n {
            if !normalized_eq(&old[i].text, new[i]) {
                let sim = similarity(&old[i].text, new[i]);
                if sim < SCRAMBLE_MIN_SIM {
                    return Err(format!(
                        "{label}: sentence {} changed too drastically to match safely \
                         (similarity {sim:.2}); use `pnpm db:reset` + re-import",
                        i + 1
                    ));
                }
            }
            assignment[i] = Some(old[i].id);
        }
        return Ok(BlockPlan {
            assignment,
            retired,
            splits,
        });
    }

    // Count changed → anchor exact-equal sentences from both ends; whatever is
    // left in the middle is the single structural edit we must classify.
    let mut p = 0;
    while p < m && p < n && normalized_eq(&old[p].text, new[p]) {
        p += 1;
    }
    let mut s = 0;
    while s < (m - p) && s < (n - p) && normalized_eq(&old[m - 1 - s].text, new[n - 1 - s]) {
        s += 1;
    }
    for i in 0..p {
        assignment[i] = Some(old[i].id);
    }
    for k in 0..s {
        assignment[n - 1 - k] = Some(old[m - 1 - k].id);
    }

    let mid_old = &old[p..m - s];
    let mid_new = &new[p..n - s];

    match (mid_old.len(), mid_new.len()) {
        // split: one old sentence became two
        (1, 2) => {
            let sim = concat_similarity(&mid_old[0].text, &[mid_new[0], mid_new[1]]);
            if sim < SPLIT_MERGE_MIN_SIM {
                return Err(format!(
                    "{label}: ambiguous split of \"{}\" (similarity {sim:.2}); \
                     edit one boundary at a time or `pnpm db:reset`",
                    trunc(&mid_old[0].text)
                ));
            }
            assignment[p] = Some(mid_old[0].id);
            assignment[p + 1] = None;
            splits.push((mid_old[0].id, p + 1));
        }
        // merge: two old sentences became one
        (2, 1) => {
            let sim = concat_similarity(mid_new[0], &[&mid_old[0].text, &mid_old[1].text]);
            if sim < SPLIT_MERGE_MIN_SIM {
                return Err(format!(
                    "{label}: ambiguous merge into \"{}\" (similarity {sim:.2}); \
                     edit one boundary at a time or `pnpm db:reset`",
                    trunc(mid_new[0])
                ));
            }
            assignment[p] = Some(mid_old[0].id);
            retired.push((mid_old[1].id, Some(mid_old[0].id)));
        }
        // a brand-new sentence inserted at a boundary
        (0, 1) => {
            assignment[p] = None;
        }
        // a sentence removed at a boundary
        (1, 0) => {
            retired.push((mid_old[0].id, None));
        }
        (mo, mn) => {
            return Err(format!(
                "{label}: ambiguous change ({mo} old vs {mn} new sentences in the edited region); \
                 edit one boundary at a time or `pnpm db:reset`"
            ));
        }
    }

    Ok(BlockPlan {
        assignment,
        retired,
        splits,
    })
}

type SourceSentenceMap = HashMap<(String, i16, i16), Uuid>;
type SourceFnSentenceMap = HashMap<(i32, i16), Uuid>;

#[allow(clippy::too_many_arguments)]
pub async fn reconcile(
    tx: &mut Transaction<'_, Postgres>,
    book_id: Uuid,
    output: &Output,
    is_translation: bool,
    source_sentence_map: &SourceSentenceMap,
    source_fn_sentence_map: &SourceFnSentenceMap,
    system_ids: &HashMap<String, Uuid>,
    force: bool,
) -> Result<ReconcileReport, Box<dyn std::error::Error>> {
    let mut report = ReconcileReport::default();

    // --- Load existing structure -------------------------------------------
    let node_rows: Vec<(Uuid, String)> =
        sqlx::query_as("SELECT id, source_ref FROM toc_nodes WHERE book_id = $1")
            .bind(book_id)
            .fetch_all(&mut **tx)
            .await?;
    let node_id_by_ref: HashMap<String, Uuid> = node_rows
        .iter()
        .map(|(id, sref)| (sref.clone(), *id))
        .collect();

    let block_rows: Vec<(Uuid, String, i16)> = sqlx::query_as(
        "SELECT cb.id, tn.source_ref, cb.position
         FROM content_blocks cb JOIN toc_nodes tn ON cb.node_id = tn.id
         WHERE cb.book_id = $1",
    )
    .bind(book_id)
    .fetch_all(&mut **tx)
    .await?;
    let block_id_by_pos: HashMap<(String, i16), Uuid> = block_rows
        .iter()
        .map(|(id, sref, pos)| ((sref.clone(), *pos), *id))
        .collect();

    // block sentences grouped by (source_ref, block_position), ordered by position
    let sent_rows: Vec<(Uuid, String, i16, i16, String)> = sqlx::query_as(
        "SELECT s.id, tn.source_ref, cb.position, s.position, s.text
         FROM sentences s
         JOIN content_blocks cb ON s.block_id = cb.id
         JOIN toc_nodes tn ON cb.node_id = tn.id
         WHERE s.book_id = $1
         ORDER BY tn.source_ref, cb.position, s.position",
    )
    .bind(book_id)
    .fetch_all(&mut **tx)
    .await?;
    let mut existing_by_block: HashMap<(String, i16), Vec<Existing>> = HashMap::new();
    for (id, sref, block_pos, _spos, text) in sent_rows {
        existing_by_block
            .entry((sref, block_pos))
            .or_default()
            .push(Existing { id, text });
    }

    // existing footnote sentences grouped by footnote number, ordered by position
    let fn_sent_rows: Vec<(Uuid, i32, String)> = sqlx::query_as(
        "SELECT s.id, f.number, s.text
         FROM sentences s JOIN footnotes f ON s.footnote_id = f.id
         WHERE s.book_id = $1
         ORDER BY f.number, s.position",
    )
    .bind(book_id)
    .fetch_all(&mut **tx)
    .await?;
    let mut existing_fn_by_number: HashMap<i32, Vec<Existing>> = HashMap::new();
    for (id, number, text) in fn_sent_rows {
        existing_fn_by_number
            .entry(number)
            .or_default()
            .push(Existing { id, text });
    }

    // footnote row id by number (footnotes themselves are stable; only their
    // sentences may split/merge)
    let footnote_rows: Vec<(i32, Uuid)> =
        sqlx::query_as("SELECT number, id FROM footnotes WHERE book_id = $1")
            .bind(book_id)
            .fetch_all(&mut **tx)
            .await?;
    let footnote_id_by_number: HashMap<i32, Uuid> = footnote_rows.iter().copied().collect();

    // --- Pre-flight: structure must be stable ------------------------------
    let desired_node_refs: std::collections::HashSet<&str> = output
        .toc_nodes
        .iter()
        .map(|n| n.source_ref.as_str())
        .collect();
    let existing_node_refs: std::collections::HashSet<&str> =
        node_id_by_ref.keys().map(|s| s.as_str()).collect();
    if desired_node_refs != existing_node_refs {
        return Err(
            "TOC changed (nodes added/removed); not reconcilable — use `pnpm db:reset` + re-import"
                .into(),
        );
    }

    for node in &output.toc_nodes {
        let desired_block_pos: std::collections::HashSet<i16> =
            node.content_blocks.iter().map(|b| b.position).collect();
        let existing_block_pos: std::collections::HashSet<i16> = block_id_by_pos
            .keys()
            .filter(|(sref, _)| sref == &node.source_ref)
            .map(|(_, pos)| *pos)
            .collect();
        if desired_block_pos != existing_block_pos {
            return Err(format!(
                "node {}: paragraphs added/removed; not reconcilable — use `pnpm db:reset` + re-import",
                node.source_ref
            )
            .into());
        }
    }

    // The set of footnote numbers must be stable — adding/removing a whole
    // footnote renumbers the rest globally (abort-to-reset). Sentence counts
    // *within* a footnote may change: those splits/merges are reconciled below.
    let desired_fn_numbers: std::collections::HashSet<i32> = output
        .toc_nodes
        .iter()
        .flat_map(|n| &n.content_blocks)
        .flat_map(|b| &b.sentences)
        .flat_map(|s| &s.footnotes)
        .map(|f| f.number)
        .collect();
    let existing_fn_numbers: std::collections::HashSet<i32> =
        footnote_id_by_number.keys().copied().collect();
    if desired_fn_numbers != existing_fn_numbers {
        return Err(
            "footnotes added/removed; not reconcilable — use `pnpm db:reset` + re-import".into(),
        );
    }

    // --- Plan every block (abort on ambiguity) -----------------------------
    struct BlockWork {
        node_idx: usize,
        block_idx: usize,
        node_id: Uuid,
        block_id: Uuid,
        plan: BlockPlan,
    }
    let mut works: Vec<BlockWork> = Vec::new();
    let mut all_retired: Vec<(Uuid, Option<Uuid>)> = Vec::new();

    for (node_idx, node) in output.toc_nodes.iter().enumerate() {
        let node_id = node_id_by_ref[&node.source_ref];
        for (block_idx, block) in node.content_blocks.iter().enumerate() {
            let key = (node.source_ref.clone(), block.position);
            let block_id = block_id_by_pos[&key];
            let empty: Vec<Existing> = Vec::new();
            let old = existing_by_block.get(&key).unwrap_or(&empty);
            let new: Vec<&str> = block.sentences.iter().map(|s| s.text.as_str()).collect();
            let label = format!("node {} / block {}", node.source_ref, block.position);
            let plan = plan_block(&label, old, &new)?;
            all_retired.extend(plan.retired.iter().copied());
            works.push(BlockWork {
                node_idx,
                block_idx,
                node_id,
                block_id,
                plan,
            });
        }
    }

    // Plan each footnote's sentences with the same aligner (footnote number is
    // stable; its sentences may split/merge just like a paragraph's).
    let mut fn_plans: HashMap<i32, BlockPlan> = HashMap::new();
    for node in &output.toc_nodes {
        for block in &node.content_blocks {
            for sent in &block.sentences {
                for footnote in &sent.footnotes {
                    let empty: Vec<Existing> = Vec::new();
                    let old = existing_fn_by_number
                        .get(&footnote.number)
                        .unwrap_or(&empty);
                    let new: Vec<&str> =
                        footnote.sentences.iter().map(|s| s.text.as_str()).collect();
                    let label = format!("footnote {}", footnote.number);
                    let plan = plan_block(&label, old, &new)?;
                    all_retired.extend(plan.retired.iter().copied());
                    fn_plans.insert(footnote.number, plan);
                }
            }
        }
    }

    // Pure deletes (no survivor) that still carry user data are unsafe.
    for (retired_id, survivor) in &all_retired {
        if survivor.is_none() && !force && sentence_has_dependents(tx, *retired_id).await? {
            return Err(format!(
                "sentence {retired_id} would be deleted but has quotations/resources/cross-references \
                 anchored to it; aborting (pass --force to delete anyway, or `pnpm db:reset`)"
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

    // 2. Shove all block sentences out of the unique-index space so we can
    //    freely reassign positions / sentence_numbers / natural_keys. Partial
    //    unique indexes can't be deferred; a constant offset can't collide.
    sqlx::query(
        "UPDATE sentences
         SET position = position + 10000,
             sentence_number = CASE WHEN sentence_number IS NOT NULL THEN sentence_number + 1000000 END,
             natural_key = NULL
         WHERE book_id = $1 AND block_id IS NOT NULL",
    )
    .bind(book_id)
    .execute(&mut **tx)
    .await?;
    // ...and footnote sentences, under their own partial indexes.
    sqlx::query(
        "UPDATE sentences
         SET position = position + 10000,
             sentence_number = CASE WHEN sentence_number IS NOT NULL THEN sentence_number + 1000000 END,
             natural_key = NULL
         WHERE book_id = $1 AND footnote_id IS NOT NULL",
    )
    .bind(book_id)
    .execute(&mut **tx)
    .await?;

    // 3. Apply keeps + inserts; build desired→uuid map for markers/footnotes.
    let mut resolved: HashMap<(String, i16, i16), Uuid> = HashMap::new();
    // new uuid per split second-half, keyed by (block work index, desired index)
    let mut split_new_ids: Vec<(Uuid, Uuid)> = Vec::new(); // (first_half_id, new_second_half_id)

    for work in &works {
        let node = &output.toc_nodes[work.node_idx];
        let block = &node.content_blocks[work.block_idx];
        for (i, sent) in block.sentences.iter().enumerate() {
            let source_start = if is_translation {
                source_sentence_map
                    .get(&(node.source_ref.clone(), block.position, sent.position))
                    .copied()
            } else {
                None
            };
            let natural_key = format!("{}/b{}/s{}", node.source_ref, block.position, sent.position);

            let sentence_id = match work.plan.assignment[i] {
                Some(existing_id) => {
                    sqlx::query(
                        "UPDATE sentences
                         SET position = $2, sentence_number = $3, segment = $4,
                             source_sentence_start_id = $5, source_sentence_end_id = NULL,
                             text = $6, html = $7, original_text = $8, original_html = $9,
                             natural_key = $10, updated_at = now()
                         WHERE id = $1",
                    )
                    .bind(existing_id)
                    .bind(sent.position)
                    .bind(sent.sentence_number)
                    .bind(sent.segment)
                    .bind(source_start)
                    .bind(&sent.text)
                    .bind(&sent.html)
                    .bind(&sent.original_text)
                    .bind(&sent.original_html)
                    .bind(&natural_key)
                    .execute(&mut **tx)
                    .await?;
                    report.updated += 1;
                    existing_id
                }
                None => {
                    let id: Uuid = sqlx::query_scalar(
                        "INSERT INTO sentences (book_id, node_id, block_id, position, sentence_number, segment, source_sentence_start_id, text, html, original_text, original_html, natural_key)
                         VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9, $10, $11, $12)
                         RETURNING id",
                    )
                    .bind(book_id)
                    .bind(work.node_id)
                    .bind(work.block_id)
                    .bind(sent.position)
                    .bind(sent.sentence_number)
                    .bind(sent.segment)
                    .bind(source_start)
                    .bind(&sent.text)
                    .bind(&sent.html)
                    .bind(&sent.original_text)
                    .bind(&sent.original_html)
                    .bind(&natural_key)
                    .fetch_one(&mut **tx)
                    .await?;
                    report.inserted += 1;
                    id
                }
            };
            resolved.insert(
                (node.source_ref.clone(), block.position, sent.position),
                sentence_id,
            );
        }
        // record split second-half uuids for quotation extension
        for (first_half_id, second_idx) in &work.plan.splits {
            let block_ref = &output.toc_nodes[work.node_idx].content_blocks[work.block_idx];
            let sec = &block_ref.sentences[*second_idx];
            let new_id = resolved[&(node.source_ref.clone(), block_ref.position, sec.position)];
            split_new_ids.push((*first_half_id, new_id));
            report.split += 1;
        }
    }

    // 3b. Apply footnote sentences (same aligner outcome). The footnote's anchor
    //     is repointed onto its (possibly renumbered) block sentence, and the
    //     global footnote sentence_number is recomputed in document order.
    let mut fn_split_new_ids: Vec<(Uuid, Uuid)> = Vec::new();
    let mut fn_sentence_number = 1i32;
    for node in &output.toc_nodes {
        let node_id = node_id_by_ref[&node.source_ref];
        for block in &node.content_blocks {
            for sent in &block.sentences {
                let anchor_id = resolved[&(node.source_ref.clone(), block.position, sent.position)];
                for footnote in &sent.footnotes {
                    sqlx::query("UPDATE footnotes SET anchor_sentence_id = $2 WHERE book_id = $1 AND number = $3")
                        .bind(book_id)
                        .bind(anchor_id)
                        .bind(footnote.number)
                        .execute(&mut **tx)
                        .await?;
                    let footnote_id = footnote_id_by_number[&footnote.number];
                    let plan = &fn_plans[&footnote.number];
                    let mut idx_uuid: Vec<Uuid> = Vec::with_capacity(footnote.sentences.len());
                    for (i, fn_sent) in footnote.sentences.iter().enumerate() {
                        let natural_key = format!(
                            "{}/fn{}/s{}",
                            node.source_ref, footnote.number, fn_sent.position
                        );
                        let source_start = if is_translation {
                            source_fn_sentence_map
                                .get(&(footnote.number, fn_sent.position))
                                .copied()
                        } else {
                            None
                        };
                        let sid = match plan.assignment[i] {
                            Some(existing_id) => {
                                sqlx::query(
                                    "UPDATE sentences
                                     SET position = $2, sentence_number = $3,
                                         source_sentence_start_id = $4, source_sentence_end_id = NULL,
                                         text = $5, html = $6, original_text = $7, original_html = $8,
                                         natural_key = $9, updated_at = now()
                                     WHERE id = $1",
                                )
                                .bind(existing_id)
                                .bind(fn_sent.position)
                                .bind(fn_sentence_number)
                                .bind(source_start)
                                .bind(&fn_sent.text)
                                .bind(&fn_sent.html)
                                .bind(&fn_sent.original_text)
                                .bind(&fn_sent.original_html)
                                .bind(&natural_key)
                                .execute(&mut **tx)
                                .await?;
                                report.footnote_sentences_updated += 1;
                                existing_id
                            }
                            None => {
                                let id: Uuid = sqlx::query_scalar(
                                    "INSERT INTO sentences (book_id, node_id, footnote_id, position, sentence_number, source_sentence_start_id, text, html, original_text, original_html, natural_key)
                                     VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9, $10, $11)
                                     RETURNING id",
                                )
                                .bind(book_id)
                                .bind(node_id)
                                .bind(footnote_id)
                                .bind(fn_sent.position)
                                .bind(fn_sentence_number)
                                .bind(source_start)
                                .bind(&fn_sent.text)
                                .bind(&fn_sent.html)
                                .bind(&fn_sent.original_text)
                                .bind(&fn_sent.original_html)
                                .bind(&natural_key)
                                .fetch_one(&mut **tx)
                                .await?;
                                report.inserted += 1;
                                id
                            }
                        };
                        idx_uuid.push(sid);
                        fn_sentence_number += 1;
                    }
                    for (first_half_id, second_idx) in &plan.splits {
                        fn_split_new_ids.push((*first_half_id, idx_uuid[*second_idx]));
                        report.split += 1;
                    }
                }
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

    // 5. Extend single-sentence anchors over each split's new second half
    //    (block and footnote splits alike).
    for (first_half_id, new_id) in split_new_ids.iter().chain(fn_split_new_ids.iter()) {
        report.deps_repointed += extend_anchors_to(tx, *first_half_id, *new_id).await?;
    }

    // 6. Rebuild page markers from the desired struct.
    sqlx::query(
        "DELETE FROM page_markers WHERE sentence_id IN (SELECT id FROM sentences WHERE book_id = $1)",
    )
    .bind(book_id)
    .execute(&mut **tx)
    .await?;
    for node in &output.toc_nodes {
        for block in &node.content_blocks {
            for sent in &block.sentences {
                let sid = resolved[&(node.source_ref.clone(), block.position, sent.position)];
                for pm in &sent.page_markers {
                    let system_id = system_ids
                        .get(&pm.system)
                        .ok_or_else(|| format!("Unknown reference system: {}", pm.system))?;
                    sqlx::query(
                        "INSERT INTO page_markers (system_id, sentence_id, ref_value, sort_order, char_offset)
                         VALUES ($1, $2, $3, $4, $5)",
                    )
                    .bind(system_id)
                    .bind(sid)
                    .bind(&pm.ref_value)
                    .bind(pm.sort_order)
                    .bind(pm.char_offset)
                    .execute(&mut **tx)
                    .await?;
                    report.markers_rebuilt += 1;
                }
            }
        }
    }

    // 7. Update block + node text in place (paragraph/heading content edits).
    for node in &output.toc_nodes {
        let node_id = node_id_by_ref[&node.source_ref];
        let label_html = if node.label_html != node.label {
            Some(&node.label_html)
        } else {
            None
        };
        sqlx::query("UPDATE toc_nodes SET label = $2, label_html = $3 WHERE id = $1")
            .bind(node_id)
            .bind(&node.label)
            .bind(label_html)
            .execute(&mut **tx)
            .await?;
        for block in &node.content_blocks {
            let block_id = block_id_by_pos[&(node.source_ref.clone(), block.position)];
            sqlx::query(
                "UPDATE content_blocks
                 SET text = $2, html = $3, original_text = $4, original_html = $5, updated_at = now()
                 WHERE id = $1",
            )
            .bind(block_id)
            .bind(&block.text)
            .bind(&block.html)
            .bind(&block.original_text)
            .bind(&block.original_html)
            .execute(&mut **tx)
            .await?;
        }
    }

    Ok(report)
}

/// Does this sentence have user/editor data (or a footnote anchor) hanging off it?
async fn sentence_has_dependents(
    tx: &mut Transaction<'_, Postgres>,
    id: Uuid,
) -> Result<bool, Box<dyn std::error::Error>> {
    let count: i64 = sqlx::query_scalar(
        "SELECT
           (SELECT count(*) FROM quotations WHERE anchor_sentence_start_id = $1 OR anchor_sentence_end_id = $1)
         + (SELECT count(*) FROM resources  WHERE anchor_sentence_start_id = $1 OR anchor_sentence_end_id = $1)
         + (SELECT count(*) FROM cross_references WHERE source_sentence_start_id = $1 OR source_sentence_end_id = $1 OR target_sentence_start_id = $1 OR target_sentence_end_id = $1)
         + (SELECT count(*) FROM footnotes WHERE anchor_sentence_id = $1)",
    )
    .bind(id)
    .fetch_one(&mut **tx)
    .await?;
    Ok(count > 0)
}

/// Repoint every reference from `from` onto `to` (merge survivor). Returns the
/// number of rows moved.
async fn migrate_dependents(
    tx: &mut Transaction<'_, Postgres>,
    from: Uuid,
    to: Uuid,
) -> Result<u32, Box<dyn std::error::Error>> {
    let mut moved = 0u32;
    let stmts = [
        "UPDATE quotations SET anchor_sentence_start_id = $2 WHERE anchor_sentence_start_id = $1",
        "UPDATE quotations SET anchor_sentence_end_id = $2 WHERE anchor_sentence_end_id = $1",
        "UPDATE resources SET anchor_sentence_start_id = $2 WHERE anchor_sentence_start_id = $1",
        "UPDATE resources SET anchor_sentence_end_id = $2 WHERE anchor_sentence_end_id = $1",
        "UPDATE cross_references SET source_sentence_start_id = $2 WHERE source_sentence_start_id = $1",
        "UPDATE cross_references SET source_sentence_end_id = $2 WHERE source_sentence_end_id = $1",
        "UPDATE cross_references SET target_sentence_start_id = $2 WHERE target_sentence_start_id = $1",
        "UPDATE cross_references SET target_sentence_end_id = $2 WHERE target_sentence_end_id = $1",
        "UPDATE footnotes SET anchor_sentence_id = $2 WHERE anchor_sentence_id = $1",
        // translation links from the peer book
        "UPDATE sentences SET source_sentence_start_id = $2 WHERE source_sentence_start_id = $1",
        "UPDATE sentences SET source_sentence_end_id = $2 WHERE source_sentence_end_id = $1",
    ];
    for sql in stmts {
        let res = sqlx::query(sql)
            .bind(from)
            .bind(to)
            .execute(&mut **tx)
            .await?;
        moved += res.rows_affected() as u32;
    }
    Ok(moved)
}

/// After a split, a single-sentence anchor on the first half (`end IS NULL`)
/// should grow to cover the new second half so the quoted text is preserved.
async fn extend_anchors_to(
    tx: &mut Transaction<'_, Postgres>,
    first_half: Uuid,
    second_half: Uuid,
) -> Result<u32, Box<dyn std::error::Error>> {
    let mut moved = 0u32;
    let stmts = [
        "UPDATE quotations SET anchor_sentence_end_id = $2 WHERE anchor_sentence_start_id = $1 AND anchor_sentence_end_id IS NULL",
        "UPDATE resources SET anchor_sentence_end_id = $2 WHERE anchor_sentence_start_id = $1 AND anchor_sentence_end_id IS NULL",
        "UPDATE cross_references SET source_sentence_end_id = $2 WHERE source_sentence_start_id = $1 AND source_sentence_end_id IS NULL",
        "UPDATE cross_references SET target_sentence_end_id = $2 WHERE target_sentence_start_id = $1 AND target_sentence_end_id IS NULL",
    ];
    for sql in stmts {
        let res = sqlx::query(sql)
            .bind(first_half)
            .bind(second_half)
            .execute(&mut **tx)
            .await?;
        moved += res.rows_affected() as u32;
    }
    Ok(moved)
}

#[cfg(test)]
mod tests {
    use super::*;

    fn ex(id: u128, text: &str) -> Existing {
        Existing {
            id: Uuid::from_u128(id),
            text: text.to_string(),
        }
    }

    #[test]
    fn unchanged_block_keeps_all_uuids() {
        let old = vec![ex(1, "A sentence."), ex(2, "Another one.")];
        let plan = plan_block("t", &old, &["A sentence.", "Another one."]).unwrap();
        assert_eq!(
            plan.assignment,
            vec![Some(Uuid::from_u128(1)), Some(Uuid::from_u128(2))]
        );
        assert!(plan.retired.is_empty());
    }

    #[test]
    fn edited_sentence_keeps_uuid() {
        let old = vec![ex(1, "The cat sat on the mat."), ex(2, "End.")];
        let plan = plan_block("t", &old, &["The cat sat on the rug.", "End."]).unwrap();
        assert_eq!(plan.assignment[0], Some(Uuid::from_u128(1)));
        assert_eq!(plan.assignment[1], Some(Uuid::from_u128(2)));
    }

    #[test]
    fn split_keeps_first_uuid_inserts_second() {
        let old = vec![
            ex(1, "All bodies are extended, and they are heavy."),
            ex(2, "Done."),
        ];
        let plan = plan_block(
            "t",
            &old,
            &["All bodies are extended,", "and they are heavy.", "Done."],
        )
        .unwrap();
        assert_eq!(plan.assignment[0], Some(Uuid::from_u128(1)));
        assert_eq!(plan.assignment[1], None);
        assert_eq!(plan.assignment[2], Some(Uuid::from_u128(2)));
        assert_eq!(plan.splits, vec![(Uuid::from_u128(1), 1)]);
    }

    #[test]
    fn merge_retires_second_onto_first() {
        let old = vec![
            ex(1, "All bodies are extended,"),
            ex(2, "and they are heavy."),
            ex(3, "Done."),
        ];
        let plan = plan_block(
            "t",
            &old,
            &["All bodies are extended, and they are heavy.", "Done."],
        )
        .unwrap();
        assert_eq!(plan.assignment[0], Some(Uuid::from_u128(1)));
        assert_eq!(plan.assignment[1], Some(Uuid::from_u128(3)));
        assert_eq!(
            plan.retired,
            vec![(Uuid::from_u128(2), Some(Uuid::from_u128(1)))]
        );
    }

    #[test]
    fn ambiguous_double_change_aborts() {
        let old = vec![ex(1, "One."), ex(2, "Two."), ex(3, "Three.")];
        // two new sentences in the middle that don't cleanly concat from one old
        let err = plan_block(
            "t",
            &old,
            &["One.", "Totally different.", "New thing.", "Three."],
        );
        assert!(err.is_err());
    }

    #[test]
    fn insert_at_boundary() {
        let old = vec![ex(1, "One."), ex(2, "Two.")];
        let plan = plan_block("t", &old, &["One.", "Inserted.", "Two."]).unwrap();
        assert_eq!(plan.assignment[0], Some(Uuid::from_u128(1)));
        assert_eq!(plan.assignment[1], None);
        assert_eq!(plan.assignment[2], Some(Uuid::from_u128(2)));
    }

    #[test]
    fn delete_at_boundary() {
        let old = vec![ex(1, "One."), ex(2, "Two."), ex(3, "Three.")];
        let plan = plan_block("t", &old, &["One.", "Three."]).unwrap();
        assert_eq!(plan.retired, vec![(Uuid::from_u128(2), None)]);
    }
}
