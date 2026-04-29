use regex::Regex;
use std::sync::LazyLock;

/// Strip `|||` markers from text, returning (cleaned text, byte positions of forced splits).
/// Each position is the byte offset in the cleaned text where a new sentence should begin.
pub fn strip_forced_splits(text: &str) -> (String, Vec<usize>) {
    let mut cleaned = String::with_capacity(text.len());
    let mut positions = Vec::new();
    let mut last_end = 0;

    for (idx, _) in text.match_indices("|||") {
        cleaned.push_str(&text[last_end..idx]);
        positions.push(cleaned.len());
        last_end = idx + 3;
    }
    cleaned.push_str(&text[last_end..]);
    (cleaned, positions)
}

/// Strip `|||` markers from text without tracking positions.
pub fn strip_forced_split_markers(text: &str) -> String {
    text.replace("|||", "")
}

/// Regex matching a sentence-ending punctuation followed by whitespace and an uppercase letter or ».
/// We capture the punctuation+space so we can check abbreviation context before accepting the split.
static SPLIT_RE: LazyLock<Regex> = LazyLock::new(|| {
    Regex::new(r#"[.!?][)»""\u{201c}\u{201d}\u{201e}\u{201f}]*\s+(?:[A-ZÄÖÜ»\u{201e}(])"#).unwrap()
});

/// Known German abbreviations that should NOT trigger a sentence split.
/// We store them in two groups:
/// - Single-word abbreviations: checked against the end of preceding text
/// - Multi-word abbreviations: checked by looking at a window around the split point
static SINGLE_ABBREVS: LazyLock<Vec<&'static str>> = LazyLock::new(|| {
    vec![
        "bzw.",
        "usf.",
        "usw.",
        "vgl.",
        "sog.",
        "evtl.",
        "bes.",
        "Anm.",
        "Bd.",
        "Kap.",
        "Nr.",
        "St.",
        "Dr.",
        "Fr.",
        "Hr.",
        "Prof.",
        // Honorific/title abbreviations common in older German texts
        "Sr.",
        "Ew.",
        "Königl.",
        "Hochfürstl.",
        "Hochgräfl.",
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
    LazyLock::new(|| Regex::new(r"^\d{1,2}\.\s*$").unwrap());

/// Split a paragraph into sentences, returning (text, html) pairs.
///
/// The algorithm:
/// 1. Find candidate split positions in the plain text
/// 2. Filter out false positives (abbreviations, initials)
/// 3. Map text split positions to HTML positions (walking both in parallel)
/// 4. Re-balance open inline tags at each split boundary
pub fn split_sentences(text: &str, html: &str) -> Vec<(String, String)> {
    split_sentences_forced(text, html, &[])
}

/// Split a paragraph into sentences with additional forced split positions.
///
/// `forced` contains byte offsets in `text` where sentence boundaries should be inserted
/// regardless of punctuation.
pub fn split_sentences_forced(text: &str, html: &str, forced: &[usize]) -> Vec<(String, String)> {
    if text.is_empty() {
        return vec![];
    }

    // Step 1: Find split positions in plain text, merge with forced positions.
    let mut split_positions = find_text_split_positions(text);
    split_positions.extend_from_slice(forced);
    split_positions.sort_unstable();
    split_positions.dedup();

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

/// Find byte positions in `text` where new sentences begin (German).
fn find_text_split_positions(text: &str) -> Vec<usize> {
    find_text_split_positions_with(text, &SINGLE_ABBREVS, &MULTI_ABBREV_RE)
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
            out.push_str(tag_name_of(tag));
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
/// Each entry in open_tags is the full tag content (e.g. `span class="antiqua"`)
/// so it can be reopened with attributes intact.
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
            if matches!(tag_name, "i" | "b" | "sup" | "sub" | "span") {
                if is_closing {
                    // Remove the last matching open tag (match by tag name prefix)
                    if let Some(pos) = open_tags
                        .iter()
                        .rposition(|t| t == tag_name || t.starts_with(&format!("{tag_name} ")))
                    {
                        open_tags.remove(pos);
                    }
                } else {
                    // Check it's not a self-closing tag
                    let tag_content = &segment[tag_start..i.min(segment.len())];
                    if !tag_content.ends_with("/>") {
                        // Store full tag content (between < and >) for reopening
                        let full_tag = &segment[tag_start + 1..i - 1];
                        open_tags.push(full_tag.to_string());
                    }
                }
            }
        } else {
            i += 1;
        }
    }
}

/// Extract the tag name from a full tag string (e.g. `span class="x"` → `span`).
fn tag_name_of(full_tag: &str) -> &str {
    full_tag.split_once(' ').map_or(full_tag, |(name, _)| name)
}

// ---------------------------------------------------------------------------
// English sentence splitting
// ---------------------------------------------------------------------------

/// English single-word abbreviations that should NOT trigger a sentence split.
static SINGLE_ABBREVS_EN: LazyLock<Vec<&'static str>> = LazyLock::new(|| {
    vec![
        "Mr.", "Mrs.", "Ms.", "Dr.", "Prof.", "Rev.", "St.", "Jr.", "Sr.", "vs.", "Vol.", "No.",
        "Gen.", "Gov.", "Sgt.", "Corp.", "Inc.", "Ltd.", "Jan.", "Feb.", "Mar.", "Apr.", "Aug.",
        "Sept.", "Oct.", "Nov.", "Dec.",
    ]
});

