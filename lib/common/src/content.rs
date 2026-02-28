use crate::epub_reader::EpubReader;
use crate::model::{BlockType, ContentBlock, TocNode};
use crate::ncx::{NcxNode, flatten_ncx};
use scraper::{ElementRef, Html, Selector};
use std::collections::HashMap;

/// Fill content blocks into TocNode tree by extracting from XHTML files.
pub fn extract_all_content(
    reader: &mut EpubReader,
    nodes: &mut [TocNode],
    ncx_nodes: &[NcxNode],
) -> Result<(), Box<dyn std::error::Error>> {
    // Build a flat list of (ncx_id, play_order, label, src, depth) for boundary calculation.
    let flat = flatten_ncx(ncx_nodes, 1);

    // Build a map: file -> list of (fragment, ncx_id, play_order) for nodes in that file,
    // ordered by play_order.
    let mut file_nodes: HashMap<String, Vec<(Option<String>, String, u32)>> = HashMap::new();
    for (ncx_id, play_order, _label, src, _depth) in &flat {
        file_nodes
            .entry(src.file.clone())
            .or_default()
            .push((src.fragment.clone(), ncx_id.clone(), *play_order));
    }
    for v in file_nodes.values_mut() {
        v.sort_by_key(|x| x.2);
    }

    // Build a map from ncx_id -> Vec<ContentBlock> by processing each file.
    let mut content_map: HashMap<String, Vec<ContentBlock>> = HashMap::new();

    // Get unique files to process
    let mut files_to_process: Vec<String> = file_nodes.keys().cloned().collect();
    files_to_process.sort();

    for file in &files_to_process {
        let html_str = match reader.read_file(file) {
            Ok(s) => s,
            Err(e) => {
                eprintln!("Warning: could not read {}: {}", file, e);
                continue;
            }
        };

        let doc = Html::parse_document(&html_str);
        let nodes_in_file = &file_nodes[file.as_str()];

        if nodes_in_file.len() == 1 && nodes_in_file[0].0.is_none() {
            // Single node owns the entire file, no fragment
            let blocks = extract_blocks_from_body(&doc);
            content_map.insert(nodes_in_file[0].1.clone(), blocks);
        } else {
            // Multiple nodes share this file or nodes have fragment anchors.
            // We need to split content by anchor boundaries.
            extract_segmented_content(&doc, nodes_in_file, file, &mut content_map);
        }
    }

    // Now recursively assign content from the map into the TocNode tree.
    assign_content(nodes, &mut content_map);

    Ok(())
}

/// Assign content blocks from the map to TocNode tree recursively.
fn assign_content(nodes: &mut [TocNode], content_map: &mut HashMap<String, Vec<ContentBlock>>) {
    for node in nodes {
        if let Some(blocks) = content_map.remove(&node.ncx_id) {
            node.content = blocks;
        }
        assign_content(&mut node.children, content_map);
    }
}

/// Extract all content blocks from the entire <body> of a document.
fn extract_blocks_from_body(doc: &Html) -> Vec<ContentBlock> {
    let body_sel = Selector::parse("body").unwrap();
    let Some(body) = doc.select(&body_sel).next() else {
        return Vec::new();
    };
    extract_blocks_from_children(body, None)
}

