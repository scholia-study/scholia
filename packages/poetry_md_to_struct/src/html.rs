//! Verse/prose markdown → HTML / plain text.
//!
//! The only inline markup in the curated MD is `*word*` for italicised words
//! (proper names in the Quarto/1674 originals). HTML metacharacters are escaped
//! first so the only tags in the output are the ones we add.

use std::sync::LazyLock;

use regex::Regex;

static ITALIC_RE: LazyLock<Regex> = LazyLock::new(|| Regex::new(r"\*([^*]+)\*").unwrap());

pub fn escape_html(s: &str) -> String {
    s.replace('&', "&amp;")
        .replace('<', "&lt;")
        .replace('>', "&gt;")
}

/// `*x*` → `<i>x</i>`, after escaping HTML metacharacters.
pub fn md_to_html(text: &str) -> String {
    let escaped = escape_html(text);
    ITALIC_RE.replace_all(&escaped, "<i>$1</i>").into_owned()
}

/// Strip `*` emphasis markers, leaving plain text.
pub fn md_to_plain(text: &str) -> String {
    ITALIC_RE.replace_all(text, "$1").into_owned()
}

/// Join verse lines into one HTML blob with `<br>` line breaks.
pub fn join_html(lines: &[String]) -> String {
    lines
        .iter()
        .map(|l| md_to_html(l))
        .collect::<Vec<_>>()
        .join("<br>\n")
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn italic() {
        assert_eq!(
            md_to_html("beauties *Rose* might"),
            "beauties <i>Rose</i> might"
        );
        assert_eq!(md_to_plain("beauties *Rose* might"), "beauties Rose might");
    }

    #[test]
    fn escapes_metachars() {
        assert_eq!(md_to_html("a & b"), "a &amp; b");
    }

    #[test]
    fn plain_line() {
        assert_eq!(
            md_to_html("From fairest creatures"),
            "From fairest creatures"
        );
    }
}