/// English multi-word abbreviation patterns.
static MULTI_ABBREV_RE_EN: LazyLock<Vec<Regex>> = LazyLock::new(|| {
    let patterns = vec![
        r"e\.?\s*g\.", // e.g.
        r"i\.?\s*e\.", // i.e.
        r"c\.?\s*f\.", // c.f.
    ];
    patterns
        .into_iter()
        .map(|p| Regex::new(p).unwrap())
        .collect()
});

/// Split English text into sentences, returning (text, html) pairs.
///
/// Uses English-specific abbreviation lists but the same splitting algorithm.
pub fn split_sentences_en(text: &str, html: &str) -> Vec<(String, String)> {
    split_sentences_en_forced(text, html, &[])
}

/// Split English text into sentences with additional forced split positions.
pub fn split_sentences_en_forced(
    text: &str,
    html: &str,
    forced: &[usize],
) -> Vec<(String, String)> {
    if text.is_empty() {
        return vec![];
    }

    let mut split_positions =
        find_text_split_positions_with(text, &SINGLE_ABBREVS_EN, &MULTI_ABBREV_RE_EN);
    split_positions.extend_from_slice(forced);
    split_positions.sort_unstable();
    split_positions.dedup();

    if split_positions.is_empty() {
        return vec![(text.to_string(), html.to_string())];
    }

    let text_parts = split_at_positions(text, &split_positions);
    let html_split_positions = map_text_positions_to_html(text, html, &split_positions);
    let html_parts = split_html_with_rebalance(html, &html_split_positions);

    assert_eq!(text_parts.len(), html_parts.len());

    text_parts
        .into_iter()
        .zip(html_parts)
        .map(|(t, h)| (t.trim().to_string(), h.trim().to_string()))
        .filter(|(t, _)| !t.is_empty())
        .collect()
}

/// Generalized split-position finder parameterized by abbreviation lists.
fn find_text_split_positions_with(
    text: &str,
    single_abbrevs: &[&str],
    multi_abbrev_res: &[Regex],
) -> Vec<usize> {
    let mut positions = Vec::new();

    for m in SPLIT_RE.find_iter(text) {
        let match_start = m.start();

        let matched = m.as_str();
        let ws_start = matched.find(|c: char| c.is_whitespace()).unwrap();
        let after_ws = matched[ws_start..]
            .find(|c: char| !c.is_whitespace())
            .unwrap();
        let split_pos = match_start + ws_start + after_ws;

        let preceding = &text[..match_start + ws_start];

        // Check single-word abbreviations
        let trimmed = preceding.trim_end();
        let is_single = single_abbrevs
            .iter()
            .any(|abbrev| trimmed.ends_with(abbrev));
        if is_single {
            continue;
        }

        // Check multi-word abbreviations
        if is_multi_word_abbreviation_with(text, match_start, multi_abbrev_res) {
            continue;
        }

        // Check single-letter initials
        if INITIAL_RE.is_match(preceding) {
            continue;
        }

        // Check numbered paragraph labels
        if NUMBERED_LABEL_RE.is_match(preceding) {
            continue;
        }

        positions.push(split_pos);
    }

    positions
}

