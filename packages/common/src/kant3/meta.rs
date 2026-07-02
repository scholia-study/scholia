//! Book + reference-system metadata for the Kritik der Urteilskraft editions
//! (German Akademie-Ausgabe Band V source + English translation), consumed by
//! `md_prose_to_struct::corpus`. Pure data — string constants only, so the
//! parser crate owns the struct shapes.

pub const MODERNIZED_DIR: &str = "assets/kant3/curated/md_modernized";
pub const REVIEWED_DIR: &str = "assets/kant3/curated/md_reviewed";
pub const TRANSLATED_DIR: &str = "assets/kant3/curated/md_modernized_translated";
pub const OUTPUT_FILE: &str = "assets/kant3/derived/output.json";
pub const TRANSLATION_OUTPUT_FILE: &str = "assets/kant3/derived/translation_output.json";

pub const AUTHOR: &str = "Immanuel Kant";

// German source edition.
pub const BOOK_SLUG: &str = "kritik-der-urteilskraft";
pub const BOOK_TITLE: &str = "Kritik der Urteilskraft";
pub const LANGUAGE: &str = "de";
pub const SOURCE: &str = "Akademie-Ausgabe Band V";
pub const YEAR: &str = "1790";
pub const ABOUT: &str = "This German edition reproduces the text of Kant's Kritik der Urteilskraft as printed in the \
         Akademie-Ausgabe (Band V). Margin markers refer to AA page numbers; the original 1790 \
         first-edition pagination is preserved inline. The text itself is in the public domain. \
         The digital edition on Scholia is a community-driven project. Corrections and refinements \
         are welcome.";

// English translation edition.
pub const BOOK_SLUG_EN: &str = "critique-of-the-power-of-judgment";
pub const BOOK_TITLE_EN: &str = "Critique of the Power of Judgment";
pub const LANGUAGE_EN: &str = "en";
pub const SOURCE_EN: &str = "Scholia Community Edition";
pub const YEAR_EN: &str = "2026";
pub const ABOUT_EN: &str = "This English translation of Kant's Kritik der Urteilskraft (Critique of the Power of \
         Judgment) is a Scholia community project. It is prepared from the Akademie-Ausgabe (Band V) \
         German text, which serves as the underlying source on Scholia.";

// Reference systems: the AA (block) system + the edition (inline) page system.
pub const AA_SYSTEM_SLUG: &str = "aa_v";
pub const AA_SYSTEM_LABEL: &str = "Akademie-Ausgabe Band V";
pub const AA_SYSTEM_LABEL_EN: &str = "Akademie-Ausgabe Band V";
pub const AA_CITE_TEMPLATE: &str = "AA V {ref}";
pub const EDITION_SYSTEM_SLUG: &str = "e1790";
pub const EDITION_SYSTEM_LABEL: &str = "Erstausgabe 1790";
pub const EDITION_SYSTEM_LABEL_EN: &str = "First Edition 1790";
pub const EDITION_CITE_TEMPLATE: &str = "E {ref}";