/// For files shared by multiple navpoints, split content by anchor IDs.
fn extract_segmented_content(
    doc: &Html,
    nodes_in_file: &[(Option<String>, String, u32)],
    filename: &str,
    content_map: &mut HashMap<String, Vec<ContentBlock>>,
) {
    let body_sel = Selector::parse("body").unwrap();
    let Some(body) = doc.select(&body_sel).next() else {
        return;
    };

    // Collect all body children as elements
    let children: Vec<ElementRef> = body.children().filter_map(ElementRef::wrap).collect();

    // For each navpoint, determine which anchor ID marks its start.
    // Nodes with fragments have explicit anchor IDs (e.g., "an20009177264").
    // Nodes without fragments in shared files use "an{FILENAME_STEM}" as their anchor.
    let filename_stem = filename.strip_suffix(".html").unwrap_or(filename);

    let mut anchor_indices: Vec<usize> = Vec::new();
    for (frag, _ncx_id, _) in nodes_in_file {
        let anchor_id = match frag {
            Some(f) => f.clone(),
            None => format!("an{}", filename_stem),
        };

        let idx = children
            .iter()
            .position(|el| {
                el.value().id() == Some(anchor_id.as_str())
                    || has_descendant_with_id(el, &anchor_id)
            })
            .unwrap_or(0);

        anchor_indices.push(idx);
    }

    // Segment: for each node, content runs from its anchor to the next anchor (or EOF).
    for (i, (_, ncx_id, _)) in nodes_in_file.iter().enumerate() {
        let start_idx = anchor_indices[i];
        let end_idx = if i + 1 < anchor_indices.len() {
            anchor_indices[i + 1]
        } else {
            children.len()
        };

        let segment = &children[start_idx..end_idx];

        let mut blocks = Vec::new();
        let mut pos = 0u32;
        let mut in_footnotes = false;
        for el in segment {
            if is_footnote_header(el) {
                in_footnotes = true;
                continue;
            }
            if in_footnotes {
                if let Some(block) = classify_as_footnote(el, &mut pos) {
                    blocks.push(block);
                }
            } else if let Some(block) = classify_element(el, &mut pos) {
                blocks.push(block);
            }
        }

        content_map.insert(ncx_id.clone(), blocks);
    }
}

/// Check if an element has a descendant with the given ID.
fn has_descendant_with_id(el: &ElementRef, id: &str) -> bool {
    let sel = Selector::parse(&format!("[id=\"{}\"]", id)).unwrap();
    el.select(&sel).next().is_some()
}

/// Extract content blocks from body children, optionally stopping at a given anchor.
fn extract_blocks_from_children(body: ElementRef, _stop_at: Option<&str>) -> Vec<ContentBlock> {
    let mut blocks = Vec::new();
    let mut pos = 0u32;
    let mut in_footnotes = false;

    for child in body.children().filter_map(ElementRef::wrap) {
        // Check if this is the "Fußnoten" marker
        if is_footnote_header(&child) {
            in_footnotes = true;
            continue;
        }

        if in_footnotes {
            if let Some(block) = classify_as_footnote(&child, &mut pos) {
                blocks.push(block);
            }
            continue;
        }

        if let Some(block) = classify_element(&child, &mut pos) {
            blocks.push(block);
        }
    }

    blocks
}

/// Check if element is a "Fußnoten" header that starts the footnote section.
fn is_footnote_header(el: &ElementRef) -> bool {
    let tag = el.value().name();
    if tag == "p" {
        if let Some(class) = el.value().attr("class") {
            if class == "fn" {
                let text = el.text().collect::<String>().trim().to_string();
                return text.starts_with("Fußnote");
            }
        }
    }
    false
}

/// Classify a body-level element into a ContentBlock.
fn classify_element(el: &ElementRef, pos: &mut u32) -> Option<ContentBlock> {
    let tag = el.value().name();

    match tag {
        // Empty anchor placeholder — skip
        "p" if el.value().id().is_some() && is_empty_or_whitespace(el) => None,

        // Separator
        "div" => {
            if el.value().attr("class") == Some("emptyLine") {
                let block = ContentBlock {
                    position: *pos,
                    block_type: BlockType::Separator,
                    text: String::new(),
                    html: String::new(),
                    page_ref: None,
                };
                *pos += 1;
                Some(block)
            } else {
                None
            }
        }

        // Headings — h1 through h5 all treated as heading
        "h1" | "h2" | "h3" | "h4" | "h5" => {
            let text = el.text().collect::<String>().trim().to_string();
            if text.is_empty() {
                return None;
            }
            let block = ContentBlock {
                position: *pos,
                block_type: BlockType::Heading,
                text,
                html: inner_html(el),
                page_ref: None,
            };
            *pos += 1;
            Some(block)
        }

        // Paragraphs
        "p" => {
            let class = el.value().attr("class").unwrap_or("");

            // Footnote header
            if class == "fn" {
                return None;
            }

            let (text, html, page_ref) = extract_paragraph_content(el);

            // Skip empty/whitespace-only paragraphs
            if text.trim().is_empty() || text.trim() == "\u{a0}" {
                return None;
            }

            let block = ContentBlock {
                position: *pos,
                block_type: BlockType::Paragraph,
                text,
                html,
                page_ref,
            };
            *pos += 1;
            Some(block)
        }

        // Tables and other elements — treat as paragraph with text extraction
        "table" => {
            let text = el.text().collect::<String>().trim().to_string();
            if text.is_empty() {
                return None;
            }
            let block = ContentBlock {
                position: *pos,
                block_type: BlockType::Paragraph,
                text,
                html: el.inner_html(),
                page_ref: None,
            };
            *pos += 1;
            Some(block)
        }

        _ => None,
    }
}

