use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize, Deserialize)]
pub struct Output {
    pub book: BookData,
    pub reference_systems: Vec<ReferenceSystemData>,
    pub toc_nodes: Vec<TocNodeData>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct BookData {
    pub slug: String,
    pub title: String,
    pub author: String,
    pub language: String,
    pub source: String,
    pub source_date: String,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct ReferenceSystemData {
    pub slug: String,
    pub label: String,
    pub ref_type: String,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct TocNodeData {
    pub source_ref: String,
    pub slug: String,
    pub path: String,
    pub sort_order: i32,
    pub depth: i16,
    pub label: String,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub parent_source_ref: Option<String>,
    pub content_blocks: Vec<ContentBlockData>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct ContentBlockData {
    pub position: i16,
    pub block_type: String,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub paragraph_number: Option<i32>,
    pub text: String,
    pub html: String,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub original_text: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub original_html: Option<String>,
    pub sentences: Vec<SentenceData>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct SentenceData {
    pub position: i16,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub sentence_number: Option<i32>,
    pub text: String,
    pub html: String,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub original_text: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub original_html: Option<String>,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub page_markers: Vec<PageMarkerData>,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub footnotes: Vec<FootnoteData>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct FootnoteData {
    pub number: i32,
    pub sentences: Vec<FootnoteSentenceData>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct FootnoteSentenceData {
    pub position: i16,
    pub text: String,
    pub html: String,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub original_text: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub original_html: Option<String>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct PageMarkerData {
    pub system: String,
    pub ref_value: String,
    pub sort_order: i32,
    pub char_offset: i32,
}
