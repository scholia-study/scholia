//! Reconciling re-import: update an already-imported book in place from the
//! freshly parsed struct, preserving the UUIDs of unchanged sentences (and the
//! quotations / resources / cross-references anchored to them).
//!
//! Identity is anchored to the block: a split/merge only reshuffles ordinals
//! inside the one affected paragraph, so we reconcile per block, aligning old
//! rows against the new struct by text — paragraphs and footnotes alike. A change
//! we cannot attribute confidently (a changed TOC, an added/removed paragraph or
//! footnote, or two structural edits in one paragraph) aborts to `db:reset`.
//!
//! Reconcile is incremental via content hashes: each node stores a hash and the
//! book stores the root. An unchanged root short-circuits the run in one query;
//! only nodes whose hash differs are loaded and applied. Within a changed node,
//! a same-count block writes just the sentences whose content differs, while a
//! block whose sentence count changed is re-laid-out and triggers a set-based
//! global `sentence_number` renumber. `--full-rewrite` bypasses the hash checks
//! and rewrites everything. See docs/architecture/reconcile-incremental-hashing.md.
//!
//! The book-agnostic alignment + dependent-migration logic lives in the shared
//! `reconcile` crate; this module is the Kant-specific orchestration on top.

use std::collections::{HashMap, HashSet};

use kant1_md_to_struct::model::{
    ContentBlockData, FootnoteData, Output, SentenceData, TocNodeData,
};
use reconcile::{
    BlockContent, BlockPlan, Existing, FootnoteContent, MarkerContent, NodeContent,
    SentenceContent, extend_anchors_to, migrate_dependents, node_hash, plan_block, root_hash,
    sentence_has_dependents,
};
use sqlx::{Postgres, Transaction};
use uuid::Uuid;

// Temp `sentence_number` base for rows inserted during apply. Existing rows keep
// their real numbers until the set-based renumber reassigns everything, so a
// fresh insert needs a non-null, unique placeholder that is out of the dense
// [1, N] range (any real book is far smaller than this). The renumber then
// rewrites these to their final dense values.
const TEMP_SENTENCE_NUMBER_BASE: i32 = 8_000_000;

// --- Content hashing (tier-2 incremental reconcile) ------------------------
// Build the book-agnostic `reconcile::NodeContent` from the Kant struct so the
// insert path and the reconcile path hash *identical* content. The field set
// here must mirror what reconcile writes (text/html/original_*/segment, page
// markers, footnote content, block + label fields) — never the recomputed
// numbering/positional fields. See docs/architecture/reconcile-incremental-hashing.md.

pub(crate) fn node_content(node: &TocNodeData) -> NodeContent<'_> {
    NodeContent {
        label: &node.label,
        label_html: &node.label_html,
        blocks: node.content_blocks.iter().map(block_content).collect(),
    }
}

fn block_content(block: &ContentBlockData) -> BlockContent<'_> {
    BlockContent {
        block_type: &block.block_type,
        text: &block.text,
        html: &block.html,
        original_text: block.original_text.as_deref(),
        original_html: block.original_html.as_deref(),
        sentences: block.sentences.iter().map(sentence_content).collect(),
    }
}

fn sentence_content(s: &SentenceData) -> SentenceContent<'_> {
    SentenceContent {
        text: &s.text,
        html: &s.html,
        original_text: s.original_text.as_deref(),
        original_html: s.original_html.as_deref(),
        segment: s.segment,
        markers: s
            .page_markers
            .iter()
            .map(|m| MarkerContent {
                system: &m.system,
                ref_value: &m.ref_value,
                char_offset: Some(m.char_offset),
            })
            .collect(),
        footnotes: s.footnotes.iter().map(footnote_content).collect(),
    }
}

fn footnote_content(f: &FootnoteData) -> FootnoteContent<'_> {
    FootnoteContent {
        number: f.number,
        sentences: f
            .sentences
            .iter()
            .map(|fs| SentenceContent {
                text: &fs.text,
                html: &fs.html,
                original_text: fs.original_text.as_deref(),
                original_html: fs.original_html.as_deref(),
                segment: None,
                markers: Vec::new(),
                footnotes: Vec::new(),
            })
            .collect(),
    }
}

