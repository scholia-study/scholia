//! Book + reference-system metadata for peirce1 (*Essays in Pragmaticism*,
//! 1867–1908).

/// The one curated layer. Peirce's English is already modern, so unlike the
/// other English corpora there is no modernized/reviewed pair — the parser
/// builds this corpus in `single` mode.
pub const CURATED_DIR: &str = "assets/peirce1/curated/md_reviewed";
pub const OUTPUT_FILE: &str = "assets/peirce1/derived/output.json";

pub const AUTHOR: &str = "Charles Sanders Peirce";
pub const BOOK_SLUG: &str = "essays-in-pragmaticism";
pub const BOOK_TITLE: &str = "Essays in Pragmaticism";
pub const LANGUAGE: &str = "en";
/// Publication year of this Scholia edition; the papers' own years run
/// 1867–1908 and the earliest is the identity year (`original_year`).
pub const YEAR: &str = "2026";
pub const PUBLISHER: &str = "Scholia Sodalitas";

pub const SOURCE: &str = "Set from the original periodical printings — Proceedings of the \
    American Academy of Arts and Sciences, the Journal of Speculative Philosophy, Popular \
    Science Monthly, The Monist, and the Hibbert Journal — via proofread transcriptions \
    checked against page scans.";

pub const ABOUT: &str = "Sixteen papers Peirce published in the philosophical and scientific \
    journals between 1867 and 1908, in order of publication: the new list of categories, the \
    cognition series, the six-part Illustrations of the Logic of Science in which he states \
    the pragmatic maxim, the Monist metaphysical series, and a late argument for the reality \
    of God. The text is set from the original printings, with their pagination. The title \
    takes the name Peirce coined in 1905 for his own version of pragmatism, to mark it \
    off from what others had made of the word; these essays are its foundations.";

/// The `{{{ }}}` system: the original printing's volume and page, qualified by
/// periodical because the selection spans five of them and the volume numbers
/// collide (JSP 2 and Monist 2; PAAAS 7 and Hibbert 7). This is the citation
/// standard for the journal papers and the only system the corpus carries —
/// CP/W/EP handles cannot be derived without the copyrighted editions.
pub const PAGE_SYSTEM_SLUG: &str = "orig-pub";
pub const PAGE_SYSTEM_LABEL: &str = "Original publication";
pub const PAGE_SYSTEM_REF_TYPE: &str = "block";
pub const PAGE_CITE_PRIORITY: i16 = 0;
pub const PAGE_CITE_TEMPLATE: &str = "{self} · {ref}";

/// Each paper's original printing, parallel to `filenames::FILENAMES`:
/// (publication year, periodical, volume). These become per-paper sub-work
/// sources, so a quotation cites the essay in its original venue rather than
/// the collection.
pub const PAPER_IMPRINTS: &[(i16, &str, &str)] = &[
    (
        1868,
        "Proceedings of the American Academy of Arts and Sciences",
        "7",
    ),
    (1868, "The Journal of Speculative Philosophy", "2"),
    (1868, "The Journal of Speculative Philosophy", "2"),
    (1869, "The Journal of Speculative Philosophy", "2"),
    (1877, "Popular Science Monthly", "12"),
    (1878, "Popular Science Monthly", "12"),
    (1878, "Popular Science Monthly", "12"),
    (1878, "Popular Science Monthly", "12"),
    (1878, "Popular Science Monthly", "13"),
    (1878, "Popular Science Monthly", "13"),
    (1891, "The Monist", "1"),
    (1892, "The Monist", "2"),
    (1892, "The Monist", "2"),
    (1892, "The Monist", "3"),
    (1893, "The Monist", "3"),
    (1908, "The Hibbert Journal", "7"),
];
