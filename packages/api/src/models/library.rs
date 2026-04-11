use serde::Serialize;
use utoipa::ToSchema;

#[derive(Debug, Serialize, ToSchema)]
pub struct LibraryResponse {
    pub authors: Vec<LibraryAuthor>,
    pub stats: LibraryStats,
}

#[derive(Debug, Serialize, ToSchema)]
pub struct LibraryAuthor {
    pub id: String,
    pub name: String,
    pub sort_name: String,
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
