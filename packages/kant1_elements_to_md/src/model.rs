use serde::Deserialize;

use crate::stitch::BPageAnchor;

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
// Intermediate types for markdown output
// ---------------------------------------------------------------------------

pub struct MdTocNode {
    pub flat_index: usize,
    pub label: String,
    pub aa_page: u16,
    pub depth: u16,
    pub blocks: Vec<MdBlock>,
    pub footnotes: Vec<MdFootnote>,
}

pub struct MdBlock {
    pub block_type: MdBlockType,
    pub text: String,
    pub aa_page: u16,
    pub b_page_anchors: Vec<BPageAnchor>,
}

pub struct MdFootnote {
    pub marker: String,
    pub text: String,
}

pub enum MdBlockType {
    Heading,
    Paragraph,
}
