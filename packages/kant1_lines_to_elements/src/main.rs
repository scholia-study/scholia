use std::collections::BTreeSet;
use std::fs;
use std::path::Path;
use std::sync::LazyLock;

use clap::Parser;
use regex::Regex;
use serde::{Deserialize, Serialize};

// ---------------------------------------------------------------------------
// Constants
// ---------------------------------------------------------------------------

const IMG_H: f64 = 2434.0;
const HEADER_Y_MAX: f64 = 220.0;
const FOOTNOTE_Y_MIN_FRAC: f64 = 0.65;
const FOOTNOTE_GAP_PX: f64 = 30.0;
const FRONT_MATTER_END: usize = 12;

// ---------------------------------------------------------------------------
// Compiled regexes
// ---------------------------------------------------------------------------

static ROMAN_RE: LazyLock<Regex> = LazyLock::new(|| Regex::new(r"^[IVXLCDM]{2,}$").unwrap());
static FOOTNOTE_MARKER_RE: LazyLock<Regex> =
    LazyLock::new(|| Regex::new(r"[¹²³⁴⁵⁶⁷⁸⁹⁰\d]+\)|[*†‡][)\.]").unwrap());
static DIGIT_ONLY_RE: LazyLock<Regex> = LazyLock::new(|| Regex::new(r"^\d+$").unwrap());
static DIGIT_1_2_RE: LazyLock<Regex> = LazyLock::new(|| Regex::new(r"^\d{1,2}$").unwrap());
static LINE_NUM_START_RE: LazyLock<Regex> = LazyLock::new(|| Regex::new(r"^(\d{1,2})\s+").unwrap());
static B_REF_ROMAN_RE: LazyLock<Regex> =
    LazyLock::new(|| Regex::new(r"\s+([IVXLCDM]{2,})\s*$").unwrap());
static B_REF_ARABIC_RE: LazyLock<Regex> =
    LazyLock::new(|| Regex::new(r"\s+(\d{1,4})\s*$").unwrap());
static LINE_NUM_END_RE: LazyLock<Regex> =
    LazyLock::new(|| Regex::new(r"\s+(\d{1,2})\s*$").unwrap());
static B_REF_ROMAN_START_RE: LazyLock<Regex> =
    LazyLock::new(|| Regex::new(r"^([IVXLCDM]{2,})\s+").unwrap());
static B_REF_ARABIC_START_RE: LazyLock<Regex> =
    LazyLock::new(|| Regex::new(r"^(\d{1,4})\s+").unwrap());
static BULLET_RE: LazyLock<Regex> = LazyLock::new(|| Regex::new(r"^[•]\s*").unwrap());
static FN_DIGIT_MARKER_RE: LazyLock<Regex> =
    LazyLock::new(|| Regex::new(r"([¹²³⁴⁵⁶⁷⁸⁹⁰\d]+)\)").unwrap());
static FN_SYMBOL_MARKER_RE: LazyLock<Regex> = LazyLock::new(|| Regex::new(r"([*†‡§])").unwrap());
static FN_SPLIT_RE: LazyLock<Regex> =
    LazyLock::new(|| Regex::new(r"(?:^|\n)\s*(\d+|\*|†|‡)\)\s*").unwrap());
static STANDALONE_B_REF_RE: LazyLock<Regex> = LazyLock::new(|| Regex::new(r"^\d{1,4}$").unwrap());

// ---------------------------------------------------------------------------
// CLI
// ---------------------------------------------------------------------------

#[derive(Parser)]
#[command(about = "Extract structured elements from OCR line data")]
struct Args {
    /// Directory containing per-page OCR line JSON
    #[arg(long, default_value = "assets/kant1_ocr_to_lines")]
    input_dir: String,

    /// Directory for per-page element JSON output
    #[arg(long, default_value = "assets/kant1_lines_to_elements")]
    output_dir: String,

