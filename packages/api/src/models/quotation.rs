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
    /// Slug of the book this quotation lives in. Used to render the
    /// translation badge ("KJV"/"WEB" — see `translation_label`) and
    /// to differentiate cross-translation peer quotations from
    /// own-book ones in the reader. Optional for backward-compat
    /// across endpoints that haven't been wired yet; populated by
    /// `list_quotations_for_node`.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub book_slug: Option<String>,
    /// Short display label for the translation badge — derived from
    /// the source's `publisher` field (which the Bible importer sets
    /// to "KJV"/"WEB"). Falls back to the book title when absent.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub translation_label: Option<String>,
    /// `toc_nodes.source_ref` of the anchor — translation-invariant
    /// (e.g. `"genesis:5"`). Used as the chapter-grouping key for
    /// verse-level visual marker projection.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub anchor_source_ref: Option<String>,
    /// Verse `ref_value` of the start anchor sentence (e.g. `"5:1"`).
    /// Populated when the book has a `verse` reference system; null
    /// for books without verse-style markers (Kant). Together with
    /// `anchor_source_ref` this is the cross-translation marker key.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub anchor_verse_start: Option<String>,
    /// Verse `ref_value` of the end anchor sentence (only when the
    /// quotation spans multiple verses; otherwise null and the
    /// quotation is treated as covering a single verse).
    #[serde(skip_serializing_if = "Option::is_none")]
    pub anchor_verse_end: Option<String>,
}

#[derive(Debug, Serialize, ToSchema)]
pub struct QuotationListResponse {
    pub quotations: Vec<QuotationResponse>,
}

#[derive(Debug, Serialize, ToSchema)]
pub struct QuotationLimitsResponse {
    pub max: i32,
    pub current: i64,
}

#[derive(Debug, Serialize, ToSchema)]
pub struct NoteLimitsResponse {
    pub max: i32,
    pub current: i64,
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
    /// Compact translation badge — the source's `publisher` when short
    /// (e.g. "KJV"/"WEB" for Bible) or the language code (e.g. "DE"/"EN"
    /// for Kant). Used in My Quotations / reader badge UI to show
    /// which translation a quotation belongs to.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub translation_label: Option<String>,
    /// Title of the cited bibliographic work (Shape 3): the effective
    /// source resolved for the quotation's anchor — either the per-book
    /// child source within a compilation (e.g. "Genesis") or the hosted
    /// text's root source for non-compilations (e.g. "Critique of Pure
    /// Reason").
    pub book_title: String,
    /// When the cited source is a child of a compilation, the compilation's
    /// display title. Frontend renders as "[book_title] (in: [parent])".
    #[serde(skip_serializing_if = "Option::is_none")]
    pub parent_compilation_title: Option<String>,
    pub node_label: String,
    pub node_slug: String,
    pub anchor_sentence_start_number: i32,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub anchor_sentence_end_number: Option<i32>,
    pub sentence_kind: String,
    /// For footnote-kind anchors: the body sentence number the footnote is
    /// attached to. None for body-kind anchors.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub anchor_main_sentence_number: Option<i32>,
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
    /// See `QuotationWithContextResponse::translation_label`.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub translation_label: Option<String>,
    /// See `QuotationWithContextResponse::book_title`.
    pub book_title: String,
    /// See `QuotationWithContextResponse::parent_compilation_title`.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub parent_compilation_title: Option<String>,
    pub node_label: String,
    pub node_slug: String,
    pub anchor_sentence_start_number: i32,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub anchor_sentence_end_number: Option<i32>,
    pub sentence_kind: String,
    /// For footnote-kind anchors: the body sentence number the footnote is
    /// attached to. None for body-kind anchors.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub anchor_main_sentence_number: Option<i32>,
    pub quotation_id: String,
    pub created_at: String,
    pub updated_at: String,
}

#[derive(Debug, Serialize, ToSchema)]
pub struct NoteWithContextListResponse {
    pub notes: Vec<NoteWithContextResponse>,
    pub limits: NoteLimitsResponse,
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
