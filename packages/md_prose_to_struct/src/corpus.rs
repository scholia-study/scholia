//! Per-corpus configuration for the shared annotated-prose parser. Canonical
//! data lives in `common::kant1` / `common::kant3` (TOC tables, filename rules,
//! book metadata); this assembles it into a [`Corpus`] per corpus × edition. A
//! new prose corpus (e.g. hegel1) = a new `common::<corpus>` module + a builder
//! arm here — never a new parser.

use text_struct::model::{BookData, ReferenceSystemData};

/// One flattened TOC row: (flat_index, page, depth, label, slug_override).
/// The page is the corpus page key's value as a string — kant's numeric AA
/// page, hegel1's Roman-or-Arabic 1807 page — `None` for a node that opens
/// before the first numbered page.
pub type FlatEntry = common::FlatTocEntry;

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
    /// The source edition's own language is English (hobbes1), so both layers
    /// split with the English sentence splitter; false = German source
    /// (kant1/kant3/hegel1).
    pub source_splitter_en: bool,
    /// The copy-text points with period-strength colons (Early Modern
    /// convention — hobbes1): a capitalized word after ": " begins a new
    /// sentence.
    pub strong_colon_splits: bool,
}

struct SystemSpec {
    slug: &'static str,
    label: &'static str,
    template: &'static str,
    priority: Option<i16>,
    margin_prefix: Option<&'static str>,
}

// Default-citation system per corpus, matching scholarly convention:
// kant1 cites by B pagination ("B 132"), kant3 by AA volume page
// ("AA V 212"). The sibling system stays citation-capable but
// non-default (priority None).
fn systems(aa: SystemSpec, edition: SystemSpec) -> Vec<ReferenceSystemData> {
    vec![
        ReferenceSystemData {
            slug: aa.slug.to_string(),
            label: aa.label.to_string(),
            ref_type: "block".to_string(),
            cite_priority: aa.priority,
            cite_template: Some(aa.template.to_string()),
            margin_prefix: aa.margin_prefix.map(str::to_string),
        },
        ReferenceSystemData {
            slug: edition.slug.to_string(),
            label: edition.label.to_string(),
            ref_type: "inline".to_string(),
            cite_priority: edition.priority,
            cite_template: Some(edition.template.to_string()),
            margin_prefix: edition.margin_prefix.map(str::to_string),
        },
    ]
}

/// Bibliographic imprint of the record the edition's source row asserts —
/// the copy-text's own publication facts, plus the identity year of the
/// edition it presents. The Scholia translation editions are their own
/// record (no imprint), carrying only the identity year.
struct Imprint {
    publisher: Option<&'static str>,
    place: Option<&'static str>,
    volume: Option<&'static str>,
    edition: Option<&'static str>,
    original_year: Option<i16>,
}

struct BookSpec {
    slug: &'static str,
    title: &'static str,
    author: &'static str,
    language: &'static str,
    source: &'static str,
    year: &'static str,
    about: &'static str,
    imprint: Imprint,
}

