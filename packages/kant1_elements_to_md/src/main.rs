mod markdown;
mod model;
mod stitch;
mod toc;

use std::collections::HashMap;
use std::fs;
use std::path::Path;

use clap::Parser;

use model::*;
use stitch::{stitch_lines, BPageAnchor};

// ---------------------------------------------------------------------------
// CLI
// ---------------------------------------------------------------------------

#[derive(Parser)]
#[command(about = "Build per-section markdown files from per-page element data and authoritative TOC")]
struct Args {
    /// Directory containing per-page element JSON
    #[arg(long, default_value = "assets/kant1_lines_to_elements")]
    input_dir: String,

    /// Output directory for markdown files
    #[arg(long, default_value = "assets/kant1_elements_to_md")]
    output_dir: String,
}

// ---------------------------------------------------------------------------
// Constants
// ---------------------------------------------------------------------------

/// Offset from scan page_index to AA III page number.
/// Derived empirically: page_0017 = AA 8, so aa_page = page_index - 9.
const PAGE_INDEX_TO_AA_OFFSET: i32 = 9;

/// Scan page indices to skip entirely (e.g. the original title page at page_index 10 = AA 1,
/// which is a volume title page, not Kant's text).
const SKIP_PAGE_INDICES: &[usize] = &[10];

// ---------------------------------------------------------------------------
// Main
// ---------------------------------------------------------------------------

fn main() {
    let args = Args::parse();

    // 1. Read all per-page JSONs
    let mut pages = read_pages(&args.input_dir);
    eprintln!("Read {} page files.", pages.len());

    // 2. Apply page-specific fixups
    fixup_pages(&mut pages);

    // 3. Infer AA page numbers for pages without explicit page_number
    infer_page_numbers(&mut pages);

    // 4. Stitch hyphenated words across page boundaries
    let joins = stitch::stitch_across_pages(&mut pages);
    for (idx, word) in &joins {
        eprintln!("  Cross-page stitch at page {idx}: {word}");
    }

    // 5. Build flat list of (aa_page, element, footnotes) from content pages
    let page_elements = flatten_page_elements(&pages);
    eprintln!(
        "Flattened {} elements from {} content pages.",
        page_elements.iter().map(|(_, elems, _)| elems.len()).sum::<usize>(),
        page_elements.len()
    );

    // 6. Build MdTocNodes from page elements
    let flat_entries = toc::flat_toc_entries();
    let nodes = build_md_nodes(&flat_entries, &page_elements);

    // 7. Write markdown files
    let output_dir = Path::new(&args.output_dir);
    fs::create_dir_all(output_dir).expect("Failed to create output directory");

    let mut file_count = 0;
    for node in &nodes {
        let fname = markdown::filename(node.flat_index, &node.label);
        let md = markdown::render_md(node);
        fs::write(output_dir.join(&fname), &md).expect("Failed to write markdown file");
        file_count += 1;
    }
    eprintln!("Wrote {} markdown files to {}", file_count, args.output_dir);
}

// ---------------------------------------------------------------------------
// Build MdTocNodes
// ---------------------------------------------------------------------------

