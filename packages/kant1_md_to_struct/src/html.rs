use regex::Regex;
use std::sync::LazyLock;

// Footnote ref: [^marker] — must be parsed BEFORE bold
pub static FOOTNOTE_REF_RE: LazyLock<Regex> =
    LazyLock::new(|| Regex::new(r"\[\^([^\]]+)\]").unwrap());

// Emphasis: _text_ — simple pattern, no lookaround needed in these files
static EMPHASIS_RE: LazyLock<Regex> =
    LazyLock::new(|| Regex::new(r"_([^_]+)_").unwrap());

// Bold: **text**
static BOLD_RE: LazyLock<Regex> =
    LazyLock::new(|| Regex::new(r"\*\*([^*]+)\*\*").unwrap());

/// Strip markdown formatting markers, returning plain text.
///
/// Same processing order as md_to_html but replaces with content only.
pub fn md_to_plain(text: &str) -> String {
    // Strip footnote refs entirely from plain text (they only appear in HTML as <sup>)
    let result = FOOTNOTE_REF_RE.replace_all(text, "");
    let result = BOLD_RE.replace_all(&result, "$1");
    let result = EMPHASIS_RE.replace_all(&result, "$1");
    result.into_owned()
}

/// Convert markdown-formatted text to HTML.
///
/// Processing order matters:
/// 1. Footnote refs `[^marker]` → `<sup>marker</sup>` (before bold, since `[^**]` is valid)
/// 2. Bold `**text**` → `<span class="sperrdruck">text</span>`
/// 3. Emphasis `_text_` → `<span class="antiqua">text</span>`
pub fn md_to_html(text: &str) -> String {
    // 1. Footnote refs — use placeholder for stars to prevent bold regex matching across <sup> tags
    let result = FOOTNOTE_REF_RE.replace_all(text, |caps: &regex::Captures| {
        let marker = caps[1].replace('*', "\u{FFFC}");
        format!("<sup>{marker}</sup>")
    });
    // 2. Bold
    let result = BOLD_RE.replace_all(&result, "<span class=\"sperrdruck\">$1</span>");
    // 3. Emphasis
    let result = EMPHASIS_RE.replace_all(&result, "<span class=\"antiqua\">$1</span>");
    // 4. Restore star placeholders
    result.replace('\u{FFFC}', "*")
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
    fn test_footnote_stars_dont_interfere_with_bold() {
        // [^***] and [^****] in same text should not create false bold matches
        assert_eq!(
            md_to_html("end.[^***] Middle text.[^****]"),
            "end.<sup>***</sup> Middle text.<sup>****</sup>"
        );
    }

    #[test]
    fn test_combined() {
        assert_eq!(
            md_to_html("_italic_ and **bold** and [^1]"),
            "<span class=\"antiqua\">italic</span> and <span class=\"sperrdruck\">bold</span> and <sup>1</sup>",
        );
    }

    #[test]
    fn test_combined_with_star_footnote() {
        assert_eq!(
            md_to_html("**bold** and [^*]"),
            "<span class=\"sperrdruck\">bold</span> and <sup>*</sup>"
        );
    }

    #[test]
    fn test_plain() {
        assert_eq!(md_to_html("plain text"), "plain text");
    }

    #[test]
    fn test_plain_strips_footnote_refs() {
        assert_eq!(md_to_plain("end.[^5] Next sentence."), "end. Next sentence.");
        assert_eq!(md_to_plain("text[^1] more"), "text more");
        assert_eq!(md_to_plain("[^*]"), "");
    }

    #[test]
    fn test_plain_keeps_bold_and_emphasis() {
        assert_eq!(md_to_plain("**bold** and _italic_"), "bold and italic");
    }
}
