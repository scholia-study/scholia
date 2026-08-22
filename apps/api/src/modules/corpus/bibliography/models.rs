use serde::{Deserialize, Serialize};
use utoipa::{IntoParams, ToSchema};

/// Stored as the Postgres enum `source_type`.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, ToSchema, sqlx::Type)]
#[serde(rename_all = "lowercase")]
#[sqlx(type_name = "source_type", rename_all = "lowercase")]
pub enum SourceType {
    Book,
    Article,
    Chapter,
    Journal,
    Web,
}

/// Stored as the Postgres enum `source_person_role`.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, ToSchema, sqlx::Type)]
#[serde(rename_all = "lowercase")]
#[sqlx(type_name = "source_person_role", rename_all = "lowercase")]
pub enum SourcePersonRole {
    Author,
    Editor,
    Translator,
    Contributor,
}

/// Stored as the Postgres enum `resource_type`.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, ToSchema, sqlx::Type)]
#[serde(rename_all = "lowercase")]
#[sqlx(type_name = "resource_type", rename_all = "lowercase")]
pub enum ResourceType {
    Verbatim,
    Paraphrase,
    Allusion,
}

/// Stored as the Postgres enum `verbatim_kind`.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, ToSchema, sqlx::Type)]
#[serde(rename_all = "lowercase")]
#[sqlx(type_name = "verbatim_kind", rename_all = "lowercase")]
pub enum VerbatimKind {
    Entirety,
    Fragmentary,
}

/// Stored as the Postgres enum `sentence_kind`.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, ToSchema, sqlx::Type)]
#[serde(rename_all = "lowercase")]
#[sqlx(type_name = "sentence_kind", rename_all = "lowercase")]
pub enum SentenceKind {
    Body,
    Footnote,
    /// Figure anchors sit outside the body enumeration and are addressed
    /// by `content_blocks.figure_number` (migration 0012).
    Figure,
}

impl SentenceKind {
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::Body => "body",
            Self::Footnote => "footnote",
            Self::Figure => "figure",
        }
    }

    /// Graceful parse for untrusted content-derived strings (article
    /// passage directives); None for anything unknown.
    pub fn parse(value: &str) -> Option<Self> {
        match value {
            "body" => Some(Self::Body),
            "footnote" => Some(Self::Footnote),
            "figure" => Some(Self::Figure),
            _ => None,
        }
    }
}

/// Stored as the Postgres enum `resource_scope` (ADR 0008).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, ToSchema, sqlx::Type)]
#[serde(rename_all = "lowercase")]
#[sqlx(type_name = "resource_scope", rename_all = "lowercase")]
pub enum ResourceScope {
    Work,
    Language,
    Edition,
}

use crate::system::serde_util::double_option;

#[derive(Debug, Serialize, ToSchema)]
pub struct ResourceResponse {
    pub id: String,
    pub resource_type: ResourceType,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub verbatim_kind: Option<VerbatimKind>,
    pub anchor_sentence_start_number: i32,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub anchor_sentence_end_number: Option<i32>,
    pub sentence_kind: SentenceKind,
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
    /// Lifecycle: "approved" for live resources, "pending" for a community
    /// submission awaiting review, "rejected" for a declined one. The reader
    /// only ever receives "approved" rows plus the caller's own "pending" ones.
    pub review_status: ResourceReviewStatus,
    /// Cataloguer's claim about what the resource is about: "work" (the
    /// passage in any form), "language" (this language's translation layer),
    /// or "edition" (this one edition's actual text). Label only — never
    /// affects which editions the resource appears on.
    pub scope: ResourceScope,
    /// True when this entry is projected from a sibling edition of the same
    /// work (its anchor lives in `origin_book_slug`, not the requested book).
    /// Projected entries are read-only in the requesting edition.
    pub is_projected: bool,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub origin_book_slug: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub origin_language: Option<String>,
    /// Target-local sentence range covering the projected anchor, for
    /// placement in the requesting edition. Absent when the anchor's
    /// passage has no counterpart there.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub projected_sentence_start_number: Option<i32>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub projected_sentence_end_number: Option<i32>,
    pub created_at: String,
}

#[derive(Debug, Serialize, ToSchema)]
pub struct ResourceListResponse {
    pub resources: Vec<ResourceResponse>,
}

/// Review state of a resource that entered via community submission.
#[derive(Debug, Clone, Copy, sqlx::Type, Serialize, Deserialize, ToSchema, PartialEq, Eq)]
#[sqlx(type_name = "resource_review_status", rename_all = "snake_case")]
#[serde(rename_all = "snake_case")]
pub enum ResourceReviewStatus {
    Pending,
    Approved,
    Rejected,
}

