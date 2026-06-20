//! Glue between the poetry struct and the shared reconcile crate: build the
//! content-hash inputs from the struct (so the fresh-insert and
//! reconcile paths hash *identical* content) and map the struct → the
//! book-agnostic [`reconcile::ReconcileInput`] IR. The orchestration itself —
//! alignment, split/merge, dependent migration, hash short-circuit, renumber —
//! lives in `reconcile::orchestrate`.

use poetry_md_to_struct::model::{ContentBlockData, Output, SentenceData, TocNodeData};
use reconcile::{
    BlockContent, BlockInput, MarkerContent, MarkerInput, NodeAnchor, NodeContent, NodeInput,
    ReconcileInput, SentenceContent, SentenceInput, node_hash, root_hash,
};
use uuid::Uuid;

// --- Content hashing (tier-2 incremental reconcile) ------------------------
// Build the book-agnostic `reconcile::NodeContent` from the Shakespeare struct so
// the insert path and the reconcile path hash *identical* content. The field set
// here must mirror what reconcile writes (text/html/original_*/segment, page
// markers, block + label fields) — never the recomputed numbering/positional
// fields. Shakespeare has no footnotes, so the footnote list is always empty.
//
// NOTE: `indent` is intentionally NOT part of the hash (the shared
// `reconcile::SentenceContent` has no `indent` field). An indent-only edit is
// therefore not hash-detected and needs `--full-rewrite` to take effect.

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
        footnotes: Vec::new(),
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
// A node carrying its own `source` (a Bible-shape sub-work, e.g. "Sonnets")
// becomes a `WorkSource` anchor: the orchestration creates a `source_type
// 'chapter'` source under the book's compilation source and links the author.
// The caller passes the book's bibliographic source + the upserted author + the
// system user so the anchor can be built without re-querying. Shakespeare has no
// translation layer and no footnotes.

pub(crate) fn to_input(
    output: &Output,
    bib_source_id: Uuid,
    author_person_id: Uuid,
    created_by: Uuid,
) -> ReconcileInput {
    ReconcileInput {
        nodes: output
            .toc_nodes
            .iter()
            .map(|n| node_input(n, bib_source_id, author_person_id, created_by))
            .collect(),
    }
}

fn node_input(
    node: &TocNodeData,
    bib_source_id: Uuid,
    author_person_id: Uuid,
    created_by: Uuid,
) -> NodeInput {
    let anchor = match &node.source {
        Some(src) => NodeAnchor::WorkSource {
            title: src.title.clone(),
            publication_year: src.publication_year,
            parent_source_id: bib_source_id,
            author_person_id,
            created_by,
        },
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
        indent: s.indent,
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
        footnotes: Vec::new(),
    }
}
