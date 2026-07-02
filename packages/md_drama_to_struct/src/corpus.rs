//! Per-corpus configuration for the shared drama parser. Canonical data lives in
//! `common::ibsen1`; this assembles it into a [`Corpus`]. A new drama corpus =
//! a new `common::<corpus>` module + a builder here — never a new parser.

use common::ibsen1;
use text_struct::model::{BookData, ReferenceSystemData};

pub struct Corpus {
    pub book: BookData,
    pub reference_systems: Vec<ReferenceSystemData>,
    /// The primary reading layer (md_modernized for the source corpus;
    /// md_modernized_translated for the translation corpus).
    pub modernized_dir: String,
    /// The secondary `original_*` layer (md_reviewed). `None` for a translation
    /// edition, which is single-layer (English only) — its node labels come from
    /// the file front matter, not the source-corpus `NodeSpec.label`.
    pub reviewed_dir: Option<String>,
    /// Default derived-struct output path (CLI `--output-file` overrides).
    pub output_file: String,
    pub nodes: Vec<NodeSpec>,
}

pub struct NodeSpec {
    pub source_ref: String,
    pub slug: String,
    pub path: String,
    pub depth: i16,
    pub parent_source_ref: Option<String>,
    pub filename: String,
    pub expected_position: u32,
}

/// Resolve a corpus by CLI name + translation flag.
pub fn by_name(name: &str, translation: bool) -> Option<Corpus> {
    match (name, translation) {
        ("ibsen1" | "ibsen", false) => Some(ibsen1()),
        ("ibsen1" | "ibsen", true) => Some(ibsen1_translation()),
        _ => None,
    }
}

fn ibsen1_nodes() -> Vec<NodeSpec> {
    ibsen1::nodes()
        .into_iter()
        .map(|n| NodeSpec {
            source_ref: n.source_ref.into(),
            slug: n.slug.into(),
            path: n.path.into(),
            depth: n.depth,
            parent_source_ref: n.parent_source_ref.map(Into::into),
            filename: n.filename.into(),
            expected_position: n.position,
        })
        .collect()
}

/// The 1873 printed page is drama's default citation (`p. N`); the template
/// embeds the part + act node labels, e.g. "Cæsars Frafald, Første handling ·
/// p. 12". The translation edition reuses the source book's system (the
/// importer maps markers by slug), so both corpora carry the same config.
fn page_system() -> Vec<ReferenceSystemData> {
    vec![ReferenceSystemData {
        slug: ibsen1::PAGE_SYSTEM_SLUG.into(),
        label: ibsen1::PAGE_SYSTEM_LABEL.into(),
        ref_type: "block".into(),
        cite_priority: Some(0),
        cite_template: Some("{parent}, {self} · p. {ref}".into()),
    }]
}

pub fn ibsen1() -> Corpus {
    Corpus {
        book: BookData {
            slug: ibsen1::BOOK_SLUG.into(),
            title: ibsen1::BOOK_TITLE.into(),
            author: ibsen1::AUTHOR.into(),
            language: ibsen1::LANGUAGE.into(),
            publisher: None,
            source: ibsen1::SOURCE.into(),
            source_date: ibsen1::YEAR.to_string(),
            about_text: ibsen1::ABOUT.into(),
            // Acts are few but very large nodes (like Milton's Books) — load a
            // couple per page so an act boundary is prefetched before reached.
            nodes_per_page: Some(2),
        },
        reference_systems: page_system(),
        modernized_dir: ibsen1::MODERNIZED_DIR.into(),
        reviewed_dir: Some(ibsen1::REVIEWED_DIR.into()),
        output_file: ibsen1::OUTPUT_FILE.into(),
        nodes: ibsen1_nodes(),
    }
}

/// The English translation edition: single-layer (`md_modernized_translated`),
/// a separate book locked 1:1 to `ibsen1()` and imported with
/// `--source-book-slug emperor-and-galilean`.
pub fn ibsen1_translation() -> Corpus {
    Corpus {
        book: BookData {
            slug: ibsen1::BOOK_SLUG_EN.into(),
            title: ibsen1::BOOK_TITLE_EN.into(),
            author: ibsen1::AUTHOR.into(),
            language: ibsen1::LANGUAGE_EN.into(),
            publisher: None,
            source: ibsen1::SOURCE_EN.into(),
            source_date: ibsen1::YEAR_EN.to_string(),
            about_text: ibsen1::ABOUT_EN.into(),
            nodes_per_page: Some(2),
        },
        reference_systems: page_system(),
        modernized_dir: ibsen1::TRANSLATED_DIR.into(),
        reviewed_dir: None,
        output_file: ibsen1::TRANSLATION_OUTPUT_FILE.into(),
        nodes: ibsen1_nodes(),
    }
}
