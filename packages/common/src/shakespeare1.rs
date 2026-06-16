//! Canonical structure of Shakespeare's Sonnets (1609): 154 sonnets, each a
//! flat depth-1 node "Sonnet N".

pub const SONNET_COUNT: u32 = 154;
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

/// Zero-padded structure-stable id (matches Kant's `{:03}` convention).
pub fn source_ref(n: u32) -> String {
    format!("{n:03}")
}

/// ltree path label (flat — no hierarchy among sonnets).
pub fn path(n: u32) -> String {
    format!("s{n:03}")
}

/// Curated MD filename, e.g. `001_sonnet_1.md`.
pub fn filename(n: u32) -> String {
    format!("{n:03}_sonnet_{n}.md")
}

/// Expected `(n, filename)` pairs for the whole corpus.
pub fn all_filenames() -> Vec<(u32, String)> {
    sonnet_numbers().map(|n| (n, filename(n))).collect()
}
