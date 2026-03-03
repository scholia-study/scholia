use serde::{Deserialize, Serialize};

// ---------------------------------------------------------------------------
// Input types (from kant1_lines_to_elements per-page JSON)
// ---------------------------------------------------------------------------

#[derive(Debug, Clone, Deserialize)]
pub struct InputPage {
    pub page_index: usize,
    pub page_number: Option<String>,
    pub page_type: String,
    pub elements: Vec<InputElement>,
    pub footnotes: Vec<InputFootnote>,
}

#[derive(Debug, Clone, Deserialize)]
pub struct InputElement {
    #[serde(rename = "type")]
    pub elem_type: String,
    pub text: String,
    pub lines: Vec<InputLine>,
    #[serde(default)]
    pub b_page_refs: Vec<String>,
}

#[derive(Debug, Clone, Deserialize)]
pub struct InputLine {
    pub text: String,
    #[allow(dead_code)]
    pub line_number: Option<i64>,
    pub b_page_ref: Option<String>,
}

#[derive(Debug, Clone, Deserialize)]
pub struct InputFootnote {
    pub marker: String,
    pub text: String,
}

// ---------------------------------------------------------------------------
// Output types (Kant-specific, no html field, with aa_page/b_page)
// ---------------------------------------------------------------------------

#[derive(Debug, Clone, Serialize)]
pub struct KantBook {
    pub title: String,
    pub author: String,
    pub language: String,
    pub source: String,
    pub date: String,
    pub nodes: Vec<KantTocNode>,
}

#[derive(Debug, Clone, Serialize)]
pub struct KantTocNode {
    pub position: u32,
    pub label: String,
    pub aa_page: u16,
    pub depth: u16,
    pub children: Vec<KantTocNode>,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub content: Vec<KantContentBlock>,
}

#[derive(Debug, Clone, Serialize)]
pub struct KantContentBlock {
    pub position: u32,
    #[serde(rename = "type")]
    pub block_type: KantBlockType,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub paragraph_number: Option<u32>,
    pub text: String,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub b_page_refs: Vec<String>,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub sentences: Vec<KantSentence>,
}

#[derive(Debug, Clone, Serialize)]
pub struct KantSentence {
    pub position: u32,
    pub sentence_number: u32,
    pub text: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub b_page_ref: Option<String>,
}

#[derive(Debug, Clone, Serialize, PartialEq)]
#[serde(rename_all = "lowercase")]
pub enum KantBlockType {
    Paragraph,
    Heading,
    Footnote,
}
