//! Book + reference-system metadata for the hegel3 edition, consumed by
//! `md_prose_to_struct::corpus`. Pure data — string constants only.

pub const MODERNIZED_DIR: &str = "assets/hegel3/curated/md_modernized";
pub const REVIEWED_DIR: &str = "assets/hegel3/curated/md_reviewed";
pub const TRANSLATED_DIR: &str = "assets/hegel3/curated/md_modernized_translated";
pub const OUTPUT_FILE: &str = "assets/hegel3/derived/output.json";
pub const TRANSLATION_OUTPUT_FILE: &str = "assets/hegel3/derived/translation_output.json";

pub const AUTHOR: &str = "Georg Wilhelm Friedrich Hegel";

pub const BOOK_SLUG: &str = "das-seyn-1812";
pub const BOOK_TITLE: &str = "Wissenschaft der Logik. Das Seyn (1812)";
pub const LANGUAGE: &str = "de";
pub const YEAR: &str = "2026";
pub const ORIGINAL_YEAR: i16 = 1812;
pub const PUBLISHER: &str = "Scholia Sodalitas";
pub const SOURCE_URL: &str = "https://www.deutschestextarchiv.de/hegel_logik0101_1812";
pub const SOURCE: &str = "Deutsches Textarchiv TEI P5 transcription of the 1812 first edition \
(Nürnberg: Johann Leonhard Schrag).";
pub const ABOUT: &str = "The first-edition Doctrine of Being: Wissenschaft der Logik, Bd. 1,1 — \
Die objektive Logik: Das Seyn (Nürnberg: Johann Leonhard Schrag, 1812). Hegel rewrote this book \
completely for the 1832 second edition (which the Wissenschaft der Logik on Scholia follows), \
making the 1812 text a work of its own, the original opening of the Logic as the system first \
had it. The text follows the Deutsches Textarchiv transcription of the first edition; page \
markers refer to its pagination. The text itself is in the public domain. The digital edition \
on Scholia, including its modernized reading text, is prepared by Scholia Sodalitas, a \
community-driven project. The German text layers follow the DTA transcription's CC BY-SA 4.0 \
terms; Scholia asserts no further rights in them. Corrections and refinements are welcome.";

pub const PAGE_SYSTEM_SLUG: &str = "orig1812";
pub const PAGE_SYSTEM_LABEL: &str = "Ausgabe 1812";
pub const PAGE_SYSTEM_LABEL_EN: &str = "1812 edition";
pub const PAGE_SYSTEM_REF_TYPE: &str = "block";
pub const PAGE_CITE_PRIORITY: i16 = 0;
pub const PAGE_CITE_TEMPLATE: &str = "{self} · S. {ref}";

pub const BOOK_SLUG_EN: &str = "doctrine-of-being-1812";
pub const BOOK_TITLE_EN: &str = "The Science of Logic. Being (1812)";
pub const LANGUAGE_EN: &str = "en";
pub const YEAR_EN: &str = "2026";
pub const PUBLISHER_EN: &str = "Scholia Sodalitas";
pub const SOURCE_EN: &str = "Scholia's translation from the German of the 1812 first edition, \
via the Deutsches Textarchiv transcription.";
pub const ABOUT_EN: &str = "The first-edition Doctrine of Being: Wissenschaft der Logik, Bd. 1,1 \
— Die objektive Logik: Das Seyn (Nürnberg: Johann Leonhard Schrag, 1812). Hegel rewrote this \
book completely for the 1832 second edition (which The Science of Logic on Scholia follows), \
making the 1812 text a work of its own, the original opening of the Logic as the system first \
had it. This is Scholia's own English translation, made directly from the 1812 German. \
Page markers refer to the first edition's pagination. \
The digital edition is prepared by Scholia Sodalitas, a community-driven project. The translation \
is Scholia's own work under the regular Scholia assets licence. Corrections and refinements are \
welcome.";
