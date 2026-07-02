//! Shared curated-markdown parsing utilities used by every genre parser:
//! front-matter extraction, curated-dir scanning, verse indent stripping, and
//! page-marker → sentence resolution. Genre grammars (verse stanzas, drama
//! speech runs, prose footnotes) stay in the genre crates; only the
//! genre-agnostic mechanics live here.

use std::fs;
use std::io;
use std::path::Path;

/// The `---`-fenced header every curated file starts with. `aa_page` is only
/// present in annotated-prose corpora (Kant); verse/drama files carry the
/// three-key form.
#[derive(Debug, Clone)]
pub struct FrontMatter {
    pub position: u32,
    pub label: String,
    pub depth: i16,
    pub aa_page: Option<u16>,
}

/// Parse the front matter, returning it and the body. BOM-tolerant; the label
/// is unquoted (`"…"` stripped, `\"` unescaped). Exactly one newline after the
/// closing `---` is consumed — any further body trimming (verse corpora also
/// drop leading/trailing blank lines) is the caller's business.
pub fn parse_front_matter(content: &str) -> Option<(FrontMatter, &str)> {
    let rest = content
        .trim_start_matches('\u{feff}')
        .trim_start()
        .strip_prefix("---")?;
    let close = rest.find("\n---")?;
    let fm_text = &rest[..close];
    let body = rest[close + "\n---".len()..]
        .strip_prefix('\n')
        .unwrap_or(&rest[close + "\n---".len()..]);

    let (mut position, mut label, mut depth, mut aa_page) = (None, None, None, None);
    for line in fm_text.lines() {
        let line = line.trim();
        if let Some(v) = line.strip_prefix("position:") {
            position = v.trim().parse().ok();
        } else if let Some(v) = line.strip_prefix("label:") {
            label = Some(v.trim().trim_matches('"').replace("\\\"", "\"").to_string());
        } else if let Some(v) = line.strip_prefix("depth:") {
            depth = v.trim().parse().ok();
        } else if let Some(v) = line.strip_prefix("aa_page:") {
            aa_page = v.trim().parse().ok();
        }
    }
    Some((
        FrontMatter {
            position: position?,
            label: label?,
            depth: depth?,
            aa_page,
        },
        body,
    ))
}

/// Content files in a curated layer dir: every `*.md` except the reviewer
/// convenience `000_toc.md`. Sorted for deterministic error reporting.
pub fn scan_md_files(dir: &Path) -> io::Result<Vec<String>> {
    let mut out: Vec<String> = fs::read_dir(dir)?
        .filter_map(|e| e.ok())
        .map(|e| e.file_name().to_string_lossy().to_string())
        .filter(|name| name.ends_with(".md") && name != "000_toc.md")
        .collect();
    out.sort();
    Ok(out)
}

/// Leading whitespace → indent level (2 spaces per level), and the de-indented
/// line. Flush lines yield `(None, line)`.
pub fn strip_indent(line: &str) -> (Option<i16>, String) {
    let trimmed = line.trim_start();
    let spaces = line.len() - trimmed.len();
    let indent = (spaces >= 2).then_some((spaces / 2) as i16);
    (indent, trimmed.trim_end().to_string())
}

/// Resolve a marker's char offset (into a block's joined plain text) to
/// `(sentence_index, char_offset_within_sentence)`. `cumulative_chars[i]` is the
/// char offset at the start of sentence `i` in that same joined text.
pub fn resolve_marker_to_sentence(
    cumulative_chars: &[usize],
    marker_offset: usize,
) -> (usize, i32) {
    for i in (0..cumulative_chars.len()).rev() {
        if marker_offset >= cumulative_chars[i] {
            return (i, (marker_offset - cumulative_chars[i]) as i32);
        }
    }
    (0, 0)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn front_matter_three_key() {
        let (fm, body) = parse_front_matter(
            "---\nposition: 3\nlabel: \"Første handling\"\ndepth: 1\n---\n\nBody",
        )
        .unwrap();
        assert_eq!(fm.position, 3);
        assert_eq!(fm.label, "Første handling");
        assert_eq!(fm.depth, 1);
        assert_eq!(fm.aa_page, None);
        // exactly one newline consumed; the blank line stays for the caller
        assert_eq!(body, "\nBody");
    }

    #[test]
    fn front_matter_with_aa_page_and_escaped_quote() {
        // Interior escaped quotes unescape; kant labels never *end* in one
        // (trailing `\""` would be eaten by the quote trim, a quirk kept
        // verbatim from the original kant parser).
        let (fm, _) = parse_front_matter(
            "---\nposition: 12\nlabel: \"Die \\\"Idee\\\" selbst\"\ndepth: 2\naa_page: 251\n---\nX",
        )
        .unwrap();
        assert_eq!(fm.label, "Die \"Idee\" selbst");
        assert_eq!(fm.aa_page, Some(251));
    }

    #[test]
    fn front_matter_bom_tolerant() {
        let (fm, _) =
            parse_front_matter("\u{feff}---\nposition: 1\nlabel: x\ndepth: 0\n---\nB").unwrap();
        assert_eq!(fm.position, 1);
    }

    #[test]
    fn resolves_to_owning_sentence() {
        // sentence 0 = chars 0..10, sentence 1 = chars 11..
        let cumulative = vec![0usize, 11usize];
        assert_eq!(resolve_marker_to_sentence(&cumulative, 0), (0, 0));
        assert_eq!(resolve_marker_to_sentence(&cumulative, 11), (1, 0));
        assert_eq!(resolve_marker_to_sentence(&cumulative, 14), (1, 3));
    }

    #[test]
    fn indent_levels() {
        assert_eq!(strip_indent("flush line"), (None, "flush line".into()));
        assert_eq!(strip_indent("  one level"), (Some(1), "one level".into()));
        assert_eq!(strip_indent("    two  "), (Some(2), "two".into()));
    }
}
