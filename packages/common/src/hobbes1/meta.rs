//! Book + reference-system metadata for hobbes1 (Leviathan, 1651).

pub const MODERNIZED_DIR: &str = "assets/hobbes1/curated/md_modernized";
pub const REVIEWED_DIR: &str = "assets/hobbes1/curated/md_reviewed";
pub const OUTPUT_FILE: &str = "assets/hobbes1/derived/output.json";

pub const AUTHOR: &str = "Thomas Hobbes";
pub const BOOK_SLUG: &str = "leviathan";
pub const BOOK_TITLE: &str = "Leviathan";
pub const LANGUAGE: &str = "en";
pub const YEAR: &str = "1651";
pub const PUBLISHER: &str = "Andrew Crooke";
pub const PUBLICATION_PLACE: &str = "London";
pub const SOURCE_URL: &str = "https://quod.lib.umich.edu/e/eebo/A43998.0001.001";
pub const SOURCE: &str = "EEBO-TCP A43998 (CC0): keyed full-text transcription of the 1651 \
    first (\"Head\") edition, London: Andrew Crooke. Diplomatic layer keeps the 1651 \
    orthography as transcribed (TCP normalizes long s); the modernized layer is a \
    modern-spelling reading text derived from it.";

pub const ABOUT: &str = "Hobbes's Leviathan (1651), the founding work of modern political \
    philosophy: from the mechanics of sense and thought to the state of nature, the \
    covenant, and the rights and duties of sovereigns — with the famous authorial \
    margin notes running alongside the text. This presentation offers the 1651 text \
    in original and modernized spelling, with the original pagination. The printed \
    table of contents and errata leaf are omitted (the errata's corrections are \
    applied in the modernized text), as is a later owner's bookplate found in the \
    source copy.";

/// The `{{{ }}}` system: the printed 1651 folio numbers (with `b` suffixes on
/// the five twice-printed pages). The scholarly citation standard for the
/// Head edition, hence the citation default.
pub const PAGE_SYSTEM_SLUG: &str = "orig1651";
pub const PAGE_SYSTEM_LABEL: &str = "1651 edition";
pub const PAGE_SYSTEM_REF_TYPE: &str = "block";
pub const PAGE_CITE_PRIORITY: i16 = 0;
pub const PAGE_CITE_TEMPLATE: &str = "{self} · p. {ref}";
