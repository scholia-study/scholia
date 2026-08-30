//! Canonical TOC for peirce1 — one row per curated file, in order of
//! publication. Flat: each paper is a top-level node, with no series headings
//! above them. The four series Peirce published as units (the cognition
//! papers, the Illustrations, the Monist metaphysical series) fall contiguous
//! in date order anyway, so grouping headings would add a layer of our own
//! invention for no navigational gain.
//!
//! Pages are the opening page of the original printing, venue-qualified
//! because the volume spans five periodicals whose volume numbers collide
//! (JSP 2 and Monist 2; PAAAS 7 and Hibbert 7).

/// (page_pub, depth, label)
pub const TOC: &[(Option<&str>, u16, &str)] = &[
    (Some("PAAAS 7:287"), 1, "On a New List of Categories"),
    (
        Some("JSP 2:103"),
        1,
        "Questions Concerning Certain Faculties Claimed for Man",
    ),
    (
        Some("JSP 2:140"),
        1,
        "Some Consequences of Four Incapacities",
    ),
    (
        Some("JSP 2:193"),
        1,
        "Grounds of Validity of the Laws of Logic: Further Consequences of Four Incapacities",
    ),
    (Some("PSM 12:1"), 1, "The Fixation of Belief"),
    (Some("PSM 12:286"), 1, "How to Make Our Ideas Clear"),
    (Some("PSM 12:604"), 1, "The Doctrine of Chances"),
    (Some("PSM 12:705"), 1, "The Probability of Induction"),
    (Some("PSM 13:203"), 1, "The Order of Nature"),
    (
        Some("PSM 13:470"),
        1,
        "Deduction, Induction, and Hypothesis",
    ),
    (Some("Monist 1:161"), 1, "The Architecture of Theories"),
    (
        Some("Monist 2:321"),
        1,
        "The Doctrine of Necessity Examined",
    ),
    (Some("Monist 2:533"), 1, "The Law of Mind"),
    (Some("Monist 3:1"), 1, "Man's Glassy Essence"),
    (Some("Monist 3:176"), 1, "Evolutionary Love"),
    (
        Some("Hibbert 7:90"),
        1,
        "A Neglected Argument for the Reality of God",
    ),
];

pub fn flat_toc_entries() -> Vec<crate::FlatTocEntry> {
    TOC.iter()
        .enumerate()
        .map(|(i, (page, depth, label))| (i, page.map(str::to_string), *depth, *label, None))
        .collect()
}
