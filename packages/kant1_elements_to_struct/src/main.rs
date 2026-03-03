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

    // 6. Build the TOC tree
    let mut tree = toc::build_toc_tree();

    // 7. Assign content to TOC nodes
    let flat_entries = toc::flat_toc_entries();
    assign_content(&mut tree, &flat_entries, &page_elements);

    // 8. Number paragraphs and sentences globally
    let mut para_counter = 0u32;
    let mut sentence_counter = 0u32;
    number_tree(&mut tree, &mut para_counter, &mut sentence_counter);
    eprintln!("Numbered {para_counter} paragraphs, {sentence_counter} sentences.");

    // 9. Output
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
    //    Elem 4 starts with "-\nveniant..." — the "-" is a continuation of
    //    elem 3's "et ipsi in partem" → "et ipsi in partem-\nveniant..." → "partemveniant..."
    //    Wait, actually it's "in partem" + "-" + "veniant" = "in partemveniant"? No.
    //    Looking at the actual text: elem[3] = "et ipsi in partem", elem[4] starts with "-\nveniant."
    //    This is "partem" being continued as a word that was split: the original has
    //    "in partem-/veniant" but the hyphen landed on a separate line. So it should just
    //    concatenate without the hyphen: "in partem" is complete and "-" is stray.
    //    The actual Latin: "et ipsi in partem veniant" — so drop the stray hyphen.
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
    // Raw OCR line at y=215: "änderlichkeit wird sich dieses System, wie ich hoffe, auch fernerhin be-"
    // Without it, cross-page stitch from page 31 produces "Unverhaupten" instead of "Unveränderlichkeit".
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
        // (Fraktur 'e' + hyphen misread as 'es'; continuation on page 33 starts "wußt;")
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
    // (the AA editors inserted a "1) Dieses Beharrliche werden kann." note
    // inside Kant's own *) footnote — we drop it entirely)
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

    // The header parser ate the line at y=203:
    //   "XLI sehung des übrigen auch kein Mißverstand sachkundiger und unparteiischer"
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

    // The header parser ate the line at y=214:
    //   "gebührenden Lobe nennen darf, die Rücksicht, die ich auf ihre Erinnerun- XLII"
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
            // Skip printer's signature marks (e.g. "1*", "2*")
            let trimmed = elem.text.trim();
            if trimmed.len() <= 3
                && trimmed.ends_with('*')
                && trimmed[..trimmed.len() - 1].parse::<u16>().is_ok()
            {
                continue;
            }

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

/// Convert an InputElement to a KantContentBlock with stitched lines and sentences.
fn element_to_content_block(elem: &InputElement) -> KantContentBlock {
    let (raw_text, line_anchors) = stitch_lines(&elem.lines);
    let text = clean_ocr_text(&raw_text);
    let b_page_refs = collect_element_b_refs(&elem.b_page_refs, &line_anchors);

    // Detect inline footnotes: paragraphs starting with *), **), etc.
    // These are Kant's own footnotes that appear in the body text rather than
    // in the footnote zone at the bottom of the page.
    if let Some(marker) = detect_inline_footnote_marker(&text) {
        let body = text[marker.len()..]
            .trim_start_matches(')')
            .trim()
            .to_string();
        return KantContentBlock {
            position: 0,
            block_type: KantBlockType::Footnote,
            paragraph_number: None,
            text: format!("[{}] {}", marker.trim_end_matches(')'), body),
            b_page_refs,
            sentences: Vec::new(),
        };
    }

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
/// position fields are 0-based indices within their parent:
///   - KantContentBlock.position = index within node.content
///   - KantSentence.position = index within block.sentences (already set at construction)
fn number_tree(
    nodes: &mut [KantTocNode],
    para_counter: &mut u32,
    sentence_counter: &mut u32,
) {
    for node in nodes.iter_mut() {
        for (block_idx, block) in node.content.iter_mut().enumerate() {
            block.position = block_idx as u32;

            if block.block_type == KantBlockType::Paragraph {
                *para_counter += 1;
                block.paragraph_number = Some(*para_counter);
            }

            // sentence.position is already 0-based from element_to_content_block;
            // here we only assign the global sentence_number
            for sentence in block.sentences.iter_mut() {
                *sentence_counter += 1;
                sentence.sentence_number = *sentence_counter;
            }
        }

        number_tree(&mut node.children, para_counter, sentence_counter);
    }
}
