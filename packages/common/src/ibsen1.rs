//! Canonical structure of Ibsen's *Emperor and Galilean* (*Kejser og
//! Galilæer*, 1873) as a standalone authored work — the first **drama** on
//! Scholia. Two-layer text: `md_modernized` (modern Norwegian Bokmål, the
//! primary reading layer) + `md_reviewed` (the faithful 1873 first edition).
//!
//! The whole work nests two depth-0 **part** title-pages (`cf` Cæsars Frafall,
//! `kj` Keiser Julian), each parenting a cast list + five acts at depth 1.
//! Both parts are fully modernized and listed. Part Two child slugs carry a
//! `kj-` prefix because act and cast names repeat across parts and node slugs
//! are unique per book. See ADR 0005 and the `dano-norwegian-drama-modernize`
//! skill.
//!
//! Node **labels are not declared here** — they come from each file's
//! front-matter `label:` (the modernized spelling for the source book, the
//! translated spelling for the translation edition), so the markdown stays the
//! single source of truth. This module owns only the *structure* (source_ref,
//! slug, ltree path, depth, parent, filename, position).

/// The Norwegian (modernized-Bokmål) source edition.
use crate::drama::Node;

pub const BOOK_SLUG: &str = "keiser-og-galileer";
pub const BOOK_TITLE: &str = "Keiser og Galileer";
pub const AUTHOR: &str = "Henrik Ibsen";
/// The primary reading layer is modern Norwegian Bokmål.
pub const LANGUAGE: &str = "nb";
/// The Bokmål modernization is judgment-heavy enough to be Scholia's own
/// edition: published by Scholia Sodalitas, dated by the edition itself,
/// with 1873 as identity year. The first-edition imprint ("København, Den
/// Gyldendalske Boghandel (F. Hegel), 1873" per the HIS TEI biblStruct —
/// `assets/ibsen1/raw/DRVIT_KG_KG73.xml`) is noted in the about text; the
/// original layer reproduces that printing.
pub const YEAR: i16 = 2026;
pub const ORIGINAL_YEAR: i16 = 1873;
pub const PUBLISHER: &str = "Scholia Sodalitas";
/// The exact HIS page the TEI was taken from.
pub const SOURCE_URL: &str = "https://ibsen.uio.no/DRVIT_KG%7CKG73.html?facs=Ja";

pub const MODERNIZED_DIR: &str = "assets/ibsen1/curated/md_modernized";
pub const REVIEWED_DIR: &str = "assets/ibsen1/curated/md_reviewed";
pub const OUTPUT_FILE: &str = "assets/ibsen1/derived/output.json";
pub const TRANSLATION_OUTPUT_FILE: &str = "assets/ibsen1/derived/translation_output.json";

/// English translation layer — a separate "translation edition" book locked 1:1
/// to the Norwegian source book (`BOOK_SLUG`) and shown as its side-by-side
/// companion. Mirrors kant1's `critique-of-pure-reason-b` ↔ translation pair.
pub const BOOK_SLUG_EN: &str = "emperor-and-galilean";
pub const BOOK_TITLE_EN: &str = "Emperor and Galilean";
pub const LANGUAGE_EN: &str = "en";
/// Publication year of the *translation* edition. Both editions are now
/// 2026 Scholia Sodalitas publications; their distinct titles keep the
/// `sources (title, source_type, publication_year)` unique key clear.
pub const YEAR_EN: i16 = 2026;
/// The community imprint the translation is published under (matches
/// the Kant EN editions).
pub const PUBLISHER_EN: &str = "Scholia Sodalitas";
pub const TRANSLATED_DIR: &str = "assets/ibsen1/curated/md_modernized_translated";
pub const SOURCE_EN: &str = "English reading translation prepared from the modern Norwegian Bokmål \
text; the underlying source is Ibsen's 1873 first edition (Henrik Ibsens Skrifter).";
pub const ABOUT_EN: &str = "An English reading translation of Henrik Ibsen's Emperor and Galilean \
(Kejser og Galilæer, 1873), prepared from the modern Norwegian Bokmål edition that serves as the \
underlying text on Scholia. A community project published by Scholia Sodalitas; corrections are welcome.";
pub const LICENCE_EN: &str = "CC BY-NC-ND 4.0";

/// The `1873` page reference system — the printed-page markers (`{{{ N }}}`) of
/// the first edition, and drama's **default** citation (`p. N`).
pub const PAGE_SYSTEM_SLUG: &str = "1873";
pub const PAGE_SYSTEM_LABEL: &str = "1873 page";

pub const SOURCE: &str = "Modern Norwegian Bokmål reading text; the original layer reproduces the \
1873 first edition (Dano-Norwegian) from Henrik Ibsens Skrifter (HIS), University of Oslo.";
pub const ABOUT: &str = "Keiser og Galileer (Kejser og Galilæer, 1873) by Henrik Ibsen, a \
two-part world-historical drama. The reading text is a modern Norwegian Bokmål modernization \
prepared by Scholia Sodalitas; the original layer reproduces the 1873 first edition — Den \
Gyldendalske Boghandel (F. Hegel), København. The digital edition on Scholia is a \
community-driven project; corrections are welcome.";
pub const LICENCE: &str = "Public Domain (source text); CC BY-NC-ND 4.0 (modernized reading text)";

