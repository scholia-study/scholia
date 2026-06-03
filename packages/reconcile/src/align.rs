//! Text-based alignment of a unit's existing sentence rows against the desired
//! sentence texts. A "unit" is whatever the caller reconciles atomically — a
//! paragraph, a footnote, or a Bible verse. Identity is anchored to the unit: a
//! split/merge only reshuffles ordinals inside the one affected unit, so the
//! aligner just needs each unit's old rows and new texts.

use common::textmatch::{concat_similarity, normalized_eq, similarity};
use uuid::Uuid;

/// Concatenation similarity required to accept a split/merge hypothesis.
const SPLIT_MERGE_MIN_SIM: f64 = 0.90;
/// Below this, a same-count sentence is too different to be a plausible
/// in-place edit — treat as a scrambled unit and abort.
const SCRAMBLE_MIN_SIM: f64 = 0.25;

/// An existing sentence row in the unit being reconciled.
pub struct Existing {
    pub id: Uuid,
    pub text: String,
}

/// Per-unit alignment outcome.
pub struct BlockPlan {
    /// One entry per desired sentence (in order): `Some(id)` reuses an existing
    /// row, `None` inserts a fresh one.
    pub assignment: Vec<Option<Uuid>>,
    /// Existing rows to retire: `(retired_id, survivor)`. `Some(survivor)` is a
    /// merge (migrate dependents onto the survivor); `None` is a plain delete.
    pub retired: Vec<(Uuid, Option<Uuid>)>,
    /// `(first_half_id, desired_index_of_second_half)` for each split — used to
    /// extend single-sentence quotations onto the new second half.
    pub splits: Vec<(Uuid, usize)>,
}

fn trunc(s: &str) -> String {
    let t: String = s.chars().take(50).collect();
    if s.chars().count() > 50 {
        format!("{t}…")
    } else {
        t
    }
}

/// Align one unit's existing rows against the desired sentence texts. Returns an
/// error (caller aborts the whole run) for any ambiguous change. `label`
/// identifies the unit in error messages (e.g. `"node 001 / block 2"`,
/// `"footnote 7"`, `"romans:14:24"`).
pub fn plan_block(label: &str, old: &[Existing], new: &[&str]) -> Result<BlockPlan, String> {
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
