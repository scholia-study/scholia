//! The per-paper source table: where each paper's scanned pages live on
//! Wikisource, and what its original printing calls them.
//!
//! Positions and labels must agree with `common::peirce1::{toc, filenames}` —
//! the parser validates the emitted front matter against those tables, so a
//! drift here fails loudly at `just struct peirce1` rather than silently.
//!
//! Page numbers are NOT derived from the scan index. The djvu-to-printed
//! offset varies within a single volume (Popular Science Monthly vol. 12 maps
//! djvu 11 to page 1, but djvu 300 to page 286, because covers and
//! advertisements sit between issues), so the printed number is read off each
//! page's `{{rh}}` running head instead.

pub struct Paper {
    pub position: usize,
    pub label: &'static str,
    pub filename: &'static str,
    /// Wikisource Index: file backing the scan.
    pub index: &'static str,
    /// Inclusive scan-page range, as the article's `<pages …/>` transclusion
    /// declares it.
    pub from: u32,
    pub to: u32,
    /// Named section to take when the first/last scan page is shared with a
    /// neighbouring article.
    pub from_section: Option<&'static str>,
    pub to_section: Option<&'static str>,
    pub venue: &'static str,
    pub volume: u32,
    /// Printed page the paper opens on — the one page with no running head.
    pub first_page: u32,
}

pub const PAPERS: &[Paper] = &[
    Paper {
        position: 2000,
        label: "Questions Concerning Certain Faculties Claimed for Man",
        filename: "02000_questions_concerning_certain_faculties_claimed_for_man.md",
        index: "Questions concerning certain Faculties claimed for Man.pdf",
        from: 1,
        to: 12,
        from_section: None,
        to_section: None,
        venue: "JSP",
        volume: 2,
        first_page: 103,
    },
    Paper {
        position: 3000,
        label: "Some Consequences of Four Incapacities",
        filename: "03000_some_consequences_of_four_incapacities.md",
        index: "Some Consequences of Four Incapacities.pdf",
        from: 1,
        to: 18,
        from_section: None,
        to_section: None,
        venue: "JSP",
        volume: 2,
        first_page: 140,
    },
    Paper {
        position: 5000,
        label: "The Fixation of Belief",
        filename: "05000_the_fixation_of_belief.md",
        index: "Popular Science Monthly Volume 12.djvu",
        from: 11,
        to: 25,
        from_section: None,
        to_section: Some("E25"),
        venue: "PSM",
        volume: 12,
        first_page: 1,
    },
    Paper {
        position: 6000,
        label: "How to Make Our Ideas Clear",
        filename: "06000_how_to_make_our_ideas_clear.md",
        index: "Popular Science Monthly Volume 12.djvu",
        from: 300,
        to: 316,
        from_section: Some("B300"),
        to_section: Some("E316"),
        venue: "PSM",
        volume: 12,
        first_page: 286,
    },
    Paper {
        position: 7000,
        label: "The Doctrine of Chances",
        filename: "07000_the_doctrine_of_chances.md",
        index: "Popular Science Monthly Volume 12.djvu",
        from: 622,
        to: 633,
        from_section: Some("B622"),
        to_section: None,
        venue: "PSM",
        volume: 12,
        first_page: 604,
    },
    Paper {
        position: 8000,
        label: "The Probability of Induction",
        filename: "08000_the_probability_of_induction.md",
        index: "Popular Science Monthly Volume 12.djvu",
        from: 725,
        to: 738,
        from_section: Some("B725"),
        to_section: None,
        venue: "PSM",
        volume: 12,
        first_page: 705,
    },
    Paper {
        position: 9000,
        label: "The Order of Nature",
        filename: "09000_the_order_of_nature.md",
        index: "Popular Science Monthly Volume 13.djvu",
        from: 215,
        to: 229,
        from_section: Some("B215"),
        to_section: Some("E229"),
        venue: "PSM",
        volume: 13,
        first_page: 203,
    },
    Paper {
        position: 10000,
        label: "Deduction, Induction, and Hypothesis",
        filename: "10000_deduction_induction_and_hypothesis.md",
        index: "Popular Science Monthly Volume 13.djvu",
        from: 486,
        to: 498,
        from_section: None,
        to_section: Some("E498"),
        venue: "PSM",
        volume: 13,
        first_page: 470,
    },
    Paper {
        position: 11000,
        label: "The Architecture of Theories",
        filename: "11000_the_architecture_of_theories.md",
        index: "The Monist Volume 1.djvu",
        from: 176,
        to: 191,
        from_section: None,
        to_section: None,
        venue: "Monist",
        volume: 1,
        first_page: 161,
    },
    Paper {
        position: 12000,
        label: "The Doctrine of Necessity Examined",
        filename: "12000_the_doctrine_of_necessity_examined.md",
        index: "The Monist Volume 2.djvu",
        from: 333,
        to: 349,
        from_section: None,
        to_section: None,
        venue: "Monist",
        volume: 2,
        first_page: 321,
    },
    Paper {
        position: 13000,
        label: "The Law of Mind",
        filename: "13000_the_law_of_mind.md",
        index: "The Monist Volume 2.djvu",
        from: 545,
        to: 571,
        from_section: None,
        to_section: None,
        venue: "Monist",
        volume: 2,
        first_page: 533,
    },
    Paper {
        position: 14000,
        label: "Man's Glassy Essence",
        filename: "14000_mans_glassy_essence.md",
        index: "The Monist Volume 3.djvu",
        from: 14,
        to: 35,
        from_section: None,
        to_section: None,
        venue: "Monist",
        volume: 3,
        first_page: 1,
    },
    Paper {
        position: 15000,
        label: "Evolutionary Love",
        filename: "15000_evolutionary_love.md",
        index: "The Monist Volume 3.djvu",
        from: 199,
        to: 223,
        from_section: None,
        to_section: None,
        venue: "Monist",
        volume: 3,
        first_page: 176,
    },
    Paper {
        position: 16000,
        label: "A Neglected Argument for the Reality of God",
        filename: "16000_a_neglected_argument_for_the_reality_of_god.md",
        index: "NeglectedArgument.pdf",
        from: 1,
        to: 23,
        from_section: None,
        to_section: None,
        venue: "Hibbert",
        volume: 7,
        first_page: 90,
    },
];

/// Papers this converter does NOT produce. Both were built from other sources
/// and are maintained by hand; see `assets/peirce1/curated/CURATION_NOTES.md`
/// for their full provenance.
///
/// - 01000 *On a New List of Categories* — Wikisource holds it as a plain
///   transcription with no page breaks at all; its eleven internal boundaries
///   were aligned from the Proceedings vol. 7 scan.
/// - 04000 *Grounds of Validity of the Laws of Logic* — transcribed nowhere.
///   Collated from peirce.org against the Journal of Speculative Philosophy
///   vol. 2 scan, with peirce.org's two systematic editorial interventions
///   (`premiss` for `premise`, `[Ergo,]` for `∴`) reverted to the 1869
///   readings.
pub const HAND_CURATED: &[usize] = &[1000, 4000];

pub fn by_position(position: usize) -> Option<&'static Paper> {
    PAPERS.iter().find(|p| p.position == position)
}
