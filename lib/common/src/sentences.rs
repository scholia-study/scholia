use regex::Regex;
use std::sync::LazyLock;

/// Regex matching a sentence-ending punctuation followed by whitespace and an uppercase letter or ».
/// We capture the punctuation+space so we can check abbreviation context before accepting the split.
static SPLIT_RE: LazyLock<Regex> = LazyLock::new(|| {
    Regex::new(r"[.!?][)»\u{201c}\u{201d}\u{201e}\u{201f}]*\s+(?:[A-ZÄÖÜ»\u{201e}(])").unwrap()
});

/// Known German abbreviations that should NOT trigger a sentence split.
/// We store them in two groups:
/// - Single-word abbreviations: checked against the end of preceding text
/// - Multi-word abbreviations: checked by looking at a window around the split point
static SINGLE_ABBREVS: LazyLock<Vec<&'static str>> = LazyLock::new(|| {
    vec![
        "bzw.", "usf.", "usw.", "vgl.", "sog.", "evtl.", "bes.", "Anm.", "Bd.", "Kap.", "Nr.",
        "St.", "Dr.", "Fr.", "Hr.", "Prof.",
    ]
});

/// Multi-word abbreviation patterns. Each is a sequence of tokens (lowercase or uppercase)
/// that form an abbreviation when joined. We pre-compile regexes for these.
static MULTI_ABBREV_RE: LazyLock<Vec<Regex>> = LazyLock::new(|| {
    let patterns = vec![
        r"d\.\s*i\.",   // d. i.
        r"d\.\s*h\.",   // d. h.
        r"z\.\s*B\.",   // z. B.
        r"u\.\s*dgl\.", // u. dgl.
        r"u\.\s*a\.",   // u. a.
        r"a\.\s*a\.",   // a. a.
        r"u\.\s*ö\.",   // u. ö.
        r"s\.\s*o\.",   // s. o.
        r"s\.\s*u\.",   // s. u.
        r"o\.\s*ä\.",   // o. ä.
    ];
    patterns
        .into_iter()
        .map(|p| Regex::new(p).unwrap())
        .collect()
});

/// Single uppercase-letter initial pattern: detects "X. " where X is a single uppercase letter.
/// This handles initials like "G. W. F. Hegel".
static INITIAL_RE: LazyLock<Regex> = LazyLock::new(|| Regex::new(r"\b[A-ZÄÖÜ]\.\s*$").unwrap());

/// Numbered label pattern: detects "1." "2." "12." at the start of text or after whitespace.
/// These are paragraph numbering markers, not sentence endings.
static NUMBERED_LABEL_RE: LazyLock<Regex> =
    LazyLock::new(|| Regex::new(r"(?:^|\s)\d+\.\s*$").unwrap());

/// Split a paragraph into sentences, returning (text, html) pairs.
///
/// The algorithm:
/// 1. Find candidate split positions in the plain text
/// 2. Filter out false positives (abbreviations, initials)
/// 3. Map text split positions to HTML positions (walking both in parallel)
/// 4. Re-balance open inline tags at each split boundary
pub fn split_sentences(text: &str, html: &str) -> Vec<(String, String)> {
    if text.is_empty() {
        return vec![];
    }

    // Step 1: Find split positions in plain text.
    // A split position is the byte index where a new sentence starts
    // (i.e., the first character of the new sentence).
    let split_positions = find_text_split_positions(text);

    if split_positions.is_empty() {
        return vec![(text.to_string(), html.to_string())];
    }

    // Step 2: Split the plain text
    let text_parts = split_at_positions(text, &split_positions);

    // Step 3: Map text split positions to HTML positions
    let html_split_positions = map_text_positions_to_html(text, html, &split_positions);

    // Step 4: Split and re-balance HTML
    let html_parts = split_html_with_rebalance(html, &html_split_positions);

    assert_eq!(text_parts.len(), html_parts.len());

    text_parts
        .into_iter()
        .zip(html_parts)
        .map(|(t, h)| (t.trim().to_string(), h.trim().to_string()))
        .filter(|(t, _)| !t.is_empty())
        .collect()
}

