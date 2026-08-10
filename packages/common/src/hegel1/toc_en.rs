//! English TOC labels for the Phänomenologie des Geistes translation
//! edition — the kant1 `toc_en` pattern: labels validated against this
//! table, English filenames derived from these labels by flat index.
//!
//! Renderings follow the ruled terminology table
//! (`assets/hegel1/curated/translate_terms.tsv`): mastery/servitude,
//! culture, the matter at hand, the light-essence, the master builder,
//! revealed religion, absolute knowing. Structural punctuation mirrors the
//! German label (semicolons, commas — including the printed `a,` of the
//! Lichtwesen entry) so heading sentence-splitting stays 1:1.

/// English labels, one per `toc` entry, in document order.
pub const LABELS_EN: &[&str] = &[
    "Preface",
    "First Part. Science of the Experience of Consciousness",
    "Introduction",
    "I. Sense-Certainty; or the This and Meaning",
    "II. Perception; or the Thing, and Deception",
    "III. Force and the Understanding, Appearance and Supersensible World",
    "IV. The Truth of Self-Certainty",
    "A. Self-Subsistence and Non-Self-Subsistence of Self-Consciousness; Mastery and Servitude",
    "B. Freedom of Self-Consciousness; Stoicism, Skepticism, and the Unhappy Consciousness",
    "V. Certainty and Truth of Reason",
    "A. Observing Reason",
    "a. Observation of Nature",
    "b. The Observation of Self-Consciousness in its Purity and its Relation to External Actuality; Logical and Psychological Laws",
    "c. Observation of the Relation of Self-Consciousness to its Immediate Actuality; Physiognomy and Phrenology",
    "B. The Actualization of Rational Self-Consciousness through Itself",
    "a. Pleasure and Necessity",
    "b. The Law of the Heart, and the Madness of Self-Conceit",
    "c. Virtue and the Way of the World",
    "C. Individuality, Which to Itself Is Real In and For Itself",
    "a. The Spiritual Animal Kingdom and Deceit, or the Matter at Hand",
    "b. Law-Giving Reason",
    "c. Law-Testing Reason",
    "VI. Spirit",
    "A. True Spirit, Ethical Life",
    "a. The Ethical World, Human and Divine Law, Man and Woman",
    "b. Ethical Action, Human and Divine Knowledge, Guilt and Fate",
    "c. The Condition of Right",
    "B. Self-Estranged Spirit; Culture",
    "I. The World of Self-Estranged Spirit",
    "a. Culture and its Realm of Actuality",
    "b. Faith and Pure Insight",
    "II. The Enlightenment",
    "a. The Struggle of the Enlightenment with Superstition",
    "b. The Truth of the Enlightenment",
    "III. Absolute Freedom and Terror",
    "C. Spirit Certain of Itself. Morality",
    "a. The Moral Worldview",
    "b. Dissemblance",
    "c. Conscience, the Beautiful Soul, Evil and its Forgiveness",
    "VII. Religion",
    "A. Natural Religion",
    "a, The Light-Essence",
    "b. The Plant and the Animal",
    "c. The Master Builder",
    "B. The Religion of Art",
    "a. The Abstract Work of Art",
    "b. The Living Work of Art",
    "c. The Spiritual Work of Art",
    "C. Revealed Religion",
    "VIII. Absolute Knowing",
];

/// Flattened rows for `md_prose_to_struct::corpus` — structural fields from
/// `toc`, labels from this table.
pub fn flat_toc_entries_en() -> Vec<crate::FlatTocEntry> {
    super::toc::entries()
        .iter()
        .enumerate()
        .map(|(i, e)| {
            (
                i,
                e.page.map(|p| p.to_string()),
                e.depth,
                LABELS_EN[i],
                None,
            )
        })
        .collect()
}

#[cfg(test)]
mod tests {
    use super::super::toc;
    use super::*;

    #[test]
    fn one_label_per_toc_entry() {
        assert_eq!(LABELS_EN.len(), toc::entries().len());
    }

    #[test]
    fn lichtwesen_keeps_the_printed_comma() {
        assert!(LABELS_EN[41].starts_with("a,"));
    }
}
