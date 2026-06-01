use common::sentences::RUN_BREAK;
use regex::Regex;
use std::sync::LazyLock;

// {{{ N }}} — AA page marker
static AA_MARKER_RE: LazyLock<Regex> =
    LazyLock::new(|| Regex::new(r"\{\{\{\s*(\d+)\s*\}\}\}").unwrap());

// {{ VALUE }} — B-edition page marker
static B_MARKER_RE: LazyLock<Regex> =
    LazyLock::new(|| Regex::new(r"\{\{\s*([^}]+?)\s*\}\}").unwrap());

// Combined: either marker type
static ANY_MARKER_RE: LazyLock<Regex> =
    LazyLock::new(|| Regex::new(r"\{\{\{?\s*[^}]+?\s*\}?\}\}").unwrap());

// <figcaption>…</figcaption> — the editorial label inside a figure block.
// `(?s)` lets `.` span newlines so multi-line captions match.
static FIGCAPTION_RE: LazyLock<Regex> =
    LazyLock::new(|| Regex::new(r"(?s)<figcaption[^>]*>(.*?)</figcaption>").unwrap());

// Any HTML tag — used to flatten figure markup to plain text.
static HTML_TAG_RE: LazyLock<Regex> = LazyLock::new(|| Regex::new(r"<[^>]+>").unwrap());

/// YAML front matter parsed from a reviewed markdown file.
#[derive(Debug)]
pub struct FrontMatter {
    pub position: usize,
    pub label: String,
    pub depth: u16,
    pub aa_page: u16,
}

/// A raw page marker found in the text.
#[derive(Debug, Clone)]
pub struct RawMarker {
    pub kind: MarkerKind,
    pub value: String,
    /// Char offset in the stripped text where this marker appeared.
    pub char_offset: usize,
}

#[derive(Debug, Clone, PartialEq)]
pub enum MarkerKind {
    Aa,
    BEdition,
}

/// A parsed block from the markdown content.
#[derive(Debug)]
pub struct ParsedBlock {
    pub block_type: ParsedBlockType,
    pub text: String,
    pub markers: Vec<RawMarker>,
}

#[derive(Debug, PartialEq)]
pub enum ParsedBlockType {
    Heading,
    Paragraph,
    Footnote {
        marker: String,
    },
    /// A diagram-like insertion (e.g. Kant's table of judgments) authored
    /// as verbatim `<figure>` HTML. Its `text` is the raw figure markup
    /// (page markers stripped); rendering and sentence-splitting bypass the
    /// markdown pipeline entirely. See `figure_caption` for the label.
    Figure,
    /// A thematic break authored as a lone token line. `---` is a plain
    /// horizontal rule; `***` is a centered, bold "* * *" ornament (a
    /// dinkus). Carries no text and produces no sentences — the variant is
    /// all that survives, recorded in `dinkus`. Front-matter `---` never
    /// reaches here, as it is consumed before `parse_blocks` runs.
    Separator {
        dinkus: bool,
    },
}

/// Flatten HTML markup to plain text by removing tags and collapsing
/// whitespace. Used to derive a figure's full-text-search content and its
/// caption label.
pub fn strip_html_tags(html: &str) -> String {
    let no_tags = HTML_TAG_RE.replace_all(html, " ");
    no_tags.split_whitespace().collect::<Vec<_>>().join(" ")
}

/// Extract the plain-text label from a `<figure>`'s `<figcaption>`.
/// Returns `None` when the figure has no caption — callers treat that as a
/// hard error, since every figure needs a label for quotation/search/anchoring.
pub fn figure_caption(figure_html: &str) -> Option<String> {
    let caps = FIGCAPTION_RE.captures(figure_html)?;
    let plain = strip_html_tags(&caps[1]);
    if plain.is_empty() { None } else { Some(plain) }
}

