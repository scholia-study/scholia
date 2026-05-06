use serde::{Deserialize, Serialize};
use utoipa::{IntoParams, ToSchema};

use crate::models::quotation::QuotationLimitsResponse;

// ── Response types ─────────────────────────────────────────

#[derive(Debug, Serialize, ToSchema)]
pub struct ArticleQuotationResponse {
    pub id: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub article_id: Option<String>,
    pub article_title: String,
    pub author_display_name: String,
    pub text: String,
    pub html: String,
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

// ── Request types ──────────────────────────────────────────

#[derive(Debug, Deserialize, ToSchema)]
pub struct CreateArticleQuotationRequest {
    pub article_id: String,
    pub text: String,
    pub html: String,
}

// ── Unified listing types ──────────────────────────────────

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
        sentence_kind: String,
        /// For footnote-kind anchors: the body sentence number the footnote
        /// is attached to. None for body-kind anchors.
        #[serde(skip_serializing_if = "Option::is_none")]
        anchor_main_sentence_number: Option<i32>,
        #[serde(skip_serializing_if = "Option::is_none")]
        start_text_snippet: Option<String>,
        #[serde(skip_serializing_if = "Option::is_none")]
        end_text_snippet: Option<String>,
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
