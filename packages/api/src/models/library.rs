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
