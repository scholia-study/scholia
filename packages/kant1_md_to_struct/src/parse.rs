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
    Footnote { marker: String },
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
            label = Some(
                val.trim_matches('"')
                    .replace("\\\"", "\"")
                    .to_string(),
            );
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

        // Paragraph: collect lines until blank line or footnote
        let mut para_lines = vec![line.to_string()];
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
            para_lines.push(next.to_string());
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
        assert!(matches!(&blocks[3].block_type, ParsedBlockType::Footnote { marker } if marker == "*"));
    }

    #[test]
    fn test_footnote_parse() {
        let line = "[^**]: Some footnote text here.";
        let (marker, text) = try_parse_footnote_start(line).unwrap();
        assert_eq!(marker, "**");
        assert_eq!(text, "Some footnote text here.");
    }
}
