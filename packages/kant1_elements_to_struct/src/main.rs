mod model;
mod stitch;
mod toc;

use std::fs;
use std::path::Path;

use clap::Parser;
use common::sentences::split_sentences;

use model::*;
use stitch::{collect_element_b_refs, stitch_lines};

// ---------------------------------------------------------------------------
// CLI
// ---------------------------------------------------------------------------

#[derive(Parser)]
#[command(about = "Build structured Book JSON from per-page element data and authoritative TOC")]
struct Args {
    /// Directory containing per-page element JSON
    #[arg(long, default_value = "assets/kant1_lines_to_elements")]
    input_dir: String,

    /// Output JSON path
    #[arg(long, default_value = "assets/kant1_kritik.json")]
    output: String,
}

// ---------------------------------------------------------------------------
// Constants
// ---------------------------------------------------------------------------

/// Offset from scan page_index to AA III page number.
/// Derived empirically: page_0017 = AA 8, so aa_page = page_index - 9.
const PAGE_INDEX_TO_AA_OFFSET: i32 = 9;

// ---------------------------------------------------------------------------
// Main
// ---------------------------------------------------------------------------

fn main() {
    let args = Args::parse();

    // 1. Read all per-page JSONs
    let mut pages = read_pages(&args.input_dir);
    eprintln!("Read {} page files.", pages.len());

    // 2. Infer AA page numbers for pages without explicit page_number
    infer_page_numbers(&mut pages);

    // 3. Stitch hyphenated words across page boundaries
    let joins = stitch::stitch_across_pages(&mut pages);
    for (idx, word) in &joins {
        eprintln!("  Cross-page stitch at page {idx}: {word}");
    }

    // 4. Build flat list of (aa_page, element, footnotes) from content pages
    let page_elements = flatten_page_elements(&pages);
    eprintln!(
        "Flattened {} elements from {} content pages.",
        page_elements.iter().map(|(_, elems, _)| elems.len()).sum::<usize>(),
        page_elements.len()
    );

    // 5. Build the TOC tree
    let mut tree = toc::build_toc_tree();

    // 6. Assign content to TOC nodes
    let flat_entries = toc::flat_toc_entries();
    assign_content(&mut tree, &flat_entries, &page_elements);

    // 7. Number paragraphs and sentences globally
    let mut para_counter = 0u32;
    let mut sentence_counter = 0u32;
    let mut block_position = 0u32;
    number_tree(&mut tree, &mut para_counter, &mut sentence_counter, &mut block_position);
    eprintln!("Numbered {para_counter} paragraphs, {sentence_counter} sentences.");

    // 8. Output
    let book = KantBook {
        title: "Kritik der reinen Vernunft".to_string(),
        author: "Immanuel Kant".to_string(),
        language: "de".to_string(),
        source: "Akademie-Ausgabe Band III (B-Auflage 1787)".to_string(),
        date: "1787".to_string(),
        nodes: tree,
    };

    let json = serde_json::to_string_pretty(&book).unwrap();
    fs::write(&args.output, &json).expect("Failed to write output");
    eprintln!("Wrote {}", args.output);
}

// ---------------------------------------------------------------------------
// Page reading
// ---------------------------------------------------------------------------

fn read_pages(input_dir: &str) -> Vec<InputPage> {
    let pattern = format!("{input_dir}/page_*.json");
    let mut files: Vec<String> = glob::glob(&pattern)
        .expect("Invalid glob pattern")
        .filter_map(|e| e.ok())
        .map(|p| p.to_string_lossy().to_string())
        .collect();
    files.sort();

    let mut pages = Vec::new();
    for f in &files {
        let data = fs::read_to_string(f).unwrap_or_else(|e| {
            panic!("Failed to read {}: {e}", f);
        });
        let page: InputPage = serde_json::from_str(&data).unwrap_or_else(|e| {
            panic!(
                "Failed to parse {}: {e}",
                Path::new(f).file_name().unwrap().to_string_lossy()
            );
        });
        pages.push(page);
    }

    pages.sort_by_key(|p| p.page_index);
    pages
}

// ---------------------------------------------------------------------------
// AA page number inference
// ---------------------------------------------------------------------------

/// For pages without an explicit page_number, infer from the linear relationship:
/// aa_page = page_index - PAGE_INDEX_TO_AA_OFFSET
///
/// We store the inferred number back into page_number.
fn infer_page_numbers(pages: &mut [InputPage]) {
    for page in pages.iter_mut() {
        if page.page_number.is_some() {
            continue;
        }
        let inferred = page.page_index as i32 - PAGE_INDEX_TO_AA_OFFSET;
        if inferred > 0 {
            page.page_number = Some(inferred.to_string());
        }
    }
}

/// Parse page_number string to u16. Returns None for non-numeric values.
fn parse_aa_page(page: &InputPage) -> Option<u16> {
    page.page_number
        .as_ref()
        .and_then(|s| s.parse::<u16>().ok())
}

// ---------------------------------------------------------------------------
// Flatten page elements
// ---------------------------------------------------------------------------

/// Extract (aa_page, elements, footnotes) from content pages.
/// Skips blank, title, and toc pages.
fn flatten_page_elements(pages: &[InputPage]) -> Vec<(u16, Vec<InputElement>, Vec<InputFootnote>)> {
    let mut result = Vec::new();

    for page in pages {
        // Skip non-content pages
        if matches!(page.page_type.as_str(), "blank" | "title" | "toc") {
            continue;
        }
        if page.elements.is_empty() {
            continue;
        }

        let aa_page = match parse_aa_page(page) {
            Some(p) => p,
            None => continue,
        };

        result.push((aa_page, page.elements.clone(), page.footnotes.clone()));
    }

    result
}