    /// Final merged JSON output path
    #[arg(long, default_value = "assets/kant1_kritik_docai.json")]
    output: String,

    /// Start page index, 1-based
    #[arg(long, default_value_t = 1)]
    start: usize,

    /// End page index, 1-based inclusive
    #[arg(long)]
    end: Option<usize>,
}

fn find_input_files(input_dir: &str) -> Vec<String> {
    let pattern = format!("{input_dir}/*.json");
    let mut files: Vec<String> = glob::glob(&pattern)
        .expect("Invalid glob pattern")
        .filter_map(|entry| entry.ok())
        .map(|path| path.to_string_lossy().to_string())
        .collect();
    files.sort();
    files
}

fn main() {
    let args = Args::parse();

    fs::create_dir_all(&args.output_dir).expect("Failed to create output directory");

    let input_files = find_input_files(&args.input_dir);
    if input_files.is_empty() {
        eprintln!("No OCR line files found in {}/", args.input_dir);
        eprintln!("Run kant1_ocr_to_lines first.");
        return;
    }

    let total = input_files.len();
    let end_idx = args.end.unwrap_or(total);

    eprintln!(
        "Found {} OCR line files. Processing pages {}–{}.",
        total, args.start, end_idx
    );

    for i in (args.start - 1)..end_idx.min(total) {
        let page_num = i + 1;
        let out_path = format!("{}/page_{:04}.json", args.output_dir, page_num);
        let filename = Path::new(&input_files[i])
            .file_name()
            .unwrap()
            .to_string_lossy();

        eprint!("  [{page_num}/{end_idx}] {filename}... ");

        let data = match fs::read_to_string(&input_files[i]) {
            Ok(d) => d,
            Err(e) => {
                eprintln!("FAILED: {e}");
                continue;
            }
        };

        let ocr_lines: Vec<OcrLine> = match serde_json::from_str(&data) {
            Ok(l) => l,
            Err(e) => {
                eprintln!("FAILED: {e}");
                continue;
            }
        };

        let result = process_page(&ocr_lines, page_num);
        let n_elem = result.elements.len();
        let n_fn = result.footnotes.len();

        let json = serde_json::to_string_pretty(&result).unwrap();
        if let Err(e) = fs::write(&out_path, &json) {
            eprintln!("FAILED: {e}");
            continue;
        }

        eprintln!(
            "done ({}, {} elements, {} footnotes)",
            result.page_type, n_elem, n_fn
        );
    }

    // Merge step — combine all per-page JSONs into single output
    let merge_pattern = format!("{}/*.json", args.output_dir);
    let mut page_files: Vec<String> = glob::glob(&merge_pattern)
        .expect("Invalid glob pattern")
        .filter_map(|entry| entry.ok())
        .map(|path| path.to_string_lossy().to_string())
        .filter(|p| {
            Path::new(p)
                .file_name()
                .unwrap()
                .to_string_lossy()
                .starts_with("page_")
        })
        .collect();
    page_files.sort();

    if page_files.is_empty() {
        eprintln!("No page JSON files to merge.");
        return;
    }

    let mut pages: Vec<PageResult> = Vec::new();
    for pf in &page_files {
        let data = fs::read_to_string(pf).expect("Failed to read page file");
        let page: PageResult = serde_json::from_str(&data).expect("Failed to parse page JSON");
        pages.push(page);
    }

    let merged = MergedOutput { pages };
    let json = serde_json::to_string_pretty(&merged).unwrap();
    fs::write(&args.output, &json).expect("Failed to write merged output");

    eprintln!("Merged {} pages into {}", page_files.len(), args.output);
}

// ---------------------------------------------------------------------------
// Types
// ---------------------------------------------------------------------------

#[derive(Debug, Clone, Deserialize)]
struct OcrLine {
    text: String,
    x: f64,
    y: f64,
    #[allow(dead_code)]
    width: f64,
    height: f64,
}

