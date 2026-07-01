//! `{{{ N }}}` printed-page markers: strip them out of rendered text (recording
//! each one's char offset) and resolve each to the sentence it falls in. Mirrors
//! the kant1 approach, narrowed to the single drama marker form.

use std::sync::LazyLock;

use regex::Regex;

static MARKER_RE: LazyLock<Regex> =
    LazyLock::new(|| Regex::new(r"\{\{\{\s*(\d+)\s*\}\}\}").unwrap());

/// A page marker lifted out of the text.
#[derive(Debug, Clone)]
pub struct RawMarker {
    /// The printed page number.
    pub value: String,
    /// Char offset in the **stripped** text where the marker sat (the point at
    /// which the new page begins).
    pub char_offset: usize,
}

/// Strip every `{{{ N }}}` from `text`, returning the cleaned text and the
/// markers with char offsets into it. A single trailing space after a marker is
/// absorbed so a mid-line marker leaves exactly one space (the one before it).
pub fn strip_markers(text: &str) -> (String, Vec<RawMarker>) {
    let mut markers = Vec::new();
    let mut stripped = String::with_capacity(text.len());
    let mut last_end = 0;

    for m in MARKER_RE.find_iter(text) {
        stripped.push_str(&text[last_end..m.start()]);
        let char_offset = stripped.chars().count();
        let value = MARKER_RE
            .captures(m.as_str())
            .expect("matched marker has a capture")[1]
            .to_string();
        markers.push(RawMarker { value, char_offset });

        let end = m.end();
        last_end = if text[end..].starts_with(' ') {
            end + 1
        } else {
            end
        };
    }
    stripped.push_str(&text[last_end..]);
    (stripped, markers)
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
    fn strips_leading_marker_and_trailing_space() {
        let (text, markers) = strip_markers("{{{ 12 }}} Phocion the dyer");
        assert_eq!(text, "Phocion the dyer");
        assert_eq!(markers.len(), 1);
        assert_eq!(markers[0].value, "12");
        assert_eq!(markers[0].char_offset, 0);
    }

    #[test]
    fn strips_inline_marker_keeping_one_space() {
        let (text, markers) = strip_markers("sought you {{{ 23 }}} these two days");
        assert_eq!(text, "sought you these two days");
        assert_eq!(markers.len(), 1);
        assert_eq!(markers[0].value, "23");
        assert_eq!(markers[0].char_offset, 11); // "sought you " = 11 chars
    }

    #[test]
    fn resolves_to_owning_sentence() {
        // sentence 0 = chars 0..10, sentence 1 = chars 11..
        let cumulative = vec![0usize, 11usize];
        assert_eq!(resolve_marker_to_sentence(&cumulative, 0), (0, 0));
        assert_eq!(resolve_marker_to_sentence(&cumulative, 11), (1, 0));
        assert_eq!(resolve_marker_to_sentence(&cumulative, 14), (1, 3));
    }
}