/// Check multi-word abbreviations with a given set of patterns.
fn is_multi_word_abbreviation_with(text: &str, match_start: usize, patterns: &[Regex]) -> bool {
    let mut window_start = match_start.saturating_sub(10);
    while window_start > 0 && !text.is_char_boundary(window_start) {
        window_start -= 1;
    }
    let mut window_end = (match_start + 15).min(text.len());
    while window_end < text.len() && !text.is_char_boundary(window_end) {
        window_end += 1;
    }
    let window = &text[window_start..window_end];

    for re in patterns {
        if let Some(m) = re.find(window) {
            let abbrev_start = window_start + m.start();
            let abbrev_end = window_start + m.end();
            if match_start >= abbrev_start && match_start < abbrev_end {
                return true;
            }
        }
    }
    false
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
    fn test_honorific_abbreviations_not_split() {
        let text = "Sr. Excellenz, dem Königl. Staatsminister Freiherrn von Zedlitz.";
        let result = split_sentences(text, text);
        assert_eq!(result.len(), 1);

        let text2 = "heißt an Ew. Excellenz eigenem Interesse arbeiten.";
        let result2 = split_sentences(text2, text2);
        assert_eq!(result2.len(), 1);
    }

    #[test]
    fn test_html_rebalance_span_with_class() {
        let text = "Erster Satz. Zweiter Satz.";
        let html = "<span class=\"antiqua\">Erster Satz. Zweiter Satz.</span>";
        let result = split_sentences(text, html);
        assert_eq!(result.len(), 2);
        assert_eq!(result[0].1, "<span class=\"antiqua\">Erster Satz.</span>");
        assert_eq!(result[1].1, "<span class=\"antiqua\">Zweiter Satz.</span>");
    }

    #[test]
    fn test_empty_text() {
        let result = split_sentences("", "");
        assert_eq!(result.len(), 0);
    }

    // === English sentence splitting tests ===

    #[test]
    fn test_en_two_sentences() {
        let text = "First sentence. Second sentence.";
        let result = split_sentences_en(text, text);
        assert_eq!(result.len(), 2);
        assert_eq!(result[0].0, "First sentence.");
        assert_eq!(result[1].0, "Second sentence.");
    }

    #[test]
    fn test_en_eg_not_split() {
        let text = "For example, e.g. this case. Next sentence.";
        let result = split_sentences_en(text, text);
        assert_eq!(result.len(), 2);
        assert_eq!(result[0].0, "For example, e.g. this case.");
    }

    #[test]
    fn test_en_ie_not_split() {
        let text = "That is, i.e. the thing. Another sentence.";
        let result = split_sentences_en(text, text);
        assert_eq!(result.len(), 2);
        assert_eq!(result[0].0, "That is, i.e. the thing.");
    }

    #[test]
    fn test_en_dr_not_split() {
        let text = "Dr. Smith arrived. He sat down.";
        let result = split_sentences_en(text, text);
        assert_eq!(result.len(), 2);
        assert_eq!(result[0].0, "Dr. Smith arrived.");
    }

    #[test]
    fn test_en_mr_mrs_not_split() {
        let text = "Mr. Jones and Mrs. Smith agree.";
        let result = split_sentences_en(text, text);
        assert_eq!(result.len(), 1);
    }

    #[test]
    fn test_en_single_sentence() {
        let text = "This is a single sentence about reason.";
        let result = split_sentences_en(text, text);
        assert_eq!(result.len(), 1);
    }

    #[test]
    fn test_en_empty() {
        let result = split_sentences_en("", "");
        assert_eq!(result.len(), 0);
    }

    #[test]
    fn test_en_page_ref_not_suppressed() {
        let text = "proof of the objective reality of outer intuition p. 275. However innocent idealism may be.";
        let result = split_sentences_en(text, text);
        assert_eq!(result.len(), 2);
        assert_eq!(
            result[0].0,
            "proof of the objective reality of outer intuition p. 275."
        );
        assert_eq!(result[1].0, "However innocent idealism may be.");
    }

    #[test]
    fn test_en_split_after_closing_quote() {
        let text = r#"their change can be determined." Against this proof one will say something."#;
        let result = split_sentences_en(text, text);
        assert_eq!(result.len(), 2);
        assert_eq!(result[0].0, r#"their change can be determined.""#);
        assert_eq!(result[1].0, "Against this proof one will say something.");
    }

    // === Forced split tests ===

    #[test]
    fn test_strip_forced_splits() {
        let (text, positions) = strip_forced_splits("before:||| after text");
        assert_eq!(text, "before: after text");
        assert_eq!(positions, vec![7]); // byte offset of " after"
    }

    #[test]
    fn test_strip_forced_splits_multiple() {
        let (text, positions) = strip_forced_splits("a||| b||| c");
        assert_eq!(text, "a b c");
        assert_eq!(positions, vec![1, 3]);
    }

    #[test]
    fn test_strip_forced_splits_none() {
        let (text, positions) = strip_forced_splits("no markers here");
        assert_eq!(text, "no markers here");
        assert!(positions.is_empty());
    }

    #[test]
    fn test_strip_forced_split_markers() {
        assert_eq!(strip_forced_split_markers("a||| b||| c"), "a b c");
    }

    #[test]
    fn test_forced_split_de() {
        let text = "as follows: \"This permanent thing.\"";
        let html = "as follows: \"This permanent thing.\"";
        let result = split_sentences_forced(text, html, &[12]); // split before "This
        assert_eq!(result.len(), 2);
        assert_eq!(result[0].0, "as follows:");
        assert_eq!(result[1].0, "\"This permanent thing.\"");
    }

    #[test]
    fn test_forced_split_en() {
        let text = "altered as follows: \"This permanent, however, cannot be an intuition in me.\"";
        let html = "altered as follows: \"This permanent, however, cannot be an intuition in me.\"";
        let result = split_sentences_en_forced(text, html, &[20]); // split before the quote
        assert_eq!(result.len(), 2);
        assert_eq!(result[0].0, "altered as follows:");
        assert_eq!(
            result[1].0,
            "\"This permanent, however, cannot be an intuition in me.\""
        );
    }

    #[test]
    fn test_forced_split_combined_with_auto() {
        let text = "First part: Second part. Third sentence.";
        let result = split_sentences_forced(text, text, &[12]); // forced split before "Second"
        assert_eq!(result.len(), 3);
        assert_eq!(result[0].0, "First part:");
        assert_eq!(result[1].0, "Second part.");
        assert_eq!(result[2].0, "Third sentence.");
    }

    /// Diagnostic test for Kant 009 block 3 sentence parity.
    /// The EN and DE paragraphs both have 7 sentences visually,
    /// but the EN splitter produces 6. This test reveals which boundary is missed.
    #[test]
    fn test_kant_009_block3_sentence_parity() {
        // Plain text after md_to_plain processing (no markdown markers)
        let en_plain = "One might initially indeed think: that the proposition 7+5 = 12 is a merely analytic proposition, which follows from the concept of a sum of seven and five in accordance with the principle of contradiction. But if one considers it more closely, one finds that the concept of the sum of 7 and 5 contains nothing further than the unification of both numbers into a single one, through which it is not at all thought what this single number is that comprehends both. The concept of twelve is by no means already thought merely by my thinking of that unification of seven and five, and I may analyze my concept of such a possible sum for as long as I please, I will still not encounter the twelve in it. One must go beyond these concepts, by taking assistance from the intuition that corresponds to one of the two, such as one\u{2019}s five fingers or (like Segner in his arithmetic) five points, and thus successively add the units of the five given in intuition to the concept of seven. For I take first the number 7, and by taking the fingers of my hand as an intuition to assist with the concept of 5, I now successively add the units, which I previously brought together in order to make up the number 5, to the number 7 by means of that image of mine, and thus see the number 12 arise. That 5 should be added to 7 I have indeed thought in the concept of a sum = 7+5, but not that this sum is equal to the number 12. The arithmetical proposition is therefore always synthetic, of which one becomes all the more clearly aware if one takes somewhat larger numbers, since it then becomes clearly evident that, twist and turn our concepts as we will, we could never find the sum by means of the mere analysis of our concepts, without taking assistance from intuition.";

        let de_plain = "Man sollte anfänglich zwar denken: dass der Satz 7+5 = 12 ein bloß analytischer Satz sei, der aus dem Begriffe einer Summe von Sieben und Fünf nach dem Satze des Widerspruches erfolge. Allein wenn man es näher betrachtet, so findet man, dass der Begriff der Summe von 7 und 5 nichts weiter enthalte, als die Vereinigung beider Zahlen in eine einzige, wodurch ganz und gar nicht gedacht wird, welches diese einzige Zahl sei, die beide zusammenfasst. Der Begriff von Zwölf ist keineswegs dadurch schon gedacht, dass ich mir bloß jene Vereinigung von Sieben und Fünf denke, und ich mag meinen Begriff von einer solchen möglichen Summe noch so lange zergliedern, so werde ich doch darin die Zwölf nicht antreffen. Man muss über diese Begriffe hinausgehen, indem man die Anschauung zu Hilfe nimmt, die einem von beiden korrespondiert, etwa seine fünf Finger oder (wie Segner in seiner Arithmetik) fünf Punkte, und so nach und nach die Einheiten der in der Anschauung gegebenen Fünf zu dem Begriffe der Sieben hinzutut. Denn ich nehme zuerst die Zahl 7, und indem ich für den Begriff der 5 die Finger meiner Hand als Anschauung zu Hilfe nehme, so tue ich die Einheiten, die ich vorher zusammennahm, um die Zahl 5 auszumachen, nun an jenem meinem Bilde nach und nach zur Zahl 7 und sehe so die Zahl 12 entspringen. Dass 5 zu 7 hinzugetan werden sollten, habe ich zwar in dem Begriff einer Summe = 7+5 gedacht, aber nicht, dass diese Summe der Zahl 12 gleich sei. Der arithmetische Satz ist also jederzeit synthetisch, welches man desto deutlicher inne wird, wenn man etwas größere Zahlen nimmt, da es denn klar einleuchtet, dass, wir möchten unsere Begriffe drehen und wenden, wie wir wollen, wir, ohne die Anschauung zu Hilfe zu nehmen, vermittelst der bloßen Zergliederung unserer Begriffe, die Summe niemals finden könnten.";

        let en_sentences = split_sentences_en(en_plain, en_plain);
        let de_sentences = split_sentences(de_plain, de_plain);

        eprintln!("\n=== EN sentences ({}) ===", en_sentences.len());
        for (i, (text, _)) in en_sentences.iter().enumerate() {
            eprintln!("  [{}] {}", i, &text[..text.len().min(120)]);
        }
        eprintln!("\n=== DE sentences ({}) ===", de_sentences.len());
        for (i, (text, _)) in de_sentences.iter().enumerate() {
            eprintln!("  [{}] {}", i, &text[..text.len().min(120)]);
        }

        assert_eq!(de_sentences.len(), 7, "DE should have 7 sentences");
        assert_eq!(
            en_sentences.len(),
            7,
            "EN should have 7 sentences (currently produces 6 — which boundary is missed?)"
        );
    }
}