/// Classify an element as a footnote (used after the "Fußnoten" marker).
fn classify_as_footnote(el: &ElementRef, pos: &mut u32) -> Option<ContentBlock> {
    let tag = el.value().name();
    if tag != "p" {
        return None;
    }

    let (text, html, page_ref) = extract_paragraph_content(el);

    if text.trim().is_empty() || text.trim() == "\u{a0}" {
        return None;
    }

    let block = ContentBlock {
        position: *pos,
        block_type: BlockType::Footnote,
        text,
        html,
        page_ref,
    };
    *pos += 1;
    Some(block)
}

/// Extract text, html, and optional page_ref from a paragraph element.
fn extract_paragraph_content(el: &ElementRef) -> (String, String, Option<String>) {
    let page_sel = Selector::parse("a.page").unwrap();

    // Extract page reference
    let page_ref = el.select(&page_sel).next().and_then(|a| {
        let text = a.text().collect::<String>();
        // Page refs look like [83] — extract the number
        text.trim()
            .strip_prefix('[')
            .and_then(|s| s.strip_suffix(']'))
            .map(|s| s.to_string())
    });

    // Build plain text (excluding page ref links)
    let text = build_plain_text(el);

    // Build HTML (preserving <i> but stripping page ref links)
    let html = build_html(el);

    (text, html, page_ref)
}

/// Build plain text from element, excluding page reference links.
fn build_plain_text(el: &ElementRef) -> String {
    let mut text = String::new();
    collect_text_recursive(el, &mut text);
    text.trim().to_string()
}

/// Recursively collect text, skipping <a class="page"> elements.
fn collect_text_recursive(el: &ElementRef, buf: &mut String) {
    use scraper::Node;
    for child in el.children() {
        match child.value() {
            Node::Text(t) => buf.push_str(t),
            Node::Element(_) => {
                if let Some(child_el) = ElementRef::wrap(child) {
                    // Skip page reference links
                    if child_el.value().name() == "a"
                        && child_el.value().attr("class") == Some("page")
                    {
                        continue;
                    }
                    collect_text_recursive(&child_el, buf);
                }
            }
            _ => {}
        }
    }
}

/// Build inner HTML preserving formatting but stripping page reference links.
fn build_html(el: &ElementRef) -> String {
    let mut html = String::new();
    build_html_recursive(el, &mut html);
    html.trim().to_string()
}

fn build_html_recursive(el: &ElementRef, buf: &mut String) {
    use scraper::Node;
    for child in el.children() {
        match child.value() {
            Node::Text(t) => buf.push_str(t),
            Node::Element(e) => {
                if let Some(child_el) = ElementRef::wrap(child) {
                    let tag = e.name.local.as_ref();

                    // Skip page reference links entirely
                    if tag == "a" && child_el.value().attr("class") == Some("page") {
                        continue;
                    }

                    // Preserve <i>, <b>, <sup>, <sub> formatting
                    if matches!(tag, "i" | "b" | "sup" | "sub") {
                        buf.push('<');
                        buf.push_str(tag);
                        buf.push('>');
                        build_html_recursive(&child_el, buf);
                        buf.push_str("</");
                        buf.push_str(tag);
                        buf.push('>');
                    } else {
                        // For other elements, just include their text content
                        build_html_recursive(&child_el, buf);
                    }
                }
            }
            _ => {}
        }
    }
}

/// Check if element is empty or contains only whitespace/&nbsp;
fn is_empty_or_whitespace(el: &ElementRef) -> bool {
    let text = el.text().collect::<String>();
    text.trim().is_empty() || text.trim() == "\u{a0}"
}

/// Get inner HTML of an element (its child nodes rendered as HTML).
fn inner_html(el: &ElementRef) -> String {
    let mut html = String::new();
    build_html_recursive(el, &mut html);
    html.trim().to_string()
}
