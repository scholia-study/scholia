use serde::{Deserialize, Serialize};
use utoipa::{IntoParams, ToSchema};

// ── Response types ─────────────────────────────────────────

#[derive(Debug, Serialize, ToSchema)]
pub struct ResourceResponse {
    pub id: String,
    pub resource_type: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub verbatim_kind: Option<String>,
    pub anchor_sentence_start_number: i32,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub anchor_sentence_end_number: Option<i32>,
    pub sentence_kind: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub quoted_text: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub editor_note: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub source: Option<SourceResponse>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub source_page_start: Option<i32>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub source_page_end: Option<i32>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub source_location_freeform: Option<String>,
    pub is_featured: bool,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub admin_notes: Option<String>,
    pub created_at: String,
}

#[derive(Debug, Serialize, ToSchema)]
pub struct ResourceListResponse {
    pub resources: Vec<ResourceResponse>,
}

#[derive(Debug, Clone, Serialize, ToSchema)]
pub struct SourceResponse {
    pub id: String,
    pub source_type: String,
    pub title: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub publication_year: Option<i16>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub publisher: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub isbn: Option<Vec<String>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub doi: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub edition: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub volume: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub journal_name: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub url: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub page_start: Option<i32>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub page_end: Option<i32>,
    pub persons: Vec<SourcePersonResponse>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub parent: Option<Box<ParentSourceResponse>>,
}

#[derive(Debug, Clone, Serialize, ToSchema)]
pub struct ParentSourceResponse {
    pub id: String,
    pub source_type: String,
    pub title: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub publication_year: Option<i16>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub publisher: Option<String>,
    pub persons: Vec<SourcePersonResponse>,
}

#[derive(Debug, Clone, Serialize, ToSchema)]
pub struct SourcePersonResponse {
    pub person_id: String,
    pub name: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub sort_name: Option<String>,
    pub role: String,
    pub position: i16,
}

#[derive(Debug, Serialize, ToSchema)]
pub struct PersonResponse {
    pub id: String,
    pub name: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub sort_name: Option<String>,
}

#[derive(Debug, Serialize, ToSchema)]
pub struct SourceSearchResponse {
    pub id: String,
    pub source_type: String,
    pub title: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub publication_year: Option<i16>,
    pub persons: Vec<SourcePersonResponse>,
}

// ── Request types ──────────────────────────────────────────

#[derive(Debug, Deserialize, IntoParams)]
pub struct ResourceQuery {
    pub start: i32,
    pub end: i32,
    #[serde(default = "default_body")]
    pub kind: String,
}

fn default_body() -> String {
    "body".to_string()
}

#[derive(Debug, Deserialize, IntoParams)]
pub struct SearchQuery {
    #[serde(default)]
    pub q: String,
}

#[derive(Debug, Deserialize, ToSchema)]
pub struct CreateResourceRequest {
    pub resource_type: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub verbatim_kind: Option<String>,
    pub sentence_start: i32,
    pub sentence_end: Option<i32>,
    pub sentence_kind: String,
    pub source_id: Option<String>,
    pub source_page_start: Option<i32>,
    pub source_page_end: Option<i32>,
    pub source_location_freeform: Option<String>,
    pub quoted_text: Option<String>,
    pub editor_note: Option<String>,
    pub is_featured: Option<bool>,
    pub admin_notes: Option<String>,
}

#[derive(Debug, Deserialize, ToSchema)]
pub struct UpdateResourceRequest {
    pub resource_type: Option<String>,
    pub verbatim_kind: Option<String>,
    pub sentence_start: Option<i32>,
    pub sentence_end: Option<i32>,
    pub sentence_kind: Option<String>,
    pub source_id: Option<String>,
    pub source_page_start: Option<i32>,
    pub source_page_end: Option<i32>,
    pub source_location_freeform: Option<String>,
    pub quoted_text: Option<String>,
    pub editor_note: Option<String>,
    pub is_featured: Option<bool>,
    pub admin_notes: Option<String>,
}

#[derive(Debug, Deserialize, ToSchema)]
pub struct CreateSourceRequest {
    pub source_type: String,
    pub title: String,
    pub publication_year: Option<i16>,
    pub publisher: Option<String>,
    pub isbn: Option<Vec<String>>,
    pub doi: Option<String>,
    pub edition: Option<String>,
    pub volume: Option<String>,
    pub journal_name: Option<String>,
    pub url: Option<String>,
    pub page_start: Option<i32>,
    pub page_end: Option<i32>,
    pub parent_source_id: Option<String>,
}

#[derive(Debug, Deserialize, ToSchema)]
pub struct UpdateSourceRequest {
    pub source_type: Option<String>,
    pub title: Option<String>,
    pub publication_year: Option<i16>,
    pub publisher: Option<String>,
    pub isbn: Option<Vec<String>>,
    pub doi: Option<String>,
    pub edition: Option<String>,
    pub volume: Option<String>,
    pub journal_name: Option<String>,
    pub url: Option<String>,
    pub page_start: Option<i32>,
    pub page_end: Option<i32>,
    pub parent_source_id: Option<String>,
}

#[derive(Debug, Deserialize, ToSchema)]
pub struct CreatePersonRequest {
    pub name: String,
    pub sort_name: Option<String>,
}

#[derive(Debug, Deserialize, ToSchema)]
pub struct UpdatePersonRequest {
    pub name: Option<String>,
    pub sort_name: Option<String>,
}

#[derive(Debug, Deserialize, ToSchema)]
pub struct LinkSourcePersonRequest {
    pub person_id: String,
    pub role: String,
    pub position: Option<i16>,
}

#[derive(Debug, Serialize, ToSchema)]
pub struct ReferenceCheckResponse {
    pub count: i64,
    pub resource_ids: Vec<String>,
}
