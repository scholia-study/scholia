use serde::Serialize;
use utoipa::ToSchema;

#[derive(Debug, Serialize, ToSchema)]
pub struct LibraryResponse {
    pub groups: Vec<LibraryGroup>,
    pub stats: LibraryStats,
}

/// A library entry as the index renders it: a heading plus a list of
/// works under it. The heading is either an author (`primary_kind =
/// "author"`) or the work itself (`primary_kind = "self"` — used for
/// authorless top-level works, including compilations like the Bible
/// and singletons like Gilgamesh).
#[derive(Debug, Serialize, ToSchema)]
pub struct LibraryGroup {
    /// Stable identifier. Person id for `"author"` groups; source id
    /// for `"self"` groups. UUIDs are globally unique so the id is
    /// safe to use as a React key without a kind prefix.
    pub id: String,
    /// `"author"` — heading is a person; `books` are works by them.
    /// `"self"` — heading IS the work; `books` are its child sources
    /// (compilation case) or empty (singleton case). Click the
    /// heading to open the work in the reader.
    pub primary_kind: String,
    pub primary_label: String,
    pub sort_name: String,
    /// Reader path for `primary_kind = "self"` headings (e.g.
    /// `/books/king-james-bible`). Absent for author headings until
    /// per-author pages exist.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub primary_slug: Option<String>,
    pub books: Vec<LibraryWork>,
    /// "Bible-shape" navigation: when this group is a compilation that is
    /// available in multiple translations, the frontend renders the
    /// compilation's child works (e.g. Genesis, John) as primary book
    /// pills here, and the multiple translations collapse into a single
    /// subtle translation chooser. Empty for regular author groups and
    /// for compilations available in a single translation.
    pub book_pills: Vec<BookPill>,
}

/// One entry in a Bible-shape group's primary pill row.
/// `node_slug` is the toc-node slug (translation-invariant by import
/// guard) — the frontend composes the URL as
/// `/books/<active-translation>/<node_slug>`.
#[derive(Debug, Serialize, ToSchema)]
pub struct BookPill {
    pub node_slug: String,
    pub label: String,
    pub sort_order: i32,
}

#[derive(Debug, Serialize, ToSchema)]
pub struct LibraryWork {
    pub work_id: String,
    pub title: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub publication_year: Option<i16>,
    pub co_authors: Vec<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub editor_names: Option<Vec<String>>,
    pub versions: Vec<LibraryVersion>,
}

#[derive(Debug, Serialize, ToSchema)]
pub struct LibraryVersion {
    pub book_slug: String,
    /// For nested works (a child source hosted inside a compilation):
    /// the toc-node slug to deep-link to. Absent for top-level books.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub node_slug: Option<String>,
    pub language: String,
    pub is_original: bool,
    pub translator_names: Vec<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub publisher: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub publication_year: Option<i16>,
}

#[derive(Debug, Serialize, ToSchema)]
pub struct LibraryStats {
    pub works: i64,
    pub authors: i64,
    pub languages: i64,
}
