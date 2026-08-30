//! Curated filename roster + slug rules for peirce1.
//!
//! Unlike the fixed-extent corpora, this edition is expected to GROW: papers
//! get added as transcriptions become available (the 1905–06 pragmaticism
//! papers are the known next candidates, and belong between 15000 and 16000).
//!
//! That makes `position_number` load-bearing. It feeds `source_ref`, which is
//! the key `struct_to_db` reconciles sentence UUIDs through — and anchored
//! quotations ride on those UUIDs. A positional `flat_index + 1` would shift
//! every later paper's `source_ref` the moment one is inserted, silently
//! breaking every quotation after the insertion point. So the numbers are
//! assigned from a table instead, spaced a thousand apart.
//!
//! Rules for adding a paper: give it an unused number in its date position,
//! and never renumber or reuse an existing one.
//!
//! Stable numbering is necessary but NOT sufficient, so there is a second rule:
//! **curate in chronological order, so every import appends.** `paragraph_number`
//! and footnote numbers are book-global counters, and inserting a paper ahead of
//! one already imported shifts them, which the reconciler rejects outright
//! (`reconcile::orchestrate::classify_added_block_positions`) — the only remedy
//! being a full reload, which re-mints sentence UUIDs and breaks anchored
//! quotations. Appending is verified safe; inserting is not currently possible.
//! Lifting that restriction means teaching the reconciler to accept renumbering
//! explained by an inserted node, which is a change to shared code and has not
//! been made.

pub const FILENAMES: &[&str] = &[
    "01000_on_a_new_list_of_categories.md",
    "02000_questions_concerning_certain_faculties_claimed_for_man.md",
    "03000_some_consequences_of_four_incapacities.md",
    "04000_grounds_of_validity_of_the_laws_of_logic.md",
    "05000_the_fixation_of_belief.md",
    "06000_how_to_make_our_ideas_clear.md",
    "07000_the_doctrine_of_chances.md",
    "08000_the_probability_of_induction.md",
    "09000_the_order_of_nature.md",
    "10000_deduction_induction_and_hypothesis.md",
    "11000_the_architecture_of_theories.md",
    "12000_the_doctrine_of_necessity_examined.md",
    "13000_the_law_of_mind.md",
    "14000_mans_glassy_essence.md",
    "15000_evolutionary_love.md",
    "16000_a_neglected_argument_for_the_reality_of_god.md",
];

/// Permanent position number per paper, parallel to [`FILENAMES`]. Spaced a
/// thousand apart so papers can be inserted without disturbing neighbours.
///
/// Filenames zero-pad the number to five digits so `ls` sorts in reading
/// order; `source_ref` carries it unpadded ("1000") — the shared `{:03}`
/// format is a minimum width, and widening it would churn every other
/// corpus's live refs.
pub const POSITIONS: &[usize] = &[
    1000, 2000, 3000, 4000, 5000, 6000, 7000, 8000, 9000, 10000, 11000, 12000, 13000, 14000, 15000,
    16000,
];

pub fn all_filenames() -> Vec<(usize, String)> {
    FILENAMES
        .iter()
        .enumerate()
        .map(|(i, f)| (i, f.to_string()))
        .collect()
}

pub fn position_number(flat_index: usize) -> usize {
    POSITIONS[flat_index]
}

/// ASCII slug: lowercased alphanumeric runs joined by `_`, `&` → "and".
pub fn slugify(label: &str) -> String {
    let mut s = String::new();
    let mut last_us = true;
    for ch in label.chars() {
        if ch == '&' {
            if !last_us {
                s.push('_');
            }
            s.push_str("and");
            last_us = false;
        } else if ch.is_ascii_alphanumeric() {
            s.push(ch.to_ascii_lowercase());
            last_us = false;
        } else if !last_us {
            s.push('_');
            last_us = true;
        }
    }
    s.trim_matches('_').to_string()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn every_file_has_a_position() {
        assert_eq!(FILENAMES.len(), POSITIONS.len());
    }

    #[test]
    fn positions_are_unique_and_ascending() {
        assert!(POSITIONS.windows(2).all(|w| w[0] < w[1]));
    }

    #[test]
    fn positions_leave_room_for_insertions() {
        // The 1905-06 pragmaticism papers land between Evolutionary Love
        // (15000) and A Neglected Argument (16000) without renumbering either.
        let evolutionary_love = POSITIONS[14];
        let neglected_argument = POSITIONS[15];
        assert!(neglected_argument - evolutionary_love >= 3);
    }

    #[test]
    fn filename_prefix_matches_the_position() {
        for (i, name) in FILENAMES.iter().enumerate() {
            let prefix: usize = name[..5].parse().expect("filenames open with NNNNN_");
            assert_eq!(prefix, POSITIONS[i], "{name}");
        }
    }
}
