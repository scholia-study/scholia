//! Canonical structure of Ibsen's *Emperor and Galilean* (*Kejser og
//! Galilæer*, 1873) as a standalone authored work — the first **drama** on
//! Scholia. Two-layer text: `md_modernized` (modern Norwegian Bokmål, the
//! primary reading layer) + `md_reviewed` (the faithful 1873 first edition).
//!
//! The whole work nests two depth-0 **part** title-pages (`cf` Cæsars Frafall,
//! `kj` Keiser Julian), each parenting a cast list + five acts at depth 1.
//! Part One (`cf`) is fully modernized; Part Two (`kj`) is listed as far as it
//! is modernized (title-page, cast, act one so far) — its remaining acts slot
//! in here as they land. Part Two child slugs carry a `kj-` prefix because act
//! and cast names repeat across parts and node slugs are unique per book. See
//! ADR 0005 and the `dano-norwegian-drama-modernize` skill.
//!
//! Node **labels are not declared here** — they come from each file's
//! front-matter `label:` (the modernized spelling for the source book, the
//! translated spelling for the translation edition), so the markdown stays the
//! single source of truth. This module owns only the *structure* (source_ref,
//! slug, ltree path, depth, parent, filename, position).

/// The Norwegian (modernized-Bokmål) source edition.
pub const BOOK_SLUG: &str = "keiser-og-galileer";
pub const BOOK_TITLE: &str = "Keiser og Galileer";
pub const AUTHOR: &str = "Henrik Ibsen";
/// The primary reading layer is modern Norwegian Bokmål.
pub const LANGUAGE: &str = "nb";
pub const YEAR: i16 = 1873;

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
/// Publication year of the *translation* edition (a present-day Scholia
/// translation), distinct from the source's 1873 so the two editions don't
/// collide on the `sources (title, source_type, publication_year)` unique key.
pub const YEAR_EN: i16 = 2026;
pub const TRANSLATED_DIR: &str = "assets/ibsen1/curated/md_modernized_translated";
pub const SOURCE_EN: &str = "English reading translation prepared from the modern Norwegian Bokmål \
text; the underlying source is Ibsen's 1873 first edition (Henrik Ibsens Skrifter).";
pub const ABOUT_EN: &str = "An English reading translation of Henrik Ibsen's Emperor and Galilean \
(Kejser og Galilæer, 1873), prepared from the modern Norwegian Bokmål edition that serves as the \
underlying text on Scholia. A Scholia community project; corrections are welcome.";

/// The `1873` page reference system — the printed-page markers (`{{{ N }}}`) of
/// the first edition, and drama's **default** citation (`p. N`).
pub const PAGE_SYSTEM_SLUG: &str = "1873";
pub const PAGE_SYSTEM_LABEL: &str = "1873 page";

pub const SOURCE: &str = "Modern Norwegian Bokmål reading text; the original layer reproduces the \
1873 first edition (Dano-Norwegian) from Henrik Ibsens Skrifter (HIS), University of Oslo.";
pub const ABOUT: &str = "Keiser og Galileer (Kejser og Galilæer, 1873) by Henrik Ibsen, a \
two-part world-historical drama. The reading text is a modern Norwegian Bokmål modernization; \
the original layer reproduces the 1873 first edition. The digital edition on Scholia is a \
community-driven project; corrections are welcome.";

/// One reading node — its structure only; the display label comes from the
/// file's front matter at parse time.
pub struct DramaNode {
    pub source_ref: &'static str,
    pub slug: &'static str,
    pub path: &'static str,
    pub depth: i16,
    pub parent_source_ref: Option<&'static str>,
    pub filename: &'static str,
    pub position: u32,
}

/// The part title-pages (depth 0), each parenting a cast list + acts
/// (depth 1). The canonical file set the parser validates the two curated
/// layers against — a missing, extra, or misnamed file is an error.
pub fn nodes() -> Vec<DramaNode> {
    vec![
        DramaNode {
            source_ref: "cf",
            slug: "caesars-frafall",
            path: "caesars-frafall",
            depth: 0,
            parent_source_ref: None,
            filename: "001_cf_titelblad.md",
            position: 1,
        },
        DramaNode {
            source_ref: "cf-de-opptredende",
            slug: "de-opptredende",
            path: "caesars-frafall.de-opptredende",
            depth: 1,
            parent_source_ref: Some("cf"),
            filename: "002_cf_de_optraedende.md",
            position: 2,
        },
        DramaNode {
            source_ref: "cf-foerste-handling",
            slug: "foerste-handling",
            path: "caesars-frafall.foerste-handling",
            depth: 1,
            parent_source_ref: Some("cf"),
            filename: "003_cf_foerste_handling.md",
            position: 3,
        },
        DramaNode {
            source_ref: "cf-annen-handling",
            slug: "annen-handling",
            path: "caesars-frafall.annen-handling",
            depth: 1,
            parent_source_ref: Some("cf"),
            filename: "004_cf_anden_handling.md",
            position: 4,
        },
        DramaNode {
            source_ref: "cf-tredje-handling",
            slug: "tredje-handling",
            path: "caesars-frafall.tredje-handling",
            depth: 1,
            parent_source_ref: Some("cf"),
            filename: "005_cf_tredje_handling.md",
            position: 5,
        },
        DramaNode {
            source_ref: "cf-fjerde-handling",
            slug: "fjerde-handling",
            path: "caesars-frafall.fjerde-handling",
            depth: 1,
            parent_source_ref: Some("cf"),
            filename: "006_cf_fjerde_handling.md",
            position: 6,
        },
        DramaNode {
            source_ref: "cf-femte-handling",
            slug: "femte-handling",
            path: "caesars-frafall.femte-handling",
            depth: 1,
            parent_source_ref: Some("cf"),
            filename: "007_cf_femte_handling.md",
            position: 7,
        },
        DramaNode {
            source_ref: "kj",
            slug: "keiser-julian",
            path: "keiser-julian",
            depth: 0,
            parent_source_ref: None,
            filename: "008_kj_titelblad.md",
            position: 8,
        },
        DramaNode {
            source_ref: "kj-de-opptredende",
            slug: "kj-de-opptredende",
            path: "keiser-julian.de-opptredende",
            depth: 1,
            parent_source_ref: Some("kj"),
            filename: "009_kj_de_optraedende.md",
            position: 9,
        },
        DramaNode {
            source_ref: "kj-foerste-handling",
            slug: "kj-foerste-handling",
            path: "keiser-julian.foerste-handling",
            depth: 1,
            parent_source_ref: Some("kj"),
            filename: "010_kj_foerste_handling.md",
            position: 10,
        },
    ]
}