// ---------------------------------------------------------------------------
// Content assignment to TOC nodes
// ---------------------------------------------------------------------------

/// For each element, find the most recent TOC entry (in flat document order)
/// whose aa_page <= element's aa_page.
fn find_flat_section(flat_entries: &[(usize, u16, u16, &str)], aa_page: u16) -> usize {
    let mut best = 0;
    for (i, &(_, entry_page, _, _)) in flat_entries.iter().enumerate() {
        if entry_page <= aa_page {
            best = i;
        } else {
            break;
        }
    }
    best
}

/// Assign content from page elements to the TOC tree.
fn assign_content(
    tree: &mut [KantTocNode],
    flat_entries: &[(usize, u16, u16, &str)],
    page_elements: &[(u16, Vec<InputElement>, Vec<InputFootnote>)],
) {
    // Build a map: flat_index -> Vec<ContentBlock>
    let num_entries = toc::toc_len();
    let mut content_map: Vec<Vec<KantContentBlock>> = vec![Vec::new(); num_entries];

    for (aa_page, elements, footnotes) in page_elements {
        // Find which section this page's content belongs to
        let section_idx = find_flat_section(flat_entries, *aa_page);

        // Convert elements to content blocks
        for elem in elements {
            let block = element_to_content_block(elem);
            content_map[section_idx].push(block);
        }

        // Convert footnotes to content blocks
        for footnote in footnotes {
            content_map[section_idx].push(KantContentBlock {
                position: 0,
                block_type: KantBlockType::Footnote,
                paragraph_number: None,
                text: format!("[{}] {}", footnote.marker, footnote.text),
                b_page_refs: Vec::new(),
                sentences: Vec::new(),
            });
        }
    }

    // Distribute content to the tree using the flat index mapping
    distribute_content(tree, flat_entries, &mut content_map, &mut 0);
}

/// Convert an InputElement to a KantContentBlock with stitched lines and sentences.
fn element_to_content_block(elem: &InputElement) -> KantContentBlock {
    let (text, line_anchors) = stitch_lines(&elem.lines);
    let b_page_refs = collect_element_b_refs(&elem.b_page_refs, &line_anchors);

    let block_type = match elem.elem_type.as_str() {
        "heading" => KantBlockType::Heading,
        _ => KantBlockType::Paragraph,
    };

    // Split into sentences (pass text as both text and html since we have no HTML)
    let sentence_pairs = split_sentences(&text, &text);
    let sentences: Vec<KantSentence> = sentence_pairs
        .iter()
        .enumerate()
        .map(|(i, (sent_text, _))| {
            // Find if any b_page_ref anchor falls within this sentence
            let b_ref = find_b_ref_for_sentence(&text, sent_text, &line_anchors);
            KantSentence {
                position: i as u32,
                sentence_number: 0, // numbered later
                text: sent_text.clone(),
                b_page_ref: b_ref,
            }
        })
        .collect();

    KantContentBlock {
        position: 0,
        block_type,
        paragraph_number: None,
        text,
        b_page_refs,
        sentences,
    }
}

/// Find the b_page_ref that anchors within a given sentence's range in the full text.
fn find_b_ref_for_sentence(
    full_text: &str,
    sentence_text: &str,
    anchors: &[stitch::BPageAnchor],
) -> Option<String> {
    // Find where this sentence occurs in the full text
    if let Some(start) = full_text.find(sentence_text) {
        let end = start + sentence_text.len();
        for anchor in anchors {
            if anchor.char_offset >= start && anchor.char_offset < end {
                return Some(anchor.b_page.clone());
            }
        }
    }
    None
}

/// Recursively distribute content from the flat content map to the tree nodes.
fn distribute_content(
    nodes: &mut [KantTocNode],
    flat_entries: &[(usize, u16, u16, &str)],
    content_map: &mut [Vec<KantContentBlock>],
    flat_idx: &mut usize,
) {
    for node in nodes.iter_mut() {
        if *flat_idx < flat_entries.len() {
            // Take the content for this flat index
            let content = std::mem::take(&mut content_map[*flat_idx]);
            node.content = content;
            *flat_idx += 1;
        }

        // Recurse into children
        distribute_content(&mut node.children, flat_entries, content_map, flat_idx);
    }
}

// ---------------------------------------------------------------------------
// Numbering
// ---------------------------------------------------------------------------

/// Assign globally incrementing paragraph_number and sentence_number.
fn number_tree(
    nodes: &mut [KantTocNode],
    para_counter: &mut u32,
    sentence_counter: &mut u32,
    block_position: &mut u32,
) {
    for node in nodes.iter_mut() {
        for block in node.content.iter_mut() {
            block.position = *block_position;
            *block_position += 1;

            if block.block_type == KantBlockType::Paragraph {
                *para_counter += 1;
                block.paragraph_number = Some(*para_counter);
            }

            for sentence in block.sentences.iter_mut() {
                *sentence_counter += 1;
                sentence.sentence_number = *sentence_counter;
                sentence.position = *sentence_counter;
            }
        }

        number_tree(&mut node.children, para_counter, sentence_counter, block_position);
    }
}
