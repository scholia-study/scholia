//! Per-corpus configuration for the shared drama parser. Canonical data lives in
//! `common::<corpus>`; this assembles it into a [`Corpus`]. A new drama corpus =
//! a new `common::<corpus>` module + a builder here — never a new parser.

use common::{ibsen1, plato2};
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
    /// Treat a page marker's single-letter suffix as a Stephanus sub-page
    /// address (`447a` → 4470 … `447e` → 4474) rather than parsing the value
    /// as a plain integer. See `md_prose_to_struct::roman::block_sort_order_subpage`,
    /// whose formula this mirrors.
    pub subpage_letter_sort: bool,
    /// Split dialogue prose with `common::sentences::split_sentences_grc`
    /// instead of the structural (paren-aware, stage-direction-peeling)
    /// splitter. Burnet's Greek carries no parenthetical stage directions, so
    /// direction peeling and paren protection don't apply under this flag.
    pub greek_splitter: bool,
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
        ("plato2", false) => Some(plato2()),
        ("plato2", true) => Some(plato2_translation()),
        _ => None,
    }
}

fn node_specs(nodes: Vec<common::drama::Node>) -> Vec<NodeSpec> {
    nodes
        .into_iter()
        .map(|n| NodeSpec {
            source_ref: n.source_ref,
            slug: n.slug,
            path: n.path,
            depth: n.depth,
            parent_source_ref: n.parent_source_ref,
            filename: n.filename,
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
        margin_prefix: None,
    }]
}

pub fn ibsen1() -> Corpus {
    Corpus {
        book: BookData {
            slug: ibsen1::BOOK_SLUG.into(),
            title: ibsen1::BOOK_TITLE.into(),
            author: ibsen1::AUTHOR.into(),
            language: ibsen1::LANGUAGE.into(),
            publisher: Some(ibsen1::PUBLISHER.into()),
            publication_place: None,
            original_year: Some(ibsen1::ORIGINAL_YEAR),
            original_year_circa: false,
            edition: None,
            volume: None,
            url: Some(ibsen1::SOURCE_URL.into()),
            source: ibsen1::SOURCE.into(),
            source_date: ibsen1::YEAR.to_string(),
            about_text: ibsen1::ABOUT.into(),
            licence: ibsen1::LICENCE.into(),
            // Acts are few but very large nodes (like Milton's Books) — load a
            // couple per page so an act boundary is prefetched before reached.
            nodes_per_page: Some(2),
        },
        reference_systems: page_system(),
        modernized_dir: ibsen1::MODERNIZED_DIR.into(),
        reviewed_dir: Some(ibsen1::REVIEWED_DIR.into()),
        output_file: ibsen1::OUTPUT_FILE.into(),
        nodes: node_specs(ibsen1::nodes()),
        subpage_letter_sort: false,
        greek_splitter: false,
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
            publisher: Some(ibsen1::PUBLISHER_EN.into()),
            publication_place: None,
            original_year: Some(ibsen1::ORIGINAL_YEAR),
            original_year_circa: false,
            edition: None,
            volume: None,
            url: None,
            source: ibsen1::SOURCE_EN.into(),
            source_date: ibsen1::YEAR_EN.to_string(),
            about_text: ibsen1::ABOUT_EN.into(),
            licence: ibsen1::LICENCE_EN.into(),
            nodes_per_page: Some(2),
        },
        reference_systems: page_system(),
        modernized_dir: ibsen1::TRANSLATED_DIR.into(),
        reviewed_dir: None,
        output_file: ibsen1::TRANSLATION_OUTPUT_FILE.into(),
        nodes: node_specs(ibsen1::nodes()),
        subpage_letter_sort: false,
        greek_splitter: false,
    }
}

/// Stephanus pagination — page and section together (`447a`), which is how
/// Plato is cited. The book title supplies the work, so the template is the
/// bare reference and there is no margin prefix. The translation edition
/// reuses the source book's system (the importer maps markers by slug).
fn stephanus_system() -> Vec<ReferenceSystemData> {
    vec![ReferenceSystemData {
        slug: plato2::meta::PAGE_SYSTEM_SLUG.into(),
        label: plato2::meta::PAGE_SYSTEM_LABEL.into(),
        ref_type: plato2::meta::PAGE_SYSTEM_REF_TYPE.into(),
        cite_priority: Some(plato2::meta::PAGE_CITE_PRIORITY),
        cite_template: Some(plato2::meta::PAGE_CITE_TEMPLATE.into()),
        margin_prefix: None,
    }]
}

