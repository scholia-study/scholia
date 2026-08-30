//! Mid-book insertion pre-alignment — the opt-in relaxation of "strictly
//! additive" for books that grow (peirce1). See ADR 0012.
//!
//! Inserting a node ahead of existing ones shifts every book-global display
//! sequence after it: `paragraph_number`, `figure_number`, and footnote /
//! margin-note numbers. Body-sentence identity is untouched (natural keys are
//! `{source_ref}/b{pos}/s{pos}` — no global counter involved), so an insertion
//! is safe *in principle*; what breaks is the pre-flight, which reads any
//! numbering shift as structural drift and aborts.
//!
//! This pass runs inside the reconcile transaction, before pre-flight, and only
//! when the caller opted in AND an added node actually sits mid-book. It
//! rewrites the stored numbering of *existing* rows to the desired values, so
//! that the untouched pipeline then sees a plain strictly-additive run:
//!
//! - blocks keep their identity by (node source_ref, position); their
//!   paragraph/figure numbers are set to the desired values (offset first —
//!   both columns carry partial unique indexes);
//! - footnotes and margin notes keep their identity by document order: existing
//!   and desired notes in non-added nodes are paired 1:1 in order, anchors must
//!   agree pairwise, numbers are rewritten via the pairing, and the note
//!   sentences' natural keys (which embed the number) are rewritten with them.
//!
//! The anchor sentences' HTML — which bakes in footnote reference numbers — is
//! NOT touched here: it is content, so the shifted nodes' hashes differ and the
//! normal edit path updates it while carrying sentence UUIDs.
//!
//! Deliberate limits, enforced with clear errors rather than guessed around:
//! translation editions are unsupported (they are 1:1-locked to a source), and
//! a single run may not combine a mid-book insertion with new notes appended to
//! *existing* nodes (the order-pairing would be ambiguous) — land those as two
//! runs.

use std::collections::{HashMap, HashSet};

use sqlx::{Postgres, Transaction};
use uuid::Uuid;

use crate::orchestrate::ReconcileInput;

/// Offset applied to a whole numbering sequence before reassigning, so the
/// partial unique indexes never see an intermediate collision.
const RENUMBER_OFFSET: i64 = 1_000_000;

pub struct PreAlignReport {
    pub blocks_renumbered: u64,
    pub footnotes_renumbered: u64,
    pub margin_notes_renumbered: u64,
}

/// One existing note row: (id, number, anchor node source_ref, anchor block
/// position, anchor sentence position).
type NoteRow = (Uuid, i32, String, i16, i16);

/// One existing block row: (id, node source_ref, position, paragraph_number,
/// figure_number).
type BlockRow = (Uuid, String, i16, Option<i32>, Option<i32>);

/// Desired (paragraph_number, figure_number) keyed by (node source_ref,
/// block position).
type DesiredNumbering = HashMap<(String, i16), (Option<i32>, Option<i32>)>;

/// A block's planned numbering: (id, paragraph_number, figure_number).
type PlannedBlock = (Uuid, Option<i32>, Option<i32>);

/// Detect whether the desired input inserts a node ahead of any existing one.
/// Pure append (every added node sorts after all existing) needs no alignment.
fn has_mid_book_insertion(
    desired: &[(String, i32)],
    existing_sort_by_ref: &HashMap<String, i32>,
) -> bool {
    let max_existing = existing_sort_by_ref
        .values()
        .copied()
        .max()
        .unwrap_or(i32::MIN);
    desired
        .iter()
        .any(|(sref, sort)| !existing_sort_by_ref.contains_key(sref) && *sort < max_existing)
}

/// Pair existing notes with desired notes of non-added nodes, in document
/// order. Returns (existing_id, new_number, source_ref) per note. Anchors must
/// agree pairwise — a mismatch means the two sequences are not the same notes,
/// which is drift, not insertion.
fn pair_notes(
    kind: &str,
    existing: &[NoteRow],
    desired: &[(i32, String, i16, i16)],
) -> Result<Vec<(Uuid, i32, String)>, String> {
    if existing.len() != desired.len() {
        return Err(format!(
            "insertion pre-align: {} existing {kind}s but {} desired in existing nodes — \
             adding a {kind} to an existing node cannot be combined with a mid-book \
             insertion in one run; land it as a separate run",
            existing.len(),
            desired.len()
        ));
    }
    existing
        .iter()
        .zip(desired)
        .map(|((id, old, e_sref, e_bpos, e_spos), (new, d_sref, d_bpos, d_spos))| {
            if e_sref != d_sref || e_bpos != d_bpos || e_spos != d_spos {
                return Err(format!(
                    "insertion pre-align: {kind} {old} anchored at {e_sref}/b{e_bpos}/s{e_spos} \
                     pairs with desired {kind} {new} at {d_sref}/b{d_bpos}/s{d_spos} — the \
                     sequences disagree; not an insertion"
                ));
            }
            Ok((*id, *new, e_sref.clone()))
        })
        .collect()
}

