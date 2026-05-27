// ---------------------------------------------------------------------------
// Authoritative English TOC for Kant's KrV B-edition (Akademie-Ausgabe Band III)
// ---------------------------------------------------------------------------
//
// Each entry: (aa_page, depth, label)
// English translations of the German TOC labels.
// Must be in exact 1:1 correspondence with toc.rs entries.

struct FlatEntry {
    aa_page: u16,
    depth: u16,
    label: &'static str,
    slug_override: Option<&'static str>,
}

const TOC: &[FlatEntry] = &[
    // -----------------------------------------------------------------------
    // Front matter
    // -----------------------------------------------------------------------
    FlatEntry {
        aa_page: 2,
        depth: 1,
        label: "Motto",
        slug_override: None,
    },
    FlatEntry {
        aa_page: 3,
        depth: 1,
        label: "Dedication",
        slug_override: None,
    },
    FlatEntry {
        aa_page: 7,
        depth: 1,
        label: "Preface to the Second Edition",
        slug_override: None,
    },
    // -----------------------------------------------------------------------
    // Introduction
    // -----------------------------------------------------------------------
    FlatEntry {
        aa_page: 27,
        depth: 1,
        label: "Introduction",
        slug_override: None,
    },
    FlatEntry {
        aa_page: 27,
        depth: 2,
        label: "I. Of the difference between pure and empirical cognition",
        slug_override: None,
    },
    FlatEntry {
        aa_page: 28,
        depth: 2,
        label: "II. We are in possession of certain _a priori_ cognitions, and even the common understanding is never without them",
        slug_override: None,
    },
    FlatEntry {
        aa_page: 30,
        depth: 2,
        label: "III. Philosophy stands in need of a science which shall determine the possibility, the principles, and the extent of all _a priori_ cognition",
        slug_override: None,
    },
    FlatEntry {
        aa_page: 33,
        depth: 2,
        label: "IV. Of the difference between analytic and synthetic judgments",
        slug_override: None,
    },
    FlatEntry {
        aa_page: 36,
        depth: 2,
        label: "V. In all theoretical sciences of reason synthetic _a priori_ judgments are contained as principles",
        slug_override: None,
    },
    FlatEntry {
        aa_page: 39,
        depth: 2,
        label: "VI. The general problem of pure reason",
        slug_override: None,
    },
    FlatEntry {
        aa_page: 42,
        depth: 2,
        label: "VII. Idea and division of a special science, under the title of a Critique of Pure Reason",
        slug_override: None,
    },
    // -----------------------------------------------------------------------
    // I. Transcendental Doctrine of Elements
    // -----------------------------------------------------------------------
    FlatEntry {
        aa_page: 49,
        depth: 1,
        label: "I. Transcendental Doctrine of Elements",
        slug_override: None,
    },
    // -- First Part: Transcendental Aesthetic --
    FlatEntry {
        aa_page: 49,
        depth: 2,
        label: "First Part. Transcendental Aesthetic",
        slug_override: None,
    },
    FlatEntry {
        aa_page: 49,
        depth: 3,
        label: "Introduction",
        slug_override: Some("introduction-aesthetic"),
    },
    FlatEntry {
        aa_page: 51,
        depth: 3,
        label: "Section 1. Of Space",
        slug_override: None,
    },
    FlatEntry {
        aa_page: 57,
        depth: 3,
        label: "Section 2. Of Time",
        slug_override: None,
    },
    FlatEntry {
        aa_page: 65,
        depth: 3,
        label: "General Remarks on the Transcendental Aesthetic",
        slug_override: None,
    },
    FlatEntry {
        aa_page: 73,
        depth: 3,
        label: "Conclusion of the Transcendental Aesthetic",
        slug_override: None,
    },
    // -- Second Part: Transcendental Logic --
    FlatEntry {
        aa_page: 74,
        depth: 2,
        label: "Second Part. Transcendental Logic",
        slug_override: None,
    },
    // Introduction to Transcendental Logic
    FlatEntry {
        aa_page: 74,
        depth: 3,
        label: "Introduction. Idea of a Transcendental Logic",
        slug_override: None,
    },
    FlatEntry {
        aa_page: 74,
        depth: 4,
        label: "I. Of Logic in General",
        slug_override: None,
    },
    FlatEntry {
        aa_page: 77,
        depth: 4,
        label: "II. Of Transcendental Logic",
        slug_override: None,
    },
    FlatEntry {
        aa_page: 79,
        depth: 4,
        label: "III. Of the Division of General Logic into Analytic and Dialectic",
        slug_override: None,
    },
    FlatEntry {
        aa_page: 81,
        depth: 4,
        label: "IV. Of the Division of Transcendental Logic into Transcendental Analytic and Dialectic",
        slug_override: None,
    },
    // -- First Division: Transcendental Analytic --
    FlatEntry {
        aa_page: 83,
        depth: 3,
        label: "First Division. Transcendental Analytic",
        slug_override: None,
    },
    // Book I: Analytic of Concepts
    FlatEntry {
        aa_page: 83,
        depth: 4,
        label: "Book I. Analytic of Concepts",
        slug_override: None,
    },
    FlatEntry {
        aa_page: 84,
        depth: 5,
        label: "Chapter 1. Of the clue to the discovery of all pure concepts of the understanding",
        slug_override: None,
    },
    FlatEntry {
        aa_page: 85,
        depth: 6,
        label: "Section 1. Of the logical use of the understanding in general",
        slug_override: None,
    },
    FlatEntry {
        aa_page: 86,
        depth: 6,
        label: "Section 2. Of the logical function of the understanding in judgments. \u{00A7}9",
        slug_override: None,
    },
    FlatEntry {
        aa_page: 90,
        depth: 6,
        label: "Section 3. Of the pure concepts of the understanding, or categories. \u{00A7}10\u{2013}12",
        slug_override: None,
    },
    FlatEntry {
        aa_page: 99,
        depth: 5,
        label: "Chapter 2. Of the Deduction of the Pure Concepts of Understanding",
        slug_override: None,
    },
    FlatEntry {
        aa_page: 99,
        depth: 6,
        label: "Section 1. Of the principles of a transcendental deduction in general. \u{00A7}13",
        slug_override: None,
    },
    FlatEntry {
        aa_page: 104,
        depth: 6,
        label: "Transition to the transcendental deduction of the categories. \u{00A7}14",
        slug_override: None,
    },
    FlatEntry {
        aa_page: 107,
        depth: 6,
        label: "Section 2. Transcendental deduction of the pure concepts of the understanding. \u{00A7}15\u{2013}27",
        slug_override: None,
    },
    // Book II: Analytic of Principles
    FlatEntry {
        aa_page: 130,
        depth: 4,
        label: "Book II. Analytic of Principles",
        slug_override: None,
    },
    FlatEntry {
        aa_page: 131,
        depth: 5,
        label: "Introduction. Of the transcendental power of judgment in general",
        slug_override: None,
    },
    FlatEntry {
        aa_page: 133,
        depth: 5,
        label: "Chapter 1. Of the schematism of the pure concepts of the understanding",
        slug_override: None,
    },
    FlatEntry {
        aa_page: 140,
        depth: 5,
        label: "Chapter 2. System of all principles of pure understanding",
        slug_override: None,
    },
    FlatEntry {
        aa_page: 141,
        depth: 6,
        label: "Section 1. Of the highest principle of all analytic judgments",
        slug_override: None,
    },
    FlatEntry {
        aa_page: 143,
        depth: 6,
        label: "Section 2. Of the highest principle of all synthetic judgments",
        slug_override: None,
    },
    FlatEntry {
        aa_page: 146,
        depth: 6,
        label: "Section 3. Systematic representation of all synthetic principles of pure understanding",
        slug_override: None,
    },
    FlatEntry {
        aa_page: 148,
        depth: 7,
        label: "1. Axioms of Intuition",
        slug_override: None,
    },
    FlatEntry {
        aa_page: 151,
        depth: 7,
        label: "2. Anticipations of Perception",
        slug_override: None,
    },
    FlatEntry {
        aa_page: 158,
        depth: 7,
        label: "3. Analogies of Experience",
        slug_override: None,
    },
    FlatEntry {
        aa_page: 162,
        depth: 8,
        label: "First Analogy. Principle of the permanence of substance",
        slug_override: None,
    },
    FlatEntry {
        aa_page: 166,
        depth: 8,
        label: "Second Analogy. Principle of succession in time in accordance with the law of causality",
        slug_override: None,
    },
    FlatEntry {
        aa_page: 180,
        depth: 8,
        label: "Third Analogy. Principle of coexistence in accordance with the law of reciprocity or community",
        slug_override: None,
    },
    FlatEntry {
        aa_page: 185,
        depth: 7,
        label: "4. The postulates of empirical thought in general",
        slug_override: None,
    },
    FlatEntry {
        aa_page: 198,
        depth: 7,
        label: "General remark on the system of principles",
        slug_override: None,
    },
    FlatEntry {
        aa_page: 202,
        depth: 5,
        label: "Chapter 3. On the ground of the distinction of all objects in general into phenomena and noumena",
        slug_override: None,
    },
    FlatEntry {
        aa_page: 214,
        depth: 5,
        label: "Appendix. Of the amphiboly of concepts of reflection",
        slug_override: None,
    },
    // -- Second Division: Transcendental Dialectic --
    FlatEntry {
        aa_page: 234,
        depth: 3,
        label: "Second Division. Transcendental Dialectic",
        slug_override: None,
    },
    // Introduction to Dialectic
    FlatEntry {
        aa_page: 234,
        depth: 4,
        label: "Introduction",
        slug_override: Some("introduction-dialectic"),
    },
    FlatEntry {
        aa_page: 234,
        depth: 5,
        label: "I. Of transcendental illusion",
        slug_override: None,
    },
    FlatEntry {
        aa_page: 237,
        depth: 5,
        label: "II. Of pure reason as the seat of transcendental illusion",
        slug_override: None,
    },
    FlatEntry {
        aa_page: 237,
        depth: 6,
        label: "Of reason in general",
        slug_override: None,
    },
    FlatEntry {
        aa_page: 240,
        depth: 6,
        label: "Of the logical use of reason",
        slug_override: None,
    },
    FlatEntry {
        aa_page: 241,
        depth: 6,
        label: "Of the pure use of reason",
        slug_override: None,
    },
    // Book I of Dialectic
    FlatEntry {
        aa_page: 244,
        depth: 4,
        label: "Book I. Of the Concepts of Pure Reason",
        slug_override: None,
    },
    FlatEntry {
        aa_page: 245,
        depth: 5,
        label: "Section 1. Of ideas in general",
        slug_override: None,
    },
    FlatEntry {
        aa_page: 250,
        depth: 5,
        label: "Section 2. Of transcendental ideas",
        slug_override: None,
    },
    FlatEntry {
        aa_page: 257,
        depth: 5,
        label: "Section 3. System of transcendental ideas",
        slug_override: None,
    },
    // Book II of Dialectic
    FlatEntry {
        aa_page: 261,
        depth: 4,
        label: "Book II. Of the Dialectical Inferences of Pure Reason",
        slug_override: None,
    },
    FlatEntry {
        aa_page: 262,
        depth: 5,
        label: "Chapter 1. Of the Paralogisms of Pure Reason",
        slug_override: None,
    },
    FlatEntry {
        aa_page: 262,
        depth: 5,
        label: "General remark concerning the transition from rational psychology to cosmology",
        slug_override: None,
    },
    FlatEntry {
        aa_page: 281,
        depth: 5,
        label: "Chapter 2. The Antinomy of Pure Reason",
        slug_override: None,
    },
    FlatEntry {
        aa_page: 283,
        depth: 6,
        label: "Section 1. System of cosmological ideas",
        slug_override: None,
    },
    FlatEntry {
        aa_page: 290,
        depth: 6,
        label: "Section 2. Antithetic of pure reason",
        slug_override: None,
    },
    FlatEntry {
        aa_page: 294,
        depth: 7,
        label: "First Antinomy",
        slug_override: None,
    },
    FlatEntry {
        aa_page: 300,
        depth: 7,
        label: "Second Antinomy",
        slug_override: None,
    },
    FlatEntry {
        aa_page: 308,
        depth: 7,
        label: "Third Antinomy",
        slug_override: None,
    },
    FlatEntry {
        aa_page: 314,
        depth: 7,
        label: "Fourth Antinomy",
        slug_override: None,
    },
    FlatEntry {
        aa_page: 322,
        depth: 6,
        label: "Section 3. Of the interest of reason in these its conflicts",
        slug_override: None,
    },
    FlatEntry {
        aa_page: 330,
        depth: 6,
        label: "Section 4. Of the transcendental problems of pure reason, in so far as they absolutely must be capable of a solution",
        slug_override: None,
    },
    FlatEntry {
        aa_page: 335,
        depth: 6,
        label: "Section 5. Skeptical representation of the cosmological questions through all four transcendental ideas",
        slug_override: None,
    },
    FlatEntry {
        aa_page: 338,
        depth: 6,
        label: "Section 6. Transcendental idealism as the key to the solution of the cosmological dialectic",
        slug_override: None,
    },
    FlatEntry {
        aa_page: 342,
        depth: 6,
        label: "Section 7. Critical decision of the cosmological conflict of reason with itself",
        slug_override: None,
    },
    FlatEntry {
        aa_page: 348,
        depth: 6,
        label: "Section 8. Regulative principle of pure reason in respect of the cosmological ideas",
        slug_override: None,
    },
    FlatEntry {
        aa_page: 353,
        depth: 6,
        label: "Section 9. Of the empirical use of the regulative principle of reason, in respect of all cosmological ideas",
        slug_override: None,
    },
    FlatEntry {
        aa_page: 354,
        depth: 7,
        label: "I. Solution of the cosmological idea of the totality of the composition of the appearances of a cosmic whole",
        slug_override: None,
    },
    FlatEntry {
        aa_page: 357,
        depth: 7,
        label: "II. Solution of the cosmological idea of the totality of the division of a given whole in intuition",
        slug_override: None,
    },
    FlatEntry {
        aa_page: 360,
        depth: 7,
        label: "Concluding remark on the solution of the mathematical-transcendental ideas, and preliminary remark on the solution of the dynamical-transcendental ideas",
        slug_override: None,
    },
    FlatEntry {
        aa_page: 362,
        depth: 7,
        label: "III. Solution of the cosmological ideas of the totality of the derivation of cosmic events from their causes",
        slug_override: None,
    },
    FlatEntry {
        aa_page: 366,
        depth: 7,
        label: "Possibility of causality through freedom, in harmony with the universal law of natural necessity",
        slug_override: None,
    },
    FlatEntry {
        aa_page: 368,
        depth: 7,
        label: "Explanation of the cosmological idea of freedom in connection with universal natural necessity",
        slug_override: None,
    },
    FlatEntry {
        aa_page: 378,
        depth: 7,
        label: "IV. Solution of the cosmological idea of the totality of the dependence of appearances regarding their existence in general",
        slug_override: None,
    },
    FlatEntry {
        aa_page: 381,
        depth: 7,
        label: "Concluding remark on the entire antinomy of pure reason",
        slug_override: None,
    },
    FlatEntry {
        aa_page: 383,
        depth: 5,
        label: "Chapter 3. The Ideal of Pure Reason",
        slug_override: None,
    },
    FlatEntry {
        aa_page: 383,
        depth: 6,
        label: "Section 1. Of the ideal in general",
        slug_override: None,
    },
    FlatEntry {
        aa_page: 385,
        depth: 6,
        label: "Section 2. Of the transcendental ideal (Prototypon transcendentale)",
        slug_override: None,
    },
    FlatEntry {
        aa_page: 392,
        depth: 6,
        label: "Section 3. Of the arguments of speculative reason in inferring the existence of a supreme being",
        slug_override: None,
    },
    FlatEntry {
        aa_page: 397,
        depth: 6,
        label: "Section 4. Of the impossibility of an ontological proof of the existence of God",
        slug_override: None,
    },
    FlatEntry {
        aa_page: 403,
        depth: 6,
        label: "Section 5. Of the impossibility of a cosmological proof of the existence of God",
        slug_override: None,
    },
    FlatEntry {
        aa_page: 410,
        depth: 7,
        label: "Discovery and explanation of the dialectical illusion in all transcendental proofs of the existence of a necessary being",
        slug_override: None,
    },
    FlatEntry {
        aa_page: 413,
        depth: 6,
        label: "Section 6. Of the impossibility of the physico-theological proof",
        slug_override: None,
    },
    FlatEntry {
        aa_page: 420,
        depth: 6,
        label: "Section 7. Critique of all theology based upon speculative principles of reason",
        slug_override: None,
    },
    // Appendix to Transcendental Dialectic
    FlatEntry {
        aa_page: 426,
        depth: 6,
        label: "Appendix to the transcendental dialectic",
        slug_override: None,
    },
    FlatEntry {
        aa_page: 426,
        depth: 7,
        label: "Of the regulative use of the ideas of pure reason",
        slug_override: None,
    },
    FlatEntry {
        aa_page: 442,
        depth: 7,
        label: "Of the ultimate end of the natural dialectic of human reason",
        slug_override: None,
    },
    // -----------------------------------------------------------------------
    // II. Transcendental Doctrine of Method
    // -----------------------------------------------------------------------
    FlatEntry {
        aa_page: 465,
        depth: 1,
        label: "II. Transcendental Doctrine of Method",
        slug_override: None,
    },
    FlatEntry {
        aa_page: 466,
        depth: 2,
        label: "Introduction",
        slug_override: Some("introduction-methodology"),
    },
    FlatEntry {
        aa_page: 466,
        depth: 2,
        label: "Chapter 1. The Discipline of Pure Reason",
        slug_override: None,
    },
    FlatEntry {
        aa_page: 468,
        depth: 3,
        label: "Section 1. The discipline of pure reason in its dogmatic use",
        slug_override: None,
    },
    FlatEntry {
        aa_page: 484,
        depth: 3,
        label: "Section 2. The discipline of pure reason in respect of its polemical use",
        slug_override: None,
    },
    FlatEntry {
        aa_page: 495,
        depth: 4,
        label: "Of the impossibility of a skeptical satisfaction of pure reason in conflict with itself",
        slug_override: None,
    },
    FlatEntry {
        aa_page: 502,
        depth: 3,
        label: "Section 3. The discipline of pure reason in respect of hypotheses",
        slug_override: None,
    },
    FlatEntry {
        aa_page: 509,
        depth: 3,
        label: "Section 4. The discipline of pure reason in respect of its proofs",
        slug_override: None,
    },
    FlatEntry {
        aa_page: 517,
        depth: 2,
        label: "Chapter 2. The Canon of Pure Reason",
        slug_override: None,
    },
    FlatEntry {
        aa_page: 518,
        depth: 3,
        label: "Section 1. Of the ultimate end of the pure use of our reason",
        slug_override: None,
    },
    FlatEntry {
        aa_page: 522,
        depth: 3,
        label: "Section 2. Of the ideal of the highest good, as a determining ground of the ultimate end of pure reason",
        slug_override: None,
    },
    FlatEntry {
        aa_page: 531,
        depth: 3,
        label: "Section 3. Of opining, knowing, and believing",
        slug_override: None,
    },
    FlatEntry {
        aa_page: 538,
        depth: 2,
        label: "Chapter 3. The Architectonic of Pure Reason",
        slug_override: None,
    },
    FlatEntry {
        aa_page: 550,
        depth: 2,
        label: "Chapter 4. The History of Pure Reason",
        slug_override: None,
    },
];