pub fn by_name(name: &str, translation: bool) -> Option<Corpus> {
    match name {
        "kant1" => {
            use common::kant1::{filenames, filenames_en, meta, toc, toc_en, toc_mod};
            let (book, figure_label, output_file) = if translation {
                (
                    book_data(BookSpec {
                        slug: meta::BOOK_SLUG_EN,
                        title: meta::BOOK_TITLE_EN,
                        author: meta::AUTHOR,
                        language: meta::LANGUAGE_EN,
                        source: meta::SOURCE_EN,
                        year: meta::YEAR_EN,
                        about: meta::ABOUT_EN,
                        imprint: Imprint {
                            publisher: Some(meta::PUBLISHER_EN),
                            place: None,
                            volume: None,
                            edition: None,
                            original_year: Some(meta::ORIGINAL_YEAR),
                        },
                    }),
                    "Figure",
                    meta::TRANSLATION_OUTPUT_FILE,
                )
            } else {
                (
                    book_data(BookSpec {
                        slug: meta::BOOK_SLUG,
                        title: meta::BOOK_TITLE,
                        author: meta::AUTHOR,
                        language: meta::LANGUAGE,
                        source: meta::SOURCE,
                        year: meta::YEAR,
                        about: meta::ABOUT,
                        imprint: Imprint {
                            publisher: Some(meta::PUBLISHER),
                            place: Some(meta::PUBLICATION_PLACE),
                            volume: Some(meta::VOLUME),
                            edition: Some(meta::EDITION),
                            original_year: Some(meta::ORIGINAL_YEAR),
                        },
                    }),
                    "Abbildung",
                    meta::OUTPUT_FILE,
                )
            };
            Some(Corpus {
                name: "kant1",
                book,
                reference_systems: systems(
                    SystemSpec {
                        slug: meta::AA_SYSTEM_SLUG,
                        label: if translation {
                            meta::AA_SYSTEM_LABEL_EN
                        } else {
                            meta::AA_SYSTEM_LABEL
                        },
                        template: meta::AA_CITE_TEMPLATE,
                        priority: None,
                        margin_prefix: Some("AA "),
                    },
                    SystemSpec {
                        slug: meta::EDITION_SYSTEM_SLUG,
                        label: if translation {
                            meta::EDITION_SYSTEM_LABEL_EN
                        } else {
                            meta::EDITION_SYSTEM_LABEL
                        },
                        template: meta::EDITION_CITE_TEMPLATE,
                        priority: Some(0),
                        margin_prefix: Some("B "),
                    },
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
                source_splitter_en: false,
                strong_colon_splits: false,
            })
        }
        "kant3" => {
            use common::kant3::{filenames, meta, toc, toc_mod};
            let (book, figure_label, output_file) = if translation {
                (
                    book_data(BookSpec {
                        slug: meta::BOOK_SLUG_EN,
                        title: meta::BOOK_TITLE_EN,
                        author: meta::AUTHOR,
                        language: meta::LANGUAGE_EN,
                        source: meta::SOURCE_EN,
                        year: meta::YEAR_EN,
                        about: meta::ABOUT_EN,
                        imprint: Imprint {
                            publisher: Some(meta::PUBLISHER_EN),
                            place: None,
                            volume: None,
                            edition: None,
                            original_year: Some(meta::ORIGINAL_YEAR),
                        },
                    }),
                    "Figure",
                    meta::TRANSLATION_OUTPUT_FILE,
                )
            } else {
                (
                    book_data(BookSpec {
                        slug: meta::BOOK_SLUG,
                        title: meta::BOOK_TITLE,
                        author: meta::AUTHOR,
                        language: meta::LANGUAGE,
                        source: meta::SOURCE,
                        year: meta::YEAR,
                        about: meta::ABOUT,
                        imprint: Imprint {
                            publisher: Some(meta::PUBLISHER),
                            place: Some(meta::PUBLICATION_PLACE),
                            volume: Some(meta::VOLUME),
                            edition: None,
                            original_year: Some(meta::ORIGINAL_YEAR),
                        },
                    }),
                    "Abbildung",
                    meta::OUTPUT_FILE,
                )
            };
            Some(Corpus {
                name: "kant3",
                book,
                reference_systems: systems(
                    SystemSpec {
                        slug: meta::AA_SYSTEM_SLUG,
                        label: if translation {
                            meta::AA_SYSTEM_LABEL_EN
                        } else {
                            meta::AA_SYSTEM_LABEL
                        },
                        template: meta::AA_CITE_TEMPLATE,
                        priority: Some(0),
                        margin_prefix: Some("AA "),
                    },
                    SystemSpec {
                        slug: meta::EDITION_SYSTEM_SLUG,
                        label: if translation {
                            meta::EDITION_SYSTEM_LABEL_EN
                        } else {
                            meta::EDITION_SYSTEM_LABEL
                        },
                        template: meta::EDITION_CITE_TEMPLATE,
                        priority: None,
                        margin_prefix: Some("E "),
                    },
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
                source_splitter_en: false,
                strong_colon_splits: false,
            })
        }
        "hegel1" => {
            use common::hegel1::{filenames, filenames_en, meta, toc, toc_en, toc_mod};
            let (book, figure_label, output_file) = if translation {
                (
                    book_data(BookSpec {
                        slug: meta::BOOK_SLUG_EN,
                        title: meta::BOOK_TITLE_EN,
                        author: meta::AUTHOR,
                        language: meta::LANGUAGE_EN,
                        source: meta::SOURCE_EN,
                        year: meta::YEAR_EN,
                        about: meta::ABOUT_EN,
                        imprint: Imprint {
                            publisher: Some(meta::PUBLISHER_EN),
                            place: None,
                            volume: None,
                            edition: None,
                            original_year: Some(meta::ORIGINAL_YEAR),
                        },
                    }),
                    "Figure",
                    meta::TRANSLATION_OUTPUT_FILE,
                )
            } else {
                (
                    book_data(BookSpec {
                        slug: meta::BOOK_SLUG,
                        title: meta::BOOK_TITLE,
                        author: meta::AUTHOR,
                        language: meta::LANGUAGE,
                        source: meta::SOURCE,
                        year: meta::YEAR,
                        about: meta::ABOUT,
                        imprint: Imprint {
                            publisher: Some(meta::PUBLISHER),
                            place: None,
                            volume: None,
                            edition: None,
                            original_year: Some(meta::ORIGINAL_YEAR),
                        },
                    }),
                    "Abbildung",
                    meta::OUTPUT_FILE,
                )
            };
            Some(Corpus {
                name: "hegel1",
                book,
                // Two systems: the 1807 pages ({{{ }}}, displayable, non-
                // default) and the GW 9 pages ({{ }}, the scholarly citation
                // standard, hence the citation default).
                reference_systems: vec![
                    ReferenceSystemData {
                        slug: meta::PAGE_SYSTEM_SLUG.to_string(),
                        label: if translation {
                            meta::PAGE_SYSTEM_LABEL_EN.to_string()
                        } else {
                            meta::PAGE_SYSTEM_LABEL.to_string()
                        },
                        ref_type: meta::PAGE_SYSTEM_REF_TYPE.to_string(),
                        cite_priority: None,
                        cite_template: Some(meta::PAGE_CITE_TEMPLATE.to_string()),
                        margin_prefix: None,
                    },
                    ReferenceSystemData {
                        slug: meta::GW_SYSTEM_SLUG.to_string(),
                        label: if translation {
                            meta::GW_SYSTEM_LABEL_EN.to_string()
                        } else {
                            meta::GW_SYSTEM_LABEL.to_string()
                        },
                        ref_type: "inline".to_string(),
                        cite_priority: Some(meta::GW_CITE_PRIORITY),
                        cite_template: Some(meta::GW_CITE_TEMPLATE.to_string()),
                        margin_prefix: Some(meta::GW_MARGIN_PREFIX.to_string()),
                    },
                ],
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
                aa_system_slug: meta::PAGE_SYSTEM_SLUG,
                edition_system_slug: meta::GW_SYSTEM_SLUG,
                // GW pages are Arabic throughout; without the fallback every
                // marker would land on sort 0 (the kant1 b_edition bug).
                edition_sort_arabic_fallback: true,
                marker_labels: ("1807", "GW"),
                source_splitter_en: false,
                strong_colon_splits: false,
            })
        }
        "hegel2" => {
            use common::hegel2::{filenames, meta, toc, toc_mod};
            if translation {
                panic!("hegel2's English translation edition is not yet in scope");
            }
            Some(Corpus {
                name: "hegel2",
                book: book_data(BookSpec {
                    slug: meta::BOOK_SLUG,
                    title: meta::BOOK_TITLE,
                    author: meta::AUTHOR,
                    language: meta::LANGUAGE,
                    source: meta::SOURCE,
                    year: meta::YEAR,
                    about: meta::ABOUT,
                    imprint: Imprint {
                        publisher: Some(meta::PUBLISHER),
                        place: None,
                        volume: None,
                        edition: None,
                        original_year: Some(meta::ORIGINAL_YEAR),
                    },
                }),
                // One system: GW volume.page ("21.68"), the scholarly
                // citation standard and therefore the citation default.
                reference_systems: vec![ReferenceSystemData {
                    slug: meta::GW_SYSTEM_SLUG.to_string(),
                    label: meta::GW_SYSTEM_LABEL.to_string(),
                    ref_type: meta::GW_SYSTEM_REF_TYPE.to_string(),
                    cite_priority: Some(meta::GW_CITE_PRIORITY),
                    cite_template: Some(meta::GW_CITE_TEMPLATE.to_string()),
                    margin_prefix: None,
                }],
                toc_reviewed: toc::flat_toc_entries(),
                toc_modernized: toc_mod::flat_toc_entries(),
                toc_en: None,
                filenames: filenames::all_filenames(),
                position_number: filenames::position_number,
                slugify: filenames::slugify,
                modernized_dir: meta::MODERNIZED_DIR.to_string(),
                reviewed_dir: meta::REVIEWED_DIR.to_string(),
                translated_dir: String::new(),
                output_file: meta::OUTPUT_FILE.to_string(),
                figure_label: "Abbildung",
                aa_system_slug: meta::GW_SYSTEM_SLUG,
                // No `{{ }}` system in this corpus; the empty slug fails
                // loudly at import if such a marker ever appears.
                edition_system_slug: "",
                edition_sort_arabic_fallback: false,
                marker_labels: ("GW", "(none)"),
                source_splitter_en: false,
                strong_colon_splits: false,
            })
        }
        "hegel3" => {
            use common::hegel3::{filenames, meta, toc, toc_mod};
            if translation {
                panic!("hegel3 is a German-only corpus — no --translation build");
            }
            Some(Corpus {
                name: "hegel3",
                book: book_data(BookSpec {
                    slug: meta::BOOK_SLUG,
                    title: meta::BOOK_TITLE,
                    author: meta::AUTHOR,
                    language: meta::LANGUAGE,
                    source: meta::SOURCE,
                    year: meta::YEAR,
                    about: meta::ABOUT,
                    imprint: Imprint {
                        publisher: Some(meta::PUBLISHER),
                        place: None,
                        volume: None,
                        edition: None,
                        original_year: Some(meta::ORIGINAL_YEAR),
                    },
                }),
                // One system: the 1812 first edition's own pages (Roman
                // front matter, Arabic body), the citation default.
                reference_systems: vec![ReferenceSystemData {
                    slug: meta::PAGE_SYSTEM_SLUG.to_string(),
                    label: meta::PAGE_SYSTEM_LABEL.to_string(),
                    ref_type: meta::PAGE_SYSTEM_REF_TYPE.to_string(),
                    cite_priority: Some(meta::PAGE_CITE_PRIORITY),
                    cite_template: Some(meta::PAGE_CITE_TEMPLATE.to_string()),
                    margin_prefix: None,
                }],
                toc_reviewed: toc::flat_toc_entries(),
                toc_modernized: toc_mod::flat_toc_entries(),
                toc_en: None,
                filenames: filenames::all_filenames(),
                position_number: filenames::position_number,
                slugify: filenames::slugify,
                modernized_dir: meta::MODERNIZED_DIR.to_string(),
                reviewed_dir: meta::REVIEWED_DIR.to_string(),
                translated_dir: String::new(),
                output_file: meta::OUTPUT_FILE.to_string(),
                figure_label: "Abbildung",
                aa_system_slug: meta::PAGE_SYSTEM_SLUG,
                // No `{{ }}` system in this corpus; the empty slug fails
                // loudly at import if such a marker ever appears.
                edition_system_slug: "",
                edition_sort_arabic_fallback: false,
                marker_labels: ("1812", "(none)"),
                source_splitter_en: false,
                strong_colon_splits: false,
            })
        }
        "hobbes1" => {
            use common::hobbes1::{filenames, meta, toc, toc_mod};
            if translation {
                panic!("hobbes1 is a single-edition English corpus — no --translation build");
            }
            Some(Corpus {
                name: "hobbes1",
                book: book_data(BookSpec {
                    slug: meta::BOOK_SLUG,
                    title: meta::BOOK_TITLE,
                    author: meta::AUTHOR,
                    language: meta::LANGUAGE,
                    source: meta::SOURCE,
                    year: meta::YEAR,
                    about: meta::ABOUT,
                    imprint: Imprint {
                        publisher: Some(meta::PUBLISHER),
                        place: Some(meta::PUBLICATION_PLACE),
                        volume: None,
                        edition: None,
                        original_year: None,
                    },
                }),
                // One system: the printed 1651 folio numbers, the scholarly
                // citation standard for the Head edition.
                reference_systems: vec![ReferenceSystemData {
                    slug: meta::PAGE_SYSTEM_SLUG.to_string(),
                    label: meta::PAGE_SYSTEM_LABEL.to_string(),
                    ref_type: meta::PAGE_SYSTEM_REF_TYPE.to_string(),
                    cite_priority: Some(meta::PAGE_CITE_PRIORITY),
                    cite_template: Some(meta::PAGE_CITE_TEMPLATE.to_string()),
                    margin_prefix: None,
                }],
                toc_reviewed: toc::flat_toc_entries(),
                toc_modernized: toc_mod::flat_toc_entries(),
                toc_en: None,
                filenames: filenames::all_filenames(),
                position_number: filenames::position_number,
                slugify: filenames::slugify,
                modernized_dir: meta::MODERNIZED_DIR.to_string(),
                reviewed_dir: meta::REVIEWED_DIR.to_string(),
                translated_dir: String::new(),
                output_file: meta::OUTPUT_FILE.to_string(),
                figure_label: "Figure",
                aa_system_slug: meta::PAGE_SYSTEM_SLUG,
                // No `{{ }}` system in this corpus; the empty slug fails
                // loudly at import if such a marker ever appears.
                edition_system_slug: "",
                edition_sort_arabic_fallback: false,
                marker_labels: ("1651", "(none)"),
                source_splitter_en: true,
                strong_colon_splits: true,
            })
        }
        _ => None,
    }
}

fn book_data(spec: BookSpec) -> BookData {
    BookData {
        slug: spec.slug.to_string(),
        title: spec.title.to_string(),
        author: spec.author.to_string(),
        language: spec.language.to_string(),
        publisher: spec.imprint.publisher.map(str::to_string),
        publication_place: spec.imprint.place.map(str::to_string),
        original_year: spec.imprint.original_year,
        edition: spec.imprint.edition.map(str::to_string),
        volume: spec.imprint.volume.map(str::to_string),
        url: None,
        source: spec.source.to_string(),
        source_date: spec.year.to_string(),
        about_text: spec.about.to_string(),
        nodes_per_page: None,
    }
}
