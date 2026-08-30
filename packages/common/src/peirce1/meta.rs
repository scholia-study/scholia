//! Book + reference-system metadata for peirce1 (*Pragmaticism*, 1867–1908).

/// The one curated layer. Peirce's English is already modern, so unlike the
/// other English corpora there is no modernized/reviewed pair — the parser
/// builds this corpus in `single` mode.
pub const CURATED_DIR: &str = "assets/peirce1/curated/md_reviewed";
pub const OUTPUT_FILE: &str = "assets/peirce1/derived/output.json";

pub const AUTHOR: &str = "Charles Sanders Peirce";
pub const BOOK_SLUG: &str = "pragmaticism";
pub const BOOK_TITLE: &str = "Pragmaticism";
pub const LANGUAGE: &str = "en";
pub const YEAR: &str = "1867–1908";

pub const SOURCE: &str = "Set from the original periodical printings — Proceedings of the \
    American Academy of Arts and Sciences, the Journal of Speculative Philosophy, Popular \
    Science Monthly, The Monist, and the Hibbert Journal — via proofread transcriptions \
    checked against page scans.";

pub const ABOUT: &str = "Sixteen papers Peirce published in the philosophical and scientific \
    journals between 1867 and 1908, in order of publication: the new list of categories, the \
    cognition series, the six-part Illustrations of the Logic of Science in which he states \
    the pragmatic maxim, the Monist metaphysical series, and a late argument for the reality \
    of God. The text is set from the original printings, with their pagination. The title \
    takes the name Peirce coined in 1905 for his own version of pragmatism to mark it off \
    from what others had made of the word.";

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