/// Build MdTocNodes by assigning page elements to flat TOC sections.
fn build_md_nodes(
    flat_entries: &[(usize, u16, u16, &str)],
    page_elements: &[(u16, Vec<InputElement>, Vec<InputFootnote>)],
) -> Vec<MdTocNode> {
    let num_entries = toc::toc_len();

    // Per-section accumulators
    let mut section_blocks: Vec<Vec<MdBlock>> = (0..num_entries).map(|_| Vec::new()).collect();
    let mut section_footnotes: Vec<Vec<(String, String)>> = (0..num_entries).map(|_| Vec::new()).collect();

    // Footnote marker dedup state per section:
    // star_count: how many star-based footnotes we've seen so far
    // numeric_counter: next available numeric footnote number
    let mut star_counts: Vec<usize> = vec![0; num_entries];
    let mut numeric_counters: Vec<u32> = vec![1; num_entries];

    for (aa_page, elements, footnotes) in page_elements {
        // Find all TOC entries starting on this page
        let same_page_entries: Vec<usize> = flat_entries
            .iter()
            .filter(|(_, p, _, _)| *p == *aa_page)
            .map(|(i, _, _, _)| *i)
            .collect();

        // Start with the first TOC entry on this page (not last-match)
        let base_section = if same_page_entries.is_empty() {
            find_flat_section(flat_entries, *aa_page)
        } else {
            same_page_entries[0]
        };
        let mut section_idx = base_section;

        // Build a marker remap for this page's footnotes.
        // Footnotes go to whatever section the last element was assigned to,
        // but we process them after elements so we use base_section initially.
        let mut marker_remap: HashMap<String, String> = HashMap::new();

        // Process page footnotes: remap markers to avoid collisions
        for footnote in footnotes {
            let new_marker = remap_marker(
                &footnote.marker,
                &mut star_counts[base_section],
                &mut numeric_counters[base_section],
            );
            marker_remap.insert(footnote.marker.clone(), new_marker.clone());
            section_footnotes[base_section].push((new_marker, footnote.text.clone()));
        }

        // Pre-scan elements for inline footnotes (paragraphs starting with *), **), etc.)
        // so that body text references can be replaced in the second pass.
        for elem in elements {
            let (raw_text, _) = stitch_lines(&elem.lines);
            let text = clean_ocr_text(&raw_text);
            if let Some(marker_with_paren) = detect_inline_footnote_marker(&text) {
                let star_marker = marker_with_paren.trim_end_matches(')');
                let body = text[marker_with_paren.len()..].trim().to_string();
                let new_marker = remap_marker(
                    star_marker,
                    &mut star_counts[base_section],
                    &mut numeric_counters[base_section],
                );
                marker_remap.insert(star_marker.to_string(), new_marker.clone());
                section_footnotes[base_section].push((new_marker, body));
            }
        }

        // Process body text elements
        for elem in elements {
            // Skip printer's signature marks (e.g. "1*", "2*")
            let trimmed = elem.text.trim();
            if trimmed.len() <= 3
                && trimmed.ends_with('*')
                && trimmed[..trimmed.len() - 1].parse::<u16>().is_ok()
            {
                continue;
            }

            let (raw_text, mut line_anchors) = stitch_lines(&elem.lines);
            let text = clean_ocr_text(&raw_text);

            // Add element-level b_page_refs not already in line anchors at offset 0
            for b_ref in &elem.b_page_refs {
                if !line_anchors.iter().any(|a| &a.b_page == b_ref) {
                    line_anchors.insert(
                        0,
                        BPageAnchor {
                            b_page: b_ref.clone(),
                            char_offset: 0,
                        },
                    );
                }
            }

            // Skip inline footnotes (already processed in pre-scan)
            if detect_inline_footnote_marker(&text).is_some() {
                continue;
            }

            // When multiple TOC entries share this aa_page, advance section_idx
            // when a heading matches a later TOC entry's label.
            if elem.elem_type == "heading" && same_page_entries.len() > 1 {
                if let Some(matched) = match_heading_to_toc(&text, flat_entries, &same_page_entries)
                {
                    section_idx = matched;
                }
            }

            let block_type = match elem.elem_type.as_str() {
                "heading" => MdBlockType::Heading,
                _ => MdBlockType::Paragraph,
            };

            // Replace inline footnote references in body text
            let text = replace_footnote_refs(&text, &marker_remap);

            section_blocks[section_idx].push(MdBlock {
                block_type,
                text,
                aa_page: *aa_page,
                b_page_anchors: line_anchors,
            });
        }
    }

    // Build final nodes — only emit sections that have content
    let mut nodes = Vec::new();
    for &(flat_index, aa_page, depth, label) in flat_entries {
        let blocks = std::mem::take(&mut section_blocks[flat_index]);
        let raw_footnotes = std::mem::take(&mut section_footnotes[flat_index]);

        if blocks.is_empty() && raw_footnotes.is_empty() {
            continue;
        }

        let footnotes: Vec<MdFootnote> = raw_footnotes
            .into_iter()
            .map(|(marker, text)| MdFootnote { marker, text })
            .collect();

        nodes.push(MdTocNode {
            flat_index,
            label: label.to_string(),
            aa_page,
            depth,
            blocks,
            footnotes,
        });
    }

    nodes
}

/// Remap a footnote marker to avoid collisions within a section.
///
/// Star markers (`*`, `**`, `***`): accumulate — first `*` stays `*`, second becomes `**`, etc.
/// Numeric markers (`1`, `2`, ...): use running counter.
fn remap_marker(marker: &str, star_count: &mut usize, numeric_counter: &mut u32) -> String {
    if marker.chars().all(|c| c == '*') {
        *star_count += 1;
        "*".repeat(*star_count)
    } else if marker.parse::<u32>().is_ok() {
        let n = *numeric_counter;
        *numeric_counter += 1;
        n.to_string()
    } else {
        // Unknown marker type — pass through
        marker.to_string()
    }
}

