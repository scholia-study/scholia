//! Struct-JSON schema for the structured-text ingest pipeline, shared by every
//! genre parser that feeds `struct_to_db` — verse (Shakespeare's Sonnets,
//! Milton's *Paradise Lost*) and drama (Ibsen) alike. Mirrors the Kant `Output`
//! tree (so the importer logic stays familiar) with one addition:
//! `SentenceData.indent` for verse line indentation (ADR 0003).

use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Output {
    pub book: BookData,
    pub reference_systems: Vec<ReferenceSystemData>,
    pub toc_nodes: Vec<TocNodeData>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BookData {
    pub slug: String,
    pub title: String,
    pub author: String,
    pub language: String,
    pub source: String,
    pub source_date: String,
    /// Editorial "about this book" copy → `books.about_text`.
    #[serde(default)]
    pub about_text: String,
    /// Reader page size (nodes per next/prev fetch) → `books.nodes_per_page`.
    /// `None` = default (20); set small for texts with few but huge nodes.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub nodes_per_page: Option<i16>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ReferenceSystemData {
    pub slug: String,
    pub label: String,
    pub ref_type: String,
    /// Lowest-wins default-citation rank; `None` = not a default (see
    /// migration 0008).
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub cite_priority: Option<i16>,
    /// Citation render template (tokens `{parent}`/`{self}`/`{ref}`).
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub cite_template: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TocNodeData {
    pub source_ref: String,
    pub slug: String,
    pub path: String,
    pub sort_order: i32,
    pub depth: i16,
    pub label: String,
    pub label_html: String,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub parent_source_ref: Option<String>,
    /// When set, the importer creates a sub-work source (source_type 'chapter',
    /// parented to the book's compilation source) and points this node's
    /// `source_id` at it — the Bible-shape anchor that makes the node a work pill.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub source: Option<NodeSource>,
    pub content_blocks: Vec<ContentBlockData>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NodeSource {
    pub title: String,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub publication_year: Option<i16>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ContentBlockData {
    pub position: i16,
    pub block_type: String,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub paragraph_number: Option<i32>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub figure_number: Option<i32>,
    pub text: String,
    pub html: String,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub original_text: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub original_html: Option<String>,
    pub sentences: Vec<SentenceData>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SentenceData {
    pub position: i16,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub sentence_number: Option<i32>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub segment: Option<i16>,
    /// Verse line indent level (0 = flush). `None` keeps the JSON quiet for the
    /// common flush case; the importer treats absent as 0.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub indent: Option<i16>,
    pub text: String,
    pub html: String,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub original_text: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub original_html: Option<String>,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub page_markers: Vec<PageMarkerData>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PageMarkerData {
    pub system: String,
    pub ref_value: String,
    pub sort_order: i32,
    pub char_offset: i32,
}
