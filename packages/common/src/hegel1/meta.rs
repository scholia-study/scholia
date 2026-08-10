//! Book + reference-system metadata for the Phänomenologie des Geistes
//! edition, consumed by `md_prose_to_struct::corpus`. Pure data — string
//! constants only, so the parser crate owns the struct shapes.

pub const MODERNIZED_DIR: &str = "assets/hegel1/curated/md_modernized";
pub const REVIEWED_DIR: &str = "assets/hegel1/curated/md_reviewed";
pub const TRANSLATED_DIR: &str = "assets/hegel1/curated/md_modernized_translated";
pub const OUTPUT_FILE: &str = "assets/hegel1/derived/output.json";
pub const TRANSLATION_OUTPUT_FILE: &str = "assets/hegel1/derived/translation_output.json";

pub const AUTHOR: &str = "Georg Wilhelm Friedrich Hegel";

pub const BOOK_SLUG: &str = "phaenomenologie-des-geistes";
pub const BOOK_TITLE: &str = "Phänomenologie des Geistes";
pub const LANGUAGE: &str = "de";
pub const YEAR: &str = "2026";
pub const ORIGINAL_YEAR: i16 = 1807;
pub const PUBLISHER: &str = "Scholia Sodalitas";
pub const SOURCE_URL: &str = "https://www.deutschestextarchiv.de/hegel_phaenomenologie_1807";
pub const SOURCE: &str = "Deutsches Textarchiv TEI P5 transcription of the 1807 first edition \
(Bamberg und Würzburg: Joseph Anton Goebhardt).";
pub const ABOUT: &str = "Hegel's Phänomenologie des Geistes (1807), the first part of the System \
der Wissenschaft, published at Bamberg und Würzburg by Joseph Anton Goebhardt. The text follows \
the Deutsches Textarchiv transcription of that first edition; page markers refer to its \
pagination and to the Gesammelte Werke (vol. 9) concordance. The text itself is in the public \
domain. The digital edition on Scholia — including its modernized reading text — is prepared by \
Scholia Sodalitas, a community-driven project. The German text layers follow the DTA \
transcription's CC BY-SA 4.0 terms; Scholia asserts no further rights in them. Corrections and \
refinements are welcome.";

pub const PAGE_SYSTEM_SLUG: &str = "orig1807";
pub const PAGE_SYSTEM_LABEL: &str = "Ausgabe 1807";
pub const PAGE_SYSTEM_LABEL_EN: &str = "1807 edition";
pub const PAGE_SYSTEM_REF_TYPE: &str = "block";
pub const PAGE_CITE_TEMPLATE: &str = "{self} · S. {ref}";

pub const GW_SYSTEM_SLUG: &str = "gw9";
pub const GW_SYSTEM_LABEL: &str = "Gesammelte Werke Bd. 9";
pub const GW_SYSTEM_LABEL_EN: &str = "Gesammelte Werke vol. 9";
pub const GW_CITE_PRIORITY: i16 = 0;
pub const GW_CITE_TEMPLATE: &str = "{self} · GW 9, {ref}";
pub const GW_MARGIN_PREFIX: &str = "9.";

pub const BOOK_SLUG_EN: &str = "phenomenology-of-spirit";
pub const BOOK_TITLE_EN: &str = "The Phenomenology of Spirit";
pub const LANGUAGE_EN: &str = "en";
pub const YEAR_EN: &str = "2026";
pub const PUBLISHER_EN: &str = "Scholia Sodalitas";
pub const SOURCE_EN: &str = "Scholia's translation from the 1807 first edition \
(Deutsches Textarchiv transcription).";
pub const ABOUT_EN: &str = "Hegel's Phenomenology of Spirit (1807), the first part of the System \
of Science. This is Scholia's own English translation, made directly from the first edition's \
German and independent of every existing translation; page markers refer to the 1807 pagination \
and to the Gesammelte Werke (vol. 9) concordance. The digital edition is prepared by Scholia \
Sodalitas, a community-driven project. Unlike the German text layers (CC BY-SA 4.0, following \
the DTA transcription), the translation is Scholia's own work under the regular Scholia assets \
licence. Corrections and refinements are welcome.";
