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
    pub blocks: Vec<ContentBlockResponse>,
}

#[derive(Debug, Serialize, ToSchema)]
pub struct ContentBlockResponse {
    pub id: String,
    pub position: i16,
    pub block_type: String,
    pub paragraph_number: Option<i32>,
    pub html: String,
    pub sentences: Vec<SentenceResponse>,
}

#[derive(Debug, Serialize, ToSchema)]
pub struct SentenceResponse {
    pub id: String,
    pub position: i16,
    pub sentence_number: Option<i32>,
    pub text: String,
    pub html: String,
    pub page_markers: Vec<PageMarkerResponse>,
}

#[derive(Debug, Serialize, ToSchema)]
pub struct PageMarkerResponse {
    pub system_slug: String,
    pub ref_value: String,
    pub sort_order: i32,
    pub char_offset: Option<i32>,
}
