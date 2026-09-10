//! Authoritative TOC for plato1. Each entry is `(stephanus, depth, label)`:
//! depth 0 is one of the ten books, depth 1 a Scholia division within it. The
//! `stephanus` value is the division's opening section — the same value its
//! first `{{{ ... }}}` marker carries.

pub struct FlatEntry {
    pub stephanus: &'static str,
    pub depth: u16,
    pub label: &'static str,
}

pub const TOC: &[FlatEntry] = &[
    // Book I
    FlatEntry {
        stephanus: "327a",
        depth: 0,
        label: "Book I",
    },
    FlatEntry {
        stephanus: "327a",
        depth: 1,
        label: "The Descent to the Piraeus",
    },
    FlatEntry {
        stephanus: "328b",
        depth: 1,
        label: "Cephalus on Old Age",
    },
    FlatEntry {
        stephanus: "329e",
        depth: 1,
        label: "Wealth, and Justice as Telling the Truth and Paying Debts",
    },
    FlatEntry {
        stephanus: "331e",
        depth: 1,
        label: "Simonides: Justice as a Craft, and What It Is Good For",
    },
    FlatEntry {
        stephanus: "334a",
        depth: 1,
        label: "The Guardian Who Is Also a Thief",
    },
    FlatEntry {
        stephanus: "336b",
        depth: 1,
        label: "Thrasymachus Breaks In",
    },
    FlatEntry {
        stephanus: "338c",
        depth: 1,
        label: "Justice Is the Advantage of the Stronger",
    },
    FlatEntry {
        stephanus: "340d",
        depth: 1,
        label: "The Craftsman in the Precise Sense",
    },
    FlatEntry {
        stephanus: "343a",
        depth: 1,
        label: "Thrasymachus' Speech: The Shepherd and the Flock",
    },
    FlatEntry {
        stephanus: "344d",
        depth: 1,
        label: "Whom Rule Serves, and Why the Decent Rule Reluctantly",
    },
    FlatEntry {
        stephanus: "348a",
        depth: 1,
        label: "Is Injustice Virtue and Wisdom?",
    },
    FlatEntry {
        stephanus: "350d",
        depth: 1,
        label: "Injustice Breeds Faction",
    },
    FlatEntry {
        stephanus: "352d",
        depth: 1,
        label: "The Work of the Soul, and an Inconclusive End",
    },
    // Book II
    FlatEntry {
        stephanus: "357a",
        depth: 0,
        label: "Book II",
    },
    FlatEntry {
        stephanus: "357a",
        depth: 1,
        label: "Glaucon Renews the Argument: Three Kinds of Good",
    },
    FlatEntry {
        stephanus: "358e",
        depth: 1,
        label: "The Origin of Justice as a Compact, and the Ring of Gyges",
    },
    FlatEntry {
        stephanus: "360e",
        depth: 1,
        label: "Two Portraits: The Perfectly Unjust and the Perfectly Just Man",
    },
    FlatEntry {
        stephanus: "362d",
        depth: 1,
        label: "Adeimantus: What Fathers and Poets Praise",
    },
    FlatEntry {
        stephanus: "368a",
        depth: 1,
        label: "The City as the Soul Writ Large",
    },
    FlatEntry {
        stephanus: "369b",
        depth: 1,
        label: "The First City and Its Needs",
    },
    FlatEntry {
        stephanus: "372e",
        depth: 1,
        label: "The Feverish City, and the Need for Guardians",
    },
    FlatEntry {
        stephanus: "375a",
        depth: 1,
        label: "The Guardian's Nature: Spirited and Philosophic",
    },
    FlatEntry {
        stephanus: "376c",
        depth: 1,
        label: "Education Begins: The Tales We Tell",
    },
    FlatEntry {
        stephanus: "379a",
        depth: 1,
        label: "The First Pattern: God Is Good, and Cause Only of Good",
    },
    FlatEntry {
        stephanus: "380d",
        depth: 1,
        label: "The Second Pattern: God Does Not Change or Deceive",
    },
    // Book III
    FlatEntry {
        stephanus: "386a",
        depth: 0,
        label: "Book III",
    },
    FlatEntry {
        stephanus: "386a",
        depth: 1,
        label: "Tales of Hades, and Grief",
    },
    FlatEntry {
        stephanus: "389a",
        depth: 1,
        label: "Truthfulness, Moderation, and What May Be Said of Men",
    },
    FlatEntry {
        stephanus: "392c",
        depth: 1,
        label: "Diction: Narrative, Imitation, and the Mixed Kind",
    },
    FlatEntry {
        stephanus: "394e",
        depth: 1,
        label: "Whether the Guardians May Imitate",
    },
    FlatEntry {
        stephanus: "398c",
        depth: 1,
        label: "Modes and Rhythms",
    },
    FlatEntry {
        stephanus: "400d",
        depth: 1,
        label: "Why Musical Training Matters: Grace, and the Right Love",
    },
    FlatEntry {
        stephanus: "403c",
        depth: 1,
        label: "Gymnastic and the Body",
    },
    FlatEntry {
        stephanus: "406c",
        depth: 1,
        label: "Medicine, Judges, and Lingering Illness",
    },
    FlatEntry {
        stephanus: "410b",
        depth: 1,
        label: "Music and Gymnastic Together Temper the Soul",
    },
    FlatEntry {
        stephanus: "412b",
        depth: 1,
        label: "Selecting the Rulers, and the Noble Lie",
    },
    FlatEntry {
        stephanus: "415d",
        depth: 1,
        label: "The Guardians' Way of Life: Nothing of Their Own",
    },
    // Book IV
    FlatEntry {
        stephanus: "419a",
        depth: 0,
        label: "Book IV",
    },
    FlatEntry {
        stephanus: "419a",
        depth: 1,
        label: "Are the Guardians Happy? The Happiness of the Whole",
    },
    FlatEntry {
        stephanus: "421c",
        depth: 1,
        label: "Wealth, Poverty, and the Right Size of a City",
    },
    FlatEntry {
        stephanus: "423b",
        depth: 1,
        label: "What the Rulers Must Guard: Education as the One Great Thing",
    },
    FlatEntry {
        stephanus: "427a",
        depth: 1,
        label: "The City Is Founded: Where Is Justice?",
    },
    FlatEntry {
        stephanus: "429a",
        depth: 1,
        label: "Courage as the Preservation of a Right Belief",
    },
    FlatEntry {
        stephanus: "430d",
        depth: 1,
        label: "Moderation as Agreement About Who Should Rule",
    },
    FlatEntry {
        stephanus: "432b",
        depth: 1,
        label: "Justice: Doing One's Own",
    },
    FlatEntry {
        stephanus: "434d",
        depth: 1,
        label: "Turning to the Soul: Is It One or Many?",
    },
    FlatEntry {
        stephanus: "436b",
        depth: 1,
        label: "The Principle of Opposites: Reason and Appetite",
    },
    FlatEntry {
        stephanus: "439c",
        depth: 1,
        label: "The Spirited Part",
    },
    FlatEntry {
        stephanus: "441c",
        depth: 1,
        label: "The Virtues in the Soul, and Justice as Health",
    },
    // Book V
    FlatEntry {
        stephanus: "449a",
        depth: 0,
        label: "Book V",
    },
    FlatEntry {
        stephanus: "449a",
        depth: 1,
        label: "Polemarchus Interrupts: The Community of Women and Children",
    },
    FlatEntry {
        stephanus: "451c",
        depth: 1,
        label: "First Wave: The Nature and Education of Women",
    },
    FlatEntry {
        stephanus: "457c",
        depth: 1,
        label: "Second Wave: Wives and Children in Common",
    },
    FlatEntry {
        stephanus: "460a",
        depth: 1,
        label: "The Regulation of Births",
    },
    FlatEntry {
        stephanus: "461e",
        depth: 1,
        label: "All One Family: Kinship, Unity, and the Guardians' Happiness",
    },
    FlatEntry {
        stephanus: "466d",
        depth: 1,
        label: "The Guardians at War, and How Greeks Should Treat Greeks",
    },
    FlatEntry {
        stephanus: "471c",
        depth: 1,
        label: "Is Such a City Possible? The Third Wave",
    },
    FlatEntry {
        stephanus: "474c",
        depth: 1,
        label: "Who Counts as a Philosopher",
    },
    FlatEntry {
        stephanus: "476d",
        depth: 1,
        label: "Knowledge, Opinion, and the Dreamers",
    },
    // Book VI
    FlatEntry {
        stephanus: "484a",
        depth: 0,
        label: "Book VI",
    },
    FlatEntry {
        stephanus: "484a",
        depth: 1,
        label: "The Philosopher's Nature, and Why Such Men Should Rule",
    },
    FlatEntry {
        stephanus: "487b",
        depth: 1,
        label: "Adeimantus Objects: The Ship of State",
    },
    FlatEntry {
        stephanus: "489d",
        depth: 1,
        label: "How Philosophic Natures Are Corrupted",
    },
    FlatEntry {
        stephanus: "492a",
        depth: 1,
        label: "The Great Beast: The Crowd as Educator",
    },
    FlatEntry {
        stephanus: "495c",
        depth: 1,
        label: "Unworthy Suitors of Philosophy",
    },
    FlatEntry {
        stephanus: "497a",
        depth: 1,
        label: "What City Would Suit the Philosopher",
    },
    FlatEntry {
        stephanus: "499a",
        depth: 1,
        label: "Persuading the Many: The Painter of Constitutions",
    },
    FlatEntry {
        stephanus: "502c",
        depth: 1,
        label: "The Greatest Study, and What the Good Is Not",
    },
    FlatEntry {
        stephanus: "506b",
        depth: 1,
        label: "The Sun, Offspring of the Good",
    },
    FlatEntry {
        stephanus: "509d",
        depth: 1,
        label: "The Divided Line",
    },
    // Book VII
    FlatEntry {
        stephanus: "514a",
        depth: 0,
        label: "Book VII",
    },
    FlatEntry {
        stephanus: "514a",
        depth: 1,
        label: "The Cave",
    },
    FlatEntry {
        stephanus: "517b",
        depth: 1,
        label: "Reading the Cave: The Turning of the Soul",
    },
    FlatEntry {
        stephanus: "519b",
        depth: 1,
        label: "Compelling the Philosophers to Go Back Down",
    },
    FlatEntry {
        stephanus: "521c",
        depth: 1,
        label: "What Draws the Soul Upward: Number",
    },
    FlatEntry {
        stephanus: "524d",
        depth: 1,
        label: "The Mathematical Studies: Arithmetic, Geometry, and Solids",
    },
    FlatEntry {
        stephanus: "528e",
        depth: 1,
        label: "Astronomy and Harmonics",
    },
    FlatEntry {
        stephanus: "531d",
        depth: 1,
        label: "Dialectic",
    },
    FlatEntry {
        stephanus: "535a",
        depth: 1,
        label: "Choosing and Testing the Students",
    },
    FlatEntry {
        stephanus: "537d",
        depth: 1,
        label: "The Danger of Dialectic Taken Too Early",
    },
    FlatEntry {
        stephanus: "540d",
        depth: 1,
        label: "How Such a City Could Actually Come to Be",
    },
    // Book VIII
    FlatEntry {
        stephanus: "543a",
        depth: 0,
        label: "Book VIII",
    },
    FlatEntry {
        stephanus: "543a",
        depth: 1,
        label: "Recalling the Argument: Four Defective Constitutions",
    },
    FlatEntry {
        stephanus: "545c",
        depth: 1,
        label: "Timocracy: How It Arises",
    },
    FlatEntry {
        stephanus: "548d",
        depth: 1,
        label: "The Timocratic Man",
    },
    FlatEntry {
        stephanus: "550c",
        depth: 1,
        label: "Oligarchy: How It Arises",
    },
    FlatEntry {
        stephanus: "553a",
        depth: 1,
        label: "The Oligarchic Man",
    },
    FlatEntry {
        stephanus: "555b",
        depth: 1,
        label: "Democracy: How It Arises",
    },
    FlatEntry {
        stephanus: "558c",
        depth: 1,
        label: "The Democratic Man",
    },
    FlatEntry {
        stephanus: "562a",
        depth: 1,
        label: "Freedom Turning Into Slavery",
    },
    FlatEntry {
        stephanus: "565a",
        depth: 1,
        label: "The Protector Becomes a Tyrant",
    },
    FlatEntry {
        stephanus: "566a",
        depth: 1,
        label: "The Tyrant Secures His Power",
    },
    FlatEntry {
        stephanus: "568a",
        depth: 1,
        label: "The People's Slavery",
    },
    // Book IX
    FlatEntry {
        stephanus: "571a",
        depth: 0,
        label: "Book IX",
    },
    FlatEntry {
        stephanus: "571a",
        depth: 1,
        label: "Lawless Desires, and Dreams",
    },
    FlatEntry {
        stephanus: "572b",
        depth: 1,
        label: "How the Tyrannical Man Comes to Be",
    },
    FlatEntry {
        stephanus: "573c",
        depth: 1,
        label: "A Life Ruled by Eros",
    },
    FlatEntry {
        stephanus: "575a",
        depth: 1,
        label: "The Tyrannical Man in Private and in Public",
    },
    FlatEntry {
        stephanus: "576b",
        depth: 1,
        label: "First Proof: The City and the Soul Compared",
    },
    FlatEntry {
        stephanus: "578b",
        depth: 1,
        label: "The Tyrant's Slavery and Misery",
    },
    FlatEntry {
        stephanus: "580d",
        depth: 1,
        label: "Second Proof: Three Parts, Three Pleasures, Three Lives",
    },
    FlatEntry {
        stephanus: "583b",
        depth: 1,
        label: "Third Proof: Pleasure, Pain, and the State Between",
    },
    FlatEntry {
        stephanus: "585a",
        depth: 1,
        label: "Filling the Soul: Which Pleasures Are True",
    },
    FlatEntry {
        stephanus: "587b",
        depth: 1,
        label: "By How Much the King Lives More Pleasantly",
    },
    FlatEntry {
        stephanus: "588b",
        depth: 1,
        label: "Man, Lion, and Beast: The City Laid Up in Heaven",
    },
    // Book X
    FlatEntry {
        stephanus: "595a",
        depth: 0,
        label: "Book X",
    },
    FlatEntry {
        stephanus: "595a",
        depth: 1,
        label: "Imitation: Three Removes From the Truth",
    },
    FlatEntry {
        stephanus: "598d",
        depth: 1,
        label: "The Imitator Knows Nothing",
    },
    FlatEntry {
        stephanus: "602c",
        depth: 1,
        label: "Imitation Appeals to the Inferior Part",
    },
    FlatEntry {
        stephanus: "605c",
        depth: 1,
        label: "The Charm of Poetry, and the Old Quarrel",
    },
    FlatEntry {
        stephanus: "608c",
        depth: 1,
        label: "The Rewards of Justice, and the Immortal Soul",
    },
    FlatEntry {
        stephanus: "611a",
        depth: 1,
        label: "The Soul's True Nature, Like Glaucus",
    },
    FlatEntry {
        stephanus: "612b",
        depth: 1,
        label: "Justice Rewarded, by Gods and by Men",
    },
    FlatEntry {
        stephanus: "614b",
        depth: 1,
        label: "The Myth of Er: The Journey, and the Spindle of Necessity",
    },
    FlatEntry {
        stephanus: "617d",
        depth: 1,
        label: "The Choice of Lives, and the River of Forgetfulness",
    },
];