/// Inject a bold catalogue prefix at the start of a figure's `<figcaption>`
/// content, e.g. `<figcaption>Table of Judgments</figcaption>` with prefix
/// "Figure 1." becomes `<figcaption><b>Figure 1.</b> Table of Judgments</figcaption>`.
/// Language-specific wording is the caller's responsibility.
pub fn prepend_figcaption_label(figure_html: &str, prefix: &str) -> String {
    FIGCAPTION_RE
        .replace(figure_html, |caps: &regex::Captures| {
            format!("<figcaption><b>{prefix}</b> {}</figcaption>", &caps[1])
        })
        .into_owned()
}

/// Parse YAML front matter from between `---` markers.
pub fn parse_front_matter(content: &str) -> Option<(FrontMatter, &str)> {
    let content = content.trim_start();
    if !content.starts_with("---") {
        return None;
    }
    let after_first = &content[3..];
    let end = after_first.find("---")?;
    let yaml_block = &after_first[..end];
    let rest = &after_first[end + 3..];
    // Skip leading newline after closing ---
    let rest = rest.strip_prefix('\n').unwrap_or(rest);

    let mut position = None;
    let mut label = None;
    let mut depth = None;
    let mut aa_page = None;

    for line in yaml_block.lines() {
        let line = line.trim();
        if let Some(val) = line.strip_prefix("position:") {
            position = val.trim().parse().ok();
        } else if let Some(val) = line.strip_prefix("label:") {
            let val = val.trim();
            // Strip surrounding quotes
            label = Some(val.trim_matches('"').replace("\\\"", "\"").to_string());
        } else if let Some(val) = line.strip_prefix("depth:") {
            depth = val.trim().parse().ok();
        } else if let Some(val) = line.strip_prefix("aa_page:") {
            aa_page = val.trim().parse().ok();
        }
    }

    Some((
        FrontMatter {
            position: position?,
            label: label?,
            depth: depth?,
            aa_page: aa_page?,
        },
        rest,
    ))
}

/// Strip all page markers from text, returning (stripped_text, markers).
///
/// Markers are recorded with char_offset relative to the stripped text.
/// Also removes one trailing space after each marker to avoid double spaces.
pub fn strip_markers(text: &str) -> (String, Vec<RawMarker>) {
    let mut markers = Vec::new();
    let mut stripped = String::with_capacity(text.len());
    let mut last_end = 0;

    for m in ANY_MARKER_RE.find_iter(text) {
        let before = &text[last_end..m.start()];
        stripped.push_str(before);

        // Remove trailing space after marker if present
        let after_match_end = m.end();
        let next_is_space = text[after_match_end..].starts_with(' ');

        let char_offset = stripped.chars().count();
        let matched = m.as_str();

        if let Some(caps) = AA_MARKER_RE.captures(matched) {
            markers.push(RawMarker {
                kind: MarkerKind::Aa,
                value: caps[1].to_string(),
                char_offset,
            });
        } else if let Some(caps) = B_MARKER_RE.captures(matched) {
            markers.push(RawMarker {
                kind: MarkerKind::BEdition,
                value: caps[1].to_string(),
                char_offset,
            });
        }

        if next_is_space {
            last_end = after_match_end + 1; // skip the trailing space
        } else {
            last_end = after_match_end;
        }
    }
    stripped.push_str(&text[last_end..]);

    (stripped, markers)
}

