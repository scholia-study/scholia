//! Book metadata for plato2's two editions: the Greek source text and the
//! English translation locked 1:1 to it.

pub const AUTHOR: &str = "Plato";

pub const ORIGINAL_YEAR: i16 = -380;
pub const ORIGINAL_YEAR_CIRCA: bool = true;

/// Both editions transliterate to the same word, and `books.slug` is UNIQUE, so
/// the Greek edition carries the language tag and the English keeps the plain
/// slug.
pub const BOOK_SLUG: &str = "gorgias-grc";
pub const BOOK_TITLE: &str = "Γοργίας";
pub const LANGUAGE: &str = "grc";
pub const YEAR: &str = "1903";
pub const PUBLISHER: &str = "Clarendon Press";
pub const PUBLICATION_PLACE: &str = "Oxford";
pub const VOLUME: &str = "III";
pub const MODERNIZED_DIR: &str = "assets/plato2/curated/md_modernized";
pub const OUTPUT_FILE: &str = "assets/plato2/derived/output.json";

pub const BOOK_SLUG_EN: &str = "gorgias";
pub const BOOK_TITLE_EN: &str = "Gorgias";
pub const LANGUAGE_EN: &str = "en";
pub const YEAR_EN: &str = "2026";
pub const PUBLISHER_EN: &str = "Scholia Sodalitas";
pub const TRANSLATED_DIR: &str = "assets/plato2/curated/md_modernized_translated";
pub const TRANSLATION_OUTPUT_FILE: &str = "assets/plato2/derived/translation_output.json";

/// Stephanus pagination — page and section together (`447a`), which is how
/// Plato is cited. The book title supplies the work, so the template is the
/// bare reference and there is no margin prefix.
pub const PAGE_SYSTEM_SLUG: &str = "stephanus";
pub const PAGE_SYSTEM_LABEL: &str = "Stephanus";
pub const PAGE_SYSTEM_REF_TYPE: &str = "block";
pub const PAGE_CITE_PRIORITY: i16 = 0;
pub const PAGE_CITE_TEMPLATE: &str = "{ref}";

pub const SOURCE: &str = "Greek text of John Burnet's Platonis Opera, Tomus III \
(Oxford: Clarendon Press, 1903). The words and the Stephanus pagination are Burnet's; \
the transcription was taken from the Perseus Digital Library's canonical-greekLit corpus, \
and the markup, divisions and apparatus of this edition are Scholia's own.";

pub const ABOUT: &str = "Plato's Gorgias (Γοργίας) in the Greek text constituted by John Burnet \
for the Oxford Classical Texts, 1903. Burnet's editorial brackets are kept: ⟨ ⟩ marks what he \
supplied, [ ] what he judged interpolated.\n\nThe dialogue is set out as it is spoken, one \
speaker at a time. The three conversations, with Gorgias, with Polus, with Callicles, are the \
work's own turns; the divisions within them, and their English titles, are Scholia's. Burnet \
prints no such divisions and none are transmitted with the text; they are offered as an aid to \
navigation, not as part of the work. Footnotes identifying the poets Plato quotes are likewise \
Scholia's.\n\nBurnet's text is in the public domain. Only the divisions, their titles and the \
footnotes are licensed CC BY-NC-ND 4.0. Transcription credit to the Perseus Digital Library, \
Tufts University.\n\nThe digital edition on Scholia is a community-driven project; corrections \
are welcome.";
pub const LICENCE: &str = "Public Domain";

pub const SOURCE_EN: &str = "An English reading translation prepared from Burnet's Greek text \
by Scholia Sodalitas.";

pub const ABOUT_EN: &str = "An English reading translation of Plato's Gorgias, prepared from the \
Greek text of Burnet's Platonis Opera, Tomus III (Oxford, 1903).\n\nThe three conversations, \
with Gorgias, with Polus, with Callicles, are the work's own turns. The divisions within them, \
and their titles, are Scholia's, as are the footnotes identifying the poets Plato quotes.\n\n\
This translation is Scholia's own work, licensed CC BY-NC-ND 4.0. A community project published \
by Scholia Sodalitas; corrections are welcome.";
pub const LICENCE_EN: &str = "CC BY-NC-ND 4.0";