/// Return the flat English TOC entries in document order.
/// Each entry: (index_in_flat_list, aa_page, depth, label)
pub fn flat_toc_entries_en() -> Vec<(usize, u16, u16, &'static str, Option<&'static str>)> {
    TOC.iter()
        .enumerate()
        .map(|(i, e)| (i, e.aa_page, e.depth, e.label, e.slug_override))
        .collect()
}

/// Return the total number of English TOC entries.
pub fn toc_en_len() -> usize {
    TOC.len()
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::kant1::toc;

    #[test]
    fn test_en_entry_count() {
        assert_eq!(toc_en_len(), 113);
    }

    #[test]
    fn test_en_matches_de_count() {
        assert_eq!(toc_en_len(), toc::toc_len());
    }

    #[test]
    fn test_en_matches_de_structure() {
        let de = toc::flat_toc_entries();
        let en = flat_toc_entries_en();
        for (d, e) in de.iter().zip(en.iter()) {
            assert_eq!(d.0, e.0, "index mismatch");
            assert_eq!(d.1, e.1, "aa_page mismatch at index {}", d.0);
            assert_eq!(d.2, e.2, "depth mismatch at index {}", d.0);
        }
    }

    #[test]
    fn test_en_first_entries() {
        let flat = flat_toc_entries_en();
        assert_eq!(flat[0], (0, 2, 1, "Motto", None));
        assert_eq!(flat[1], (1, 3, 1, "Dedication", None));
        assert_eq!(flat[2], (2, 7, 1, "Preface to the Second Edition", None));
    }

    #[test]
    fn test_en_last_entry() {
        let flat = flat_toc_entries_en();
        assert_eq!(
            flat.last().unwrap().3,
            "Chapter 4. The History of Pure Reason"
        );
    }
}
