//! Kant-specific glue for the shared reconcile crate: build the content-hash
//! inputs from the Kant struct (so the fresh-insert and reconcile paths hash
//! *identical* content) and map the struct → the book-agnostic
//! [`reconcile::ReconcileInput`] IR. The orchestration itself — alignment,
//! split/merge, dependent migration, hash short-circuit, renumber — lives in
//! `reconcile::orchestrate`.

use std::collections::HashMap;

use kant1_md_to_struct::model::{
    ContentBlockData, FootnoteData, Output, SentenceData, TocNodeData,
};
use reconcile::{
    BlockContent, BlockInput, FootnoteContent, FootnoteInput, MarkerContent, MarkerInput,
    NodeAnchor, NodeContent, NodeInput, ReconcileInput, SentenceContent, SentenceInput, node_hash,
    root_hash,
};
use uuid::Uuid;

// --- Content hashing (tier-2 incremental reconcile) ------------------------
// Build the book-agnostic `reconcile::NodeContent` from the Kant struct so the
// insert path and the reconcile path hash *identical* content. The field set
// here must mirror what reconcile writes (text/html/original_*/segment, page
// markers, footnote content, block + label fields) — never the recomputed
// numbering/positional fields. See docs/architecture/reconcile-incremental-hashing.md.

pub(crate) fn node_content(node: &TocNodeData) -> NodeContent<'_> {
    NodeContent {
        label: &node.label,
        label_html: &node.label_html,
        blocks: node.content_blocks.iter().map(block_content).collect(),
    }
}

fn block_content(block: &ContentBlockData) -> BlockContent<'_> {
    BlockContent {
        block_type: &block.block_type,
        text: &block.text,
        html: &block.html,
        original_text: block.original_text.as_deref(),
        original_html: block.original_html.as_deref(),
        sentences: block.sentences.iter().map(sentence_content).collect(),
    }
}

fn sentence_content(s: &SentenceData) -> SentenceContent<'_> {
    SentenceContent {
        text: &s.text,
        html: &s.html,
        original_text: s.original_text.as_deref(),
        original_html: s.original_html.as_deref(),
        segment: s.segment,
        markers: s
            .page_markers
            .iter()
            .map(|m| MarkerContent {
                system: &m.system,
                ref_value: &m.ref_value,
                char_offset: Some(m.char_offset),
            })
            .collect(),
        footnotes: s.footnotes.iter().map(footnote_content).collect(),
    }
}

fn footnote_content(f: &FootnoteData) -> FootnoteContent<'_> {
    FootnoteContent {
        number: f.number,
        sentences: f
            .sentences
            .iter()
            .map(|fs| SentenceContent {
                text: &fs.text,
                html: &fs.html,
                original_text: fs.original_text.as_deref(),
                original_html: fs.original_html.as_deref(),
                segment: None,
                markers: Vec::new(),
                footnotes: Vec::new(),
            })
            .collect(),
    }
}

/// Per-node hashes in document (sort) order, paired with `source_ref`, plus the
/// root hash. Both the insert and reconcile paths derive their stored hashes
/// from here.
pub(crate) fn compute_hashes(output: &Output) -> (Vec<(String, String)>, String) {
    let node_hashes: Vec<(String, String)> = output
        .toc_nodes
        .iter()
        .map(|n| (n.source_ref.clone(), node_hash(&node_content(n))))
        .collect();
    let root = root_hash(
        &node_hashes
            .iter()
            .map(|(_, h)| h.clone())
            .collect::<Vec<_>>(),
    );
    (node_hashes, root)
}

// --- Struct → reconcile IR --------------------------------------------------
// Kant has no Bible-shape work anchors: an added translation node points at its
// source-book node (`SourceNode`), every other added node is anchor-less. Kant's
// sentences never carry an indent (poetry support is Shakespeare-only).

pub(crate) fn to_input(output: &Output, source_node_map: &HashMap<String, Uuid>) -> ReconcileInput {
    ReconcileInput {
        nodes: output
            .toc_nodes
            .iter()
            .map(|n| node_input(n, source_node_map))
            .collect(),
    }
}

fn node_input(node: &TocNodeData, source_node_map: &HashMap<String, Uuid>) -> NodeInput {
    let anchor = match source_node_map.get(&node.source_ref) {
        Some(&id) => NodeAnchor::SourceNode(id),
        None => NodeAnchor::None,
    };
    NodeInput {
        source_ref: node.source_ref.clone(),
        parent_source_ref: node.parent_source_ref.clone(),
        slug: node.slug.clone(),
        path: node.path.clone(),
        sort_order: node.sort_order,
        depth: node.depth,
        label: node.label.clone(),
        label_html: node.label_html.clone(),
        anchor,
        blocks: node.content_blocks.iter().map(block_input).collect(),
    }
}

fn block_input(block: &ContentBlockData) -> BlockInput {
    BlockInput {
        position: block.position,
        block_type: block.block_type.clone(),
        paragraph_number: block.paragraph_number,
        figure_number: block.figure_number,
        text: block.text.clone(),
        html: block.html.clone(),
        original_text: block.original_text.clone(),
        original_html: block.original_html.clone(),
        sentences: block.sentences.iter().map(sentence_input).collect(),
    }
}

fn sentence_input(s: &SentenceData) -> SentenceInput {
    SentenceInput {
        position: s.position,
        sentence_number: s.sentence_number,
        segment: s.segment,
        indent: None,
        text: s.text.clone(),
        html: s.html.clone(),
        original_text: s.original_text.clone(),
        original_html: s.original_html.clone(),
        markers: s
            .page_markers
            .iter()
            .map(|m| MarkerInput {
                system: m.system.clone(),
                ref_value: m.ref_value.clone(),
                sort_order: m.sort_order,
                char_offset: m.char_offset,
            })
            .collect(),
        footnotes: s.footnotes.iter().map(footnote_input).collect(),
    }
}

fn footnote_input(f: &FootnoteData) -> FootnoteInput {
    FootnoteInput {
        number: f.number,
        sentences: f
            .sentences
            .iter()
            .map(|fs| SentenceInput {
                position: fs.position,
                sentence_number: fs.sentence_number,
                segment: None,
                indent: None,
                text: fs.text.clone(),
                html: fs.html.clone(),
                original_text: fs.original_text.clone(),
                original_html: fs.original_html.clone(),
                markers: Vec::new(),
                footnotes: Vec::new(),
            })
            .collect(),
    }
}
