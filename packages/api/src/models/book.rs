use serde::Serialize;
use utoipa::ToSchema;

#[derive(Debug, Serialize, ToSchema)]
pub struct BookSummary {
    pub id: String,
    pub slug: String,
    pub title: String,
    pub author: String,
    pub language: String,
}

#[derive(Debug, Serialize, ToSchema)]
pub struct BookDetail {
    pub id: String,
    pub slug: String,
    pub title: String,
    pub author: String,
    pub language: String,
    pub source: Option<String>,
    pub source_date: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub source_book_id: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub source_book_slug: Option<String>,
    pub translations: Vec<BookSummary>,
}
