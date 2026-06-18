//! Book-agnostic reconciling re-import: update an already-imported book in place
//! from a freshly parsed struct, preserving the UUIDs of unchanged sentences
//! (and the quotations / resources / cross-references anchored to them).
//!
//! Identity is anchored to the block: a split/merge only reshuffles ordinals
//! inside the one affected paragraph, so we reconcile per block, aligning old
//! rows against the new struct by text — paragraphs and footnotes alike. A change
//! we cannot attribute confidently (a removed/reordered TOC node, a removed or
//! shifted paragraph or footnote, or two structural edits in one paragraph)
//! aborts to `db:reset`.
//!
//! Strictly-additive growth, by contrast, is reconciled in place: new TOC nodes,
//! blocks appended to an existing node, and new footnotes are inserted alongside
//! the reconcile of existing rows (the batch-by-batch curation workflow, where
//! each batch appends the next run of sections). "Strictly additive" means no
//! existing identity moves: node `sort_order`, block positions and paragraph/
//! figure numbering, and footnote numbers + anchor locations are all verified in
//! pre-flight, and the run aborts if a "new" element would renumber existing
//! ones (e.g. a paragraph or footnote inserted mid-book).
//!
//! Reconcile is incremental via content hashes: each node stores a hash and the
//! book stores the root. An unchanged root short-circuits the run in one query;
//! only nodes whose hash differs are loaded and applied. Within a changed node,
//! a same-count block writes just the sentences whose content differs, while a
//! block whose sentence count changed is re-laid-out and triggers a set-based
//! global `sentence_number` renumber. `--full-rewrite` bypasses the hash checks
//! and rewrites everything. See docs/architecture/reconcile-incremental-hashing.md.
//!
//! Callers map their own struct → the owned [`ReconcileInput`] IR below, compute
//! their stored hashes themselves (so hashing stays identical to the fresh-insert
//! path), and pass everything in. This module never recomputes hashes — it only
//! compares the passed-in ones against stored values and writes them back.

use std::collections::{HashMap, HashSet};

use sqlx::{Postgres, Transaction};
use uuid::Uuid;

use crate::align::{BlockPlan, Existing, plan_block};
use crate::deps::{extend_anchors_to, migrate_dependents, sentence_has_dependents};

// Temp `sentence_number` base for rows inserted during apply. Existing rows keep
// their real numbers until the set-based renumber reassigns everything, so a
// fresh insert needs a non-null, unique placeholder that is out of the dense
// [1, N] range (any real book is far smaller than this). The renumber then
// rewrites these to their final dense values.
const TEMP_SENTENCE_NUMBER_BASE: i32 = 8_000_000;

/// How an *added* node's source link is set. Existing (non-added) nodes never
/// have their `source_id`/`source_node_id` touched.
pub enum NodeAnchor {
    /// No source link: both `source_id` and `source_node_id` stay NULL.
    None,
    /// Translation node: point `toc_nodes.source_node_id` at the source book's
    /// matching node.
    SourceNode(Uuid),
    /// Source-anchored work node (e.g. a Bible-shape sub-work): create a
    /// `source_type='chapter'` source under the book's compilation source, link
    /// the author, and point `toc_nodes.source_id` at it.
    WorkSource {
        title: String,
        publication_year: Option<i16>,
        parent_source_id: Uuid,
        author_person_id: Uuid,
        created_by: Uuid,
    },
}

pub struct MarkerInput {
    pub system: String,
    pub ref_value: String,
    pub sort_order: i32,
    pub char_offset: i32,
}

/// A footnote's sentences. Footnote sentences carry `indent = None`, no markers,
/// and no nested footnotes.
pub struct FootnoteInput {
    pub number: i32,
    pub sentences: Vec<SentenceInput>,
}

pub struct SentenceInput {
    pub position: i16,
    pub sentence_number: Option<i32>,
    pub segment: Option<i16>,
    pub indent: Option<i16>,
    pub text: String,
    pub html: String,
    pub original_text: Option<String>,
    pub original_html: Option<String>,
    pub markers: Vec<MarkerInput>,
    pub footnotes: Vec<FootnoteInput>,
}

pub struct BlockInput {
    pub position: i16,
    pub block_type: String,
    pub paragraph_number: Option<i32>,
    pub figure_number: Option<i32>,
    pub text: String,
    pub html: String,
    pub original_text: Option<String>,
    pub original_html: Option<String>,
    pub sentences: Vec<SentenceInput>,
}