/// Replace inline footnote references in body text.
///
/// Looks for patterns like `marker)` (e.g. `*)`, `1)`) and replaces with `[^remapped]`.
/// Also handles superscript digits (`¹)` → `[^1]`, etc.).
fn replace_footnote_refs(text: &str, remap: &HashMap<String, String>) -> String {
    let mut result = text.to_string();
    for (old_marker, new_marker) in remap {
        // Replace `marker)` patterns — for star markers look for `*)`, for numeric `1)` etc.
        let search = format!("{})", old_marker);
        let replacement = format!("[^{}]", new_marker);
        result = result.replace(&search, &replacement);

        // Also replace superscript digit variants (e.g. ¹) for marker "1")
        if let Ok(n) = old_marker.parse::<u32>() {
            if let Some(superscript) = digit_to_superscript(n) {
                let super_search = format!("{})", superscript);
                result = result.replace(&super_search, &replacement);
            }
        }
    }
    result
}

/// Convert a single digit (0-9) to its Unicode superscript character.
fn digit_to_superscript(n: u32) -> Option<&'static str> {
    match n {
        0 => Some("\u{2070}"),
        1 => Some("\u{00B9}"),
        2 => Some("\u{00B2}"),
        3 => Some("\u{00B3}"),
        4 => Some("\u{2074}"),
        5 => Some("\u{2075}"),
        6 => Some("\u{2076}"),
        7 => Some("\u{2077}"),
        8 => Some("\u{2078}"),
        9 => Some("\u{2079}"),
        _ => None,
    }
}

// ---------------------------------------------------------------------------
// Page-specific fixups
// ---------------------------------------------------------------------------

/// Apply manual corrections to pages where the upstream OCR/extraction
/// heuristics produce incorrect structure.
fn fixup_pages(pages: &mut [InputPage]) {
    for page in pages.iter_mut() {
        match page.page_index {
            11 => fixup_motto_page(page),
            12 => fixup_dedication_header_page(page),
            14 => fixup_dedication_body_page(page),
            16 => fixup_vorrede_page(page),
            _ => {}
        }
    }
    // Cross-page footnote spanning pages 32-34 needs access to multiple pages
    fixup_cross_page_footnote_32_34(pages);
}

/// Fix the Bacon motto page (page_index 11, AA page 2).
///
/// The upstream extraction produces:
///   [0] heading "BACO DE VERULAMIO.\nII"       — II is actually a b_page_ref
///   [1] heading "Instauratio magna. Praefatio." — correct
///   [2] paragraph (first chunk of Latin)        — these three are one paragraph,
///   [3] paragraph "et ipsi in partem"           — split by gap/indent heuristics
///   [4] paragraph "-\nveniant..."               — with a hyphenation artifact
///   [5] paragraph "1) Das Motto ist..."         — actually a footnote
fn fixup_motto_page(page: &mut InputPage) {
    if page.elements.len() < 6 {
        return;
    }

    // 1. Fix elem[0]: split "BACO DE VERULAMIO.\nII" — keep heading, extract b_page_ref
    if let Some(elem) = page.elements.get_mut(0) {
        elem.lines.retain(|l| l.text.trim() != "II");
        elem.text = "BACO DE VERULAMIO.".to_string();
        elem.b_page_refs = vec!["II".to_string()];
    }

    // 2. Merge elems [2,3,4] into one paragraph, stitching the hyphenation.
    let mut merged_lines: Vec<InputLine> = Vec::new();
    for idx in [2, 3, 4] {
        for line in &page.elements[idx].lines {
            let trimmed = line.text.trim();
            // Skip the stray hyphen line
            if trimmed == "-" {
                continue;
            }
            merged_lines.push(line.clone());
        }
    }
    let merged_text = merged_lines
        .iter()
        .map(|l| l.text.as_str())
        .collect::<Vec<_>>()
        .join("\n");

    let merged_elem = InputElement {
        elem_type: "paragraph".to_string(),
        text: merged_text,
        lines: merged_lines,
        b_page_refs: Vec::new(),
    };

    // 3. Convert elem[5] to a footnote
    let footnote_text = page.elements[5]
        .text
        .trim_start_matches("1)")
        .trim()
        .to_string();
    page.footnotes.push(InputFootnote {
        marker: "1".to_string(),
        text: footnote_text,
    });

    // 4. Rebuild elements: [0] heading, [1] heading, merged paragraph
    let elem0 = page.elements[0].clone();
    let elem1 = page.elements[1].clone();
    page.elements = vec![elem0, elem1, merged_elem];
}

