//! Authoritative TOC for plato2. Each entry is `(stephanus, depth, label)`:
//! depth 0 is one of the three conversations, depth 1 a Scholia division within
//! it. The `stephanus` value is the section **in force** where the division
//! opens — not necessarily a section whose milestone sits at that exact point,
//! since a section opened mid-speech has its `{{{ ... }}}` marker emitted in
//! the preceding division's file.
//!
//! Every anchor is the section in force at a genuine (non-`rend="merge"`)
//! speech turn, so a division never opens in the middle of a speech.

use crate::drama::Node;

use super::filenames;

pub struct FlatEntry {
    pub stephanus: &'static str,
    pub depth: u16,
    pub label: &'static str,
}

pub const TOC: &[FlatEntry] = &[
    // The Conversation with Gorgias — from 447a
    FlatEntry {
        stephanus: "447a",
        depth: 0,
        label: "The Conversation with Gorgias",
    },
    FlatEntry {
        stephanus: "447a",
        depth: 1,
        label: "Arrival, and Polus Praises without Defining",
    },
    FlatEntry {
        stephanus: "449b",
        depth: 1,
        label: "Rhetoric Defined, and Tested Against the Other Crafts",
    },
    FlatEntry {
        stephanus: "452e",
        depth: 1,
        label: "Persuasion Is Belief, Not Knowledge",
    },
    FlatEntry {
        stephanus: "456a",
        depth: 1,
        label: "Gorgias Boasts of Rhetoric's Power, and Contradicts Himself",
    },
    // The Conversation with Polus — from 461b
    FlatEntry {
        stephanus: "461b",
        depth: 0,
        label: "The Conversation with Polus",
    },
    FlatEntry {
        stephanus: "461b",
        depth: 1,
        label: "Rhetoric as Flattery, Not a Craft",
    },
    FlatEntry {
        stephanus: "466b",
        depth: 1,
        label: "Tyrants and Orators Have the Least Power",
    },
    FlatEntry {
        stephanus: "469c",
        depth: 1,
        label: "Better to Suffer Than to Do Wrong: The Paradox and Archelaus",
    },
    FlatEntry {
        stephanus: "472e",
        depth: 1,
        label: "Proving Injustice Worse Than Suffering It",
    },
    FlatEntry {
        stephanus: "476b",
        depth: 1,
        label: "Punishment as Medicine for the Soul",
    },
    // The Conversation with Callicles — from 481b
    FlatEntry {
        stephanus: "481b",
        depth: 0,
        label: "The Conversation with Callicles",
    },
    FlatEntry {
        stephanus: "481b",
        depth: 1,
        label: "Callicles Interrupts: Nature Against Convention",
    },
    FlatEntry {
        stephanus: "486d",
        depth: 1,
        label: "The Many Prove Collectively Stronger Than the One",
    },
    FlatEntry {
        stephanus: "491e",
        depth: 1,
        label: "Hedonism Declared, and the Coward's Equal Pleasure",
    },
    FlatEntry {
        stephanus: "497c",
        depth: 1,
        label: "The Choice of Lives: Philosophy or Flattery of the Crowd",
    },
    FlatEntry {
        stephanus: "501d",
        depth: 1,
        label: "What a Soul-Improving Statesman Would Look Like",
    },
    FlatEntry {
        stephanus: "505e",
        depth: 1,
        label: "The Argument's Bonds of Iron and Adamant",
    },
    FlatEntry {
        stephanus: "510a",
        depth: 1,
        label: "Currying a Tyrant's Favour, and the Craft That Saves Lives",
    },
    FlatEntry {
        stephanus: "515a",
        depth: 1,
        label: "Did the Great Statesmen Make Athens Better, or Only Bigger?",
    },
    FlatEntry {
        stephanus: "519e",
        depth: 1,
        label: "The True Political Craft, and a Trial Foreseen",
    },
    FlatEntry {
        stephanus: "523a",
        depth: 1,
        label: "The Myth of Judgment",
    },
];

/// Sort key for a Stephanus reference: page, then section letter.
pub fn stephanus_key(reference: &str) -> (u32, u32) {
    let digits: String = reference.chars().take_while(char::is_ascii_digit).collect();
    let letter = reference[digits.len()..].chars().next();
    (
        digits.parse().unwrap_or(0),
        letter.map_or(0, |c| c as u32 - 'a' as u32),
    )
}