/// Plan the paragraph/figure renumbering of existing blocks: every existing
/// block must appear in the desired input at the same (source_ref, position)
/// with the same numbering *kind* (numbered paragraph stays a numbered
/// paragraph, figure stays figure).
fn plan_block_renumber(
    existing: &[BlockRow],
    desired: &DesiredNumbering,
) -> Result<Vec<PlannedBlock>, String> {
    existing
        .iter()
        .map(|(id, sref, pos, para, fig)| {
            let (d_para, d_fig) = desired.get(&(sref.clone(), *pos)).ok_or_else(|| {
                format!(
                    "insertion pre-align: existing block {sref}/b{pos} is absent from the \
                     desired input — removal is not an insertion"
                )
            })?;
            if para.is_some() != d_para.is_some() || fig.is_some() != d_fig.is_some() {
                return Err(format!(
                    "insertion pre-align: block {sref}/b{pos} changes numbering kind \
                     (stored {para:?}/{fig:?}, desired {d_para:?}/{d_fig:?})"
                ));
            }
            Ok((*id, *d_para, *d_fig))
        })
        .collect()
}

/// Run the pre-alignment. Call inside the reconcile transaction, after the
/// root-hash short-circuit and before any other load: this pass reads its own
/// snapshot and writes the aligned numbering, so the main flow's loads see a
/// book whose numbering already matches the desired input.
pub async fn pre_align(
    tx: &mut Transaction<'_, Postgres>,
    book_id: Uuid,
    input: &ReconcileInput,
    is_translation: bool,
) -> Result<Option<PreAlignReport>, Box<dyn std::error::Error>> {
    if is_translation {
        return Err(
            "--allow-insertion is unsupported for translation editions: they are \
                    sentence-locked 1:1 to a source book; insert into the source first"
                .into(),
        );
    }

    let existing_nodes: Vec<(String, i32)> =
        sqlx::query_as("SELECT source_ref, sort_order FROM toc_nodes WHERE book_id = $1")
            .bind(book_id)
            .fetch_all(&mut **tx)
            .await?;
    if existing_nodes.is_empty() {
        return Ok(None); // fresh book, nothing to align
    }
    let existing_sort_by_ref: HashMap<String, i32> = existing_nodes.into_iter().collect();
    let desired_nodes: Vec<(String, i32)> = input
        .nodes
        .iter()
        .map(|n| (n.source_ref.clone(), n.sort_order))
        .collect();
    if !has_mid_book_insertion(&desired_nodes, &existing_sort_by_ref) {
        return Ok(None); // pure append (or no additions): the strict path handles it
    }

    let added_refs: HashSet<&str> = input
        .nodes
        .iter()
        .map(|n| n.source_ref.as_str())
        .filter(|r| !existing_sort_by_ref.contains_key(*r))
        .collect();

    // --- Blocks: renumber paragraph/figure sequences to the desired values ---
    let existing_blocks: Vec<BlockRow> = sqlx::query_as(
        "SELECT cb.id, tn.source_ref, cb.position, cb.paragraph_number, cb.figure_number
         FROM content_blocks cb JOIN toc_nodes tn ON cb.node_id = tn.id
         WHERE cb.book_id = $1",
    )
    .bind(book_id)
    .fetch_all(&mut **tx)
    .await?;
    let desired_blocks: DesiredNumbering = input
        .nodes
        .iter()
        .flat_map(|n| {
            n.blocks.iter().map(move |b| {
                (
                    (n.source_ref.clone(), b.position),
                    (b.paragraph_number, b.figure_number),
                )
            })
        })
        .collect();
    let block_plan = plan_block_renumber(&existing_blocks, &desired_blocks)?;

    let changed_blocks: Vec<&PlannedBlock> = block_plan
        .iter()
        .zip(&existing_blocks)
        .filter(|(planned, stored)| (planned.1, planned.2) != (stored.3, stored.4))
        .map(|(planned, _)| planned)
        .collect();
    let blocks_renumbered = changed_blocks.len() as u64;
    if blocks_renumbered > 0 {
        // Offset both sequences out of range, then assign every existing block
        // its desired values — set-based, immune to intermediate collisions.
        sqlx::query(
            "UPDATE content_blocks SET paragraph_number = paragraph_number + $2
             WHERE book_id = $1 AND paragraph_number IS NOT NULL",
        )
        .bind(book_id)
        .bind(RENUMBER_OFFSET as i32)
        .execute(&mut **tx)
        .await?;
        sqlx::query(
            "UPDATE content_blocks SET figure_number = figure_number + $2
             WHERE book_id = $1 AND figure_number IS NOT NULL",
        )
        .bind(book_id)
        .bind(RENUMBER_OFFSET as i32)
        .execute(&mut **tx)
        .await?;
        let (ids, paras, figs): (Vec<Uuid>, Vec<Option<i32>>, Vec<Option<i32>>) = block_plan
            .iter()
            .map(|(id, p, f)| (*id, *p, *f))
            .fold((vec![], vec![], vec![]), |mut acc, (id, p, f)| {
                acc.0.push(id);
                acc.1.push(p);
                acc.2.push(f);
                acc
            });
        sqlx::query(
            "UPDATE content_blocks cb
             SET paragraph_number = v.para, figure_number = v.fig
             FROM (SELECT unnest($1::uuid[]) AS id, unnest($2::int[]) AS para,
                          unnest($3::int[]) AS fig) v
             WHERE cb.id = v.id",
        )
        .bind(&ids)
        .bind(&paras)
        .bind(&figs)
        .execute(&mut **tx)
        .await?;
    }

    // --- Notes: renumber by document-order pairing, rewrite natural keys -----
    let footnotes_renumbered = renumber_notes(tx, book_id, input, &added_refs, "footnote").await?;
    let margin_notes_renumbered =
        renumber_notes(tx, book_id, input, &added_refs, "margin note").await?;

    Ok(Some(PreAlignReport {
        blocks_renumbered,
        footnotes_renumbered,
        margin_notes_renumbered,
    }))
}