/// Per-node hashes in document (sort) order, paired with `source_ref`, plus the
/// root hash. Both the insert and reconcile paths derive their stored hashes
/// from here.
pub(crate) fn compute_hashes(output: &Output) -> (Vec<(String, String)>, String) {
    let node_hashes: Vec<(String, String)> = output
        .toc_nodes
        .iter()
        .map(|n| (n.source_ref.clone(), node_hash(&node_content(n))))
        .collect();
    let root = root_hash(
        &node_hashes
            .iter()
            .map(|(_, h)| h.clone())
            .collect::<Vec<_>>(),
    );
    (node_hashes, root)
}

#[derive(Default)]
pub struct ReconcileReport {
    pub unchanged: bool,
    pub nodes_changed: u32,
    pub updated: u32,
    pub split: u32,
    pub merged: u32,
    pub inserted: u32,
    pub deleted: u32,
    pub deps_repointed: u32,
    pub markers_rebuilt: u32,
    pub footnote_sentences_updated: u32,
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
        eprintln!("  nodes changed:       {}", self.nodes_changed);
        eprintln!("  sentences updated:   {}", self.updated);
        eprintln!("  splits:              {}", self.split);
        eprintln!("  merges:              {}", self.merged);
        eprintln!("  sentences inserted:  {}", self.inserted);
        eprintln!("  sentences deleted:   {}", self.deleted);
        eprintln!("  dependents repointed:{}", self.deps_repointed);
        eprintln!("  page markers rebuilt:{}", self.markers_rebuilt);
        eprintln!("  footnote sents upd.: {}", self.footnote_sentences_updated);
        eprintln!(
            "  global renumber:     {}",
            if self.renumbered { "yes" } else { "skipped" }
        );
    }
}

type SourceSentenceMap = HashMap<(String, i16, i16), Uuid>;
type SourceFnSentenceMap = HashMap<(i32, i16), Uuid>;

// Loaded existing block / footnote sentence rows (changed nodes only).
type BlockSentRow = (
    Uuid,
    String,
    i16,
    i16,
    String,
    String,
    Option<String>,
    Option<String>,
    Option<i16>,
);
type FnSentRow = (Uuid, i32, String, String, Option<String>, Option<String>);

