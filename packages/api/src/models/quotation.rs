use serde::{Deserialize, Serialize};
use utoipa::{IntoParams, ToSchema};

// ── Response types ─────────────────────────────────────────

#[derive(Debug, Serialize, ToSchema)]
pub struct QuotationResponse {
    pub id: String,
    pub anchor_sentence_start_number: i32,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub anchor_sentence_end_number: Option<i32>,
    pub sentence_kind: String,
    pub note_count: i64,
    pub created_at: String,
}

#[derive(Debug, Serialize, ToSchema)]
pub struct QuotationListResponse {
    pub quotations: Vec<QuotationResponse>,
}

#[derive(Debug, Serialize, ToSchema)]
pub struct CreateQuotationResponse {
    pub quotation: QuotationResponse,
    pub created: bool,
}

#[derive(Debug, Serialize, ToSchema)]
pub struct NoteResponse {
    pub id: String,
    pub body: String,
    pub tags: Vec<TagResponse>,
    pub created_at: String,
    pub updated_at: String,
}

#[derive(Debug, Serialize, ToSchema)]
pub struct NoteListResponse {
    pub notes: Vec<NoteResponse>,
}

#[derive(Debug, Clone, Serialize, ToSchema)]
pub struct TagResponse {
    pub id: String,
    pub name: String,
}

#[derive(Debug, Serialize, ToSchema)]
pub struct TagListResponse {
    pub tags: Vec<TagResponse>,
}

// ── Global listing response types ──────────────────────────

#[derive(Debug, Serialize, ToSchema)]
pub struct QuotationWithContextResponse {
    pub id: String,
    pub book_slug: String,
    pub book_title: String,
    pub node_label: String,
    pub node_slug: String,
    pub anchor_sentence_start_number: i32,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub anchor_sentence_end_number: Option<i32>,
    pub sentence_kind: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub start_text_snippet: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub end_text_snippet: Option<String>,
    pub note_count: i64,
    pub created_at: String,
}

#[derive(Debug, Serialize, ToSchema)]
pub struct QuotationWithContextListResponse {
    pub quotations: Vec<QuotationWithContextResponse>,
}

#[derive(Debug, Serialize, ToSchema)]
pub struct NoteWithContextResponse {
    pub id: String,
    pub body: String,
    pub tags: Vec<TagResponse>,
    pub book_slug: String,
    pub book_title: String,
    pub node_label: String,
    pub node_slug: String,
    pub anchor_sentence_start_number: i32,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub anchor_sentence_end_number: Option<i32>,
    pub sentence_kind: String,
    pub quotation_id: String,
    pub created_at: String,
    pub updated_at: String,
}

#[derive(Debug, Serialize, ToSchema)]
pub struct NoteWithContextListResponse {
    pub notes: Vec<NoteWithContextResponse>,
}

#[derive(Debug, Deserialize, IntoParams)]
pub struct GlobalListQuery {
    #[serde(default)]
    pub book_slug: Option<String>,
}

// ── Request types ──────────────────────────────────────────

#[derive(Debug, Deserialize, IntoParams)]
pub struct QuotationQuery {
    pub node_id: String,
}

#[derive(Debug, Deserialize, ToSchema)]
pub struct CreateQuotationRequest {
    pub sentence_start: i32,
    pub sentence_end: Option<i32>,
    pub sentence_kind: String,
}

#[derive(Debug, Deserialize, ToSchema)]
pub struct CreateNoteRequest {
    pub body: String,
    #[serde(default)]
    pub tags: Vec<String>,
}

#[derive(Debug, Deserialize, ToSchema)]
pub struct UpdateNoteRequest {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub body: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub tags: Option<Vec<String>>,
}
