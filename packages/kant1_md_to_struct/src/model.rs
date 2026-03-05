use serde::Serialize;

#[derive(Debug, Serialize)]
pub struct Output {
    pub book: BookData,
    pub reference_systems: Vec<ReferenceSystemData>,
    pub toc_nodes: Vec<TocNodeData>,
}

#[derive(Debug, Serialize)]
pub struct BookData {
    pub slug: String,
    pub title: String,
    pub author: String,
    pub language: String,
    pub source: String,
    pub source_date: String,
}

#[derive(Debug, Serialize)]
pub struct ReferenceSystemData {
    pub slug: String,
    pub label: String,
    pub ref_type: String,
}

#[derive(Debug, Serialize)]
pub struct TocNodeData {
    pub source_ref: String,
    pub slug: String,
    pub path: String,
    pub sort_order: i32,
    pub depth: i16,
    pub label: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub parent_source_ref: Option<String>,
    pub content_blocks: Vec<ContentBlockData>,
}

#[derive(Debug, Serialize)]
pub struct ContentBlockData {
    pub position: i16,
    pub block_type: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub paragraph_number: Option<i32>,
    pub text: String,
    pub html: String,
    pub sentences: Vec<SentenceData>,
}

#[derive(Debug, Serialize)]
pub struct SentenceData {
    pub position: i16,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub sentence_number: Option<i32>,
    pub text: String,
    pub html: String,
    #[serde(skip_serializing_if = "Vec::is_empty")]
    pub page_markers: Vec<PageMarkerData>,
}

#[derive(Debug, Serialize)]
pub struct PageMarkerData {
    pub system: String,
    pub ref_value: String,
    pub sort_order: i32,
    pub char_offset: i32,
}
