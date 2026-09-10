//! Book metadata for plato1's two editions: the Greek source text and the
//! English translation locked 1:1 to it.

pub const AUTHOR: &str = "Plato";

pub const ORIGINAL_YEAR: i16 = -375;
pub const ORIGINAL_YEAR_CIRCA: bool = true;

pub const BOOK_SLUG: &str = "politeia";
pub const BOOK_TITLE: &str = "Πολιτεία";
pub const LANGUAGE: &str = "grc";
pub const YEAR: &str = "1905";
pub const PUBLISHER: &str = "Clarendon Press";
pub const PUBLICATION_PLACE: &str = "Oxford";
pub const VOLUME: &str = "IV";
pub const MODERNIZED_DIR: &str = "assets/plato1/curated/md_modernized";
pub const OUTPUT_FILE: &str = "assets/plato1/derived/output.json";

pub const BOOK_SLUG_EN: &str = "republic";
pub const BOOK_TITLE_EN: &str = "Republic";
pub const LANGUAGE_EN: &str = "en";
pub const YEAR_EN: &str = "2026";
pub const PUBLISHER_EN: &str = "Scholia Sodalitas";
pub const TRANSLATED_DIR: &str = "assets/plato1/curated/md_modernized_translated";
pub const TRANSLATION_OUTPUT_FILE: &str = "assets/plato1/derived/translation_output.json";

/// Stephanus pagination — page and section together (`327a`), which is how
/// Plato is cited. The book title supplies the work, so the template is the
/// bare reference and there is no margin prefix.
pub const PAGE_SYSTEM_SLUG: &str = "stephanus";
pub const PAGE_SYSTEM_LABEL: &str = "Stephanus";
pub const PAGE_SYSTEM_REF_TYPE: &str = "block";
pub const PAGE_CITE_PRIORITY: i16 = 0;
pub const PAGE_CITE_TEMPLATE: &str = "{ref}";

pub const SOURCE: &str = "Greek text of John Burnet's Platonis Opera, Tomus IV \
(Oxford: Clarendon Press, 1905). The words and the Stephanus pagination are Burnet's; \
the transcription was taken from the Perseus Digital Library's \
canonical-greekLit corpus, and the markup, divisions and apparatus of this edition are \
Scholia's own.";

pub const ABOUT: &str = "Plato's Republic (Πολιτεία) in the Greek text constituted by John \
Burnet for the Oxford Classical Texts, 1905. Burnet's editorial brackets are kept: ⟨ ⟩ marks \
what he supplied, [ ] what he judged interpolated.\n\nThe ten books are Plato's. The divisions \
within them, and their English titles, are Scholia's. Burnet prints no chapter divisions and \
none are transmitted with the text; there are offered as an aid to navigation, not as part of \
the work. Footnotes identifying the poets Plato quotes are likewise Scholia's.\n\nBurnet's text is \
in the public domain. Only the divisions, their titles and the footnotes are licensed \
CC BY-NC-ND 4.0. Transcription credit to the Perseus Digital Library, Tufts University.\n\n\
The digital edition on Scholia is a community-driven project; corrections are welcome.";
pub const LICENCE: &str = "Public Domain";

pub const SOURCE_EN: &str = "An English reading translation prepared from Burnet's Greek text \
by Scholia Sodalitas.";

pub const ABOUT_EN: &str = "An English reading translation of Plato's Republic, prepared from \
the Greek text of Burnet's Platonis Opera, Tomus IV (Oxford, 1905). \
\n\nThe ten books are Plato's. The divisions within \
them, and their titles, are Scholia's, as are the footnotes identifying the poets Plato \
quotes.\n\nThis translation is Scholia's own work, licensed CC BY-NC-ND 4.0. A community \
project published by Scholia Sodalitas; corrections are welcome.";
pub const LICENCE_EN: &str = "CC BY-NC-ND 4.0";