/// Renumber one note family (footnotes or margin notes) and rewrite the natural
/// keys of its sentences. Returns the count of notes whose number changed.
async fn renumber_notes(
    tx: &mut Transaction<'_, Postgres>,
    book_id: Uuid,
    input: &ReconcileInput,
    added_refs: &HashSet<&str>,
    kind: &str,
) -> Result<u64, Box<dyn std::error::Error>> {
    let (table, fk, key_tag) = match kind {
        "footnote" => ("footnotes", "footnote_id", "fn"),
        _ => ("margin_notes", "margin_note_id", "mn"),
    };

    let existing: Vec<NoteRow> = sqlx::query_as(&format!(
        "SELECT n.id, n.number, tn.source_ref, cb.position, s.position
         FROM {table} n
         JOIN sentences s ON n.anchor_sentence_id = s.id
         JOIN content_blocks cb ON s.block_id = cb.id
         JOIN toc_nodes tn ON cb.node_id = tn.id
         WHERE n.book_id = $1
         ORDER BY tn.sort_order, cb.position, s.position, n.number"
    ))
    .bind(book_id)
    .fetch_all(&mut **tx)
    .await?;

    // Desired notes of non-added nodes, in document order.
    let mut sorted_nodes: Vec<&crate::orchestrate::NodeInput> = input.nodes.iter().collect();
    sorted_nodes.sort_by_key(|n| n.sort_order);
    let mut desired: Vec<(i32, String, i16, i16)> = Vec::new();
    for node in sorted_nodes {
        if added_refs.contains(node.source_ref.as_str()) {
            continue;
        }
        for block in &node.blocks {
            for sent in &block.sentences {
                let numbers: Vec<i32> = if kind == "footnote" {
                    sent.footnotes.iter().map(|f| f.number).collect()
                } else {
                    sent.margin_notes.iter().map(|m| m.number).collect()
                };
                for number in numbers {
                    desired.push((
                        number,
                        node.source_ref.clone(),
                        block.position,
                        sent.position,
                    ));
                }
            }
        }
    }

    let pairing = pair_notes(kind, &existing, &desired)?;
    let changed: Vec<&(Uuid, i32, String)> = pairing
        .iter()
        .zip(&existing)
        .filter(|(planned, stored)| planned.1 != stored.1)
        .map(|(planned, _)| planned)
        .collect();
    if changed.is_empty() {
        return Ok(0);
    }

    sqlx::query(&format!(
        "UPDATE {table} SET number = number + $2 WHERE book_id = $1"
    ))
    .bind(book_id)
    .bind(RENUMBER_OFFSET as i32)
    .execute(&mut **tx)
    .await?;
    let (ids, numbers, srefs): (Vec<Uuid>, Vec<i32>, Vec<String>) =
        pairing
            .iter()
            .cloned()
            .fold((vec![], vec![], vec![]), |mut acc, (id, num, sref)| {
                acc.0.push(id);
                acc.1.push(num);
                acc.2.push(sref);
                acc
            });
    sqlx::query(&format!(
        "UPDATE {table} n SET number = v.num
         FROM (SELECT unnest($1::uuid[]) AS id, unnest($2::int[]) AS num) v
         WHERE n.id = v.id"
    ))
    .bind(&ids)
    .bind(&numbers)
    .execute(&mut **tx)
    .await?;

    // The note sentences' natural keys embed the number — rewrite them from the
    // same pairing so reconcile keeps matching these rows next run. Two phases,
    // like the number columns: within one node the new key of one note is often
    // the OLD key of a neighbour (hobbes chapters hold dozens of margin notes),
    // and the unique index checks per row, so a direct assignment collides
    // mid-statement. Park every affected key on a value keyed by sentence id
    // first, then assign the finals.
    sqlx::query(&format!(
        "UPDATE sentences s SET natural_key = 'realign:' || s.id
         FROM (SELECT unnest($1::uuid[]) AS id) v
         WHERE s.{fk} = v.id AND s.natural_key IS NOT NULL"
    ))
    .bind(&ids)
    .execute(&mut **tx)
    .await?;
    sqlx::query(&format!(
        "UPDATE sentences s
         SET natural_key = v.sref || '/{key_tag}' || v.num || '/s' || s.position
         FROM (SELECT unnest($1::uuid[]) AS id, unnest($2::int[]) AS num,
                      unnest($3::text[]) AS sref) v
         WHERE s.{fk} = v.id AND s.natural_key IS NOT NULL"
    ))
    .bind(&ids)
    .bind(&numbers)
    .bind(&srefs)
    .execute(&mut **tx)
    .await?;

    Ok(changed.len() as u64)
}

