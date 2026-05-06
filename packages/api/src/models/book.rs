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
    pub source_id: String,
    pub publication_year: Option<i16>,
    pub publisher: Option<String>,
    /// The hosted source-language book this book translates from
    /// (Kant EN → Kant DE). Absent when there is no hosted source —
    /// e.g. Bible translations from Hebrew/Greek that we don't host.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub source_book_slug: Option<String>,
    /// Books translated from THIS book. Populated for source-language
    /// books (Kant DE → [Kant EN]); empty for translation leaves.
    pub translations: Vec<BookSummary>,
    /// Sibling translations: every other book that shares this book's
    /// translation root. Populated when the work has no hosted source
    /// language (Bible: KJV book has [WEB]) so the reader's flat
    /// translation picker can list peer translations regardless of
    /// source/translation polarity. Empty for Kant-style works where
    /// `source_book_slug` and `translations` already express the
    /// relationship. (PLAN_BIG_BOOKS.md Q6)
    pub sibling_translations: Vec<BookSummary>,
}
