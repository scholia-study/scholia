//! Shared construction of `separator` content blocks.
//!
//! A separator is a thematic break authored as a lone token line: `---` for a
//! plain horizontal rule, `***` for a centered, bold "* * *" ornament (a
//! dinkus). Like a figure it bypasses the markdown → sentence pipeline, but
//! unlike a figure it carries *no* content at all — no sentences, no page
//! markers, no quotable text. It is pure presentation.
//!
//! The variant rides along in the block's `html` as a sentinel: `<hr>` for the
//! plain rule, `<hr class="dinkus">` for the ornament. The reader keys off
//! that class to render the bold centered asterisks; keeping the actual styling
//! in the frontend means no decorative text ever leaks into search, sentence
//! splitting, or cross-translation alignment. Both ingestion pipelines (German
//! and translation) build separators identically.

use crate::model::ContentBlockData;

/// Sentinel html stored for the dinkus (`***`) variant. The reader detects
/// the `dinkus` class and renders a centered, bold "* * *".
const DINKUS_HTML: &str = "<hr class=\"dinkus\">";
/// Sentinel html stored for the plain-rule (`---`) variant.
const RULE_HTML: &str = "<hr>";

/// Build a `separator` content block. `dinkus` selects the variant: `true`
/// for the `***` ornament, `false` for the `---` plain rule.
pub fn build_separator_block(block_pos: usize, dinkus: bool) -> ContentBlockData {
    let html = if dinkus { DINKUS_HTML } else { RULE_HTML };
    ContentBlockData {
        position: block_pos as i16,
        block_type: "separator".to_string(),
        paragraph_number: None,
        figure_number: None,
        text: String::new(),
        html: html.to_string(),
        // A divider has no language-specific layer; the reader renders it from
        // `html` regardless of the original/modernized toggle.
        original_text: None,
        original_html: None,
        sentences: Vec::new(),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn dinkus_block_carries_the_marker_and_no_content() {
        let block = build_separator_block(3, true);
        assert_eq!(block.block_type, "separator");
        assert_eq!(block.position, 3);
        assert_eq!(block.paragraph_number, None);
        assert_eq!(block.figure_number, None);
        assert!(block.text.is_empty());
        assert!(block.html.contains("dinkus"));
        assert!(block.sentences.is_empty());
    }

    #[test]
    fn rule_block_is_a_plain_hr() {
        let block = build_separator_block(0, false);
        assert_eq!(block.block_type, "separator");
        assert_eq!(block.html, "<hr>");
        assert!(!block.html.contains("dinkus"));
        assert!(block.sentences.is_empty());
    }
}
