//! Canonical structure of Shakespeare's Sonnets as a standalone authored work
//! (a normal book grouped under its author, like Kant — not a Bible-shape
//! compilation). The 154 sonnets are flat top-level `toc_nodes`.
//!
//! The range 1..=154 is the guard the pipeline validates the curated MD against:
//! a missing, extra, or misnamed sonnet file is an error.

/// The hosted book (the work itself).
pub const BOOK_SLUG: &str = "shakespeares-sonnets";
pub const BOOK_TITLE: &str = "Shakespeare's Sonnets";

/// 1609 Quarto (the original-spelling source layer).
pub const YEAR: i16 = 1609;

pub const SONNET_COUNT: u32 = 154;
/// Reading nodes are flat (top-level); no work-wrapper node.
pub const DEPTH: i16 = 0;

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

/// Structure-stable id for sonnet `n`.
pub fn source_ref(n: u32) -> String {
    format!("sonnet-{n}")
}

/// ltree path for sonnet `n` — a flat top-level label.
pub fn path(n: u32) -> String {
    format!("s{n}")
}

/// Curated MD filename, e.g. `001_sonnet_1.md`.
pub fn filename(n: u32) -> String {
    format!("{n:03}_sonnet_{n}.md")
}

/// Expected `(n, filename)` pairs for the whole corpus.
pub fn all_filenames() -> Vec<(u32, String)> {
    sonnet_numbers().map(|n| (n, filename(n))).collect()
}
