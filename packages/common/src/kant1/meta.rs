//! Book + reference-system metadata for the Kritik der reinen Vernunft
//! editions (German B-edition source + English translation), consumed by
//! `md_prose_to_struct::corpus`. Pure data — string constants only, so the
//! parser crate owns the struct shapes.

pub const MODERNIZED_DIR: &str = "assets/kant1/curated/md_modernized";
pub const REVIEWED_DIR: &str = "assets/kant1/curated/md_reviewed";
pub const TRANSLATED_DIR: &str = "assets/kant1/curated/md_modernized_translated";
pub const OUTPUT_FILE: &str = "assets/kant1/derived/output.json";
pub const TRANSLATION_OUTPUT_FILE: &str = "assets/kant1/derived/translation_output.json";

pub const AUTHOR: &str = "Immanuel Kant";

// German source edition (the B-Auflage).
pub const BOOK_SLUG: &str = "kritik-der-reinen-vernunft-b";
pub const BOOK_TITLE: &str = "Kritik der reinen Vernunft";
pub const LANGUAGE: &str = "de";
pub const SOURCE: &str = "Akademie-Ausgabe Band III";
pub const YEAR: &str = "1787";
pub const ABOUT: &str = "This German edition reproduces the text of Kant's Kritik der reinen Vernunft as printed in the \
         1911 Akademie-Ausgabe (Band III) facsimile of the second edition (B, 1787). Margin markers \
         refer to AA page numbers; inline B-edition pagination is preserved within the text. \
         The text itself is in public domain. The digital edition on Scholia is a community-driven \
         project. Corrections and refinements are welcome.";

// English translation edition.
pub const BOOK_SLUG_EN: &str = "critique-of-pure-reason-b";
pub const BOOK_TITLE_EN: &str = "Critique of Pure Reason";
pub const LANGUAGE_EN: &str = "en";
pub const SOURCE_EN: &str = "Scholia Community Edition";
pub const YEAR_EN: &str = "2026";
pub const ABOUT_EN: &str = "This English translation of Kant's Kritik der reinen Vernunft is a Scholia community project. \
         It is prepared from the 1911 Akademie-Ausgabe (Band III) facsimile of the second edition (B), \
         which serves as the underlying German text on Scholia.";

// Reference systems: the AA (block) system + the edition (inline) page system.
// Labels are per-edition-language; slugs/templates are shared.
pub const AA_SYSTEM_SLUG: &str = "aa_iii";
pub const AA_SYSTEM_LABEL: &str = "Akademie-Ausgabe Band III";
pub const AA_SYSTEM_LABEL_EN: &str = "Akademie-Ausgabe Band III";
pub const AA_CITE_TEMPLATE: &str = "AA III {ref}";
pub const EDITION_SYSTEM_SLUG: &str = "b_edition";
pub const EDITION_SYSTEM_LABEL: &str = "B-Auflage Seitenzahl";
pub const EDITION_SYSTEM_LABEL_EN: &str = "B-Edition Page Number";
pub const EDITION_CITE_TEMPLATE: &str = "B {ref}";