/// Find byte positions in `text` where new sentences begin.
fn find_text_split_positions(text: &str) -> Vec<usize> {
    let mut positions = Vec::new();

    for m in SPLIT_RE.find_iter(text) {
        let match_start = m.start();

        // The match includes the punctuation, whitespace, and the first char of new sentence.
        // We want to find where the whitespace ends (= start of new sentence).
        let matched = m.as_str();
        let ws_start = matched.find(|c: char| c.is_whitespace()).unwrap();
        let after_ws = matched[ws_start..]
            .find(|c: char| !c.is_whitespace())
            .unwrap();
        let split_pos = match_start + ws_start + after_ws;

        // Check if this is a false positive due to abbreviation
        let preceding = &text[..match_start + ws_start];
        if is_single_abbreviation(preceding) {
            continue;
        }

        // Check for multi-word abbreviations spanning the split point
        if is_multi_word_abbreviation(text, match_start) {
            continue;
        }

        // Check for single-letter initials
        if INITIAL_RE.is_match(preceding) {
            continue;
        }

        // Check for numbered paragraph labels (e.g. "1." "2." "12.")
        if NUMBERED_LABEL_RE.is_match(preceding) {
            continue;
        }

        positions.push(split_pos);
    }

    positions
}

/// Check if the text preceding a split ends with a single-word abbreviation.
fn is_single_abbreviation(preceding: &str) -> bool {
    let trimmed = preceding.trim_end();
    for abbrev in SINGLE_ABBREVS.iter() {
        if trimmed.ends_with(abbrev) {
            return true;
        }
    }
    false
}

/// Check if the candidate split point falls within a multi-word abbreviation.
/// We look at a window around the split point in the full text.
fn is_multi_word_abbreviation(text: &str, match_start: usize) -> bool {
    // Look at a window: up to 10 bytes before and 15 bytes after the match start
    // Ensure we land on char boundaries
    let mut window_start = match_start.saturating_sub(10);
    while window_start > 0 && !text.is_char_boundary(window_start) {
        window_start -= 1;
    }
    let mut window_end = (match_start + 15).min(text.len());
    while window_end < text.len() && !text.is_char_boundary(window_end) {
        window_end += 1;
    }
    let window = &text[window_start..window_end];

    for re in MULTI_ABBREV_RE.iter() {
        if let Some(m) = re.find(window) {
            // The abbreviation match's position in the original text
            let abbrev_start = window_start + m.start();
            let abbrev_end = window_start + m.end();
            // The split candidate (period) must be inside the abbreviation
            if match_start >= abbrev_start && match_start < abbrev_end {
                return true;
            }
        }
    }
    false
}

/// Split text at the given byte positions.
fn split_at_positions(text: &str, positions: &[usize]) -> Vec<String> {
    let mut parts = Vec::new();
    let mut start = 0;
    for &pos in positions {
        parts.push(text[start..pos].to_string());
        start = pos;
    }
    parts.push(text[start..].to_string());
    parts
}

/// Map text byte positions to corresponding HTML byte positions.
///
/// Walk both strings in parallel. The HTML string may contain tags like `<i>`, `</i>`, etc.
/// We advance the HTML cursor past tags to stay synchronized with the text cursor.
fn map_text_positions_to_html(text: &str, html: &str, text_positions: &[usize]) -> Vec<usize> {
    let text_bytes = text.as_bytes();
    let html_bytes = html.as_bytes();
    let mut html_positions = Vec::new();

    let mut text_cursor = 0usize;
    let mut html_cursor = 0usize;
    let mut pos_idx = 0;

    while pos_idx < text_positions.len() && html_cursor < html_bytes.len() {
        // If we've reached the text position we're looking for, record the html position
        if text_cursor == text_positions[pos_idx] {
            html_positions.push(html_cursor);
            pos_idx += 1;
            continue;
        }

        if html_cursor < html_bytes.len() && html_bytes[html_cursor] == b'<' {
            // Skip the entire tag
            while html_cursor < html_bytes.len() && html_bytes[html_cursor] != b'>' {
                html_cursor += 1;
            }
            if html_cursor < html_bytes.len() {
                html_cursor += 1; // skip '>'
            }
        } else if html_cursor < html_bytes.len() && text_cursor < text_bytes.len() {
            // Advance both cursors by one character (handling UTF-8)
            let text_char_len = char_len_at(text_bytes, text_cursor);
            let html_char_len = char_len_at(html_bytes, html_cursor);
            text_cursor += text_char_len;
            html_cursor += html_char_len;
        } else {
            break;
        }
    }

    // If we still have positions to map, they're at the end
    while pos_idx < text_positions.len() {
        html_positions.push(html_bytes.len());
        pos_idx += 1;
    }

    html_positions
}

