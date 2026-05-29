use serde::Serialize;
use utoipa::ToSchema;

use crate::modules::corpus::bibliography::models::SourceResponse;

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

/// Resolved bibliographic info for the "About this text" panel.
///
/// Walks ancestors of the active toc node looking for the deepest
/// `toc_nodes.source_id` and falls back to `books.source_id`. For
/// Bible-shape works this lets the panel show info about the
/// constituent book the reader is in (Genesis, Romans); for plain
/// works (Kant) it always falls back to the hosted book's source.
#[derive(Debug, Serialize, ToSchema)]
pub struct AboutThisTextResponse {
    pub source: SourceResponse,
    /// `"node"` when an ancestor toc_node supplied a source_id (Bible
    /// books-within-books); `"book"` when the resolver fell back to
    /// the hosted book's root source.
    pub resolved_from: String,
    /// Hosted source-language book, when this source is a translation
    /// and the original is also hosted (Kant EN → Kant DE). Absent for
    /// Bible-shape works whose originals aren't hosted.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub source_book: Option<BookSummary>,
    /// Editor's note for this hosted edition (free-form plain text).
    /// Mirrors `books.about_text` of the hosted book the reader is in
    /// (NOT walked across toc ancestors).
    #[serde(skip_serializing_if = "Option::is_none")]
    pub about_text: Option<String>,
}