#[derive(Debug, Clone, Serialize, ToSchema)]
pub struct ResourceSubmitter {
    pub id: String,
    pub display_name: String,
}

/// One entry in the editor review queue: the proposed resource plus the
/// context an editor needs to judge it (where it anchors, who suggested it).
#[derive(Debug, Serialize, ToSchema)]
pub struct ResourceSubmissionResponse {
    /// The proposed resource, in the same shape the reader renders. `review_status`
    /// on this object carries the submission's current state.
    pub resource: ResourceResponse,
    pub book_slug: String,
    pub book_title: String,
    /// `None` when the submitter's account has since been deleted.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub submitter: Option<ResourceSubmitter>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub review_note: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub reviewed_at: Option<String>,
}

#[derive(Debug, Serialize, ToSchema)]
pub struct ResourceSubmissionListResponse {
    pub submissions: Vec<ResourceSubmissionResponse>,
    pub total: i64,
    pub page: u32,
    pub per_page: u32,
}

#[derive(Debug, Deserialize, IntoParams)]
pub struct SubmissionListQuery {
    /// Filter set: "pending" (default), "all", "approved", or "rejected".
    #[serde(default)]
    pub filter: Option<String>,
    #[serde(default)]
    pub page: Option<u32>,
    #[serde(default)]
    pub per_page: Option<u32>,
}

/// An editor's verdict on a submission. `status` must be `approved` or
/// `rejected`; `review_note` is an optional message (e.g. a rejection reason).
#[derive(Debug, Deserialize, ToSchema)]
pub struct ReviewSubmissionRequest {
    pub status: ResourceReviewStatus,
    #[serde(default)]
    pub review_note: Option<String>,
}

#[derive(Debug, Clone, Serialize, ToSchema)]
pub struct SourceResponse {
    pub id: String,
    pub source_type: SourceType,
    pub title: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub title_display: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub publication_year: Option<i16>,
    /// Year of the edition this text presents, when it differs from the
    /// printing transcribed (CSL "original-date"). NULL = same as
    /// `publication_year`.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub original_year: Option<i16>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub publisher: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub publication_place: Option<String>,
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
    pub source_type: SourceType,
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
    pub role: SourcePersonRole,
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
    pub source_type: SourceType,
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
    pub kind: SentenceKind,
}

fn default_body() -> SentenceKind {
    SentenceKind::Body
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
    pub source_type: Option<SourceType>,
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
    pub resource_type: ResourceType,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub verbatim_kind: Option<VerbatimKind>,
    pub sentence_start: i32,
    pub sentence_end: Option<i32>,
    pub sentence_kind: SentenceKind,
    pub source_id: Option<String>,
    pub source_page_start: Option<i32>,
    pub source_page_end: Option<i32>,
    pub source_location_freeform: Option<String>,
    pub quoted_text: Option<String>,
    pub editor_note: Option<String>,
    pub is_featured: Option<bool>,
    pub admin_notes: Option<String>,
    /// "work" (default) | "language" | "edition" — see ResourceResponse.scope.
    pub scope: Option<ResourceScope>,
}

#[derive(Debug, Deserialize, ToSchema)]
pub struct UpdateResourceRequest {
    pub resource_type: Option<ResourceType>,
    #[serde(default, deserialize_with = "double_option")]
    pub verbatim_kind: Option<Option<VerbatimKind>>,
    pub sentence_start: Option<i32>,
    pub sentence_end: Option<i32>,
    pub sentence_kind: Option<SentenceKind>,
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
    /// NOT NULL column: plain Option — omit to leave unchanged.
    pub scope: Option<ResourceScope>,
}

#[derive(Debug, Deserialize, ToSchema)]
pub struct CreateSourceRequest {
    pub source_type: SourceType,
    pub title: String,
    pub title_display: Option<String>,
    pub publication_year: Option<i16>,
    pub original_year: Option<i16>,
    pub publisher: Option<String>,
    pub publication_place: Option<String>,
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
    pub source_type: Option<SourceType>,
    pub title: Option<String>,
    #[serde(default, deserialize_with = "double_option")]
    pub title_display: Option<Option<String>>,
    #[serde(default, deserialize_with = "double_option")]
    pub publication_year: Option<Option<i16>>,
    #[serde(default, deserialize_with = "double_option")]
    pub original_year: Option<Option<i16>>,
    #[serde(default, deserialize_with = "double_option")]
    pub publisher: Option<Option<String>>,
    #[serde(default, deserialize_with = "double_option")]
    pub publication_place: Option<Option<String>>,
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
    pub role: SourcePersonRole,
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
