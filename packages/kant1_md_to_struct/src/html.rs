use regex::Regex;
use std::sync::LazyLock;

// Footnote ref: [^marker] — must be parsed BEFORE bold
static FOOTNOTE_REF_RE: LazyLock<Regex> =
    LazyLock::new(|| Regex::new(r"\[\^([^\]]+)\]").unwrap());

// Emphasis: _text_ — simple pattern, no lookaround needed in these files
static EMPHASIS_RE: LazyLock<Regex> =
    LazyLock::new(|| Regex::new(r"_([^_]+)_").unwrap());

// Bold: **text**
static BOLD_RE: LazyLock<Regex> =
    LazyLock::new(|| Regex::new(r"\*\*([^*]+)\*\*").unwrap());

/// Convert markdown-formatted text to HTML.
///
/// Processing order matters:
/// 1. Footnote refs `[^marker]` → `<sup>marker</sup>` (before bold, since `[^**]` is valid)
/// 2. Bold `**text**` → `<span class="sperrdruck">text</span>`
/// 3. Emphasis `_text_` → `<span class="antiqua">text</span>`
pub fn md_to_html(text: &str) -> String {
    // 1. Footnote refs
    let result = FOOTNOTE_REF_RE.replace_all(text, "<sup>$1</sup>");
    // 2. Bold
    let result = BOLD_RE.replace_all(&result, "<span class=\"sperrdruck\">$1</span>");
    // 3. Emphasis
    let result = EMPHASIS_RE.replace_all(&result, "<span class=\"antiqua\">$1</span>");

    result.into_owned()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_emphasis() {
        assert_eq!(
            md_to_html("_Instauratio magna._"),
            "<span class=\"antiqua\">Instauratio magna.</span>"
        );
    }

    #[test]
    fn test_bold() {
        assert_eq!(
            md_to_html("**Sperrdruck** hier"),
            "<span class=\"sperrdruck\">Sperrdruck</span> hier"
        );
    }

    #[test]
    fn test_footnote_ref() {
        assert_eq!(md_to_html("[^*]"), "<sup>*</sup>");
        assert_eq!(md_to_html("[^**]"), "<sup>**</sup>");
        assert_eq!(md_to_html("[^1]"), "<sup>1</sup>");
    }

    #[test]
    fn test_footnote_ref_before_bold() {
        // [^**] should be a footnote ref, not bold
        assert_eq!(
            md_to_html("text[^**] more"),
            "text<sup>**</sup> more"
        );
    }

    #[test]
    fn test_combined() {
        assert_eq!(
            md_to_html("_italic_ and **bold** and [^1]"),
            "<span class=\"antiqua\">italic</span> and <span class=\"sperrdruck\">bold</span> and <sup>1</sup>"
        );
    }

    #[test]
    fn test_plain() {
        assert_eq!(md_to_html("plain text"), "plain text");
    }
}
