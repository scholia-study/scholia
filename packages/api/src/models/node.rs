use serde::Serialize;
use utoipa::ToSchema;

#[derive(Debug, Serialize, ToSchema)]
pub struct NodeDetail {
    pub id: String,
    pub ncx_id: String,
    pub label: String,
    pub depth: i16,
    pub play_order: i32,
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
    pub sentence_number: i32,
    pub text: String,
    pub html: String,
}
