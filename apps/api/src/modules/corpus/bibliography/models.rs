use serde::{Deserialize, Serialize};
use utoipa::{IntoParams, ToSchema};

use crate::system::serde_util::double_option;

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
    pub title_display: Option<String>,
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
    #[serde(skip_serializing_if = "Option::is_none")]
    pub translation_of_id: Option<String>,
    pub created_by: String,
    pub protected: bool,
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
    pub created_by: String,
    pub protected: bool,
}

#[derive(Debug, Serialize, ToSchema)]
pub struct PersonResponse {
    pub id: String,
    pub name: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub sort_name: Option<String>,
    pub created_by: String,
    pub protected: bool,
}

#[derive(Debug, Serialize, ToSchema)]
pub struct SourceSearchResponse {
    pub id: String,
    pub source_type: String,
    pub title: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub publication_year: Option<i16>,
    pub created_by: String,
    pub protected: bool,
    pub persons: Vec<SourcePersonResponse>,
}

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

#[derive(Debug, Deserialize, IntoParams)]
pub struct SourceBrowseQuery {
    #[serde(default)]
    pub q: Option<String>,
    #[serde(default)]
    pub source_type: Option<String>,
    #[serde(default)]
    pub created_by_me: Option<bool>,
    #[serde(default)]
    pub protected: Option<bool>,
    #[serde(default)]
    pub page: Option<i32>,
    #[serde(default)]
    pub per_page: Option<i32>,
}

#[derive(Debug, Serialize, ToSchema)]
pub struct SourceBrowseResponse {
    pub sources: Vec<SourceSearchResponse>,
    pub total: i64,
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
    #[serde(default, deserialize_with = "double_option")]
    pub verbatim_kind: Option<Option<String>>,
    pub sentence_start: Option<i32>,
    pub sentence_end: Option<i32>,
    pub sentence_kind: Option<String>,
    #[serde(default, deserialize_with = "double_option")]
    pub source_id: Option<Option<String>>,
    #[serde(default, deserialize_with = "double_option")]
    pub source_page_start: Option<Option<i32>>,
    #[serde(default, deserialize_with = "double_option")]
    pub source_page_end: Option<Option<i32>>,
    #[serde(default, deserialize_with = "double_option")]
    pub source_location_freeform: Option<Option<String>>,
    #[serde(default, deserialize_with = "double_option")]
    pub quoted_text: Option<Option<String>>,
    #[serde(default, deserialize_with = "double_option")]
    pub editor_note: Option<Option<String>>,
    pub is_featured: Option<bool>,
    #[serde(default, deserialize_with = "double_option")]
    pub admin_notes: Option<Option<String>>,
}

#[derive(Debug, Deserialize, ToSchema)]
pub struct CreateSourceRequest {
    pub source_type: String,
    pub title: String,
    pub title_display: Option<String>,
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
    pub translation_of_id: Option<String>,
}

/// Patch for a source. Nullable columns use `Option<Option<T>>` (via
/// `double_option`) so an omitted field is left unchanged while an explicit
/// `null` clears the column. `title` (NOT NULL) and `source_type` (immutable)
/// stay plain `Option<T>`.
#[derive(Debug, Deserialize, ToSchema)]
pub struct UpdateSourceRequest {
    pub source_type: Option<String>,
    pub title: Option<String>,
    #[serde(default, deserialize_with = "double_option")]
    pub title_display: Option<Option<String>>,
    #[serde(default, deserialize_with = "double_option")]
    pub publication_year: Option<Option<i16>>,
    #[serde(default, deserialize_with = "double_option")]
    pub publisher: Option<Option<String>>,
    #[serde(default, deserialize_with = "double_option")]
    pub isbn: Option<Option<Vec<String>>>,
    #[serde(default, deserialize_with = "double_option")]
    pub doi: Option<Option<String>>,
    #[serde(default, deserialize_with = "double_option")]
    pub edition: Option<Option<String>>,
    #[serde(default, deserialize_with = "double_option")]
    pub volume: Option<Option<String>>,
    #[serde(default, deserialize_with = "double_option")]
    pub journal_name: Option<Option<String>>,
    #[serde(default, deserialize_with = "double_option")]
    pub url: Option<Option<String>>,
    #[serde(default, deserialize_with = "double_option")]
    pub page_start: Option<Option<i32>>,
    #[serde(default, deserialize_with = "double_option")]
    pub page_end: Option<Option<i32>>,
    #[serde(default, deserialize_with = "double_option")]
    pub parent_source_id: Option<Option<String>>,
    #[serde(default, deserialize_with = "double_option")]
    pub translation_of_id: Option<Option<String>>,
    pub protected: Option<bool>,
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
    /// Total count across all categories.
    pub total: i64,
    pub resources: ReferencedResources,
    pub child_sources: ReferencedChildSources,
    pub articles: ReferencedArticles,
}

#[derive(Debug, Serialize, ToSchema)]
pub struct ReferencedResources {
    pub count: i64,
    pub ids: Vec<String>,
}

#[derive(Debug, Serialize, ToSchema)]
pub struct ReferencedChildSources {
    pub count: i64,
    pub items: Vec<ReferencedChildSource>,
}

#[derive(Debug, Serialize, ToSchema)]
pub struct ReferencedChildSource {
    pub id: String,
    pub title: String,
    pub relation: String, // "parent" | "translation"
}

#[derive(Debug, Serialize, ToSchema)]
pub struct ReferencedArticles {
    pub count: i64,
    pub items: Vec<ReferencedArticle>,
}

#[derive(Debug, Serialize, ToSchema)]
pub struct ReferencedArticle {
    pub id: String,
    pub title: String,
    pub slug: String,
    pub status: String, // "draft" | "published" | "archived"
    pub is_mine: bool,
}
