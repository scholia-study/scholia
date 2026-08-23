//! Book + reference-system metadata for the Wissenschaft der Logik edition,
//! consumed by `md_prose_to_struct::corpus`. Pure data — string constants
//! only, so the parser crate owns the struct shapes.

pub const MODERNIZED_DIR: &str = "assets/hegel2/curated/md_modernized";
pub const REVIEWED_DIR: &str = "assets/hegel2/curated/md_reviewed";
pub const TRANSLATED_DIR: &str = "assets/hegel2/curated/md_modernized_translated";
pub const OUTPUT_FILE: &str = "assets/hegel2/derived/output.json";
pub const TRANSLATION_OUTPUT_FILE: &str = "assets/hegel2/derived/translation_output.json";

pub const AUTHOR: &str = "Georg Wilhelm Friedrich Hegel";

pub const BOOK_SLUG: &str = "wissenschaft-der-logik";
pub const BOOK_TITLE: &str = "Wissenschaft der Logik";
pub const LANGUAGE: &str = "de";
pub const YEAR: &str = "2026";
/// The identity year of the edition presented: the work as completed by the
/// 1832 second edition of the Lehre vom Seyn (with the 1813 Wesen and 1816
/// Begriff, which Hegel never revised).
pub const ORIGINAL_YEAR: i16 = 1832;
pub const PUBLISHER: &str = "Scholia Sodalitas";
pub const SOURCE_URL: &str = "https://www.hegeledition.com";
pub const SOURCE: &str = "Constructed from the Digitale Hegel-Edition by Giuliano Infantino \
(hegeledition.com) and the Deutsches Textarchiv, both credited equally; the text follows the \
Gesammelte Werke (GW 21, 11, 12).";
pub const ABOUT: &str = "Hegel's Wissenschaft der Logik in the form modern scholarship reads it: \
the second edition of the Doctrine of Being (1832) together with the Doctrine of Essence (1813) \
and the Doctrine of the Concept (1816), following the text of the Gesammelten Werke (GW 21, 11, \
12); page markers refer to the GW pagination. Scholia's version is constructed from the Digitale \
Hegel-Edition by Giuliano Infantino (hegeledition.com) and the Deutsches Textarchiv, credited \
equally. The text itself is in the public domain. The digital edition on Scholia — including its \
modernized reading text — is prepared by Scholia Sodalitas, a community-driven project; the \
German text layers follow their sources' terms (CC BY 4.0 and CC BY-SA 4.0) and Scholia asserts \
no further rights in them. Corrections and refinements are welcome.";

/// The one reference system: GW volume.page ("21.68"), the scholarly
/// citation standard and therefore the citation default.
pub const GW_SYSTEM_SLUG: &str = "gw";
pub const GW_SYSTEM_LABEL: &str = "Gesammelte Werke";
pub const GW_SYSTEM_LABEL_EN: &str = "Gesammelte Werke";
pub const GW_SYSTEM_REF_TYPE: &str = "block";
pub const GW_CITE_PRIORITY: i16 = 0;
pub const GW_CITE_TEMPLATE: &str = "{self} · GW {ref}";

pub const BOOK_SLUG_EN: &str = "science-of-logic";
pub const BOOK_TITLE_EN: &str = "The Science of Logic";
pub const LANGUAGE_EN: &str = "en";
pub const YEAR_EN: &str = "2026";
pub const PUBLISHER_EN: &str = "Scholia Sodalitas";
pub const SOURCE_EN: &str = "Scholia's translation from the German of the Gesammelte Werke \
(GW 21, 11, 12), via the Digitale Hegel-Edition and the Deutsches Textarchiv.";
pub const ABOUT_EN: &str = "Hegel's Science of Logic in the form modern scholarship reads it: \
the second edition of the Doctrine of Being (1832) together with the Doctrine of Essence (1813) \
and the Doctrine of the Concept (1816). This is Scholia's own English translation, made directly \
from the German of the Gesammelten Werke and independent of every existing translation; page \
markers refer to the GW pagination. The digital edition is prepared by Scholia Sodalitas, a \
community-driven project. Unlike the German text layers (which follow their sources' terms), the \
translation is Scholia's own work under the regular Scholia assets licence. Corrections and \
refinements are welcome.";