/// Fix the dedication header page (page_index 12, AA page 3).
///
/// Upstream produces:
///   [0] paragraph "Sr. Excellenz,\ndem Königl. Staatsminister\nFreiherrn von Zedlip."
///   [1] paragraph "1*"  — page marker artifact, omit
fn fixup_dedication_header_page(page: &mut InputPage) {
    page.elements.retain(|e| e.text.trim() != "1*");
}

/// Fix the dedication body page (page_index 14, AA page 5).
///
/// Upstream produces:
///   [0] heading  "Gnädiger Herr!"                    — correct
///   [1] heading  "V"                                  — actually b_page_ref
///   [2] paragraph (first body paragraph)              — correct
///   [3] paragraph (second body paragraph, has b_page_ref VI on first line)  — correct
///   [4] heading  "Ew. Excellenz"                      — start of signature, not heading
///
/// The signature lines (Königsberg, date, "unterthänig gehorsamster Diener",
/// "Immanuel Kant.") were eaten by the footnote boundary detector because of
/// the large y-gap after "Ew. Excellenz". We reconstruct them here.
fn fixup_dedication_body_page(page: &mut InputPage) {
    if page.elements.len() < 5 {
        return;
    }

    // 1. "V" (elem[1]) is a b_page_ref, not a heading — attach to elem[0]
    if page.elements[1].text.trim() == "V" {
        page.elements[0].b_page_refs.push("V".to_string());
        page.elements.remove(1);
    }

    // After removal, indices shift: [0]=Gnädiger Herr!, [1]=para1, [2]=para2, [3]=Ew. Excellenz
    // 2. Rebuild elem[3] "Ew. Excellenz" as the full signature paragraph
    if let Some(sig_elem) = page.elements.get_mut(3) {
        sig_elem.elem_type = "paragraph".to_string();
        sig_elem.text = "Ew. Excellenz\n\
                         unterthänig gehorsamster Diener\n\
                         Immanuel Kant.\n\
                         Königsberg,\n\
                         den 23sten April 1787."
            .to_string();
        sig_elem.lines = vec![
            InputLine {
                text: "Ew. Excellenz".to_string(),
                line_number: None,
                b_page_ref: None,
            },
            InputLine {
                text: "unterthänig gehorsamster Diener".to_string(),
                line_number: None,
                b_page_ref: None,
            },
            InputLine {
                text: "Immanuel Kant.".to_string(),
                line_number: None,
                b_page_ref: None,
            },
            InputLine {
                text: "Königsberg,".to_string(),
                line_number: None,
                b_page_ref: None,
            },
            InputLine {
                text: "den 23sten April 1787.".to_string(),
                line_number: None,
                b_page_ref: None,
            },
        ];
    }
}

/// Fix the Vorrede heading (page_index 16, AA page 7).
///
/// OCR produces 'Vorrede zur zweiten Auflage."' — the trailing ." is a
/// Fraktur misread. Strip it.
fn fixup_vorrede_page(page: &mut InputPage) {
    if let Some(elem) = page.elements.first_mut() {
        if elem.text.contains("Vorrede zur zweiten Auflage") {
            elem.text = "Vorrede zur zweiten Auflage".to_string();
            if let Some(line) = elem.lines.first_mut() {
                line.text = "Vorrede zur zweiten Auflage".to_string();
            }
        }
    }
}

