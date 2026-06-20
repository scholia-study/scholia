//! Canonical structure of Milton's *Paradise Lost* as a Bible-shape compilation.
//!
//! One book, "John Milton". The work "Paradise Lost" is a depth-0,
//! source-anchored `toc_node`; its depth-1 children are "The Verse" prose
//! preface (first) then the 12 Books. Mirrors `common::shakespeare1`. The
//! per-book verse-line counts below are the canonical 1674 figures and serve as
//! the ingest guard — a curated book that parses to the wrong line count is an
//! error. Source: EEBO-TCP A50924 (1674, CC0); see `assets/milton1/`.

/// The compilation book.
pub const BOOK_SLUG: &str = "milton";
/// Named after the author so the library heading reads simply "John Milton".
pub const BOOK_TITLE: &str = "John Milton";

/// The "Paradise Lost" work — a depth-0 node anchored to its own source.
pub const PL_SOURCE_REF: &str = "paradise-lost";
pub const PL_SLUG: &str = "paradise-lost";
/// ltree path root (no hyphens — ltree labels are alphanumeric/underscore).
pub const PL_PATH: &str = "paradise_lost";
pub const PL_LABEL: &str = "Paradise Lost";
pub const PL_SOURCE_TITLE: &str = "Paradise Lost (1674)";
pub const PL_YEAR: i16 = 1674;

/// Books (and "The Verse") are depth-1 children of the work node.
pub const DEPTH: i16 = 1;
pub const BOOK_COUNT: u32 = 12;

/// "The Verse" — Milton's prose preface on blank verse; the first child node.
pub const VERSE_SOURCE_REF: &str = "paradise-lost:the-verse";
pub const VERSE_SLUG: &str = "the-verse";
pub const VERSE_PATH: &str = "paradise_lost.verse";
pub const VERSE_LABEL: &str = "The Verse";
pub const VERSE_FILENAME: &str = "000_the_verse.md";

/// Heading texts (`## ` stripped) that introduce a prose paragraph block.
pub const ARGUMENT_HEADING: &str = "THE ARGUMENT.";
pub const VERSE_NOTE_HEADING: &str = "THE VERSE.";

/// Canonical 1674 per-book verse-line counts (Book I..XII); total 10,565.
const LINE_COUNTS: [usize; 12] = [
    798, 1055, 742, 1015, 907, 912, 640, 653, 1189, 1104, 901, 649,
];

const ROMAN: [&str; 12] = [
    "I", "II", "III", "IV", "V", "VI", "VII", "VIII", "IX", "X", "XI", "XII",
];

/// All book numbers, 1..=12.
pub fn book_numbers() -> impl Iterator<Item = u32> {
    1..=BOOK_COUNT
}

/// Canonical verse-line count for book `n` (the ingest guard).
pub fn line_count(n: u32) -> usize {
    LINE_COUNTS[(n - 1) as usize]
}

pub fn roman(n: u32) -> &'static str {
    ROMAN[(n - 1) as usize]
}

pub fn label(n: u32) -> String {
    format!("Book {}", roman(n))
}

pub fn slug(n: u32) -> String {
    format!("book-{n}")
}

/// Structure-stable id for book `n`, scoped under the Paradise Lost work.
pub fn source_ref(n: u32) -> String {
    format!("paradise-lost:{n}")
}

/// ltree path for book `n` — a child of the `paradise_lost` work node.
pub fn path(n: u32) -> String {
    format!("paradise_lost.b{n}")
}

/// Curated MD filename, e.g. `001_book_1.md`.
pub fn filename(n: u32) -> String {
    format!("{n:03}_book_{n}.md")
}