#[derive(Debug, Clone)]
struct AnnotatedLine {
    text: String,
    x: f64,
    y: f64,
    line_number: Option<i64>,
    b_page_ref: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
struct LineOut {
    text: String,
    line_number: Option<i64>,
    b_page_ref: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
struct Element {
    r#type: String,
    text: String,
    lines: Vec<LineOut>,
    emphasis: Vec<String>,
    typeface: String,
    line_numbers: Vec<i64>,
    footnote_markers: Vec<String>,
    b_page_refs: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
struct Footnote {
    marker: String,
    text: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
struct PageResult {
    page_index: usize,
    page_number: Option<String>,
    page_type: String,
    elements: Vec<Element>,
    footnotes: Vec<Footnote>,
}

#[derive(Debug, Serialize)]
struct MergedOutput {
    pages: Vec<PageResult>,
}

// ---------------------------------------------------------------------------
// Superscript normalization
// ---------------------------------------------------------------------------

fn normalize_superscripts(text: &str) -> String {
    text.replace('⁰', "0")
        .replace('¹', "1")
        .replace('²', "2")
        .replace('³', "3")
        .replace('⁴', "4")
        .replace('⁵', "5")
        .replace('⁶', "6")
        .replace('⁷', "7")
        .replace('⁸', "8")
        .replace('⁹', "9")
}

// ---------------------------------------------------------------------------
// Zone partitioning
// ---------------------------------------------------------------------------

fn partition_zones(
    lines: &[OcrLine],
    page_index: usize,
) -> (Vec<OcrLine>, Vec<OcrLine>, Vec<OcrLine>) {
    let is_front_matter = page_index <= FRONT_MATTER_END;

    let mut header = Vec::new();
    let mut body = Vec::new();

    for line in lines {
        if line.y < HEADER_Y_MAX && !is_front_matter {
            header.push(line.clone());
        } else {
            body.push(line.clone());
        }
    }

    let footnote_lines = if !is_front_matter {
        let (new_body, fns) = split_footnotes(&body);
        body = new_body;
        fns
    } else {
        Vec::new()
    };

    (header, body, footnote_lines)
}

fn split_footnotes(body_lines: &[OcrLine]) -> (Vec<OcrLine>, Vec<OcrLine>) {
    if body_lines.is_empty() {
        return (body_lines.to_vec(), Vec::new());
    }

    let mut sorted_by_y: Vec<&OcrLine> = body_lines.iter().collect();
    sorted_by_y.sort_by(|a, b| a.y.partial_cmp(&b.y).unwrap());

    let page_y_min = FOOTNOTE_Y_MIN_FRAC * IMG_H;

    // Collect qualifying gaps
    let mut gaps: Vec<(f64, f64)> = Vec::new();
    for i in 1..sorted_by_y.len() {
        let prev_bottom = sorted_by_y[i - 1].y + sorted_by_y[i - 1].height;
        let curr_top = sorted_by_y[i].y;
        let gap = curr_top - prev_bottom;

        if curr_top > page_y_min && gap >= FOOTNOTE_GAP_PX {
            gaps.push((curr_top, gap));
        }
    }

    // Try each gap from earliest to latest
    for &(gap_y, _) in &gaps {
        let lines_after: Vec<&OcrLine> = body_lines.iter().filter(|l| l.y >= gap_y).collect();
        if lines_after.is_empty() {
            continue;
        }

        let mut first_lines: Vec<&&OcrLine> = lines_after.iter().collect();
        first_lines.sort_by(|a, b| {
            a.y.partial_cmp(&b.y)
                .unwrap()
                .then(a.x.partial_cmp(&b.x).unwrap())
        });
        let first_text: String = first_lines
            .iter()
            .take(5)
            .map(|l| l.text.as_str())
            .collect::<Vec<_>>()
            .join(" ");

        if FOOTNOTE_MARKER_RE.is_match(&first_text) {
            let main_body: Vec<OcrLine> =
                body_lines.iter().filter(|l| l.y < gap_y).cloned().collect();
            let fn_lines: Vec<OcrLine> = body_lines
                .iter()
                .filter(|l| l.y >= gap_y)
                .cloned()
                .collect();
            return (main_body, fn_lines);
        }
    }

    (body_lines.to_vec(), Vec::new())
}

// ---------------------------------------------------------------------------
// Header parsing
// ---------------------------------------------------------------------------

fn parse_header(header_lines: &[OcrLine]) -> Option<String> {
    if header_lines.is_empty() {
        return None;
    }

    let mut sorted_by_x: Vec<&OcrLine> = header_lines.iter().collect();
    sorted_by_x.sort_by(|a, b| a.x.partial_cmp(&b.x).unwrap());

    // Rightmost first — look for Arabic page number
    for line in sorted_by_x.iter().rev() {
        let text = line.text.trim();
        if DIGIT_ONLY_RE.is_match(text) {
            return Some(text.to_string());
        }
    }

    // Leftmost — look for Arabic page number
    for line in &sorted_by_x {
        let text = line.text.trim();
        if DIGIT_ONLY_RE.is_match(text) {
            return Some(text.to_string());
        }
    }

    // Roman numerals
    for line in &sorted_by_x {
        let text = line.text.trim();
        if ROMAN_RE.is_match(text) {
            return Some(text.to_string());
        }
    }

    None
}

// ---------------------------------------------------------------------------
// Per-line annotation (line numbers + B-refs)
// ---------------------------------------------------------------------------

fn is_valid_line_number(val: i64) -> bool {
    val % 5 == 0 && (5..=40).contains(&val)
}

/// Try to interpret a token as an OCR-garbled Roman numeral.
/// Common OCR misreadings: '1'→'I', lowercase 'x'→'X', etc.
fn try_ocr_correct_roman(token: &str) -> Option<String> {
    if token.is_empty() {
        return None;
    }
    let normalized: String = token
        .chars()
        .map(|c| match c {
            '1' => 'I',
            'i' | 'I' => 'I',
            'v' | 'V' => 'V',
            'x' | 'X' => 'X',
            'l' | 'L' => 'L',
            'c' | 'C' => 'C',
            'd' | 'D' => 'D',
            'm' | 'M' => 'M',
            _ => c,
        })
        .collect();
    if normalized.chars().all(|c| "IVXLCDM".contains(c)) {
        Some(normalized)
    } else {
        None
    }
}

fn annotate_lines(body_lines: &[OcrLine], page_index: usize) -> Vec<AnnotatedLine> {
    // In a book scan, margins alternate sides on every other page:
    //   Even scan index: line numbers at START (left), B-refs at END (right)
    //   Odd scan index:  line numbers at END (right), B-refs at START (left)
    let line_nums_at_end = page_index % 2 == 1;

    // Compute median x for spatial detection of OCR-garbled margin annotations.
    // On odd pages, B-refs at the left margin cause lines to have lower x values.
    let median_x = if body_lines.len() >= 3 {
        let mut xs: Vec<f64> = body_lines.iter().map(|l| l.x).collect();
        median(&mut xs)
    } else {
        0.0
    };

    let mut result = Vec::new();

    for line in body_lines {
        let mut text = line.text.clone();
        let mut line_number: Option<i64> = None;
        let mut b_page_ref: Option<String> = None;

        // Standalone line number (works regardless of margin side)
        let stripped = text.trim();
        if DIGIT_1_2_RE.is_match(stripped) {
            if let Ok(val) = stripped.parse::<i64>() {
                if is_valid_line_number(val) {
                    continue; // drop margin annotation
                }
            }
        }

        if line_nums_at_end {
            // Odd scan pages: line numbers at END, B-refs at START

            // B-edition ref at start — clean Roman numerals
            if let Some(cap) = B_REF_ROMAN_START_RE.captures(&text) {
                b_page_ref = Some(cap[1].to_string());
                let m = cap.get(0).unwrap();
                text = text[m.end()..].to_string();
            }
            // B-edition ref at start — Arabic (only if NOT a valid line number)
            else if let Some(cap) = B_REF_ARABIC_START_RE.captures(&text) {
                if let Ok(val) = cap[1].parse::<i64>() {
                    if !is_valid_line_number(val) {
                        b_page_ref = Some(cap[1].to_string());
                        let m = cap.get(0).unwrap();
                        text = text[m.end()..].to_string();
                    }
                }
            }

            // Spatial detection: if x is well below median, the leading token
            // may be an OCR-garbled Roman numeral B-ref (e.g. "1x" for "IX").
            if b_page_ref.is_none() && median_x > 0.0 && line.x < median_x - 30.0 {
                if let Some(space_pos) = text.find(char::is_whitespace) {
                    let token = &text[..space_pos];
                    if let Some(roman) = try_ocr_correct_roman(token) {
                        b_page_ref = Some(roman);
                        text = text[space_pos..].trim_start().to_string();
                    }
                }
            }

            // Line number at end
            if let Some(cap) = LINE_NUM_END_RE.captures(&text) {
                if let Ok(val) = cap[1].parse::<i64>() {
                    if is_valid_line_number(val) {
                        line_number = Some(val);
                        let m = cap.get(0).unwrap();
                        text = text[..m.start()].to_string();
                    }
                }
            }

            // Fallback: Roman B-ref at end (rare on these pages but possible)
            if line_number.is_none() && b_page_ref.is_none() {
                if let Some(cap) = B_REF_ROMAN_RE.captures(&text) {
                    b_page_ref = Some(cap[1].to_string());
                    let m = cap.get(0).unwrap();
                    text = text[..m.start()].to_string();
                }
            }
        } else {
            // Even scan pages: line numbers at START, B-refs at END

            // Line number at start
            if let Some(m) = LINE_NUM_START_RE.find(&text) {
                let cap = LINE_NUM_START_RE.captures(&text).unwrap();
                if let Ok(val) = cap[1].parse::<i64>() {
                    if is_valid_line_number(val) {
                        line_number = Some(val);
                        text = text[m.end()..].to_string();
                    }
                }
            }

            // B-edition ref at end — Roman numerals
            if let Some(cap) = B_REF_ROMAN_RE.captures(&text) {
                let ref_text = cap[1].to_string();
                let m = cap.get(0).unwrap();
                b_page_ref = Some(ref_text);
                text = text[..m.start()].to_string();
            } else if let Some(cap) = B_REF_ARABIC_RE.captures(&text) {
                let ref_text = cap[1].to_string();
                let m = cap.get(0).unwrap();
                b_page_ref = Some(ref_text);
                text = text[..m.start()].to_string();
            }
        }

        // Bullet / artifact at line start
        text = BULLET_RE.replace(&text, "").to_string();

        result.push(AnnotatedLine {
            text,
            x: line.x,
            y: line.y,
            line_number,
            b_page_ref,
        });
    }

    result
}

// ---------------------------------------------------------------------------
// Paragraph grouping
// ---------------------------------------------------------------------------

fn median(values: &mut [f64]) -> f64 {
    values.sort_by(|a, b| a.partial_cmp(b).unwrap());
    let len = values.len();
    if len % 2 == 0 {
        (values[len / 2 - 1] + values[len / 2]) / 2.0
    } else {
        values[len / 2]
    }
}

fn group_into_paragraphs(annotated_lines: &[AnnotatedLine]) -> Vec<Vec<AnnotatedLine>> {
    if annotated_lines.is_empty() {
        return Vec::new();
    }

    let mut sorted_lines: Vec<AnnotatedLine> = annotated_lines.to_vec();
    sorted_lines.sort_by(|a, b| a.y.partial_cmp(&b.y).unwrap());

    // Compute median line spacing
    let mut spacings: Vec<f64> = Vec::new();
    for i in 1..sorted_lines.len() {
        let spacing = sorted_lines[i].y - sorted_lines[i - 1].y;
        if spacing > 0.0 {
            spacings.push(spacing);
        }
    }

    if spacings.is_empty() {
        return vec![sorted_lines];
    }

    let median_spacing = median(&mut spacings);

    // Compute median left-x
    let mut left_xs: Vec<f64> = sorted_lines.iter().map(|l| l.x).collect();
    let median_left_x = median(&mut left_xs);

    // Group lines into paragraphs
    let mut groups: Vec<Vec<AnnotatedLine>> = vec![vec![sorted_lines[0].clone()]];

    for i in 1..sorted_lines.len() {
        let prev = &sorted_lines[i - 1];
        let curr = &sorted_lines[i];
        let spacing = curr.y - prev.y;

        let is_gap = spacing > median_spacing * 1.5;
        let is_indented = curr.x > median_left_x + 30.0;

        if is_gap || is_indented {
            groups.push(vec![curr.clone()]);
        } else {
            groups.last_mut().unwrap().push(curr.clone());
        }
    }

    groups
}

// ---------------------------------------------------------------------------
// Structure assembly
// ---------------------------------------------------------------------------

fn is_heading(text: &str, line_count: usize) -> bool {
    if text.len() >= 80 || line_count > 2 {
        return false;
    }
    let clean = text.trim_start_matches(|c: char| c.is_ascii_digit() || c == '.' || c == ' ');
    if clean.is_empty() {
        return false;
    }
    let first_char = clean.chars().next().unwrap();
    if !first_char.is_uppercase() {
        return false;
    }
    if let Some(last) = text.trim_end().chars().last() {
        if matches!(last, ',' | ';' | ':') {
            return false;
        }
    }
    true
}

fn detect_footnote_markers_in_text(text: &str) -> Vec<String> {
    let mut markers = Vec::new();
    for cap in FN_DIGIT_MARKER_RE.captures_iter(text) {
        markers.push(normalize_superscripts(&cap[1]));
    }
    for cap in FN_SYMBOL_MARKER_RE.captures_iter(text) {
        markers.push(cap[1].to_string());
    }
    markers
}

fn build_element(line_group: &[AnnotatedLine]) -> Element {
    let lines_out: Vec<LineOut> = line_group
        .iter()
        .map(|l| LineOut {
            text: l.text.clone(),
            line_number: l.line_number,
            b_page_ref: l.b_page_ref.clone(),
        })
        .collect();

    let text: String = lines_out
        .iter()
        .map(|l| l.text.as_str())
        .collect::<Vec<_>>()
        .join("\n");

    let line_numbers: Vec<i64> = {
        let set: BTreeSet<i64> = lines_out.iter().filter_map(|l| l.line_number).collect();
        set.into_iter().collect()
    };

    let b_page_refs: Vec<String> = lines_out
        .iter()
        .filter_map(|l| l.b_page_ref.clone())
        .collect();

    let elem_type = if is_heading(&text, lines_out.len()) {
        "heading"
    } else {
        "paragraph"
    };

    let markers = detect_footnote_markers_in_text(&text);

    Element {
        r#type: elem_type.to_string(),
        text,
        lines: lines_out,
        emphasis: Vec::new(),
        typeface: "fraktur".to_string(),
        line_numbers,
        footnote_markers: markers,
        b_page_refs,
    }
}

// ---------------------------------------------------------------------------
// Footnote parsing
// ---------------------------------------------------------------------------

fn parse_footnotes(footnote_lines: &[OcrLine]) -> Vec<Footnote> {
    if footnote_lines.is_empty() {
        return Vec::new();
    }

    let mut sorted_lines: Vec<&OcrLine> = footnote_lines.iter().collect();
    sorted_lines.sort_by(|a, b| {
        a.y.partial_cmp(&b.y)
            .unwrap()
            .then(a.x.partial_cmp(&b.x).unwrap())
    });

    let full_text: String = sorted_lines
        .iter()
        .map(|l| l.text.as_str())
        .collect::<Vec<_>>()
        .join("\n");

    let full_text = normalize_superscripts(&full_text);

    // Split on footnote markers: digit(s) followed by )
    let parts: Vec<&str> = FN_SPLIT_RE.split(&full_text).collect();
    let caps: Vec<regex::Match> = FN_SPLIT_RE
        .captures_iter(&full_text)
        .filter_map(|c| c.get(1))
        .collect();

    let mut footnotes = Vec::new();
    for (i, cap) in caps.iter().enumerate() {
        let marker = cap.as_str().to_string();
        if i + 1 < parts.len() {
            // parts[0] is text before first marker, parts[1] is after first marker, etc.
            let text = parts[i + 1].trim().replace('\n', " ");
            if !text.is_empty() {
                footnotes.push(Footnote { marker, text });
            }
        }
    }

    footnotes
}

// ---------------------------------------------------------------------------
// Front matter detection
// ---------------------------------------------------------------------------

fn detect_front_matter_type(page_index: usize, body_lines: &[OcrLine]) -> &'static str {
    if body_lines.is_empty() {
        return "blank";
    }