#[cfg(test)]
mod tests {
    use super::*;

    fn row(num: i32, sref: &str, bpos: i16, spos: i16) -> NoteRow {
        (Uuid::new_v4(), num, sref.to_string(), bpos, spos)
    }

    #[test]
    fn append_only_needs_no_alignment() {
        let existing: HashMap<String, i32> = [("1000".into(), 1000), ("2000".into(), 2000)].into();
        let desired = vec![
            ("1000".to_string(), 1000),
            ("2000".to_string(), 2000),
            ("3000".to_string(), 3000),
        ];
        assert!(!has_mid_book_insertion(&desired, &existing));
    }

    #[test]
    fn a_node_ahead_of_existing_ones_is_an_insertion() {
        let existing: HashMap<String, i32> = [("1000".into(), 1000), ("3000".into(), 3000)].into();
        let desired = vec![
            ("1000".to_string(), 1000),
            ("2000".to_string(), 2000),
            ("3000".to_string(), 3000),
        ];
        assert!(has_mid_book_insertion(&desired, &existing));
    }

    #[test]
    fn notes_pair_in_order_and_take_the_desired_numbers() {
        let existing = vec![row(1, "1000", 3, 0), row(2, "3000", 1, 2)];
        // A paper inserted at 2000 carries one footnote, so 3000's note is now 3.
        let desired = vec![(1, "1000".to_string(), 3, 0), (3, "3000".to_string(), 1, 2)];
        let pairing = pair_notes("footnote", &existing, &desired).unwrap();
        assert_eq!(pairing[0].1, 1);
        assert_eq!(pairing[1].1, 3);
    }

    #[test]
    fn a_note_anchor_mismatch_is_drift_not_insertion() {
        let existing = vec![row(1, "1000", 3, 0)];
        let desired = vec![(1, "1000".to_string(), 4, 0)];
        assert!(pair_notes("footnote", &existing, &desired).is_err());
    }

    #[test]
    fn a_note_added_to_an_existing_node_cannot_ride_an_insertion() {
        let existing = vec![row(1, "1000", 3, 0)];
        let desired = vec![(1, "1000".to_string(), 3, 0), (2, "1000".to_string(), 5, 0)];
        let err = pair_notes("footnote", &existing, &desired).unwrap_err();
        assert!(err.contains("separate run"), "{err}");
    }

    #[test]
    fn blocks_renumber_by_stable_position_identity() {
        let id = Uuid::new_v4();
        let existing = vec![(id, "3000".to_string(), 0_i16, Some(1), None)];
        let desired: DesiredNumbering = [(("3000".to_string(), 0_i16), (Some(4), None))].into();
        let plan = plan_block_renumber(&existing, &desired).unwrap();
        assert_eq!(plan, vec![(id, Some(4), None)]);
    }

    #[test]
    fn a_missing_desired_block_is_a_removal_and_aborts() {
        let existing = vec![(Uuid::new_v4(), "3000".to_string(), 0_i16, Some(1), None)];
        let desired: DesiredNumbering = HashMap::new();
        assert!(plan_block_renumber(&existing, &desired).is_err());
    }

    #[test]
    fn a_numbering_kind_change_aborts() {
        // A numbered paragraph must not silently become a figure.
        let existing = vec![(Uuid::new_v4(), "3000".to_string(), 0_i16, Some(1), None)];
        let desired: DesiredNumbering = [(("3000".to_string(), 0_i16), (None, Some(1)))].into();
        assert!(plan_block_renumber(&existing, &desired).is_err());
    }
}