/// Fix the cross-page `*)` footnote spanning pages 32-34 (AA 23-25).
///
/// Kant's long `*)` footnote on the refutation of idealism spans three scan pages:
///   Page 32 elem[1]: footnote start (`*) Eigentliche Vermehrung...unmittelbar bes`)
///   Page 33 elem[1]: footnote continuation (40 lines, all footnote text)
///   Page 34 elem[1]: footnote tail (2 lines: `wenig weiter erklären...hervorbringt.`)
///
/// Additionally, the header parser ate one body-text line on each of pages 33 and 34:
///   Page 33 (y=203): "sehung des übrigen auch kein Mißverstand sachkundiger und unparteiischer"
///   Page 34 (y=214): "gebührenden Lobe nennen darf, die Rücksicht, die ich auf ihre Erinnerun-"
///
/// This fixup:
/// 1. Collects all footnote lines into one complete footnote on page 32.
/// 2. Recovers the lost body-text lines on pages 33 and 34.
/// 3. Removes footnote elements so that cross-page body-text stitching works
///    (page 32 body ends "An-" → page 33 body starts "sehung..." → "Ansehung").
fn fixup_cross_page_footnote_32_34(pages: &mut [InputPage]) {
    let idx_32 = pages.iter().position(|p| p.page_index == 32);
    let idx_33 = pages.iter().position(|p| p.page_index == 33);
    let idx_34 = pages.iter().position(|p| p.page_index == 34);

    let (Some(i32), Some(i33), Some(i34)) = (idx_32, idx_33, idx_34) else {
        return;
    };

    // --- 0. Fix page 32: recover body text line eaten by header parser ---
    if !pages[i32].elements.is_empty() {
        pages[i32].elements[0].lines.insert(
            0,
            InputLine {
                text: "änderlichkeit wird sich dieses System, wie ich hoffe, auch fernerhin be-"
                    .to_string(),
                line_number: None,
                b_page_ref: None,
            },
        );
        pages[i32].elements[0].text = pages[i32].elements[0]
            .lines
            .iter()
            .map(|l| l.text.as_str())
            .collect::<Vec<_>>()
            .join("\n");
    }

    // --- 1. Collect all footnote lines and stitch into one text ---

    let mut footnote_lines: Vec<InputLine> = Vec::new();

    // Page 32 elem[1]: `*)` footnote start
    if pages[i32].elements.len() >= 2 {
        for line in &pages[i32].elements[1].lines {
            let mut l = line.clone();
            // Strip `*) ` prefix from first line
            if footnote_lines.is_empty() {
                l.text = l
                    .text
                    .trim_start()
                    .trim_start_matches("*)")
                    .trim_start()
                    .to_string();
            }
            footnote_lines.push(l);
        }
        // Fix OCR error on last line: "unmittelbar bes" should be "unmittelbar be-"
        if let Some(last) = footnote_lines.last_mut() {
            if last.text.trim_end().ends_with(" bes") {
                let t = last.text.trim_end().to_string();
                last.text = format!("{}be-", &t[..t.len() - 3]);
            }
        }
    }

    // Page 33 elem[1]: footnote continuation (40 lines)
    if pages[i33].elements.len() >= 2 {
        for line in &pages[i33].elements[1].lines {
            footnote_lines.push(line.clone());
        }
    }

    // Page 34 elem[1]: footnote tail (2 lines)
    if pages[i34].elements.len() >= 2 {
        for line in &pages[i34].elements[1].lines {
            footnote_lines.push(line.clone());
        }
    }

    // Strip the editors' sub-footnote reference ¹) from the footnote lines
    for line in &mut footnote_lines {
        line.text = line.text.replace("\u{00b9})", "");
    }

    // Stitch all footnote lines into one text
    let (footnote_text, _anchors) = stitch_lines(&footnote_lines);

    // Remove the editors' sub-footnote [1] from page 32's footnotes
    pages[i32].footnotes.retain(|f| f.marker != "1");

    // Store complete footnote on page 32
    pages[i32].footnotes.push(InputFootnote {
        marker: "*".to_string(),
        text: footnote_text,
    });

    // Remove footnote element (elem[1]) from page 32
    if pages[i32].elements.len() >= 2 {
        pages[i32].elements.remove(1);
    }

    // --- 2. Fix page 33: recover body text, remove footnote element ---

    if !pages[i33].elements.is_empty() {
        pages[i33].elements[0].elem_type = "paragraph".to_string();
        pages[i33].elements[0].lines.insert(
            0,
            InputLine {
                text: "sehung des übrigen auch kein Mißverstand sachkundiger und unparteiischer"
                    .to_string(),
                line_number: None,
                b_page_ref: Some("XLI".to_string()),
            },
        );
        pages[i33].elements[0].text = pages[i33].elements[0]
            .lines
            .iter()
            .map(|l| l.text.as_str())
            .collect::<Vec<_>>()
            .join("\n");
        if !pages[i33].elements[0]
            .b_page_refs
            .contains(&"XLI".to_string())
        {
            pages[i33].elements[0]
                .b_page_refs
                .push("XLI".to_string());
        }
    }

    // Remove footnote continuation element from page 33
    if pages[i33].elements.len() >= 2 {
        pages[i33].elements.remove(1);
    }

    // --- 3. Fix page 34: recover body text, remove footnote element ---

    if !pages[i34].elements.is_empty() {
        pages[i34].elements[0].lines.insert(
            0,
            InputLine {
                text: "gebührenden Lobe nennen darf, die Rücksicht, die ich auf ihre Erinnerun-"
                    .to_string(),
                line_number: None,
                b_page_ref: Some("XLII".to_string()),
            },
        );
        pages[i34].elements[0].text = pages[i34].elements[0]
            .lines
            .iter()
            .map(|l| l.text.as_str())
            .collect::<Vec<_>>()
            .join("\n");
        if !pages[i34].elements[0]
            .b_page_refs
            .contains(&"XLII".to_string())
        {
            pages[i34].elements[0]
                .b_page_refs
                .push("XLII".to_string());
        }
    }

    // Remove footnote tail element from page 34
    if pages[i34].elements.len() >= 2 {
        pages[i34].elements.remove(1);
    }
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
        if SKIP_PAGE_INDICES.contains(&page.page_index) {
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
// Content assignment helpers
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

/// Check if a heading element's text matches any TOC entry from the given candidates.
/// Returns the flat_index of the matched entry, if any.
///
/// Matching is fuzzy: strip trailing punctuation and compare case-insensitively,
/// or check if the heading text starts with the TOC label (or vice versa).
fn match_heading_to_toc(
    heading_text: &str,
    flat_entries: &[(usize, u16, u16, &str)],
    candidate_indices: &[usize],
) -> Option<usize> {
    let h = normalize_for_match(heading_text);
    if h.len() < 3 {
        return None;
    }
    for &idx in candidate_indices {
        let (_, _, _, label) = flat_entries[idx];
        let l = normalize_for_match(label);
        // Exact match
        if h == l {
            return Some(idx);
        }
        // Heading starts with label or label starts with heading
        if l.starts_with(&h) || h.starts_with(&l) {
            return Some(idx);
        }
        // Heading is a substantial substring of the label (OCR splits headings
        // across multiple elements, e.g. "Von dem Unterschiede der reinen und
        // empirischen" is part of "I. Von dem Unterschiede der reinen und
        // empirischen Erkenntniß")
        if h.len() >= 10 && l.contains(&h) {
            return Some(idx);
        }
    }
    None
}

/// Normalize text for heading-to-TOC matching: lowercase, strip trailing
/// punctuation, collapse whitespace.
fn normalize_for_match(s: &str) -> String {
    s.trim()
        .trim_end_matches(['.', ')', ',', ';', ':'])
        .trim()
        .to_lowercase()
}

/// Clean OCR artifacts from stitched text.
///
/// - Remove orphan ASCII `"` not part of a German „…" quote pair.
/// - Remove stray `\` before punctuation (e.g. `\')`).
fn clean_ocr_text(text: &str) -> String {
    // First strip stray backslashes before punctuation
    let text = text.replace("\\')", ")").replace("\\\")", ")");

    // Remove orphan ASCII " — keep only those preceded by a „ somewhere earlier
    let mut result = String::with_capacity(text.len());
    let mut in_quote = false;
    for ch in text.chars() {
        if ch == '\u{201E}' {
            // „ — opening German quote
            in_quote = true;
            result.push(ch);
        } else if ch == '"' {
            if in_quote {
                // Closing a „…" pair — keep it
                in_quote = false;
                result.push(ch);
            }
            // else: orphan " — skip
        } else {
            result.push(ch);
        }
    }
    result
}

/// Detect if text starts with an inline footnote marker like `*)`, `**)`, `***)`.
/// Returns the matched prefix (e.g. `"*)"`) or None.
fn detect_inline_footnote_marker(text: &str) -> Option<String> {
    let trimmed = text.trim_start();
    // Match one or more * followed by )
    let star_count = trimmed.chars().take_while(|&c| c == '*').count();
    if star_count > 0 && trimmed.as_bytes().get(star_count) == Some(&b')') {
        Some(trimmed[..star_count + 1].to_string())
    } else {
        None
    }
}