/// Get the byte length of the UTF-8 character at the given position.
fn char_len_at(bytes: &[u8], pos: usize) -> usize {
    if pos >= bytes.len() {
        return 1;
    }
    let b = bytes[pos];
    if b < 0x80 {
        1
    } else if b < 0xE0 {
        2
    } else if b < 0xF0 {
        3
    } else {
        4
    }
}

/// Split HTML at the given positions, re-balancing open inline tags at each boundary.
fn split_html_with_rebalance(html: &str, positions: &[usize]) -> Vec<String> {
    if positions.is_empty() {
        return vec![html.to_string()];
    }

    let mut parts = Vec::new();
    let mut open_tags: Vec<String> = Vec::new();
    let mut start = 0;

    for &pos in positions {
        let segment = &html[start..pos];

        // Track tags within this segment to update open_tags state
        let mut local_open = open_tags.clone();
        track_tags_in_segment(segment, &mut local_open);

        // Build the output: reopen tags at start, close at end
        let mut out = String::new();
        for tag in &open_tags {
            out.push('<');
            out.push_str(tag);
            out.push('>');
        }
        out.push_str(segment);
        // Trim trailing whitespace before adding closing tags
        if !local_open.is_empty() {
            let trimmed_len = out.trim_end().len();
            out.truncate(trimmed_len);
        }
        // Close any tags that are still open at the end of this sentence
        for tag in local_open.iter().rev() {
            out.push_str("</");
            out.push_str(tag);
            out.push('>');
        }

        parts.push(out);
        open_tags = local_open;
        start = pos;
    }

    // Last segment
    let segment = &html[start..];
    let mut out = String::new();
    for tag in &open_tags {
        out.push('<');
        out.push_str(tag);
        out.push('>');
    }
    out.push_str(segment);
    parts.push(out);

    parts
}