/// The part title-pages (depth 0), each parenting a cast list + acts
/// (depth 1). The canonical file set the parser validates the two curated
/// layers against — a missing, extra, or misnamed file is an error.
pub fn nodes() -> Vec<Node> {
    vec![
        Node {
            source_ref: "cf".into(),
            slug: "caesars-frafall".into(),
            path: "caesars-frafall".into(),
            depth: 0,
            parent_source_ref: None,
            filename: "001_cf_titelblad.md".into(),
            position: 1,
        },
        Node {
            source_ref: "cf-de-opptredende".into(),
            slug: "de-opptredende".into(),
            path: "caesars-frafall.de-opptredende".into(),
            depth: 1,
            parent_source_ref: Some("cf".into()),
            filename: "002_cf_de_optraedende.md".into(),
            position: 2,
        },
        Node {
            source_ref: "cf-foerste-handling".into(),
            slug: "foerste-handling".into(),
            path: "caesars-frafall.foerste-handling".into(),
            depth: 1,
            parent_source_ref: Some("cf".into()),
            filename: "003_cf_foerste_handling.md".into(),
            position: 3,
        },
        Node {
            source_ref: "cf-annen-handling".into(),
            slug: "annen-handling".into(),
            path: "caesars-frafall.annen-handling".into(),
            depth: 1,
            parent_source_ref: Some("cf".into()),
            filename: "004_cf_anden_handling.md".into(),
            position: 4,
        },
        Node {
            source_ref: "cf-tredje-handling".into(),
            slug: "tredje-handling".into(),
            path: "caesars-frafall.tredje-handling".into(),
            depth: 1,
            parent_source_ref: Some("cf".into()),
            filename: "005_cf_tredje_handling.md".into(),
            position: 5,
        },
        Node {
            source_ref: "cf-fjerde-handling".into(),
            slug: "fjerde-handling".into(),
            path: "caesars-frafall.fjerde-handling".into(),
            depth: 1,
            parent_source_ref: Some("cf".into()),
            filename: "006_cf_fjerde_handling.md".into(),
            position: 6,
        },
        Node {
            source_ref: "cf-femte-handling".into(),
            slug: "femte-handling".into(),
            path: "caesars-frafall.femte-handling".into(),
            depth: 1,
            parent_source_ref: Some("cf".into()),
            filename: "007_cf_femte_handling.md".into(),
            position: 7,
        },
        Node {
            source_ref: "kj".into(),
            slug: "keiser-julian".into(),
            path: "keiser-julian".into(),
            depth: 0,
            parent_source_ref: None,
            filename: "008_kj_titelblad.md".into(),
            position: 8,
        },
        Node {
            source_ref: "kj-de-opptredende".into(),
            slug: "kj-de-opptredende".into(),
            path: "keiser-julian.de-opptredende".into(),
            depth: 1,
            parent_source_ref: Some("kj".into()),
            filename: "009_kj_de_optraedende.md".into(),
            position: 9,
        },
        Node {
            source_ref: "kj-foerste-handling".into(),
            slug: "kj-foerste-handling".into(),
            path: "keiser-julian.foerste-handling".into(),
            depth: 1,
            parent_source_ref: Some("kj".into()),
            filename: "010_kj_foerste_handling.md".into(),
            position: 10,
        },
        Node {
            source_ref: "kj-annen-handling".into(),
            slug: "kj-annen-handling".into(),
            path: "keiser-julian.annen-handling".into(),
            depth: 1,
            parent_source_ref: Some("kj".into()),
            filename: "011_kj_anden_handling.md".into(),
            position: 11,
        },
        Node {
            source_ref: "kj-tredje-handling".into(),
            slug: "kj-tredje-handling".into(),
            path: "keiser-julian.tredje-handling".into(),
            depth: 1,
            parent_source_ref: Some("kj".into()),
            filename: "012_kj_tredje_handling.md".into(),
            position: 12,
        },
        Node {
            source_ref: "kj-fjerde-handling".into(),
            slug: "kj-fjerde-handling".into(),
            path: "keiser-julian.fjerde-handling".into(),
            depth: 1,
            parent_source_ref: Some("kj".into()),
            filename: "013_kj_fjerde_handling.md".into(),
            position: 13,
        },
        Node {
            source_ref: "kj-femte-handling".into(),
            slug: "kj-femte-handling".into(),
            path: "keiser-julian.femte-handling".into(),
            depth: 1,
            parent_source_ref: Some("kj".into()),
            filename: "014_kj_femte_handling.md".into(),
            position: 14,
        },
    ]
}