/// The Greek source edition. Single-layer: Burnet's constituted text has no
/// original-orthography companion, so `reviewed_dir` is `None`.
pub fn plato2() -> Corpus {
    use plato2::meta;
    Corpus {
        book: BookData {
            slug: meta::BOOK_SLUG.into(),
            title: meta::BOOK_TITLE.into(),
            author: meta::AUTHOR.into(),
            language: meta::LANGUAGE.into(),
            publisher: Some(meta::PUBLISHER.into()),
            publication_place: Some(meta::PUBLICATION_PLACE.into()),
            original_year: Some(meta::ORIGINAL_YEAR),
            original_year_circa: meta::ORIGINAL_YEAR_CIRCA,
            edition: None,
            volume: Some(meta::VOLUME.into()),
            url: None,
            source: meta::SOURCE.into(),
            source_date: meta::YEAR.to_string(),
            about_text: meta::ABOUT.into(),
            licence: meta::LICENCE.into(),
            nodes_per_page: None,
        },
        reference_systems: stephanus_system(),
        modernized_dir: meta::MODERNIZED_DIR.into(),
        reviewed_dir: None,
        output_file: meta::OUTPUT_FILE.into(),
        nodes: node_specs(plato2::toc::nodes()),
        subpage_letter_sort: true,
        greek_splitter: true,
    }
}

/// The English translation edition: a separate book locked 1:1 to `plato2()`
/// and imported with `--source-book-slug gorgias-grc`. It does NOT set
/// `greek_splitter` — the English splits on its own punctuation, and the
/// sentence-parity lock is enforced at import.
pub fn plato2_translation() -> Corpus {
    use plato2::meta;
    Corpus {
        book: BookData {
            slug: meta::BOOK_SLUG_EN.into(),
            title: meta::BOOK_TITLE_EN.into(),
            author: meta::AUTHOR.into(),
            language: meta::LANGUAGE_EN.into(),
            publisher: Some(meta::PUBLISHER_EN.into()),
            publication_place: None,
            original_year: Some(meta::ORIGINAL_YEAR),
            original_year_circa: meta::ORIGINAL_YEAR_CIRCA,
            edition: None,
            volume: None,
            url: None,
            source: meta::SOURCE_EN.into(),
            source_date: meta::YEAR_EN.to_string(),
            about_text: meta::ABOUT_EN.into(),
            licence: meta::LICENCE_EN.into(),
            nodes_per_page: None,
        },
        reference_systems: stephanus_system(),
        modernized_dir: meta::TRANSLATED_DIR.into(),
        reviewed_dir: None,
        output_file: meta::TRANSLATION_OUTPUT_FILE.into(),
        nodes: node_specs(plato2::toc::nodes()),
        subpage_letter_sort: true,
        greek_splitter: false,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn ibsen1_corpora_default_the_new_flags_off() {
        assert!(!ibsen1().subpage_letter_sort);
        assert!(!ibsen1().greek_splitter);
        assert!(!ibsen1_translation().subpage_letter_sort);
        assert!(!ibsen1_translation().greek_splitter);
    }

    #[test]
    fn plato2_greek_edition_is_single_layer_and_greek_split() {
        let c = plato2();
        assert_eq!(c.book.slug, "gorgias-grc");
        assert_eq!(c.book.title, "Γοργίας");
        assert_eq!(c.book.language, "grc");
        assert!(c.reviewed_dir.is_none(), "the Greek has no second layer");
        assert!(c.greek_splitter);
        assert!(c.subpage_letter_sort);
        assert_eq!(c.nodes.len(), 22);
    }

    /// The English splits on its own punctuation; only the Greek is
    /// Greek-split. Both editions keep the Stephanus sort, since both carry
    /// the same markers.
    #[test]
    fn plato2_translation_is_english_split_but_stephanus_sorted() {
        let c = plato2_translation();
        assert_eq!(c.book.slug, "gorgias");
        assert_eq!(c.book.language, "en");
        assert!(!c.greek_splitter);
        assert!(c.subpage_letter_sort);
        assert_eq!(c.nodes.len(), plato2().nodes.len());
    }

    #[test]
    fn plato2_editions_share_the_stephanus_system() {
        for c in [plato2(), plato2_translation()] {
            let s = &c.reference_systems[0];
            assert_eq!(s.slug, "stephanus");
            assert_eq!(s.cite_template.as_deref(), Some("{ref}"));
            assert_eq!(s.cite_priority, Some(0));
            assert!(s.margin_prefix.is_none());
        }
    }
}