/// Track opening and closing tags in a segment, updating the open tags stack.
fn track_tags_in_segment(segment: &str, open_tags: &mut Vec<String>) {
    let bytes = segment.as_bytes();
    let mut i = 0;

    while i < bytes.len() {
        if bytes[i] == b'<' {
            let tag_start = i;
            i += 1;

            let is_closing = i < bytes.len() && bytes[i] == b'/';
            if is_closing {
                i += 1;
            }

            // Read tag name
            let name_start = i;
            while i < bytes.len() && bytes[i] != b'>' && bytes[i] != b' ' {
                i += 1;
            }
            let tag_name = &segment[name_start..i];

            // Skip to end of tag
            while i < bytes.len() && bytes[i] != b'>' {
                i += 1;
            }
            if i < bytes.len() {
                i += 1; // skip '>'
            }

            // Only track inline formatting tags
            if matches!(tag_name, "i" | "b" | "sup" | "sub") {
                if is_closing {
                    // Remove the last matching open tag
                    if let Some(pos) = open_tags.iter().rposition(|t| t == tag_name) {
                        open_tags.remove(pos);
                    }
                } else {
                    // Check it's not a self-closing tag
                    let tag_content = &segment[tag_start..i.min(segment.len())];
                    if !tag_content.ends_with("/>") {
                        open_tags.push(tag_name.to_string());
                    }
                }
            }
        } else {
            i += 1;
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_single_sentence() {
        let text = "Das reine Sein und das reine Nichts ist also dasselbe.";
        let html = "Das reine Sein und das reine Nichts ist also dasselbe.";
        let result = split_sentences(text, html);
        assert_eq!(result.len(), 1);
        assert_eq!(result[0].0, text);
    }

    #[test]
    fn test_two_sentences() {
        let text = "Erster Satz. Zweiter Satz.";
        let html = "Erster Satz. Zweiter Satz.";
        let result = split_sentences(text, html);
        assert_eq!(result.len(), 2);
        assert_eq!(result[0].0, "Erster Satz.");
        assert_eq!(result[1].0, "Zweiter Satz.");
    }

    #[test]
    fn test_abbreviation_not_split() {
        let text = "Es ist d. i. dasselbe. Nächster Satz.";
        let html = "Es ist d. i. dasselbe. Nächster Satz.";
        let result = split_sentences(text, html);
        assert_eq!(result.len(), 2);
        assert_eq!(result[0].0, "Es ist d. i. dasselbe.");
        assert_eq!(result[1].0, "Nächster Satz.");
    }

    #[test]
    fn test_abbreviation_zb() {
        let text = "Etwas z. B. ist hier. Dann weiter.";
        let html = "Etwas z. B. ist hier. Dann weiter.";
        let result = split_sentences(text, html);
        assert_eq!(result.len(), 2);
        assert_eq!(result[0].0, "Etwas z. B. ist hier.");
    }

    #[test]
    fn test_initials_not_split() {
        let text = "G. W. F. Hegel schrieb dies.";
        let html = "G. W. F. Hegel schrieb dies.";
        let result = split_sentences(text, html);
        assert_eq!(result.len(), 1);
    }

    #[test]
    fn test_html_rebalance_italics() {
        let text = "Erster Satz hier. Zweiter Satz dort.";
        let html = "<i>Erster Satz</i> hier. Zweiter Satz dort.";
        let result = split_sentences(text, html);
        assert_eq!(result.len(), 2);
        assert_eq!(result[0].1, "<i>Erster Satz</i> hier.");
        assert_eq!(result[1].1, "Zweiter Satz dort.");
    }

    #[test]
    fn test_html_rebalance_italics_crossing_boundary() {
        let text = "Erster Satz. Zweiter Satz.";
        let html = "<i>Erster Satz. Zweiter Satz.</i>";
        let result = split_sentences(text, html);
        assert_eq!(result.len(), 2);
        assert_eq!(result[0].1, "<i>Erster Satz.</i>");
        assert_eq!(result[1].1, "<i>Zweiter Satz.</i>");
    }

    #[test]
    fn test_question_mark_split() {
        let text = "Was ist das? Es ist nichts.";
        let html = "Was ist das? Es ist nichts.";
        let result = split_sentences(text, html);
        assert_eq!(result.len(), 2);
        assert_eq!(result[0].0, "Was ist das?");
        assert_eq!(result[1].0, "Es ist nichts.");
    }

    #[test]
    fn test_exclamation_mark_split() {
        let text = "Nein! Das ist falsch.";
        let html = "Nein! Das ist falsch.";
        let result = split_sentences(text, html);
        assert_eq!(result.len(), 2);
    }

    #[test]
    fn test_numbered_label_not_split() {
        let text = "1. Der mathematische Schluß heißt etwas.";
        let html = "1. Der mathematische Schluß heißt etwas.";
        let result = split_sentences(text, html);
        assert_eq!(result.len(), 1);
        assert_eq!(result[0].0, text);
    }

    #[test]
    fn test_numbered_label_multidigit() {
        let text = "12. Nächster Abschnitt hier.";
        let html = "12. Nächster Abschnitt hier.";
        let result = split_sentences(text, html);
        assert_eq!(result.len(), 1);
    }

    #[test]
    fn test_numbered_label_with_later_split() {
        let text = "1. Erster Satz hier. Zweiter Satz dort.";
        let html = "1. Erster Satz hier. Zweiter Satz dort.";
        let result = split_sentences(text, html);
        assert_eq!(result.len(), 2);
        assert_eq!(result[0].0, "1. Erster Satz hier.");
        assert_eq!(result[1].0, "Zweiter Satz dort.");
    }

    #[test]
    fn test_empty_text() {
        let result = split_sentences("", "");
        assert_eq!(result.len(), 0);
    }
}
