use std::fs;
use std::path::Path;
use std::sync::LazyLock;

use clap::Parser;
use regex::Regex;
use serde::{Deserialize, Serialize};

// ---------------------------------------------------------------------------
// CLI
// ---------------------------------------------------------------------------

#[derive(Parser)]
#[command(about = "Extract line-level OCR data from raw Document AI responses")]
struct Args {
    /// Directory containing raw Document AI response JSON
    #[arg(long, default_value = "assets/kant1_png_to_ocr")]
    input_dir: String,

    /// Directory for per-page line JSON output
    #[arg(long, default_value = "assets/kant1_ocr_to_lines")]
    output_dir: String,

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
        eprintln!("No raw OCR files found in {}/", args.input_dir);
        eprintln!("Run kant1_png_to_ocr.py first.");
        return;
    }

    let total = input_files.len();
    let end_idx = args.end.unwrap_or(total);

    eprintln!(
        "Found {} raw OCR files. Processing pages {}–{}.",
        total, args.start, end_idx
    );

    let start = args.start.saturating_sub(1);
    let end = end_idx.min(total);
    for (i, path) in input_files.iter().enumerate().take(end).skip(start) {
        let page_num = i + 1;
        let filename = Path::new(path).file_name().unwrap().to_string_lossy();

        // Output uses same basename as input
        let out_path = format!("{}/{}", args.output_dir, filename);

        eprint!("  [{page_num}/{end_idx}] {filename}... ");

        let data = match fs::read_to_string(path) {
            Ok(d) => d,
            Err(e) => {
                eprintln!("FAILED: {e}");
                continue;
            }
        };

        let doc: DocAiResponse = match serde_json::from_str(&data) {
            Ok(d) => d,
            Err(e) => {
                eprintln!("FAILED: {e}");
                continue;
            }
        };

        let lines = extract_lines(&doc);
        let lines = strip_line_numbers(lines, page_num);

        let json = serde_json::to_string_pretty(&lines).unwrap();
        if let Err(e) = fs::write(&out_path, &json) {
            eprintln!("FAILED: {e}");
            continue;
        }

        eprintln!("done ({} lines)", lines.len());
    }
}

// ---------------------------------------------------------------------------
// Types — raw Document AI response (subset we need)
// ---------------------------------------------------------------------------

#[derive(Debug, Deserialize)]
struct DocAiResponse {
    text: Option<String>,
    pages: Option<Vec<DocAiPage>>,
}

#[derive(Debug, Deserialize)]
struct DocAiPage {
    dimension: Option<PageDimension>,
    lines: Option<Vec<DocAiLine>>,
}

#[derive(Debug, Deserialize)]
struct PageDimension {
    width: f64,
    height: f64,
}