/// Parse the body content (after front matter) into blocks.
pub fn parse_blocks(body: &str) -> Vec<ParsedBlock> {
    let mut blocks = Vec::new();

    // Split into raw blocks by blank lines, but handle footnotes specially
    // Footnotes start with [^marker]: and may span multiple lines
    let lines: Vec<&str> = body.lines().collect();
    let mut i = 0;

    while i < lines.len() {
        let line = lines[i].trim();

        // Skip empty lines
        if line.is_empty() {
            i += 1;
            continue;
        }

        // Footnote definition: [^marker]: text
        if let Some(rest) = try_parse_footnote_start(line) {
            let (marker, first_line) = rest;
            let mut footnote_text = first_line.to_string();
            i += 1;

            // Continuation lines: not blank, not starting with [^
            while i < lines.len() {
                let next = lines[i];
                if next.trim().is_empty() {
                    break;
                }
                if try_parse_footnote_start(next.trim()).is_some() {
                    break;
                }
                footnote_text.push(' ');
                footnote_text.push_str(next.trim());
                i += 1;
            }

            let (stripped, markers) = strip_markers(&footnote_text);
            blocks.push(ParsedBlock {
                block_type: ParsedBlockType::Footnote {
                    marker: marker.to_string(),
                },
                text: stripped,
                markers,
            });
            continue;
        }

        // Figure: a verbatim `<figure>…</figure>` HTML block. Captured raw
        // (preserving inner lines) until the closing tag, then run through
        // strip_markers so any `{{{ N }}}` in the figcaption becomes a page
        // marker on the anchor. Detection bypasses the markdown pipeline so
        // the editor's hand-written HTML survives intact.
        if line.starts_with("<figure") {
            let mut fig_lines: Vec<&str> = Vec::new();
            while i < lines.len() {
                let raw = lines[i];
                fig_lines.push(raw);
                i += 1;
                if raw.contains("</figure>") {
                    break;
                }
            }
            let fig_text = fig_lines.join("\n");
            let (stripped, markers) = strip_markers(&fig_text);
            blocks.push(ParsedBlock {
                block_type: ParsedBlockType::Figure,
                text: stripped,
                markers,
            });
            continue;
        }

        // Thematic break: a lone `---` (plain rule) or `***` (centered
        // "* * *" ornament). Matched on the exact trimmed token so it can't
        // be confused with inline emphasis (`***word***` always has content
        // between the asterisks) — and front matter `---` is already gone.
        if line == "---" || line == "***" {
            blocks.push(ParsedBlock {
                block_type: ParsedBlockType::Separator {
                    dinkus: line == "***",
                },
                text: String::new(),
                markers: Vec::new(),
            });
            i += 1;
            continue;
        }

        // Heading: ## text
        if let Some(heading_text) = line.strip_prefix("## ") {
            let (stripped, markers) = strip_markers(heading_text);
            blocks.push(ParsedBlock {
                block_type: ParsedBlockType::Heading,
                text: stripped,
                markers,
            });
            i += 1;
            continue;
        }

        // Paragraph: collect lines until blank line or footnote. A line
        // prefixed with `+ ` opens an indented run (e.g. Kant's numbered
        // `1) 2) 3)` enumerations): the prefix is replaced by a RUN_BREAK
        // sentinel so the lines still join into one paragraph block, with the
        // run boundary recorded for sentence-splitting and tagging downstream.
        let mut para_lines = vec![apply_run_marker(line)];
        i += 1;
        while i < lines.len() {
            let next = lines[i].trim();
            if next.is_empty() {
                break;
            }
            if next.starts_with("## ") {
                break;
            }
            if try_parse_footnote_start(next).is_some() {
                break;
            }
            para_lines.push(apply_run_marker(next));
            i += 1;
        }

        let para_text = para_lines.join(" ");
        let (stripped, markers) = strip_markers(&para_text);
        blocks.push(ParsedBlock {
            block_type: ParsedBlockType::Paragraph,
            text: stripped,
            markers,
        });
    }

    blocks
}

/// Replace a leading `+ ` run-marker with a `RUN_BREAK` sentinel; pass other
/// lines through unchanged. The `+ ` is our authoring convention for an
/// indented run; the sentinel carries that intent into the joined paragraph
/// text without breaking the block apart.
fn apply_run_marker(line: &str) -> String {
    match line.strip_prefix("+ ") {
        Some(rest) => format!("{RUN_BREAK}{rest}"),
        None => line.to_string(),
    }
}