pub struct NodeInput {
    pub source_ref: String,
    pub parent_source_ref: Option<String>,
    pub slug: String,
    pub path: String,
    pub sort_order: i32,
    pub depth: i16,
    pub label: String,
    pub label_html: String,
    pub anchor: NodeAnchor,
    pub blocks: Vec<BlockInput>,
}

pub struct ReconcileInput {
    pub nodes: Vec<NodeInput>,
}

#[derive(Default)]
pub struct ReconcileReport {
    pub unchanged: bool,
    pub nodes_changed: u32,
    pub nodes_added: u32,
    pub blocks_added: u32,
    pub footnotes_added: u32,
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
        eprintln!("  nodes added:         {}", self.nodes_added);
        eprintln!("  blocks added:        {}", self.blocks_added);
        eprintln!("  footnotes added:     {}", self.footnotes_added);
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

// Existing content_blocks rows: (id, node source_ref, position, paragraph_number,
// figure_number). The numbering pair doubles as the per-block stability check.
type BlockRow = (Uuid, String, i16, Option<i32>, Option<i32>);
type BlockNumbering = HashMap<i16, (Option<i32>, Option<i32>)>;

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
    indent: Option<i16>,
}

/// Reconcile a book in place from `input`, carrying unchanged sentence UUIDs (and
/// their dependents) forward across edits.
///
/// The caller computes `desired_node_hashes` (document order, paired with
/// `source_ref`) and `desired_root` from its own model — this function never
/// recomputes hashes, only compares them against stored values and writes them
/// back. `is_translation` + the two source maps drive the translation links;
/// pass empty maps and `false` for a self-standing book.
#[allow(clippy::too_many_arguments)]
pub async fn reconcile_book(
    tx: &mut Transaction<'_, Postgres>,
    book_id: Uuid,
    input: &ReconcileInput,
    desired_node_hashes: &[(String, String)],
    desired_root: &str,
    system_ids: &HashMap<String, Uuid>,
    is_translation: bool,
    source_sentence_map: &SourceSentenceMap,
    source_fn_sentence_map: &SourceFnSentenceMap,
    force: bool,
    full_rewrite: bool,
) -> Result<ReconcileReport, Box<dyn std::error::Error>> {
    let mut report = ReconcileReport::default();
    let nodes = &input.nodes;

    // --- Desired hashes + root short-circuit -------------------------------
    let desired_hash_by_ref: HashMap<&str, &str> = desired_node_hashes
        .iter()
        .map(|(r, h)| (r.as_str(), h.as_str()))
        .collect();

    let stored_root: Option<String> =
        sqlx::query_scalar("SELECT content_hash FROM books WHERE id = $1")
            .bind(book_id)
            .fetch_one(&mut **tx)
            .await?;
    if !full_rewrite && stored_root.as_deref() == Some(desired_root) {
        report.unchanged = true;
        return Ok(report);
    }

    // --- Load existing structure (cheap: no sentence text) -----------------
    let node_rows: Vec<(Uuid, String, Option<String>, i32)> = sqlx::query_as(
        "SELECT id, source_ref, content_hash, sort_order FROM toc_nodes WHERE book_id = $1",
    )
    .bind(book_id)
    .fetch_all(&mut **tx)
    .await?;
    let mut node_id_by_ref: HashMap<String, Uuid> = node_rows
        .iter()
        .map(|(id, sref, _, _)| (sref.clone(), *id))
        .collect();
    let stored_node_hash: HashMap<&str, Option<&str>> = node_rows
        .iter()
        .map(|(_, sref, h, _)| (sref.as_str(), h.as_deref()))
        .collect();

    let block_rows: Vec<BlockRow> = sqlx::query_as(
        "SELECT cb.id, tn.source_ref, cb.position, cb.paragraph_number, cb.figure_number
         FROM content_blocks cb JOIN toc_nodes tn ON cb.node_id = tn.id
         WHERE cb.book_id = $1",
    )
    .bind(book_id)
    .fetch_all(&mut **tx)
    .await?;
    let mut block_id_by_pos: HashMap<(String, i16), Uuid> = block_rows
        .iter()
        .map(|(id, sref, pos, _, _)| ((sref.clone(), *pos), *id))
        .collect();

    let footnote_rows: Vec<(i32, Uuid)> =
        sqlx::query_as("SELECT number, id FROM footnotes WHERE book_id = $1")
            .bind(book_id)
            .fetch_all(&mut **tx)
            .await?;
    let footnote_id_by_number: HashMap<i32, Uuid> = footnote_rows.iter().copied().collect();

    // Where each existing footnote is anchored — (node source_ref, block
    // position). Footnote identity is its number, so a footnote whose desired
    // anchor location differs from the stored one signals a numbering shift
    // (a footnote added/removed mid-book), not an edit.
    let fn_anchor_rows: Vec<(i32, String, i16)> = sqlx::query_as(
        "SELECT f.number, tn.source_ref, cb.position
         FROM footnotes f
         JOIN sentences s ON f.anchor_sentence_id = s.id
         JOIN content_blocks cb ON s.block_id = cb.id
         JOIN toc_nodes tn ON cb.node_id = tn.id
         WHERE f.book_id = $1",
    )
    .bind(book_id)
    .fetch_all(&mut **tx)
    .await?;

    // --- Pre-flight: existing structure must be stable; growth must be -----
    // strictly additive. Removals, reorders, and anything that would shift an
    // existing identity (node sort_order, block position, paragraph/figure
    // number, footnote number/anchor) abort to `db:reset`.
    let desired_nodes: Vec<(&str, i32)> = nodes
        .iter()
        .map(|n| (n.source_ref.as_str(), n.sort_order))
        .collect();
    let existing_node_sort: HashMap<&str, i32> = node_rows
        .iter()
        .map(|(_, sref, _, sort)| (sref.as_str(), *sort))
        .collect();
    let added_node_refs = classify_added_nodes(&desired_nodes, &existing_node_sort)?;

    let mut existing_blocks_by_node: HashMap<&str, BlockNumbering> = HashMap::new();
    for (_, sref, pos, para, fig) in &block_rows {
        existing_blocks_by_node
            .entry(sref.as_str())
            .or_default()
            .insert(*pos, (*para, *fig));
    }
    let no_blocks: BlockNumbering = HashMap::new();
    for node in nodes {
        if added_node_refs.contains(&node.source_ref) {
            continue;
        }
        let desired_blocks: Vec<(i16, Option<i32>, Option<i32>)> = node
            .blocks
            .iter()
            .map(|b| (b.position, b.paragraph_number, b.figure_number))
            .collect();
        let existing = existing_blocks_by_node
            .get(node.source_ref.as_str())
            .unwrap_or(&no_blocks);
        classify_added_block_positions(&node.source_ref, &desired_blocks, existing)?;
    }

    // Every added block (appended to an existing node or inside an added node)
    // must carry fresh paragraph/figure numbers. A collision means the addition
    // sits mid-book and would renumber existing blocks — catch it here with a
    // clear message instead of a unique-index violation at insert.
    let stored_para_numbers: HashSet<i32> =
        block_rows.iter().filter_map(|(_, _, _, p, _)| *p).collect();
    let stored_figure_numbers: HashSet<i32> =
        block_rows.iter().filter_map(|(_, _, _, _, f)| *f).collect();
    for node in nodes {
        for block in &node.blocks {
            if block_id_by_pos.contains_key(&(node.source_ref.clone(), block.position)) {
                continue;
            }
            if let Some(p) = block.paragraph_number
                && stored_para_numbers.contains(&p)
            {
                return Err(format!(
                    "node {} / block {}: paragraph number {p} already exists — the added \
                     paragraph would renumber existing ones; use `pnpm db:reset` + re-import",
                    node.source_ref, block.position
                )
                .into());
            }
            if let Some(f) = block.figure_number
                && stored_figure_numbers.contains(&f)
            {
                return Err(format!(
                    "node {} / block {}: figure number {f} already exists — the added \
                     figure would renumber existing ones; use `pnpm db:reset` + re-import",
                    node.source_ref, block.position
                )
                .into());
            }
        }
    }

    // Footnote numbers may grow but never shift. Sentence counts *within* an
    // existing footnote may change: those splits/merges are reconciled below.
    let desired_fn_anchor: HashMap<i32, (String, i16)> = nodes
        .iter()
        .flat_map(|n| {
            n.blocks.iter().flat_map(move |b| {
                b.sentences.iter().flat_map(move |s| {
                    s.footnotes
                        .iter()
                        .map(move |f| (f.number, (n.source_ref.clone(), b.position)))
                })
            })
        })
        .collect();
    let existing_fn_anchor: HashMap<i32, (String, i16)> = fn_anchor_rows
        .into_iter()
        .map(|(number, sref, pos)| (number, (sref, pos)))
        .collect();
    let added_fn_numbers = classify_added_footnotes(&desired_fn_anchor, &existing_fn_anchor)?;

    // --- Insert added nodes (skeleton rows only) ----------------------------
    // Registered in `node_id_by_ref`, an added node flows through the same
    // changed-node machinery as a hash-mismatched existing node (it has no
    // stored hash, so it always lands in the changed set): its blocks are
    // inserted in the planning loop, sentences/markers/footnotes in the apply,
    // and its content_hash in step 9.
    for node in nodes {
        if !added_node_refs.contains(node.source_ref.as_str()) {
            continue;
        }
        let parent_id: Option<Uuid> = match &node.parent_source_ref {
            Some(parent_ref) => Some(*node_id_by_ref.get(parent_ref).ok_or_else(|| {
                format!(
                    "added node {}: parent {parent_ref} not found among existing or \
                     previously added nodes",
                    node.source_ref
                )
            })?),
            None => None,
        };
        // The added node's source link comes from its anchor: a translation node
        // points at the source book's node; a Bible-shape work node creates its
        // own sub-work source + author link. Existing nodes keep their links.
        let (source_id, source_node_id): (Option<Uuid>, Option<Uuid>) = match &node.anchor {
            NodeAnchor::None => (None, None),
            NodeAnchor::SourceNode(id) => (None, Some(*id)),
            NodeAnchor::WorkSource {
                title,
                publication_year,
                parent_source_id,
                author_person_id,
                created_by,
            } => {
                let sid: Uuid = sqlx::query_scalar(
                    "INSERT INTO sources (source_type, title, publication_year, parent_source_id, protected, created_by)
                     VALUES ('chapter', $1, $2, $3, true, $4)
                     RETURNING id",
                )
                .bind(title)
                .bind(publication_year)
                .bind(parent_source_id)
                .bind(created_by)
                .fetch_one(&mut **tx)
                .await?;
                sqlx::query(
                    "INSERT INTO source_persons (source_id, person_id, role, position)
                     VALUES ($1, $2, 'author', 0)
                     ON CONFLICT DO NOTHING",
                )
                .bind(sid)
                .bind(author_person_id)
                .execute(&mut **tx)
                .await?;
                (Some(sid), None)
            }
        };
        let label_html = if node.label_html != node.label {
            Some(&node.label_html)
        } else {
            None
        };
        let node_id: Uuid = sqlx::query_scalar(
            "INSERT INTO toc_nodes (book_id, parent_id, source_id, source_node_id, source_ref, slug, path, sort_order, depth, label, label_html)
             VALUES ($1, $2, $3, $4, $5, $6, $7::ltree, $8, $9, $10, $11)
             RETURNING id",
        )
        .bind(book_id)
        .bind(parent_id)
        .bind(source_id)
        .bind(source_node_id)
        .bind(&node.source_ref)
        .bind(&node.slug)
        .bind(&node.path)
        .bind(node.sort_order)
        .bind(node.depth)
        .bind(&node.label)
        .bind(label_html)
        .fetch_one(&mut **tx)
        .await?;
        node_id_by_ref.insert(node.source_ref.clone(), node_id);
        report.nodes_added += 1;
    }

    // --- Changed set (NULL stored hash ⇒ changed; `--full-rewrite` ⇒ everything) ----
    // Added nodes have no stored hash, so they are always part of the set.
    let changed_refs: HashSet<String> = nodes
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
    report.nodes_changed = changed_refs
        .iter()
        .filter(|r| !added_node_refs.contains(r.as_str()))
        .count() as u32;

    // Nothing content-changed (e.g. a stale root with intact node hashes): just
    // refresh the stored root and return. The loops below would all no-op.
    if changed_refs.is_empty() {
        sqlx::query("UPDATE books SET content_hash = $2 WHERE id = $1")
            .bind(book_id)
            .bind(desired_root)
            .execute(&mut **tx)
            .await?;
        return Ok(report);
    }

    let changed_refs_vec: Vec<String> = changed_refs.iter().cloned().collect();
    let changed_node_ids: Vec<Uuid> = changed_refs.iter().map(|r| node_id_by_ref[r]).collect();
    let changed_fn_numbers: Vec<i32> = nodes
        .iter()
        .filter(|n| changed_refs.contains(&n.source_ref))
        .flat_map(|n| &n.blocks)
        .flat_map(|b| &b.sentences)
        .flat_map(|s| &s.footnotes)
        .map(|f| f.number)
        .collect();

    // --- Load existing sentence content, scoped to changed nodes -----------
    let sent_rows: Vec<BlockSentRow> = sqlx::query_as(
        "SELECT s.id, tn.source_ref, cb.position, s.position,
                    s.text, s.html, s.original_text, s.original_html, s.segment, s.indent
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
    for (id, sref, block_pos, _spos, text, html, original_text, original_html, segment, indent) in
        sent_rows
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
                indent,
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
                indent: None,
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

    for (node_idx, node) in nodes.iter().enumerate() {
        if !changed_refs.contains(&node.source_ref) {
            continue;
        }
        let node_id = node_id_by_ref[&node.source_ref];
        for (block_idx, block) in node.blocks.iter().enumerate() {
            let key = (node.source_ref.clone(), block.position);
            let (block_id, plan, count_delta) = match block_id_by_pos.get(&key) {
                Some(&block_id) => {
                    let empty: Vec<Existing> = Vec::new();
                    let old = existing_by_block.get(&key).unwrap_or(&empty);
                    let new: Vec<&str> = block.sentences.iter().map(|s| s.text.as_str()).collect();
                    let label = format!("node {} / block {}", node.source_ref, block.position);
                    let plan = plan_block(&label, old, &new)?;
                    let count_delta = full_rewrite || old.len() != new.len();
                    (block_id, plan, count_delta)
                }
                // Added block (pre-flight verified strictly additive): insert
                // the row now, plan every sentence as a fresh insert.
                None => {
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
                    .fetch_one(&mut **tx)
                    .await?;
                    block_id_by_pos.insert(key.clone(), block_id);
                    report.blocks_added += 1;
                    let plan = BlockPlan {
                        assignment: vec![None; block.sentences.len()],
                        retired: Vec::new(),
                        splits: Vec::new(),
                    };
                    (block_id, plan, true)
                }
            };
            all_retired.extend(plan.retired.iter().copied());
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
        // None = added footnote: the row is inserted during apply, anchored to
        // the resolved anchor sentence.
        footnote_id: Option<Uuid>,
        node_id: Uuid,
        anchor_key: (String, i16, i16),
        // The anchor sentence only moves when its containing block changed count.
        anchor_block_count_delta: bool,
        plan: BlockPlan,
        count_delta: bool,
    }
    let mut fn_works: Vec<FnWork> = Vec::new();
    for work in &works {
        let node = &nodes[work.node_idx];
        let block = &node.blocks[work.block_idx];
        for sent in &block.sentences {
            for footnote in &sent.footnotes {
                let (plan, count_delta) = if added_fn_numbers.contains(&footnote.number) {
                    let plan = BlockPlan {
                        assignment: vec![None; footnote.sentences.len()],
                        retired: Vec::new(),
                        splits: Vec::new(),
                    };
                    (plan, true)
                } else {
                    let empty: Vec<Existing> = Vec::new();
                    let old = existing_fn_by_number
                        .get(&footnote.number)
                        .unwrap_or(&empty);
                    let new: Vec<&str> =
                        footnote.sentences.iter().map(|s| s.text.as_str()).collect();
                    let label = format!("footnote {}", footnote.number);
                    let plan = plan_block(&label, old, &new)?;
                    let count_delta = full_rewrite || old.len() != new.len();
                    (plan, count_delta)
                };
                all_retired.extend(plan.retired.iter().copied());
                fn_works.push(FnWork {
                    number: footnote.number,
                    footnote_id: footnote_id_by_number.get(&footnote.number).copied(),
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
        .filter_map(|w| w.footnote_id)
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
        let node = &nodes[work.node_idx];
        let block = &node.blocks[work.block_idx];
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
                    crate::keys::natural_key(&node.source_ref, block.position, sent.position);
                match work.plan.assignment[i] {
                    Some(existing_id) => {
                        // Full reassign: positions shifted. `sentence_number` is
                        // left to the global renumber (step 8).
                        sqlx::query(
                            "UPDATE sentences
                             SET position = $2, segment = $3, indent = $4,
                                 source_sentence_start_id = $5, source_sentence_end_id = NULL,
                                 text = $6, html = $7, original_text = $8, original_html = $9,
                                 natural_key = $10, updated_at = now()
                             WHERE id = $1",
                        )
                        .bind(existing_id)
                        .bind(sent.position)
                        .bind(sent.segment)
                        .bind(sent.indent)
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
                            "INSERT INTO sentences (book_id, node_id, block_id, position, sentence_number, segment, indent, source_sentence_start_id, text, html, original_text, original_html, natural_key)
                             VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9, $10, $11, $12, $13)
                             RETURNING id",
                        )
                        .bind(book_id)
                        .bind(work.node_id)
                        .bind(work.block_id)
                        .bind(sent.position)
                        .bind(temp_number)
                        .bind(sent.segment)
                        .bind(sent.indent)
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
                            || c.indent != sent.indent
                    }
                    None => true,
                };
                if differs {
                    sqlx::query(
                        "UPDATE sentences
                         SET segment = $2, indent = $3, text = $4, html = $5,
                             original_text = $6, original_html = $7, updated_at = now()
                         WHERE id = $1",
                    )
                    .bind(existing_id)
                    .bind(sent.segment)
                    .bind(sent.indent)
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
        let footnote_id = match work.footnote_id {
            Some(id) => {
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
                id
            }
            // Added footnote: create the row, anchored to the just-resolved
            // sentence. Its sentences all take the insert path below.
            None => {
                let id: Uuid = sqlx::query_scalar(
                    "INSERT INTO footnotes (book_id, number, anchor_sentence_id)
                     VALUES ($1, $2, $3)
                     RETURNING id",
                )
                .bind(book_id)
                .bind(work.number)
                .bind(anchor_id)
                .fetch_one(&mut **tx)
                .await?;
                report.footnotes_added += 1;
                id
            }
        };

        // Re-find the footnote's sentences in the desired struct.
        let node = nodes
            .iter()
            .find(|n| node_id_by_ref.get(&n.source_ref) == Some(&work.node_id))
            .expect("footnote node present");
        let footnote = node
            .blocks
            .iter()
            .flat_map(|b| &b.sentences)
            .flat_map(|s| &s.footnotes)
            .find(|f| f.number == work.number)
            .expect("footnote present in desired struct");

        let mut idx_uuid: Vec<Uuid> = Vec::with_capacity(footnote.sentences.len());
        for (i, fn_sent) in footnote.sentences.iter().enumerate() {
            let sid = if work.count_delta {
                let natural_key = crate::keys::footnote_natural_key(
                    &node.source_ref,
                    work.number,
                    fn_sent.position,
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
                        .bind(footnote_id)
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
    for node in nodes {
        if !changed_refs.contains(&node.source_ref) {
            continue;
        }
        for block in &node.blocks {
            for sent in &block.sentences {
                let sid = resolved[&(node.source_ref.clone(), block.position, sent.position)];
                for pm in &sent.markers {
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
    for node in nodes {
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
        for block in &node.blocks {
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
    for node in nodes {
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
        .bind(desired_root)
        .execute(&mut **tx)
        .await?;

    Ok(report)
}

// --- Strictly-additive pre-flight classification ----------------------------
// Pure functions so the subtle identity rules are unit-testable without a DB.
// Each returns the *added* identities, or an error for anything non-additive.

/// Desired nodes must be a superset of existing ones, and every existing node
/// must keep its `sort_order` (a shift means the TOC was renumbered/reordered).
fn classify_added_nodes(
    desired: &[(&str, i32)],
    existing_sort: &HashMap<&str, i32>,
) -> Result<HashSet<String>, String> {
    let desired_refs: HashSet<&str> = desired.iter().map(|(r, _)| *r).collect();
    let mut removed: Vec<&str> = existing_sort
        .keys()
        .filter(|r| !desired_refs.contains(*r))
        .copied()
        .collect();
    if !removed.is_empty() {
        removed.sort();
        return Err(format!(
            "TOC nodes removed ({}); not reconcilable — use `pnpm db:reset` + re-import",
            removed.join(", ")
        ));
    }
    let mut added = HashSet::new();
    for (sref, sort_order) in desired {
        match existing_sort.get(sref) {
            None => {
                added.insert(sref.to_string());
            }
            Some(stored) if stored != sort_order => {
                return Err(format!(
                    "node {sref}: sort_order changed ({stored} → {sort_order}); TOC reordered — \
                     use `pnpm db:reset` + re-import"
                ));
            }
            Some(_) => {}
        }
    }
    Ok(added)
}

/// Within an existing node, blocks may only be appended: the existing positions
/// must be exactly the first `existing.len()` desired positions, and each
/// matched block must keep its paragraph/figure number (a mismatch means a
/// block was inserted/removed mid-node or numbering shifted across the book).
/// Returns the appended positions.
fn classify_added_block_positions(
    node_ref: &str,
    desired: &[(i16, Option<i32>, Option<i32>)],
    existing: &HashMap<i16, (Option<i32>, Option<i32>)>,
) -> Result<Vec<i16>, String> {
    if existing.len() > desired.len() {
        return Err(format!(
            "node {node_ref}: paragraphs removed; not reconcilable — use `pnpm db:reset` + re-import"
        ));
    }
    let mut desired_sorted = desired.to_vec();
    desired_sorted.sort_by_key(|(pos, _, _)| *pos);
    let (head, tail) = desired_sorted.split_at(existing.len());
    for (pos, para, fig) in head {
        match existing.get(pos) {
            None => {
                return Err(format!(
                    "node {node_ref}: block positions shifted (existing blocks are not a prefix \
                     of the desired ones); not reconcilable — use `pnpm db:reset` + re-import"
                ));
            }
            Some((stored_para, stored_fig)) => {
                if stored_para != para || stored_fig != fig {
                    return Err(format!(
                        "node {node_ref} / block {pos}: paragraph/figure numbering shifted \
                         (stored {stored_para:?}/{stored_fig:?}, desired {para:?}/{fig:?}); \
                         use `pnpm db:reset` + re-import"
                    ));
                }
            }
        }
    }
    Ok(tail.iter().map(|(pos, _, _)| *pos).collect())
}

/// Desired footnote numbers must be a superset of existing ones, and every
/// existing footnote must keep its anchor location (node source_ref + block
/// position). Identity is the number, so a moved anchor signals that footnote
/// numbering shifted (a footnote added/removed mid-book) — the alignment below
/// would then compare unrelated footnotes' sentences.
fn classify_added_footnotes(
    desired_anchor: &HashMap<i32, (String, i16)>,
    existing_anchor: &HashMap<i32, (String, i16)>,
) -> Result<HashSet<i32>, String> {
    let mut removed: Vec<i32> = existing_anchor
        .keys()
        .filter(|n| !desired_anchor.contains_key(n))
        .copied()
        .collect();
    if !removed.is_empty() {
        removed.sort();
        return Err(format!(
            "footnotes removed ({removed:?}); not reconcilable — use `pnpm db:reset` + re-import"
        ));
    }
    let mut added = HashSet::new();
    for (number, (sref, block_pos)) in desired_anchor {
        match existing_anchor.get(number) {
            None => {
                added.insert(*number);
            }
            Some((stored_ref, stored_pos)) if (stored_ref, stored_pos) != (sref, block_pos) => {
                return Err(format!(
                    "footnote {number}: anchor moved from node {stored_ref} / block {stored_pos} \
                     to node {sref} / block {block_pos} — footnote numbering shifted; \
                     use `pnpm db:reset` + re-import"
                ));
            }
            Some(_) => {}
        }
    }
    Ok(added)
}

#[cfg(test)]
mod tests {
    use super::*;

    fn node_sort(entries: &[(&'static str, i32)]) -> HashMap<&'static str, i32> {
        entries.iter().copied().collect()
    }

    #[test]
    fn nodes_unchanged_yields_no_additions() {
        let existing = node_sort(&[("001", 1), ("002", 2)]);
        let added = classify_added_nodes(&[("001", 1), ("002", 2)], &existing).unwrap();
        assert!(added.is_empty());
    }

    #[test]
    fn nodes_appended_are_added() {
        let existing = node_sort(&[("001", 1), ("002", 2)]);
        let added =
            classify_added_nodes(&[("001", 1), ("002", 2), ("003", 3), ("004", 4)], &existing)
                .unwrap();
        assert_eq!(added, HashSet::from(["003".to_string(), "004".to_string()]));
    }

    #[test]
    fn node_inserted_mid_book_is_added_when_refs_are_stable() {
        // Positions are stable per TOC entry, so a mid-book node keeps every
        // other ref + sort_order intact — that is legitimately additive.
        let existing = node_sort(&[("001", 1), ("003", 3)]);
        let added = classify_added_nodes(&[("001", 1), ("002", 2), ("003", 3)], &existing).unwrap();
        assert_eq!(added, HashSet::from(["002".to_string()]));
    }

    #[test]
    fn removed_node_aborts() {
        let existing = node_sort(&[("001", 1), ("002", 2)]);
        let err = classify_added_nodes(&[("001", 1)], &existing).unwrap_err();
        assert!(err.contains("002"), "{err}");
    }

    #[test]
    fn shifted_sort_order_aborts() {
        let existing = node_sort(&[("001", 1), ("002", 2)]);
        let err = classify_added_nodes(&[("001", 1), ("002", 3)], &existing).unwrap_err();
        assert!(err.contains("sort_order"), "{err}");
    }

    fn blocks(
        entries: &[(i16, Option<i32>, Option<i32>)],
    ) -> HashMap<i16, (Option<i32>, Option<i32>)> {
        entries.iter().map(|(p, a, f)| (*p, (*a, *f))).collect()
    }

    #[test]
    fn blocks_unchanged_yields_no_additions() {
        let existing = blocks(&[(0, None, None), (1, Some(7), None)]);
        let added = classify_added_block_positions(
            "010",
            &[(0, None, None), (1, Some(7), None)],
            &existing,
        )
        .unwrap();
        assert!(added.is_empty());
    }

    #[test]
    fn blocks_appended_are_added() {
        let existing = blocks(&[(0, None, None), (1, Some(7), None)]);
        let added = classify_added_block_positions(
            "010",
            &[
                (0, None, None),
                (1, Some(7), None),
                (2, Some(8), None),
                (3, Some(9), None),
            ],
            &existing,
        )
        .unwrap();
        assert_eq!(added, vec![2, 3]);
    }

    #[test]
    fn block_removed_aborts() {
        let existing = blocks(&[(0, None, None), (1, Some(7), None)]);
        let err = classify_added_block_positions("010", &[(0, None, None)], &existing).unwrap_err();
        assert!(err.contains("removed"), "{err}");
    }

    #[test]
    fn paragraph_renumbering_aborts() {
        // A paragraph inserted earlier in the book shifts this node's stored
        // numbers; positions still match but paragraph numbers do not.
        let existing = blocks(&[(0, Some(7), None), (1, Some(8), None)]);
        let err = classify_added_block_positions(
            "010",
            &[(0, Some(8), None), (1, Some(9), None)],
            &existing,
        )
        .unwrap_err();
        assert!(err.contains("numbering shifted"), "{err}");
    }

    #[test]
    fn non_prefix_positions_abort() {
        // Existing {0, 2} can never be a prefix of desired {0, 1, 2}.
        let existing = blocks(&[(0, None, None), (2, Some(7), None)]);
        let err = classify_added_block_positions(
            "010",
            &[(0, None, None), (1, None, None), (2, Some(7), None)],
            &existing,
        )
        .unwrap_err();
        assert!(err.contains("shifted"), "{err}");
    }

    fn anchors(entries: &[(i32, &str, i16)]) -> HashMap<i32, (String, i16)> {
        entries
            .iter()
            .map(|(n, sref, pos)| (*n, (sref.to_string(), *pos)))
            .collect()
    }

    #[test]
    fn footnotes_unchanged_yields_no_additions() {
        let existing = anchors(&[(1, "010", 2), (2, "011", 0)]);
        let desired = anchors(&[(1, "010", 2), (2, "011", 0)]);
        let added = classify_added_footnotes(&desired, &existing).unwrap();
        assert!(added.is_empty());
    }

    #[test]
    fn new_footnotes_in_new_sections_are_added() {
        let existing = anchors(&[(1, "010", 2)]);
        let desired = anchors(&[(1, "010", 2), (2, "074", 0), (3, "075", 1)]);
        let added = classify_added_footnotes(&desired, &existing).unwrap();
        assert_eq!(added, HashSet::from([2, 3]));
    }

    #[test]
    fn removed_footnote_aborts() {
        let existing = anchors(&[(1, "010", 2), (2, "011", 0)]);
        let desired = anchors(&[(1, "010", 2)]);
        let err = classify_added_footnotes(&desired, &existing).unwrap_err();
        assert!(err.contains("removed"), "{err}");
    }

    #[test]
    fn footnote_numbering_shift_aborts() {
        // A footnote inserted mid-book shifts every later number: desired is a
        // superset by numbers, but existing 2's anchor location now belongs to
        // desired 3 — caught by the anchor check, not the set check.
        let existing = anchors(&[(1, "010", 2), (2, "020", 0)]);
        let desired = anchors(&[(1, "010", 2), (2, "015", 1), (3, "020", 0)]);
        let err = classify_added_footnotes(&desired, &existing).unwrap_err();
        assert!(err.contains("anchor moved"), "{err}");
    }
}