#[derive(Debug, Deserialize)]
struct DocAiLine {
    layout: DocAiLayout,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
struct DocAiLayout {
    text_anchor: Option<TextAnchor>,
    bounding_poly: Option<BoundingPoly>,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
struct TextAnchor {
    text_segments: Option<Vec<TextSegment>>,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
struct TextSegment {
    start_index: Option<StringOrInt>,
    end_index: Option<StringOrInt>,
}

/// Document AI serializes indices as strings in proto-JSON.
#[derive(Debug, Deserialize)]
#[serde(untagged)]
enum StringOrInt {
    Str(String),
    Int(i64),
}

impl StringOrInt {
    fn as_usize(&self) -> usize {
        match self {
            StringOrInt::Str(s) => s.parse().unwrap_or(0),
            StringOrInt::Int(n) => *n as usize,
        }
    }
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
struct BoundingPoly {
    normalized_vertices: Option<Vec<NormalizedVertex>>,
}

#[derive(Debug, Deserialize)]
struct NormalizedVertex {
    x: Option<f64>,
    y: Option<f64>,
}

// ---------------------------------------------------------------------------
// Compiled regexes (line-number detection)
// ---------------------------------------------------------------------------

static DIGIT_1_2_RE: LazyLock<Regex> = LazyLock::new(|| Regex::new(r"^\d{1,2}$").unwrap());
static LINE_NUM_START_RE: LazyLock<Regex> = LazyLock::new(|| Regex::new(r"^(\d{1,2})\s+").unwrap());
static LINE_NUM_END_RE: LazyLock<Regex> =
    LazyLock::new(|| Regex::new(r"\s+(\d{1,2})\s*$").unwrap());

fn is_valid_line_number(val: i64) -> bool {
    val % 5 == 0 && (5..=40).contains(&val)
}

// ---------------------------------------------------------------------------
// Output type
// ---------------------------------------------------------------------------

#[derive(Debug, Serialize)]
struct OcrLine {
    text: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    line_number: Option<i64>,
    x: f64,
    y: f64,
    width: f64,
    height: f64,
}

// ---------------------------------------------------------------------------
// Line extraction
// ---------------------------------------------------------------------------

fn extract_lines(doc: &DocAiResponse) -> Vec<OcrLine> {
    let full_text = match &doc.text {
        Some(t) => t.as_str(),
        None => return Vec::new(),
    };

    let pages = match &doc.pages {
        Some(p) if !p.is_empty() => p,
        _ => return Vec::new(),
    };

    let page = &pages[0];
    let dim = page.dimension.as_ref();
    let img_w = dim.map_or(1575.0, |d| d.width);
    let img_h = dim.map_or(2434.0, |d| d.height);

    let doc_lines = match &page.lines {
        Some(l) => l,
        None => return Vec::new(),
    };

    let mut result = Vec::new();

    for line in doc_lines {
        // Extract text via text_anchor
        let segments = match &line.layout.text_anchor {
            Some(ta) => match &ta.text_segments {
                Some(segs) => segs,
                None => continue,
            },
            None => continue,
        };

        let text: String = segments
            .iter()
            .map(|seg| {
                let start = seg.start_index.as_ref().map_or(0, |s| s.as_usize());
                let end = seg.end_index.as_ref().map_or(0, |e| e.as_usize());
                // Indices from Document AI are code-point offsets, not byte offsets
                let start_byte = full_text
                    .char_indices()
                    .nth(start)
                    .map_or(full_text.len(), |(i, _)| i);
                let end_byte = full_text
                    .char_indices()
                    .nth(end)
                    .map_or(full_text.len(), |(i, _)| i);
                &full_text[start_byte..end_byte]
            })
            .collect::<String>()
            .trim_end_matches('\n')
            .to_string();

        if text.trim().is_empty() {
            continue;
        }

        // Extract bounding box from normalized vertices
        let verts = match &line.layout.bounding_poly {
            Some(bp) => match &bp.normalized_vertices {
                Some(nv) if !nv.is_empty() => nv,
                _ => continue,
            },
            None => continue,
        };

        let xs: Vec<f64> = verts.iter().map(|v| v.x.unwrap_or(0.0) * img_w).collect();
        let ys: Vec<f64> = verts.iter().map(|v| v.y.unwrap_or(0.0) * img_h).collect();

        let x_min = xs.iter().cloned().fold(f64::INFINITY, f64::min);
        let y_min = ys.iter().cloned().fold(f64::INFINITY, f64::min);
        let x_max = xs.iter().cloned().fold(f64::NEG_INFINITY, f64::max);
        let y_max = ys.iter().cloned().fold(f64::NEG_INFINITY, f64::max);

        result.push(OcrLine {
            text,
            line_number: None,
            x: x_min,
            y: y_min,
            width: x_max - x_min,
            height: y_max - y_min,
        });
    }

    result
}

// ---------------------------------------------------------------------------
// Line-number stripping
// ---------------------------------------------------------------------------

/// Strip margin line numbers (5, 10, 15, ... 40) from OCR text.
/// Determines margin side from page index:
///   Even scan index (page_num % 2 == 0): line numbers at START (left margin)
///   Odd scan index  (page_num % 2 == 1): line numbers at END (right margin)
fn strip_line_numbers(lines: Vec<OcrLine>, page_num: usize) -> Vec<OcrLine> {
    let line_nums_at_end = page_num % 2 == 1;

    lines
        .into_iter()
        .filter_map(|mut line| {
            let stripped = line.text.trim();

            // Drop standalone lines that are just a valid line number
            if DIGIT_1_2_RE.is_match(stripped)
                && let Ok(val) = stripped.parse::<i64>()
                && is_valid_line_number(val)
            {
                return None;
            }

            if line_nums_at_end {
                // Line number at end of text
                if let Some(cap) = LINE_NUM_END_RE.captures(&line.text)
                    && let Ok(val) = cap[1].parse::<i64>()
                    && is_valid_line_number(val)
                {
                    line.line_number = Some(val);
                    let m = cap.get(0).unwrap();
                    line.text = line.text[..m.start()].to_string();
                }
            } else {
                // Line number at start of text
                if let Some(m) = LINE_NUM_START_RE.find(&line.text) {
                    let cap = LINE_NUM_START_RE.captures(&line.text).unwrap();
                    if let Ok(val) = cap[1].parse::<i64>()
                        && is_valid_line_number(val)
                    {
                        line.line_number = Some(val);
                        line.text = line.text[m.end()..].to_string();
                    }
                }
            }

            Some(line)
        })
        .collect()
}
