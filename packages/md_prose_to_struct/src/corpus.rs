//! Per-corpus configuration for the shared annotated-prose parser. Canonical
//! data lives in `common::kant1` / `common::kant3` (TOC tables, filename rules,
//! book metadata); this assembles it into a [`Corpus`] per corpus × edition. A
//! new prose corpus (e.g. hegel1) = a new `common::<corpus>` module + a builder
//! arm here — never a new parser.

use text_struct::model::{BookData, ReferenceSystemData};

/// One flattened TOC row: (flat_index, aa_page, depth, label, slug_override).
pub type FlatEntry = (usize, u16, u16, &'static str, Option<&'static str>);

/// The English-TOC config a corpus may carry for its translation edition.
/// kant1 has a curated English TOC (labels validated against it, English
/// filenames mapped to German by flat index); kant3 has none — the translated
/// files' front-matter labels are the authority and filenames are shared.
pub struct EnToc {
    pub entries: Vec<FlatEntry>,
    pub filenames: Vec<(usize, String)>,
}

pub struct Corpus {
    pub name: &'static str,
    /// Book metadata for the edition being built (source or translation).
    pub book: BookData,
    pub reference_systems: Vec<ReferenceSystemData>,
    /// Reviewed-layer TOC (faithful labels) — source mode validates the
    /// reviewed files against it.
    pub toc_reviewed: Vec<FlatEntry>,
    /// Modernized-layer TOC (modern labels) — drives node labels in source
    /// mode and structural validation everywhere.
    pub toc_modernized: Vec<FlatEntry>,
    /// Translation-edition TOC config; `None` = front-matter labels are the
    /// authority (kant3).
    pub toc_en: Option<EnToc>,
    /// Canonical (German) filenames by flat index.
    pub filenames: Vec<(usize, String)>,
    pub position_number: fn(usize) -> usize,
    pub slugify: fn(&str) -> String,
    pub modernized_dir: String,
    pub reviewed_dir: String,
    pub translated_dir: String,
    pub output_file: String,
    /// Figure caption label word ("Abbildung" for the German edition,
    /// "Figure" for the translation).
    pub figure_label: &'static str,
    pub aa_system_slug: &'static str,
    pub edition_system_slug: &'static str,
    /// The 1790 first edition paginates its preface in Roman numerals and its
    /// body in Arabic — fall back to an Arabic parse for the edition system's
    /// sort order (kant3 only).
    pub edition_sort_arabic_fallback: bool,
    /// Summary labels for the two marker systems (stderr only).
    pub marker_labels: (&'static str, &'static str),
}

fn systems(
    aa_slug: &str,
    aa_label: &str,
    aa_template: &str,
    ed_slug: &str,
    ed_label: &str,
    ed_template: &str,
) -> Vec<ReferenceSystemData> {
    vec![
        ReferenceSystemData {
            slug: aa_slug.to_string(),
            label: aa_label.to_string(),
            ref_type: "block".to_string(),
            // Citation-capable but not the default — Kant cites by sentence.
            cite_priority: None,
            cite_template: Some(aa_template.to_string()),
        },
        ReferenceSystemData {
            slug: ed_slug.to_string(),
            label: ed_label.to_string(),
            ref_type: "inline".to_string(),
            cite_priority: None,
            cite_template: Some(ed_template.to_string()),
        },
    ]
}

