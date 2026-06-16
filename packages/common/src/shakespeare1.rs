//! Canonical structure of Shakespeare's works as a Bible-shape compilation.
//!
//! One book, "The Works of Shakespeare". Each work is a depth-0, source-anchored
//! `toc_node`; for now just "Sonnets", whose 154 sonnets are depth-1 children.
//! Plays would slot in later as sibling depth-0 works (work → act → scene).
//! See `assets/shakespeare1/WORKS_COMPILATION_MODEL.md`.
//!
//! The range 1..=154 is the guard the pipeline validates the curated MD against:
//! a missing, extra, or misnamed sonnet file is an error.

/// The compilation book.
pub const BOOK_SLUG: &str = "shakespeare";
/// Display title of the compilation, named after the author so the library
/// heading reads simply "William Shakespeare".
pub const BOOK_TITLE: &str = "William Shakespeare";

/// The "Sonnets" work — a depth-0 node anchored to its own source.
pub const SONNETS_SOURCE_REF: &str = "sonnets";
pub const SONNETS_SLUG: &str = "sonnets";
pub const SONNETS_PATH: &str = "sonnets";
pub const SONNETS_LABEL: &str = "Sonnets";
pub const SONNETS_SOURCE_TITLE: &str = "Shakespeare's Sonnets (1609 Quarto)";
pub const SONNETS_YEAR: i16 = 1609;

pub const SONNET_COUNT: u32 = 154;
/// Sonnets are depth-1 children of the "Sonnets" work node.
pub const DEPTH: i16 = 1;

/// All sonnet numbers, 1..=154.
pub fn sonnet_numbers() -> impl Iterator<Item = u32> {
    1..=SONNET_COUNT
}

pub fn label(n: u32) -> String {
    format!("Sonnet {n}")
}

pub fn slug(n: u32) -> String {
    format!("sonnet-{n}")
}

/// Structure-stable id for sonnet `n`, scoped under the Sonnets work.
pub fn source_ref(n: u32) -> String {
    format!("sonnets:{n}")
}

/// ltree path for sonnet `n` — a child of the `sonnets` work node.
pub fn path(n: u32) -> String {
    format!("sonnets.s{n}")
}

/// Curated MD filename, e.g. `001_sonnet_1.md`.
pub fn filename(n: u32) -> String {
    format!("{n:03}_sonnet_{n}.md")
}

/// Expected `(n, filename)` pairs for the whole corpus.
pub fn all_filenames() -> Vec<(u32, String)> {
    sonnet_numbers().map(|n| (n, filename(n))).collect()
}
