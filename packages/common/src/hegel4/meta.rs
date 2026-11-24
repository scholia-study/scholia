//! Book + reference-system metadata for the Enzyklopädie (1830) edition,
//! consumed by `md_prose_to_struct::corpus`. Pure data — string constants
//! only, so the parser crate owns the struct shapes.

pub const MODERNIZED_DIR: &str = "assets/hegel4/curated/md_modernized";
pub const REVIEWED_DIR: &str = "assets/hegel4/curated/md_reviewed";
pub const OUTPUT_FILE: &str = "assets/hegel4/derived/output.json";

pub const AUTHOR: &str = "Georg Wilhelm Friedrich Hegel";

pub const BOOK_SLUG: &str = "enzyklopaedie-der-philosophischen-wissenschaften";
pub const BOOK_TITLE: &str = "Enzyklopädie der philosophischen Wissenschaften im Grundrisse";
pub const LANGUAGE: &str = "de";
pub const YEAR: &str = "2026";
/// The identity year of the edition presented: the third, final edition.
pub const ORIGINAL_YEAR: i16 = 1830;
pub const PUBLISHER: &str = "Scholia Sodalitas";
pub const SOURCE_URL: &str = "https://www.hegeledition.com";
pub const SOURCE: &str = "Constructed from the Digitale Hegel-Edition by Giuliano Infantino \
(hegeledition.com); the text follows the Gesammelte Werke (GW 20).";
pub const ABOUT: &str = "Hegel's Encyclopaedia of the Philosophical Sciences in Outline in its \
third and final edition (1830), following the text of the Gesammelten Werke (GW 20); page \
markers refer to the GW pagination, and the numbered paragraphs (§§) carry their own markers, \
with the Remarks (Anmerkungen) marked as the continuation of the paragraph they belong to. \
Scholia's version is constructed from the Digitale Hegel-Edition by Giuliano Infantino \
(hegeledition.com). The text itself is in the public domain. The digital edition on Scholia — \
including its modernized reading text — is prepared by Scholia Sodalitas, a community-driven \
project; the German text layers follow their source's terms (CC BY 4.0) and Scholia asserts no \
further rights in them. Corrections and refinements are welcome.";
pub const LICENCE: &str = "Public Domain. German text layers from hegeledition.com (CC BY 4.0).";

/// GW volume.page ("20.63"): the scholarly citation standard, hence the
/// citation default.
pub const GW_SYSTEM_SLUG: &str = "gw";
pub const GW_SYSTEM_LABEL: &str = "Gesammelte Werke";
pub const GW_SYSTEM_LABEL_EN: &str = "Gesammelte Werke";
pub const GW_SYSTEM_REF_TYPE: &str = "block";
pub const GW_CITE_PRIORITY: i16 = 1;
pub const GW_CITE_TEMPLATE: &str = "GW {ref}";

/// The paragraph number ("20") or, for a Remark, the number with an `A`
/// ("20 A"): the work's own citation standard, cited as "§ 20" / "§ 20 A".
pub const PARAGRAPH_SYSTEM_SLUG: &str = "paragraph";
pub const PARAGRAPH_SYSTEM_LABEL: &str = "Paragraphen";
pub const PARAGRAPH_SYSTEM_LABEL_EN: &str = "Paragraphs";
pub const PARAGRAPH_SYSTEM_REF_TYPE: &str = "inline";
pub const PARAGRAPH_CITE_PRIORITY: i16 = 0;
pub const PARAGRAPH_CITE_TEMPLATE: &str = "§ {ref}";
pub const PARAGRAPH_MARGIN_PREFIX: &str = "§ ";
