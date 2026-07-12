use serde::Serialize;
use utoipa::ToSchema;

/// Lightweight per-node metadata for SEO: the opening text of the
/// node's content, for meta descriptions and social previews. `null`
/// for structural nodes without content blocks.
#[derive(Debug, Serialize, ToSchema)]
pub struct NodeMetaResponse {
    pub excerpt: Option<String>,
}

#[derive(Debug, Serialize, ToSchema)]
pub struct NodeDetail {
    pub id: String,
    pub source_ref: String,
    pub slug: String,
    pub label: String,
    pub depth: i16,
    pub sort_order: i32,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub source_node_id: Option<String>,
    pub blocks: Vec<ContentBlockResponse>,
}

#[derive(Debug, Serialize, ToSchema)]
pub struct ContentBlockResponse {
    pub id: String,
    pub position: i16,
    pub block_type: String,
    pub paragraph_number: Option<i32>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub figure_number: Option<i32>,
    pub html: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub original_html: Option<String>,
    pub sentences: Vec<SentenceResponse>,
}

#[derive(Debug, Serialize, ToSchema)]
pub struct SentenceResponse {
    pub id: String,
    pub position: i16,
    pub sentence_number: Option<i32>,
    /// Set only on a figure's anchor sentence — drives the `fig{N}` selection key.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub figure_number: Option<i32>,
    /// 1-based index of the indented run within the paragraph (`+ ` enumerations),
    /// or absent for normal flow. Consecutive same-`segment` sentences render as
    /// one hanging-indent block.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub segment: Option<i16>,
    pub text: String,
    pub html: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub original_text: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub original_html: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub source_sentence_start_id: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub source_sentence_end_id: Option<String>,
    pub page_markers: Vec<PageMarkerResponse>,
    #[serde(skip_serializing_if = "Vec::is_empty")]
    pub footnotes: Vec<FootnoteResponse>,
}

#[derive(Debug, Serialize, ToSchema)]
pub struct PageMarkerResponse {
    pub system_slug: String,
    pub ref_value: String,
    pub sort_order: i32,
    pub char_offset: Option<i32>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub image_url: Option<String>,
}

#[derive(Debug, Serialize, ToSchema)]
pub struct FootnoteResponse {
    pub id: String,
    pub number: i32,
    pub sentences: Vec<FootnoteSentenceResponse>,
}

#[derive(Debug, Serialize, ToSchema)]
pub struct FootnoteSentenceResponse {
    pub id: String,
    pub position: i16,
    pub sentence_number: Option<i32>,
    pub text: String,
    pub html: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub original_text: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub original_html: Option<String>,
}
