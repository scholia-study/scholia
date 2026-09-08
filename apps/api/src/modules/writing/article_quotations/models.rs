use serde::{Deserialize, Serialize};
use utoipa::{IntoParams, ToSchema};

use crate::modules::writing::quotations::models::QuotationLimitsResponse;

#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize, sqlx::Type, ToSchema)]
#[sqlx(type_name = "article_quotation_kind", rename_all = "lowercase")]
#[serde(rename_all = "lowercase")]
pub enum ArticleQuotationKind {
    Text,
    Figure,
}

/// Snapshot of a quoted figure, extracted server-side from the quoted
/// article's rendered HTML at save time.
#[derive(Debug, Serialize, ToSchema)]
pub struct ArticleQuotationFigure {
    pub src: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub alt: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub caption: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub width: Option<i32>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub height: Option<i32>,
}

#[derive(Debug, Serialize, ToSchema)]
pub struct ArticleQuotationResponse {
    pub id: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub article_id: Option<String>,
    pub article_title: String,
    pub author_display_name: String,
    pub kind: ArticleQuotationKind,
    pub text: String,
    pub html: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub figure: Option<ArticleQuotationFigure>,
    pub note_count: i64,
    pub created_at: String,
}

#[derive(Debug, Serialize, ToSchema)]
pub struct ArticleQuotationListResponse {
    pub article_quotations: Vec<ArticleQuotationResponse>,
}

#[derive(Debug, Serialize, ToSchema)]
pub struct CreateArticleQuotationResponse {
    pub article_quotation: ArticleQuotationResponse,
    pub created: bool,
}

#[derive(Debug, Deserialize, ToSchema)]
pub struct CreateArticleQuotationRequest {
    pub article_id: String,
    /// Required for text quotations; ignored for figure quotations.
    #[serde(default)]
    pub text: Option<String>,
    /// Required for text quotations; ignored for figure quotations.
    #[serde(default)]
    pub html: Option<String>,
    /// Present = save a figure quotation of the uploaded image at this
    /// `/media/` src. The alt/caption/dimension snapshot is extracted from
    /// the article's own rendered HTML, never from the client.
    #[serde(default)]
    pub figure_src: Option<String>,
}

#[derive(Debug, Serialize, ToSchema)]
#[serde(tag = "source_type")]
pub enum UnifiedQuotationResponse {
    #[serde(rename = "book")]
    Book {
        id: String,
        book_slug: String,
        /// See `QuotationWithContextResponse::translation_label`.
        #[serde(skip_serializing_if = "Option::is_none")]
        translation_label: Option<String>,
        book_title: String,
        node_label: String,
        node_slug: String,
        anchor_sentence_start_number: i32,
        #[serde(skip_serializing_if = "Option::is_none")]
        anchor_sentence_end_number: Option<i32>,
        /// See `QuotationWithContextResponse::anchor_sentence_start_id`.
        anchor_sentence_start_id: String,
        #[serde(skip_serializing_if = "Option::is_none")]
        anchor_sentence_end_id: Option<String>,
        /// See `QuotationWithContextResponse::anchor_block_type`.
        #[serde(skip_serializing_if = "Option::is_none")]
        anchor_block_type: Option<String>,
        sentence_kind: crate::modules::corpus::SentenceKind,
        /// For footnote-kind anchors: the body sentence number the footnote
        /// is attached to. None for body-kind anchors.
        #[serde(skip_serializing_if = "Option::is_none")]
        anchor_main_sentence_number: Option<i32>,
        #[serde(skip_serializing_if = "Option::is_none")]
        start_text_snippet: Option<String>,
        #[serde(skip_serializing_if = "Option::is_none")]
        end_text_snippet: Option<String>,
        /// See `QuotationWithContextResponse::has_source_view`.
        has_source_view: bool,
        note_count: i64,
        created_at: String,
    },
    #[serde(rename = "article")]
    Article {
        id: String,
        #[serde(skip_serializing_if = "Option::is_none")]
        article_id: Option<String>,
        article_title: String,
        author_display_name: String,
        text_snippet: String,
        /// Present for figure quotations: the quoted image, for a list
        /// thumbnail.
        #[serde(skip_serializing_if = "Option::is_none")]
        figure_src: Option<String>,
        #[serde(skip_serializing_if = "Option::is_none")]
        figure_alt: Option<String>,
        note_count: i64,
        created_at: String,
    },
}

#[derive(Debug, Serialize, ToSchema)]
pub struct UnifiedQuotationListResponse {
    pub quotations: Vec<UnifiedQuotationResponse>,
    pub limits: QuotationLimitsResponse,
}

#[derive(Debug, Deserialize, IntoParams)]
pub struct UnifiedListQuery {
    #[serde(default)]
    pub book_slug: Option<String>,
    #[serde(default)]
    pub source_type: Option<String>,
}