/// Divisions whose boundary snaps BACKWARD (earlier) rather than forward.
///
/// A sentence straddling a division boundary belongs to whichever division its
/// sense belongs to. Usually it concludes the previous argument, so the default
/// is to snap forward and leave it there. For these seven it OPENS the new
/// division — Republic 517b's straddling sentence is "this image must be
/// applied as a whole…", which is the interpretation of the cave the division
/// is named for — so the boundary moves earlier to take it in.
pub const BACKWARD_SNAP: &[&str] = &[
    "368a", "460a", "495c", "517b", "575a", "585a", "612b",
    // 617d: the source introduces Lachesis' proclamation with an em dash
    // ("mounting a high platform, he said—"), and a paragraph break falls
    // between. Snapping forward would leave Book X division 116 dangling on
    // "said—"; the lead-in belongs with the speech it introduces.
    "617d",
];

pub fn flat_toc_entries() -> Vec<crate::FlatTocEntry> {
    TOC.iter()
        .enumerate()
        .map(|(i, e)| (i, Some(e.stephanus.to_string()), e.depth, e.label, None))
        .collect()
}

pub fn toc_len() -> usize {
    TOC.len()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn ten_books_each_opening_a_depth_zero_node() {
        assert_eq!(TOC.iter().filter(|e| e.depth == 0).count(), 10);
    }

    #[test]
    fn every_book_has_divisions_under_it() {
        let mut seen_book = false;
        let mut kids = 0;
        for e in TOC {
            if e.depth == 0 {
                if seen_book {
                    assert!(kids > 0, "a book node has no divisions beneath it");
                }
                seen_book = true;
                kids = 0;
            } else {
                assert!(seen_book, "a division precedes its book node");
                kids += 1;
            }
        }
        assert!(kids > 0);
    }

    #[test]
    fn labels_are_unique() {
        let mut v: Vec<&str> = TOC.iter().map(|e| e.label).collect();
        v.sort_unstable();
        let n = v.len();
        v.dedup();
        assert_eq!(v.len(), n, "duplicate TOC label");
    }

    /// A book node's Stephanus value must equal its first division's — the
    /// container opens exactly where its first division does.
    #[test]
    fn book_opens_where_its_first_division_opens() {
        for (i, e) in TOC.iter().enumerate() {
            if e.depth == 0 {
                assert_eq!(e.stephanus, TOC[i + 1].stephanus, "book {}", e.label);
            }
        }
    }

    #[test]
    fn every_backward_snap_entry_is_a_real_division() {
        for s in BACKWARD_SNAP {
            assert!(
                TOC.iter().any(|e| e.depth == 1 && e.stephanus == *s),
                "BACKWARD_SNAP entry {s} is not a depth-1 division stephanus value"
            );
        }
    }
}