pub fn by_name(name: &str, translation: bool) -> Option<Corpus> {
    match name {
        "kant1" => {
            use common::kant1::{filenames, filenames_en, meta, toc, toc_en, toc_mod};
            let (book, figure_label, output_file) = if translation {
                (
                    book_data(
                        meta::BOOK_SLUG_EN,
                        meta::BOOK_TITLE_EN,
                        meta::AUTHOR,
                        meta::LANGUAGE_EN,
                        meta::SOURCE_EN,
                        meta::YEAR_EN,
                        meta::ABOUT_EN,
                    ),
                    "Figure",
                    meta::TRANSLATION_OUTPUT_FILE,
                )
            } else {
                (
                    book_data(
                        meta::BOOK_SLUG,
                        meta::BOOK_TITLE,
                        meta::AUTHOR,
                        meta::LANGUAGE,
                        meta::SOURCE,
                        meta::YEAR,
                        meta::ABOUT,
                    ),
                    "Abbildung",
                    meta::OUTPUT_FILE,
                )
            };
            Some(Corpus {
                name: "kant1",
                book,
                reference_systems: systems(
                    meta::AA_SYSTEM_SLUG,
                    if translation {
                        meta::AA_SYSTEM_LABEL_EN
                    } else {
                        meta::AA_SYSTEM_LABEL
                    },
                    meta::AA_CITE_TEMPLATE,
                    meta::EDITION_SYSTEM_SLUG,
                    if translation {
                        meta::EDITION_SYSTEM_LABEL_EN
                    } else {
                        meta::EDITION_SYSTEM_LABEL
                    },
                    meta::EDITION_CITE_TEMPLATE,
                ),
                toc_reviewed: toc::flat_toc_entries(),
                toc_modernized: toc_mod::flat_toc_entries(),
                toc_en: Some(EnToc {
                    entries: toc_en::flat_toc_entries_en(),
                    filenames: filenames_en::all_filenames_en(),
                }),
                filenames: filenames::all_filenames(),
                position_number: filenames::position_number,
                slugify: filenames::slugify,
                modernized_dir: meta::MODERNIZED_DIR.to_string(),
                reviewed_dir: meta::REVIEWED_DIR.to_string(),
                translated_dir: meta::TRANSLATED_DIR.to_string(),
                output_file: output_file.to_string(),
                figure_label,
                aa_system_slug: meta::AA_SYSTEM_SLUG,
                edition_system_slug: meta::EDITION_SYSTEM_SLUG,
                edition_sort_arabic_fallback: false,
                marker_labels: ("AA", "B-edition"),
            })
        }
        "kant3" => {
            use common::kant3::{filenames, meta, toc, toc_mod};
            let (book, figure_label, output_file) = if translation {
                (
                    book_data(
                        meta::BOOK_SLUG_EN,
                        meta::BOOK_TITLE_EN,
                        meta::AUTHOR,
                        meta::LANGUAGE_EN,
                        meta::SOURCE_EN,
                        meta::YEAR_EN,
                        meta::ABOUT_EN,
                    ),
                    "Figure",
                    meta::TRANSLATION_OUTPUT_FILE,
                )
            } else {
                (
                    book_data(
                        meta::BOOK_SLUG,
                        meta::BOOK_TITLE,
                        meta::AUTHOR,
                        meta::LANGUAGE,
                        meta::SOURCE,
                        meta::YEAR,
                        meta::ABOUT,
                    ),
                    "Abbildung",
                    meta::OUTPUT_FILE,
                )
            };
            Some(Corpus {
                name: "kant3",
                book,
                reference_systems: systems(
                    meta::AA_SYSTEM_SLUG,
                    if translation {
                        meta::AA_SYSTEM_LABEL_EN
                    } else {
                        meta::AA_SYSTEM_LABEL
                    },
                    meta::AA_CITE_TEMPLATE,
                    meta::EDITION_SYSTEM_SLUG,
                    if translation {
                        meta::EDITION_SYSTEM_LABEL_EN
                    } else {
                        meta::EDITION_SYSTEM_LABEL
                    },
                    meta::EDITION_CITE_TEMPLATE,
                ),
                toc_reviewed: toc::flat_toc_entries(),
                toc_modernized: toc_mod::flat_toc_entries(),
                toc_en: None,
                filenames: filenames::all_filenames(),
                position_number: filenames::position_number,
                slugify: filenames::slugify,
                modernized_dir: meta::MODERNIZED_DIR.to_string(),
                reviewed_dir: meta::REVIEWED_DIR.to_string(),
                translated_dir: meta::TRANSLATED_DIR.to_string(),
                output_file: output_file.to_string(),
                figure_label,
                aa_system_slug: meta::AA_SYSTEM_SLUG,
                edition_system_slug: meta::EDITION_SYSTEM_SLUG,
                edition_sort_arabic_fallback: true,
                marker_labels: ("AA Bd. V", "1790"),
            })
        }
        _ => None,
    }
}

fn book_data(
    slug: &str,
    title: &str,
    author: &str,
    language: &str,
    source: &str,
    year: &str,
    about: &str,
) -> BookData {
    BookData {
        slug: slug.to_string(),
        title: title.to_string(),
        author: author.to_string(),
        language: language.to_string(),
        publisher: Some(source.to_string()),
        source: source.to_string(),
        source_date: year.to_string(),
        about_text: about.to_string(),
        nodes_per_page: None,
    }
}
