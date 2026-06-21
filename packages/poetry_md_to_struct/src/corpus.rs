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
    /// Work nodes carry a source — the Bible-shape sub-work anchor (pill).
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

const COMPILATION_SOURCE: &str =
    "A Scholia compilation; each work carries its own source provenance.";

const SHAKESPEARE_ABOUT: &str = "The works of William Shakespeare. The modern-spelling \
reading text is drawn from public-domain sources; original-spelling layers \
reproduce the early editions (the Sonnets from the 1609 Quarto via EEBO-TCP, \
released CC0). The digital edition on Scholia is a community-driven project; \
corrections are welcome.";

const MILTON_ABOUT: &str = "The works of John Milton. The modern-spelling \
reading text is drawn from public-domain sources; original-spelling layers \
reproduce the early editions (Paradise Lost from the 1674 second edition via \
EEBO-TCP, released CC0). The digital edition on Scholia is a community-driven \
project; corrections are welcome.";

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
    let mut nodes = vec![NodeSpec {
        source_ref: sonnets::SONNETS_SOURCE_REF.into(),
        slug: sonnets::SONNETS_SLUG.into(),
        path: sonnets::SONNETS_PATH.into(),
        depth: 0,
        label: sonnets::SONNETS_LABEL.into(),
        parent_source_ref: None,
        source: Some(NodeSource {
            title: sonnets::SONNETS_SOURCE_TITLE.into(),
            publication_year: Some(sonnets::SONNETS_YEAR),
        }),
        content: None,
    }];
    for n in sonnets::sonnet_numbers() {
        nodes.push(NodeSpec {
            source_ref: sonnets::source_ref(n),
            slug: sonnets::slug(n),
            path: sonnets::path(n),
            depth: sonnets::DEPTH,
            label: sonnets::label(n),
            parent_source_ref: Some(sonnets::SONNETS_SOURCE_REF.into()),
            source: None,
            content: Some(NodeContentSpec {
                filename: sonnets::filename(n),
                expected_position: n,
                expected_lines: None,
            }),
        });
    }
    Corpus {
        book: BookData {
            slug: sonnets::BOOK_SLUG.into(),
            title: sonnets::BOOK_TITLE.into(),
            author: "William Shakespeare".into(),
            language: "en".into(),
            source: COMPILATION_SOURCE.into(),
            source_date: String::new(),
            about_text: SHAKESPEARE_ABOUT.into(),
        },
        reference_systems: line_system(Some(0), "{self} · {ref}"),
        modernized_dir: "assets/shakespeare1/curated/md_modernized".into(),
        reviewed_dir: "assets/shakespeare1/curated/md_reviewed".into(),
        prose_headings: vec![],
        nodes,
    }
}

pub fn milton() -> Corpus {
    let mut nodes = vec![
        // The "Paradise Lost" work — depth-0, source-anchored navigation node.
        NodeSpec {
            source_ref: milton1::PL_SOURCE_REF.into(),
            slug: milton1::PL_SLUG.into(),
            path: milton1::PL_PATH.into(),
            depth: 0,
            label: milton1::PL_LABEL.into(),
            parent_source_ref: None,
            source: Some(NodeSource {
                title: milton1::PL_SOURCE_TITLE.into(),
                publication_year: Some(milton1::PL_YEAR),
            }),
            content: None,
        },
        // "The Verse" prose preface — first child, prose-only (no verse guard).
        NodeSpec {
            source_ref: milton1::VERSE_SOURCE_REF.into(),
            slug: milton1::VERSE_SLUG.into(),
            path: milton1::VERSE_PATH.into(),
            depth: milton1::DEPTH,
            label: milton1::VERSE_LABEL.into(),
            parent_source_ref: Some(milton1::PL_SOURCE_REF.into()),
            source: None,
            content: Some(NodeContentSpec {
                filename: milton1::VERSE_FILENAME.into(),
                expected_position: 0,
                expected_lines: None,
            }),
        },
    ];
    for n in milton1::book_numbers() {
        nodes.push(NodeSpec {
            source_ref: milton1::source_ref(n),
            slug: milton1::slug(n),
            path: milton1::path(n),
            depth: milton1::DEPTH,
            label: milton1::label(n),
            parent_source_ref: Some(milton1::PL_SOURCE_REF.into()),
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
            source: COMPILATION_SOURCE.into(),
            source_date: String::new(),
            about_text: MILTON_ABOUT.into(),
        },
        reference_systems: line_system(Some(0), "{parent} · {self} · {ref}"),
        modernized_dir: "assets/milton1/curated/md_modernized".into(),
        reviewed_dir: "assets/milton1/curated/md_reviewed".into(),
        prose_headings: vec![
            milton1::ARGUMENT_HEADING.into(),
            milton1::VERSE_NOTE_HEADING.into(),
        ],
        nodes,
    }
}