/// The curated file set, derived from `TOC` so the table is the only place a
/// division is declared. `source_ref` is the position string, not a slug: the
/// division titles are Scholia's editorial apparatus and may be reworded, and
/// the reconciler matches nodes on `source_ref`.
pub fn nodes() -> Vec<Node> {
    let names = filenames::all_filenames();
    let mut out = Vec::with_capacity(TOC.len());
    let mut parent: Option<String> = None;
    let mut parent_slug = String::new();

    for (idx, entry) in TOC.iter().enumerate() {
        let position = filenames::position_number(idx);
        let source_ref = format!("{position:03}");
        // A conversation's slug is the interlocutor alone — it is a URL
        // segment and an ltree label, and `the_conversation_with_gorgias`
        // earns nothing over `gorgias` in either.
        let slug = if entry.depth == 0 {
            filenames::conversation_slug(entry.label)
        } else {
            filenames::slugify(entry.label)
        };

        let (path, parent_source_ref) = if entry.depth == 0 {
            parent = Some(source_ref.clone());
            parent_slug = slug.clone();
            (slug.clone(), None)
        } else {
            (format!("{parent_slug}.{slug}"), parent.clone())
        };

        out.push(Node {
            source_ref,
            slug,
            path,
            depth: entry.depth as i16,
            parent_source_ref,
            filename: names[idx].1.clone(),
            position: position as u32,
        });
    }
    out
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn stephanus_key_orders_within_and_across_pages() {
        assert!(stephanus_key("447a") < stephanus_key("447b"));
        assert!(stephanus_key("447d") < stephanus_key("448a"));
        assert!(stephanus_key("499e") < stephanus_key("500a"));
    }

    #[test]
    fn table_is_not_empty() {
        assert!(!TOC.is_empty(), "the division table has not been filled in");
    }

    #[test]
    fn anchors_are_strictly_ascending() {
        for (a, b) in TOC.iter().zip(TOC.iter().skip(1)) {
            // A conversation and its first division open at the same place.
            if a.depth == 0 && b.depth == 1 {
                assert_eq!(a.stephanus, b.stephanus, "{} vs {}", a.label, b.label);
                continue;
            }
            assert!(
                stephanus_key(a.stephanus) < stephanus_key(b.stephanus),
                "{} ({}) does not precede {} ({})",
                a.label,
                a.stephanus,
                b.label,
                b.stephanus
            );
        }
    }

    #[test]
    fn every_division_sits_under_a_conversation() {
        assert_eq!(TOC[0].depth, 0, "the table must open with a conversation");
        let mut seen_conversation = false;
        let mut children = 0;
        for entry in TOC {
            if entry.depth == 0 {
                if seen_conversation {
                    assert!(children > 0, "a conversation has no divisions");
                }
                seen_conversation = true;
                children = 0;
            } else {
                children += 1;
            }
        }
        assert!(children > 0, "the last conversation has no divisions");
    }

    #[test]
    fn there_are_three_conversations() {
        assert_eq!(TOC.iter().filter(|e| e.depth == 0).count(), 3);
    }

    #[test]
    fn labels_are_unique() {
        let mut v: Vec<&str> = TOC.iter().map(|e| e.label).collect();
        v.sort_unstable();
        let n = v.len();
        v.dedup();
        assert_eq!(v.len(), n, "duplicate TOC label");
    }

    #[test]
    fn conversation_nodes_are_slugged_by_interlocutor() {
        let conversations: Vec<String> = nodes()
            .into_iter()
            .filter(|n| n.depth == 0)
            .map(|n| n.slug)
            .collect();
        assert_eq!(conversations, ["gorgias", "polus", "callicles"]);
    }

    #[test]
    fn nodes_parent_every_division_and_none_of_the_conversations() {
        for node in nodes() {
            match node.depth {
                0 => assert!(node.parent_source_ref.is_none(), "{}", node.slug),
                _ => {
                    assert!(node.parent_source_ref.is_some(), "{}", node.slug);
                    assert!(node.path.contains('.'), "flat path: {}", node.path);
                }
            }
        }
    }
}
