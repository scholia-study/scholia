use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Book {
    pub title: String,
    pub nodes: Vec<TocNode>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TocNode {
    pub ncx_id: String,
    pub play_order: u32,
    pub label: String,
    pub depth: u16,
    pub children: Vec<TocNode>,
    pub content: Vec<ContentBlock>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ContentBlock {
    pub position: u32,
    #[serde(rename = "type")]
    pub block_type: BlockType,
    pub text: String,
    pub html: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub page_ref: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "lowercase")]
pub enum BlockType {
    Paragraph,
    Heading,
    Footnote,
    Separator,
}