/// Try to parse a footnote definition start: `[^marker]: text`
/// Returns (marker, rest_of_line) if successful.
fn try_parse_footnote_start(line: &str) -> Option<(&str, &str)> {
    let rest = line.strip_prefix("[^")?;
    let end_bracket = rest.find("]:")?;
    let marker = &rest[..end_bracket];
    let text = rest[end_bracket + 2..].trim();
    Some((marker, text))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_front_matter() {
        let content = "---\nposition: 3\nlabel: \"Vorrede zur zweiten Auflage\"\ndepth: 1\naa_page: 7\n---\n\nBody text here.";
        let (fm, rest) = parse_front_matter(content).unwrap();
        assert_eq!(fm.position, 3);
        assert_eq!(fm.label, "Vorrede zur zweiten Auflage");
        assert_eq!(fm.depth, 1);
        assert_eq!(fm.aa_page, 7);
        assert!(rest.contains("Body text here."));
    }

    #[test]
    fn test_strip_markers() {
        let (text, markers) = strip_markers("{{{ 7 }}} {{ VII }} Vorrede zur zweiten Auflage");
        assert_eq!(text, "Vorrede zur zweiten Auflage");
        assert_eq!(markers.len(), 2);
        assert_eq!(markers[0].kind, MarkerKind::Aa);
        assert_eq!(markers[0].value, "7");
        assert_eq!(markers[1].kind, MarkerKind::BEdition);
        assert_eq!(markers[1].value, "VII");
    }

    #[test]
    fn test_strip_inline_b_marker() {
        let (text, markers) = strip_markers("text before {{ XIV }} text after");
        assert_eq!(text, "text before text after");
        assert_eq!(markers.len(), 1);
        assert_eq!(markers[0].kind, MarkerKind::BEdition);
        assert_eq!(markers[0].value, "XIV");
        assert_eq!(markers[0].char_offset, 12); // "text before " = 12 chars
    }

    #[test]
    fn test_parse_blocks() {
        let body = "## {{{ 7 }}} {{ VII }} Vorrede\n\nFirst paragraph.\n\nSecond paragraph.\n\n[^*]: Footnote text.\n";
        let blocks = parse_blocks(body);
        assert_eq!(blocks.len(), 4);
        assert_eq!(blocks[0].block_type, ParsedBlockType::Heading);
        assert_eq!(blocks[0].text, "Vorrede");
        assert_eq!(blocks[1].block_type, ParsedBlockType::Paragraph);
        assert_eq!(blocks[1].text, "First paragraph.");
        assert_eq!(blocks[2].block_type, ParsedBlockType::Paragraph);
        assert_eq!(blocks[2].text, "Second paragraph.");
        assert!(
            matches!(&blocks[3].block_type, ParsedBlockType::Footnote { marker } if marker == "*")
        );
    }

    #[test]
    fn test_parse_blocks_run_markers() {
        // `+ ` lines stay inside one paragraph block; each becomes a RUN_BREAK
        // sentinel in the joined text, leaving exactly one paragraph block.
        let body = "Intro flow.\n+ 1) First item.\n+ 2) Second item.\n";
        let blocks = parse_blocks(body);
        assert_eq!(blocks.len(), 1);
        assert_eq!(blocks[0].block_type, ParsedBlockType::Paragraph);
        assert_eq!(
            blocks[0].text,
            format!("Intro flow. {RUN_BREAK}1) First item. {RUN_BREAK}2) Second item.")
        );
    }

    #[test]
    fn test_parse_blocks_run_marker_at_start() {
        // A paragraph that opens directly with an item (no intro flow).
        let body = "+ 1) Only item.\n";
        let blocks = parse_blocks(body);
        assert_eq!(blocks.len(), 1);
        assert_eq!(blocks[0].text, format!("{RUN_BREAK}1) Only item."));
    }

    #[test]
    fn test_parse_separator_blocks() {
        // `---` between paragraphs is a plain rule; `***` is the dinkus
        // ornament. Both carry no text and sit as their own blocks.
        let body = "First paragraph.\n\n---\n\nSecond paragraph.\n\n***\n\nThird paragraph.\n";
        let blocks = parse_blocks(body);
        assert_eq!(blocks.len(), 5);

        assert_eq!(blocks[0].block_type, ParsedBlockType::Paragraph);
        assert_eq!(
            blocks[1].block_type,
            ParsedBlockType::Separator { dinkus: false }
        );
        assert!(blocks[1].text.is_empty());
        assert!(blocks[1].markers.is_empty());

        assert_eq!(blocks[2].block_type, ParsedBlockType::Paragraph);
        assert_eq!(
            blocks[3].block_type,
            ParsedBlockType::Separator { dinkus: true }
        );
        assert_eq!(blocks[4].block_type, ParsedBlockType::Paragraph);
        assert_eq!(blocks[4].text, "Third paragraph.");
    }

    #[test]
    fn test_inline_emphasis_is_not_a_separator() {
        // A paragraph that merely *contains* `***word***` Sperrdruck must
        // stay a paragraph — only a line that is exactly `***` breaks.
        let body = "This has ***emphasis*** in the middle.\n";
        let blocks = parse_blocks(body);
        assert_eq!(blocks.len(), 1);
        assert_eq!(blocks[0].block_type, ParsedBlockType::Paragraph);
        assert!(blocks[0].text.contains("***emphasis***"));
    }

    #[test]
    fn test_separator_after_heading() {
        let body = "## A Heading\n\n***\n\nBody.\n";
        let blocks = parse_blocks(body);
        assert_eq!(blocks.len(), 3);
        assert_eq!(blocks[0].block_type, ParsedBlockType::Heading);
        assert_eq!(
            blocks[1].block_type,
            ParsedBlockType::Separator { dinkus: true }
        );
        assert_eq!(blocks[2].block_type, ParsedBlockType::Paragraph);
    }

    #[test]
    fn test_footnote_parse() {
        let line = "[^**]: Some footnote text here.";
        let (marker, text) = try_parse_footnote_start(line).unwrap();
        assert_eq!(marker, "**");
        assert_eq!(text, "Some footnote text here.");
    }

    #[test]
    fn test_parse_figure_block() {
        let body = "First paragraph.\n\n<figure>\n  <figcaption>{{{ 87 }}} Tafel der Urtheile</figcaption>\n  <table><tr><td>Allgemeine</td></tr></table>\n</figure>\n\nAfter paragraph.\n";
        let blocks = parse_blocks(body);
        assert_eq!(blocks.len(), 3);
        assert_eq!(blocks[0].block_type, ParsedBlockType::Paragraph);
        assert_eq!(blocks[1].block_type, ParsedBlockType::Figure);
        assert_eq!(blocks[2].block_type, ParsedBlockType::Paragraph);
        assert_eq!(blocks[2].text, "After paragraph.");

        // The page marker is lifted out of the figcaption; the figure markup
        // is preserved verbatim (minus the marker).
        assert_eq!(blocks[1].markers.len(), 1);
        assert_eq!(blocks[1].markers[0].kind, MarkerKind::Aa);
        assert_eq!(blocks[1].markers[0].value, "87");
        assert!(blocks[1].text.contains("<figure>"));
        assert!(blocks[1].text.contains("</figure>"));
        assert!(blocks[1].text.contains("<table>"));
        assert!(!blocks[1].text.contains("{{{"));

        assert_eq!(
            figure_caption(&blocks[1].text).as_deref(),
            Some("Tafel der Urtheile")
        );
    }

    #[test]
    fn test_figure_caption_missing() {
        assert_eq!(figure_caption("<figure><table></table></figure>"), None);
    }
}
