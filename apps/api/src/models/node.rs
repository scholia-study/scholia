use serde::Serialize;
use utoipa::ToSchema;

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
