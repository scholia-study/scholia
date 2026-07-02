//! Per-corpus configuration: the canonical node tree + book metadata that drives
//! the shared [`parse::build`](crate::parse::build). The canonical *data* (TOC,
//! filenames, labels, line counts) lives in `common::{shakespeare1, milton1}`;
//! this module is the thin glue that assembles it into a [`Corpus`]. Adding a new
//! verse corpus = a new `common::<corpus>` module + a builder here — never a new
//! parser (ADR 0003).

use common::{milton1, shakespeare1 as sonnets};

use crate::model::{BookData, NodeSource, ReferenceSystemData};

pub struct Corpus {
    pub book: BookData,
    pub reference_systems: Vec<ReferenceSystemData>,
    pub modernized_dir: String,
    pub reviewed_dir: String,
    /// Default derived-struct output path (CLI `--output-file` overrides).
    pub output_file: String,
    /// Heading texts (the `## ` stripped) that introduce a prose paragraph; the
    /// block immediately after such a heading is parsed as prose, not verse.
    pub prose_headings: Vec<String>,
    pub nodes: Vec<NodeSpec>,
}

pub struct NodeSpec {
    pub source_ref: String,
    pub slug: String,
    pub path: String,
    pub depth: i16,
    pub label: String,
    pub parent_source_ref: Option<String>,
    /// A node carrying its own source becomes a Bible-shape sub-work anchor
    /// (pill). The poetry corpora are standalone authored works, so this stays
    /// `None`; the field is kept for the generic importer/struct shape.
    pub source: Option<NodeSource>,
    /// `None` = pure-navigation node (no file). `Some` = a leaf parsed from
    /// `filename` in both layer dirs.
    pub content: Option<NodeContentSpec>,
}

pub struct NodeContentSpec {
    pub filename: String,
    pub expected_position: u32,
    /// Guard: the verse-line count this leaf must produce. `None` skips it
    /// (prose-only nodes like "The Verse").
    pub expected_lines: Option<usize>,
}

const SHAKESPEARE_SOURCE: &str = "Modern-spelling reading text from public-domain sources; the original-spelling \
layer reproduces the 1609 Quarto via EEBO-TCP (CC0).";
const SHAKESPEARE_ABOUT: &str = "Shakespeare's Sonnets. The modern-spelling \
reading text is drawn from public-domain sources; the original-spelling layer \
reproduces the 1609 Quarto via EEBO-TCP (released CC0). The digital edition on \
Scholia is a community-driven project; corrections are welcome.";

const MILTON_SOURCE: &str = "Modern-spelling reading text from public-domain sources; the original-spelling \
layer reproduces the 1674 second edition via EEBO-TCP A50924 (CC0).";
const MILTON_ABOUT: &str = "Paradise Lost by John Milton. The modern-spelling \
reading text is drawn from public-domain sources; the original-spelling layer \
reproduces the 1674 second edition via EEBO-TCP (released CC0). The digital \
edition on Scholia is a community-driven project; corrections are welcome.";

/// The `line` reference system. `cite_priority` decides whether lines are the
/// *default* citation (Milton: `Some(0)`; the Sonnets stay sentence-cited:
/// `None`); `cite_template` is always set so the system is citation-capable.
/// See migration 0008.
fn line_system(cite_priority: Option<i16>, cite_template: &str) -> Vec<ReferenceSystemData> {
    vec![ReferenceSystemData {
        slug: "line".into(),
        label: "Line".into(),
        ref_type: "block".into(),
        cite_priority,
        cite_template: Some(cite_template.to_string()),
    }]
}

/// Resolve a corpus by CLI name.
pub fn by_name(name: &str) -> Option<Corpus> {
    match name {
        "shakespeare1" | "shakespeare" => Some(shakespeare()),
        "milton1" | "milton" => Some(milton()),
        _ => None,
    }
}

pub fn shakespeare() -> Corpus {
    // 154 sonnets as flat top-level reading nodes.
    let nodes = sonnets::sonnet_numbers()
        .map(|n| NodeSpec {
            source_ref: sonnets::source_ref(n),
            slug: sonnets::slug(n),
            path: sonnets::path(n),
            depth: sonnets::DEPTH,
            label: sonnets::label(n),
            parent_source_ref: None,
            source: None,
            content: Some(NodeContentSpec {
                filename: sonnets::filename(n),
                expected_position: n,
                expected_lines: None,
            }),
        })
        .collect();
    Corpus {
        book: BookData {
            slug: sonnets::BOOK_SLUG.into(),
            title: sonnets::BOOK_TITLE.into(),
            author: "William Shakespeare".into(),
            language: "en".into(),
            publisher: None,
            source: SHAKESPEARE_SOURCE.into(),
            source_date: sonnets::YEAR.to_string(),
            about_text: SHAKESPEARE_ABOUT.into(),
            nodes_per_page: None,
        },
        reference_systems: line_system(Some(0), "{self} · {ref}"),
        modernized_dir: "assets/shakespeare1/curated/md_modernized".into(),
        reviewed_dir: "assets/shakespeare1/curated/md_reviewed".into(),
        output_file: "assets/shakespeare1/derived/output.json".into(),
        prose_headings: vec![],
        nodes,
    }
}

pub fn milton() -> Corpus {
    // "The Verse" preface, then the 12 Books — all flat top-level reading nodes.
    let mut nodes = vec![NodeSpec {
        source_ref: milton1::VERSE_SOURCE_REF.into(),
        slug: milton1::VERSE_SLUG.into(),
        path: milton1::VERSE_PATH.into(),
        depth: milton1::DEPTH,
        label: milton1::VERSE_LABEL.into(),
        parent_source_ref: None,
        source: None,
        content: Some(NodeContentSpec {
            filename: milton1::VERSE_FILENAME.into(),
            expected_position: 0,
            expected_lines: None,
        }),
    }];
    for n in milton1::book_numbers() {
        nodes.push(NodeSpec {
            source_ref: milton1::source_ref(n),
            slug: milton1::slug(n),
            path: milton1::path(n),
            depth: milton1::DEPTH,
            label: milton1::label(n),
            parent_source_ref: None,
            source: None,
            content: Some(NodeContentSpec {
                filename: milton1::filename(n),
                expected_position: n,
                expected_lines: Some(milton1::line_count(n)),
            }),
        });
    }
    Corpus {
        book: BookData {
            slug: milton1::BOOK_SLUG.into(),
            title: milton1::BOOK_TITLE.into(),
            author: "John Milton".into(),
            language: "en".into(),
            publisher: None,
            source: MILTON_SOURCE.into(),
            source_date: milton1::YEAR.to_string(),
            about_text: MILTON_ABOUT.into(),
            // Few but enormous nodes (a whole Book each) — load a couple per
            // page so a Book-boundary is prefetched before it's reached.
            nodes_per_page: Some(2),
        },
        reference_systems: line_system(Some(0), "{self} · {ref}"),
        modernized_dir: "assets/milton1/curated/md_modernized".into(),
        reviewed_dir: "assets/milton1/curated/md_reviewed".into(),
        output_file: "assets/milton1/derived/output.json".into(),
        prose_headings: vec![
            milton1::ARGUMENT_HEADING.into(),
            milton1::VERSE_NOTE_HEADING.into(),
        ],
        nodes,
    }
}