    let text: String = body_lines
        .iter()
        .map(|l| l.text.as_str())
        .collect::<Vec<_>>()
        .join(" ")
        .to_lowercase();

    if page_index <= 2 {
        return "title";
    }
    if text.contains("inhalt") || text.contains("inhalts") {
        return "toc";
    }

    "other"
}

// ---------------------------------------------------------------------------
// Page processing
// ---------------------------------------------------------------------------

fn process_page(ocr_lines: &[OcrLine], page_index: usize) -> PageResult {
    // Blank page detection
    let meaningful: Vec<&OcrLine> = ocr_lines
        .iter()
        .filter(|l| l.text.trim().len() > 1)
        .collect();

    if meaningful.len() < 3 {
        return PageResult {
            page_index,
            page_number: None,
            page_type: "blank".to_string(),
            elements: Vec::new(),
            footnotes: Vec::new(),
        };
    }

    let is_front = page_index <= FRONT_MATTER_END;

    // Zone partitioning
    let (header, body, footnote_lines) = partition_zones(ocr_lines, page_index);

    // Header parsing
    let page_number = parse_header(&header);

    // Page type
    let page_type = if is_front {
        detect_front_matter_type(page_index, &body).to_string()
    } else {
        "body".to_string()
    };

    // Annotate body lines
    let annotated: Vec<AnnotatedLine> = if !is_front {
        annotate_lines(&body, page_index)
    } else {
        body.iter()
            .map(|l| AnnotatedLine {
                text: l.text.clone(),
                x: l.x,
                y: l.y,
                line_number: None,
                b_page_ref: None,
            })
            .collect()
    };

    // Group into paragraphs
    let para_groups = group_into_paragraphs(&annotated);

    // Build elements
    let mut elements: Vec<Element> = para_groups.iter().map(|g| build_element(g)).collect();

    // Standalone B-ref cleanup
    if !is_front {
        let mut cleaned: Vec<Element> = Vec::new();
        for elem in elements {
            let text = elem.text.trim().to_string();
            let is_b_ref = ROMAN_RE.is_match(&text)
                || (STANDALONE_B_REF_RE.is_match(&text) && text.len() < 10);
            if is_b_ref && !cleaned.is_empty() {
                cleaned.last_mut().unwrap().b_page_refs.push(text);
                continue;
            }
            cleaned.push(elem);
        }
        elements = cleaned;
    }

    // Parse footnotes
    let footnotes = parse_footnotes(&footnote_lines);

    PageResult {
        page_index,
        page_number,
        page_type,
        elements,
        footnotes,
    }
}