/// Existing on-disk content of one sentence, loaded for changed nodes so a
/// same-count block can skip rewriting byte-identical rows (selective writes).
struct ExistingSentence {
    text: String,
    html: String,
    original_text: Option<String>,
    original_html: Option<String>,
    segment: Option<i16>,
}

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
    full_rewrite: bool,
) -> Result<ReconcileReport, Box<dyn std::error::Error>> {
    let mut report = ReconcileReport::default();

    // --- Desired hashes + root short-circuit -------------------------------
    let (desired_node_hashes, desired_root) = compute_hashes(output);
    let desired_hash_by_ref: HashMap<&str, &str> = desired_node_hashes
        .iter()
        .map(|(r, h)| (r.as_str(), h.as_str()))
        .collect();

    let stored_root: Option<String> =
        sqlx::query_scalar("SELECT content_hash FROM books WHERE id = $1")
            .bind(book_id)
            .fetch_one(&mut **tx)
            .await?;
    if !full_rewrite && stored_root.as_deref() == Some(desired_root.as_str()) {
        report.unchanged = true;
        return Ok(report);
    }

    // --- Load existing structure (cheap: no sentence text) -----------------
    let node_rows: Vec<(Uuid, String, Option<String>)> =
        sqlx::query_as("SELECT id, source_ref, content_hash FROM toc_nodes WHERE book_id = $1")
            .bind(book_id)
            .fetch_all(&mut **tx)
            .await?;
    let node_id_by_ref: HashMap<String, Uuid> = node_rows
        .iter()
        .map(|(id, sref, _)| (sref.clone(), *id))
        .collect();
    let stored_node_hash: HashMap<&str, Option<&str>> = node_rows
        .iter()
        .map(|(_, sref, h)| (sref.as_str(), h.as_deref()))
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

    let footnote_rows: Vec<(i32, Uuid)> =
        sqlx::query_as("SELECT number, id FROM footnotes WHERE book_id = $1")
            .bind(book_id)
            .fetch_all(&mut **tx)
            .await?;
    let footnote_id_by_number: HashMap<i32, Uuid> = footnote_rows.iter().copied().collect();

    // --- Pre-flight: structure must be stable ------------------------------
    let desired_node_refs: HashSet<&str> = output
        .toc_nodes
        .iter()
        .map(|n| n.source_ref.as_str())
        .collect();
    let existing_node_refs: HashSet<&str> = node_id_by_ref.keys().map(|s| s.as_str()).collect();
    if desired_node_refs != existing_node_refs {
        return Err(
            "TOC changed (nodes added/removed); not reconcilable — use `pnpm db:reset` + re-import"
                .into(),
        );
    }

    for node in &output.toc_nodes {
        let desired_block_pos: HashSet<i16> =
            node.content_blocks.iter().map(|b| b.position).collect();
        let existing_block_pos: HashSet<i16> = block_id_by_pos
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
    let desired_fn_numbers: HashSet<i32> = output
        .toc_nodes
        .iter()
        .flat_map(|n| &n.content_blocks)
        .flat_map(|b| &b.sentences)
        .flat_map(|s| &s.footnotes)
        .map(|f| f.number)
        .collect();
    let existing_fn_numbers: HashSet<i32> = footnote_id_by_number.keys().copied().collect();
    if desired_fn_numbers != existing_fn_numbers {
        return Err(
            "footnotes added/removed; not reconcilable — use `pnpm db:reset` + re-import".into(),
        );
    }

    // --- Changed set (NULL stored hash ⇒ changed; `--full-rewrite` ⇒ everything) ----
    let changed_refs: HashSet<String> = output
        .toc_nodes
        .iter()
        .filter(|n| {
            full_rewrite
                || stored_node_hash
                    .get(n.source_ref.as_str())
                    .copied()
                    .flatten()
                    != Some(desired_hash_by_ref[n.source_ref.as_str()])
        })
        .map(|n| n.source_ref.clone())
        .collect();
    report.nodes_changed = changed_refs.len() as u32;

    // Nothing content-changed (e.g. a stale root with intact node hashes): just
    // refresh the stored root and return. The loops below would all no-op.
    if changed_refs.is_empty() {
        sqlx::query("UPDATE books SET content_hash = $2 WHERE id = $1")
            .bind(book_id)
            .bind(&desired_root)
            .execute(&mut **tx)
            .await?;
        return Ok(report);
    }

    let changed_refs_vec: Vec<String> = changed_refs.iter().cloned().collect();
    let changed_node_ids: Vec<Uuid> = changed_refs.iter().map(|r| node_id_by_ref[r]).collect();
    let changed_fn_numbers: Vec<i32> = output
        .toc_nodes
        .iter()
        .filter(|n| changed_refs.contains(&n.source_ref))
        .flat_map(|n| &n.content_blocks)
        .flat_map(|b| &b.sentences)
        .flat_map(|s| &s.footnotes)
        .map(|f| f.number)
        .collect();

    // --- Load existing sentence content, scoped to changed nodes -----------
    let sent_rows: Vec<BlockSentRow> = sqlx::query_as(
        "SELECT s.id, tn.source_ref, cb.position, s.position,
                    s.text, s.html, s.original_text, s.original_html, s.segment
             FROM sentences s
             JOIN content_blocks cb ON s.block_id = cb.id
             JOIN toc_nodes tn ON cb.node_id = tn.id
             WHERE s.book_id = $1 AND tn.source_ref = ANY($2)
             ORDER BY tn.source_ref, cb.position, s.position",
    )
    .bind(book_id)
    .bind(&changed_refs_vec)
    .fetch_all(&mut **tx)
    .await?;
    let mut existing_by_block: HashMap<(String, i16), Vec<Existing>> = HashMap::new();
    let mut existing_content: HashMap<Uuid, ExistingSentence> = HashMap::new();
    for (id, sref, block_pos, _spos, text, html, original_text, original_html, segment) in sent_rows
    {
        existing_by_block
            .entry((sref, block_pos))
            .or_default()
            .push(Existing {
                id,
                text: text.clone(),
            });
        existing_content.insert(
            id,
            ExistingSentence {
                text,
                html,
                original_text,
                original_html,
                segment,
            },
        );
    }

    let fn_sent_rows: Vec<FnSentRow> = sqlx::query_as(
        "SELECT s.id, f.number, s.text, s.html, s.original_text, s.original_html
             FROM sentences s JOIN footnotes f ON s.footnote_id = f.id
             WHERE s.book_id = $1 AND f.number = ANY($2)
             ORDER BY f.number, s.position",
    )
    .bind(book_id)
    .bind(&changed_fn_numbers)
    .fetch_all(&mut **tx)
    .await?;
    let mut existing_fn_by_number: HashMap<i32, Vec<Existing>> = HashMap::new();
    for (id, number, text, html, original_text, original_html) in fn_sent_rows {
        existing_fn_by_number
            .entry(number)
            .or_default()
            .push(Existing {
                id,
                text: text.clone(),
            });
        existing_content.insert(
            id,
            ExistingSentence {
                text,
                html,
                original_text,
                original_html,
                segment: None,
            },
        );
    }

    // --- Plan every changed block (abort on ambiguity) ---------------------
    struct BlockWork {
        node_idx: usize,
        block_idx: usize,
        node_id: Uuid,
        block_id: Uuid,
        plan: BlockPlan,
        // A count change (split/merge/insert/delete) means positions shifted, so
        // the block is offset + fully reassigned; otherwise content is written
        // selectively (only the rows whose text actually differs).
        count_delta: bool,
    }
    let mut works: Vec<BlockWork> = Vec::new();
    let mut all_retired: Vec<(Uuid, Option<Uuid>)> = Vec::new();

    for (node_idx, node) in output.toc_nodes.iter().enumerate() {
        if !changed_refs.contains(&node.source_ref) {
            continue;
        }
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
            let count_delta = full_rewrite || old.len() != new.len();
            works.push(BlockWork {
                node_idx,
                block_idx,
                node_id,
                block_id,
                plan,
                count_delta,
            });
        }
    }

    // Plan each changed footnote's sentences (footnote number is stable; its
    // sentences may split/merge just like a paragraph's).
    struct FnWork {
        number: i32,
        footnote_id: Uuid,
        node_id: Uuid,
        anchor_key: (String, i16, i16),
        // The anchor sentence only moves when its containing block changed count.
        anchor_block_count_delta: bool,
        plan: BlockPlan,
        count_delta: bool,
    }
    let mut fn_works: Vec<FnWork> = Vec::new();
    for work in &works {
        let node = &output.toc_nodes[work.node_idx];
        let block = &node.content_blocks[work.block_idx];
        for sent in &block.sentences {
            for footnote in &sent.footnotes {
                let empty: Vec<Existing> = Vec::new();
                let old = existing_fn_by_number
                    .get(&footnote.number)
                    .unwrap_or(&empty);
                let new: Vec<&str> = footnote.sentences.iter().map(|s| s.text.as_str()).collect();
                let label = format!("footnote {}", footnote.number);
                let plan = plan_block(&label, old, &new)?;
                all_retired.extend(plan.retired.iter().copied());
                let count_delta = full_rewrite || old.len() != new.len();
                fn_works.push(FnWork {
                    number: footnote.number,
                    footnote_id: footnote_id_by_number[&footnote.number],
                    node_id: work.node_id,
                    anchor_key: (node.source_ref.clone(), block.position, sent.position),
                    anchor_block_count_delta: work.count_delta,
                    plan,
                    count_delta,
                });
            }
        }
    }

    let any_count_delta =
        works.iter().any(|w| w.count_delta) || fn_works.iter().any(|w| w.count_delta);

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

    // 2. Shove only the count-changed blocks/footnotes out of the per-unit
    //    position + natural_key index space, so we can freely reassign within
    //    them. `sentence_number` is owned by the set-based renumber (step 8),
    //    not offset here. Same-count units keep their positions/keys untouched.
    let offset_block_ids: Vec<Uuid> = works
        .iter()
        .filter(|w| w.count_delta)
        .map(|w| w.block_id)
        .collect();
    if !offset_block_ids.is_empty() {
        sqlx::query(
            "UPDATE sentences SET position = position + 10000, natural_key = NULL
             WHERE block_id = ANY($1)",
        )
        .bind(&offset_block_ids)
        .execute(&mut **tx)
        .await?;
    }
    let offset_fn_ids: Vec<Uuid> = fn_works
        .iter()
        .filter(|w| w.count_delta)
        .map(|w| w.footnote_id)
        .collect();
    if !offset_fn_ids.is_empty() {
        sqlx::query(
            "UPDATE sentences SET position = position + 10000, natural_key = NULL
             WHERE footnote_id = ANY($1)",
        )
        .bind(&offset_fn_ids)
        .execute(&mut **tx)
        .await?;
    }

    // 3. Apply block sentences. `resolved` maps every changed-node sentence to
    //    its (possibly new) UUID, for marker + footnote-anchor rebuild.
    let mut resolved: HashMap<(String, i16, i16), Uuid> = HashMap::new();
    let mut split_new_ids: Vec<(Uuid, Uuid)> = Vec::new();
    let mut next_block_temp: i32 = TEMP_SENTENCE_NUMBER_BASE;

    for work in &works {
        let node = &output.toc_nodes[work.node_idx];
        let block = &node.content_blocks[work.block_idx];
        for (i, sent) in block.sentences.iter().enumerate() {
            let sentence_id = if work.count_delta {
                let source_start = if is_translation {
                    source_sentence_map
                        .get(&(node.source_ref.clone(), block.position, sent.position))
                        .copied()
                } else {
                    None
                };
                let natural_key =
                    format!("{}/b{}/s{}", node.source_ref, block.position, sent.position);
                match work.plan.assignment[i] {
                    Some(existing_id) => {
                        // Full reassign: positions shifted. `sentence_number` is
                        // left to the global renumber (step 8).
                        sqlx::query(
                            "UPDATE sentences
                             SET position = $2, segment = $3,
                                 source_sentence_start_id = $4, source_sentence_end_id = NULL,
                                 text = $5, html = $6, original_text = $7, original_html = $8,
                                 natural_key = $9, updated_at = now()
                             WHERE id = $1",
                        )
                        .bind(existing_id)
                        .bind(sent.position)
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
                        // A numbered (paragraph) sentence gets a temp number the
                        // renumber will replace; an unnumbered (heading) sentence
                        // stays NULL.
                        let temp_number = sent.sentence_number.map(|_| {
                            let t = next_block_temp;
                            next_block_temp += 1;
                            t
                        });
                        let id: Uuid = sqlx::query_scalar(
                            "INSERT INTO sentences (book_id, node_id, block_id, position, sentence_number, segment, source_sentence_start_id, text, html, original_text, original_html, natural_key)
                             VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9, $10, $11, $12)
                             RETURNING id",
                        )
                        .bind(book_id)
                        .bind(work.node_id)
                        .bind(work.block_id)
                        .bind(sent.position)
                        .bind(temp_number)
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
                }
            } else {
                // Same-count block: positions/keys/number are stable. Write only
                // the rows whose content differs (selective writes). Translation
                // links are stable too — a source edit repoints them via the
                // source book's own reconcile, not here.
                let existing_id = work.plan.assignment[i]
                    .expect("same-count block: every desired sentence maps to an existing row");
                let differs = match existing_content.get(&existing_id) {
                    Some(c) => {
                        c.text != sent.text
                            || c.html != sent.html
                            || c.original_text != sent.original_text
                            || c.original_html != sent.original_html
                            || c.segment != sent.segment
                    }
                    None => true,
                };
                if differs {
                    sqlx::query(
                        "UPDATE sentences
                         SET segment = $2, text = $3, html = $4,
                             original_text = $5, original_html = $6, updated_at = now()
                         WHERE id = $1",
                    )
                    .bind(existing_id)
                    .bind(sent.segment)
                    .bind(&sent.text)
                    .bind(&sent.html)
                    .bind(&sent.original_text)
                    .bind(&sent.original_html)
                    .execute(&mut **tx)
                    .await?;
                    report.updated += 1;
                }
                existing_id
            };
            resolved.insert(
                (node.source_ref.clone(), block.position, sent.position),
                sentence_id,
            );
        }
        for (first_half_id, second_idx) in &work.plan.splits {
            let sec = &block.sentences[*second_idx];
            let new_id = resolved[&(node.source_ref.clone(), block.position, sec.position)];
            split_new_ids.push((*first_half_id, new_id));
            report.split += 1;
        }
    }

    // 3b. Apply footnote sentences. Repoint the anchor only when its block's
    //     count changed (otherwise the anchor UUID is stable). Number reassign
    //     is owned by the footnote renumber (step 8).
    let mut fn_split_new_ids: Vec<(Uuid, Uuid)> = Vec::new();
    let mut next_fn_temp: i32 = TEMP_SENTENCE_NUMBER_BASE;
    for work in &fn_works {
        let anchor_id = resolved[&work.anchor_key];
        if work.anchor_block_count_delta {
            sqlx::query(
                "UPDATE footnotes SET anchor_sentence_id = $2 WHERE book_id = $1 AND number = $3",
            )
            .bind(book_id)
            .bind(anchor_id)
            .bind(work.number)
            .execute(&mut **tx)
            .await?;
        }

        // Re-find the footnote's sentences in the desired struct.
        let node = output
            .toc_nodes
            .iter()
            .find(|n| node_id_by_ref.get(&n.source_ref) == Some(&work.node_id))
            .expect("footnote node present");
        let footnote = node
            .content_blocks
            .iter()
            .flat_map(|b| &b.sentences)
            .flat_map(|s| &s.footnotes)
            .find(|f| f.number == work.number)
            .expect("footnote present in desired struct");

        let mut idx_uuid: Vec<Uuid> = Vec::with_capacity(footnote.sentences.len());
        for (i, fn_sent) in footnote.sentences.iter().enumerate() {
            let sid = if work.count_delta {
                let natural_key = format!(
                    "{}/fn{}/s{}",
                    node.source_ref, work.number, fn_sent.position
                );
                let source_start = if is_translation {
                    source_fn_sentence_map
                        .get(&(work.number, fn_sent.position))
                        .copied()
                } else {
                    None
                };
                match work.plan.assignment[i] {
                    Some(existing_id) => {
                        sqlx::query(
                            "UPDATE sentences
                             SET position = $2,
                                 source_sentence_start_id = $3, source_sentence_end_id = NULL,
                                 text = $4, html = $5, original_text = $6, original_html = $7,
                                 natural_key = $8, updated_at = now()
                             WHERE id = $1",
                        )
                        .bind(existing_id)
                        .bind(fn_sent.position)
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
                        let temp_number = next_fn_temp;
                        next_fn_temp += 1;
                        let id: Uuid = sqlx::query_scalar(
                            "INSERT INTO sentences (book_id, node_id, footnote_id, position, sentence_number, source_sentence_start_id, text, html, original_text, original_html, natural_key)
                             VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9, $10, $11)
                             RETURNING id",
                        )
                        .bind(book_id)
                        .bind(work.node_id)
                        .bind(work.footnote_id)
                        .bind(fn_sent.position)
                        .bind(temp_number)
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
                }
            } else {
                let existing_id = work.plan.assignment[i]
                    .expect("same-count footnote: every desired sentence maps to an existing row");
                let differs = match existing_content.get(&existing_id) {
                    Some(c) => {
                        c.text != fn_sent.text
                            || c.html != fn_sent.html
                            || c.original_text != fn_sent.original_text
                            || c.original_html != fn_sent.original_html
                    }
                    None => true,
                };
                if differs {
                    sqlx::query(
                        "UPDATE sentences
                         SET text = $2, html = $3, original_text = $4, original_html = $5,
                             updated_at = now()
                         WHERE id = $1",
                    )
                    .bind(existing_id)
                    .bind(&fn_sent.text)
                    .bind(&fn_sent.html)
                    .bind(&fn_sent.original_text)
                    .bind(&fn_sent.original_html)
                    .execute(&mut **tx)
                    .await?;
                    report.footnote_sentences_updated += 1;
                }
                existing_id
            };
            idx_uuid.push(sid);
        }
        for (first_half_id, second_idx) in &work.plan.splits {
            fn_split_new_ids.push((*first_half_id, idx_uuid[*second_idx]));
            report.split += 1;
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
    for (first_half_id, new_id) in split_new_ids.iter().chain(fn_split_new_ids.iter()) {
        report.deps_repointed += extend_anchors_to(tx, *first_half_id, *new_id).await?;
    }

    // 6. Rebuild page markers for changed nodes only.
    sqlx::query(
        "DELETE FROM page_markers
         WHERE sentence_id IN (SELECT id FROM sentences WHERE node_id = ANY($1))",
    )
    .bind(&changed_node_ids)
    .execute(&mut **tx)
    .await?;
    for node in &output.toc_nodes {
        if !changed_refs.contains(&node.source_ref) {
            continue;
        }
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

    // 7. Update block + node text for changed nodes (paragraph/heading edits).
    for node in &output.toc_nodes {
        if !changed_refs.contains(&node.source_ref) {
            continue;
        }
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

    // 8. Set-based global renumber — only when some block changed its count.
    //    Two statements per sequence (offset out of [1, N], then assign dense by
    //    document order), regardless of book size. Writes only `sentence_number`,
    //    never `content_hash`, so unchanged downstream nodes keep their stored
    //    hash and stay skippable next run. Block + footnote sequences are
    //    independent (separate partial unique indexes).
    if any_count_delta {
        // Block sentence sequence.
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

        // Footnote sentence sequence — document order of the anchoring block
        // sentence, then footnote number, then position within the footnote.
        sqlx::query(
            "UPDATE sentences SET sentence_number = sentence_number + 1000000
             WHERE book_id = $1 AND sentence_number IS NOT NULL AND footnote_id IS NOT NULL",
        )
        .bind(book_id)
        .execute(&mut **tx)
        .await?;
        sqlx::query(
            "WITH ordered AS (
                 SELECT s.id, ROW_NUMBER() OVER (
                     ORDER BY tn.sort_order, cb.position, a.position, f.number, s.position
                 ) AS rn
                 FROM sentences s
                 JOIN footnotes f ON s.footnote_id = f.id
                 JOIN sentences a ON f.anchor_sentence_id = a.id
                 JOIN content_blocks cb ON a.block_id = cb.id
                 JOIN toc_nodes tn ON cb.node_id = tn.id
                 WHERE s.book_id = $1 AND s.sentence_number IS NOT NULL AND s.footnote_id IS NOT NULL
             )
             UPDATE sentences s SET sentence_number = o.rn FROM ordered o WHERE s.id = o.id",
        )
        .bind(book_id)
        .execute(&mut **tx)
        .await?;
        report.renumbered = true;
    }

    // 9. Write back hashes: changed nodes + the book root. Unchanged nodes keep
    //    their stored hash.
    for node in &output.toc_nodes {
        if !changed_refs.contains(&node.source_ref) {
            continue;
        }
        sqlx::query("UPDATE toc_nodes SET content_hash = $2 WHERE id = $1")
            .bind(node_id_by_ref[&node.source_ref])
            .bind(desired_hash_by_ref[node.source_ref.as_str()])
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
