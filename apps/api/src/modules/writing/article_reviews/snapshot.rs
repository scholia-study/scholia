//! Sentence annotation for review snapshots.
//!
//! A review request freezes the article's rendered HTML at submission time.
//! This pass tags every top-level block with `data-block="i"` and wraps each
//! sentence of prose blocks in `<span data-s="j">…</span>`, so review
//! comments can anchor to `(block_index, sentence_start, sentence_end)` the
//! same way quotations anchor into corpus texts — and, because the snapshot
//! is immutable, the anchors can never drift.
//!
//! The input is `clean_article_html` output (ammonia-sanitized, well-formed,
//! comment-free, text and attribute values entity-escaped), which is what
//! makes the lightweight tag scanner below sufficient. The injected markup is
//! plain spans and `data-*` attributes, so the annotated result needs no
//! re-sanitization.

use common::sentences::split_sentences_en;

/// Block elements whose content is inline prose to sentence-annotate.
const PROSE_TAGS: &[&str] = &["p", "h1", "h2", "h3", "h4", "h5", "h6"];

/// Block elements to recurse into, sharing one sentence counter per
/// top-level block so indices stay unique within it.
const NESTED_TAGS: &[&str] = &["blockquote", "ul", "ol", "li"];

/// Void elements (never pushed on the depth stack).
const VOID_TAGS: &[&str] = &["br", "hr", "img"];

pub fn annotate_snapshot_html(html: &str) -> String {
    let mut out = String::with_capacity(html.len() + html.len() / 4);
    let mut block_index = 0usize;
    for node in parse_children(html) {
        match node {
            Node::Text(t) => out.push_str(t),
            Node::Element(el) => {
                let mut counter = 0usize;
                out.push_str(&annotate_block(&el, Some(block_index), &mut counter));
                block_index += 1;
            }
        }
    }
    out
}

enum Node<'a> {
    Text(&'a str),
    Element(Element<'a>),
}

struct Element<'a> {
    tag: String,
    /// The opening tag, `<` through `>` inclusive.
    open: &'a str,
    inner: &'a str,
    /// The closing tag, empty for void elements.
    close: &'a str,
}

/// Split well-formed HTML into its top-level text runs and elements.
fn parse_children(html: &str) -> Vec<Node<'_>> {
    let bytes = html.as_bytes();
    let mut nodes = Vec::new();
    let mut cursor = 0;

    while cursor < bytes.len() {
        if bytes[cursor] != b'<' {
            let start = cursor;
            while cursor < bytes.len() && bytes[cursor] != b'<' {
                cursor += 1;
            }
            nodes.push(Node::Text(&html[start..cursor]));
            continue;
        }

        let open_start = cursor;
        let open_end = tag_end(html, cursor);
        let tag = tag_name(&html[open_start..open_end]);
        cursor = open_end;

        if VOID_TAGS.contains(&tag.as_str()) {
            nodes.push(Node::Element(Element {
                tag,
                open: &html[open_start..open_end],
                inner: "",
                close: "",
            }));
            continue;
        }

        // Scan to the matching close tag, tracking nesting depth.
        let inner_start = cursor;
        let mut depth = 1usize;
        let mut inner_end = cursor;
        let mut close_end = cursor;
        while cursor < bytes.len() {
            if bytes[cursor] != b'<' {
                cursor += 1;
                continue;
            }
            let t_end = tag_end(html, cursor);
            let is_close = html[cursor..].starts_with("</");
            let t_name = tag_name(&html[cursor..t_end]);
            if is_close {
                depth -= 1;
                if depth == 0 {
                    inner_end = cursor;
                    close_end = t_end;
                    cursor = t_end;
                    break;
                }
            } else if !VOID_TAGS.contains(&t_name.as_str()) {
                depth += 1;
            }
            cursor = t_end;
        }
        if depth != 0 {
            // Unbalanced input; emit the rest verbatim rather than guessing.
            nodes.push(Node::Text(&html[open_start..]));
            return nodes;
        }

        nodes.push(Node::Element(Element {
            tag,
            open: &html[open_start..open_end],
            inner: &html[inner_start..inner_end],
            close: &html[inner_end..close_end],
        }));
    }

    nodes
}

/// Byte offset just past the `>` of the tag starting at `start`, honoring
/// quoted attribute values.
fn tag_end(html: &str, start: usize) -> usize {
    let bytes = html.as_bytes();
    let mut i = start + 1;
    let mut quote: Option<u8> = None;
    while i < bytes.len() {
        match (quote, bytes[i]) {
            (Some(q), b) if b == q => quote = None,
            (None, b'"') | (None, b'\'') => quote = Some(bytes[i]),
            (None, b'>') => return i + 1,
            _ => {}
        }
        i += 1;
    }
    bytes.len()
}

fn tag_name(tag: &str) -> String {
    tag.trim_start_matches('<')
        .trim_start_matches('/')
        .chars()
        .take_while(|c| c.is_ascii_alphanumeric())
        .collect::<String>()
        .to_ascii_lowercase()
}

/// Re-emit an element, adding `data-block` when it is a top-level block and
/// sentence spans within prose content. `counter` numbers sentences across
/// the whole top-level block, including nested paragraphs and list items.
fn annotate_block(el: &Element<'_>, block_index: Option<usize>, counter: &mut usize) -> String {
    let open = match block_index {
        Some(i) => inject_attribute(el.open, &format!(" data-block=\"{i}\"")),
        None => el.open.to_string(),
    };

    let tag = el.tag.as_str();
    let inner = if PROSE_TAGS.contains(&tag) {
        annotate_sentences(el.inner, counter)
    } else if NESTED_TAGS.contains(&tag) {
        if has_element_children(el.inner) {
            parse_children(el.inner)
                .iter()
                .map(|node| match node {
                    Node::Text(t) => (*t).to_string(),
                    Node::Element(child) => annotate_block(child, None, counter),
                })
                .collect()
        } else {
            // Tight list items: inline content directly inside <li>.
            annotate_sentences(el.inner, counter)
        }
    } else {
        el.inner.to_string()
    };

    format!("{open}{inner}{}", el.close)
}

fn has_element_children(inner: &str) -> bool {
    parse_children(inner)
        .iter()
        .any(|n| matches!(n, Node::Element(_)))
}

fn inject_attribute(open_tag: &str, attr: &str) -> String {
    let insert_at = tag_end(open_tag, 0) - 1;
    format!("{}{attr}{}", &open_tag[..insert_at], &open_tag[insert_at..])
}

/// Wrap each sentence of an inline HTML run in `<span data-s="n">`.
fn annotate_sentences(inner: &str, counter: &mut usize) -> String {
    let text = inline_text(inner);
    if text.trim().is_empty() {
        return inner.to_string();
    }
    let sentences = split_sentences_en(&text, inner);
    if sentences.is_empty() {
        return inner.to_string();
    }
    let mut out = String::with_capacity(inner.len() + sentences.len() * 20);
    for (i, (_text, html)) in sentences.iter().enumerate() {
        if i > 0 {
            out.push(' ');
        }
        out.push_str(&format!("<span data-s=\"{}\">{html}</span>", *counter));
        *counter += 1;
    }
    out
}

/// Plain text of an inline HTML run for the sentence splitter: tags dropped,
/// entities left encoded, and `<sup>…</sup>` skipped wholesale — all
/// mirroring how `split_sentences_en` walks text and HTML in parallel.
fn inline_text(html: &str) -> String {
    let bytes = html.as_bytes();
    let mut out = String::with_capacity(html.len());
    let mut i = 0;
    while i < bytes.len() {
        if bytes[i] == b'<' {
            if html[i..].starts_with("<sup>") {
                match html[i..].find("</sup>") {
                    Some(rel) => i += rel + "</sup>".len(),
                    None => i = tag_end(html, i),
                }
            } else {
                i = tag_end(html, i);
            }
            continue;
        }
        let ch_len = utf8_len(bytes[i]);
        out.push_str(&html[i..i + ch_len]);
        i += ch_len;
    }
    out
}

fn utf8_len(b: u8) -> usize {
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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn tags_blocks_and_wraps_sentences() {
        let out = annotate_snapshot_html("<p>One. Two.</p><p>Three.</p>");
        assert_eq!(
            out,
            "<p data-block=\"0\"><span data-s=\"0\">One.</span> \
             <span data-s=\"1\">Two.</span></p>\
             <p data-block=\"1\"><span data-s=\"0\">Three.</span></p>"
        );
    }

    #[test]
    fn preserves_inline_markup_across_boundaries() {
        let out = annotate_snapshot_html("<p><em>One. Two.</em></p>");
        // The splitter re-balances the <em> across the split.
        assert!(out.contains("data-s=\"0\""));
        assert!(out.contains("data-s=\"1\""));
        assert_eq!(out.matches("<em>").count(), 2);
        assert_eq!(out.matches("</em>").count(), 2);
    }

    #[test]
    fn embeds_get_block_index_only() {
        let html = r#"<p>Intro.</p><div class="quotation-embed" data-quotation-book="kjv"></div>"#;
        let out = annotate_snapshot_html(html);
        assert!(
            out.contains(
                r#"<div class="quotation-embed" data-quotation-book="kjv" data-block="1">"#
            )
        );
        assert!(!out.contains(
            r#"<div class="quotation-embed" data-quotation-book="kjv" data-block="1"><span"#
        ));
    }

    #[test]
    fn blockquote_paragraphs_share_one_counter() {
        let out = annotate_snapshot_html("<blockquote><p>One.</p><p>Two.</p></blockquote>");
        assert!(out.contains("<blockquote data-block=\"0\">"));
        assert!(out.contains("<span data-s=\"0\">One.</span>"));
        assert!(out.contains("<span data-s=\"1\">Two.</span>"));
    }

    #[test]
    fn tight_and_loose_lists() {
        let tight = annotate_snapshot_html("<ul><li>One.</li><li>Two.</li></ul>");
        assert!(tight.contains("<span data-s=\"0\">One.</span>"));
        assert!(tight.contains("<span data-s=\"1\">Two.</span>"));

        let loose = annotate_snapshot_html("<ul><li><p>One.</p></li><li><p>Two.</p></li></ul>");
        assert!(loose.contains("<span data-s=\"0\">One.</span>"));
        assert!(loose.contains("<span data-s=\"1\">Two.</span>"));
    }

    #[test]
    fn headings_are_annotated() {
        let out = annotate_snapshot_html("<h2>A title</h2>");
        assert_eq!(
            out,
            "<h2 data-block=\"0\"><span data-s=\"0\">A title</span></h2>"
        );
    }

    #[test]
    fn entities_stay_encoded_and_do_not_desync() {
        let out = annotate_snapshot_html("<p>Fish &amp; chips. Second sentence.</p>");
        assert!(out.contains("<span data-s=\"0\">Fish &amp; chips.</span>"));
        assert!(out.contains("<span data-s=\"1\">Second sentence.</span>"));
    }

    #[test]
    fn bibliography_left_unsplit() {
        let html = "<section class=\"bibliography\"><h2>Bibliography</h2><ul><li>Kant, I.</li></ul></section>";
        let out = annotate_snapshot_html(html);
        assert!(out.starts_with("<section class=\"bibliography\" data-block=\"0\">"));
        assert!(!out.contains("data-s="));
    }

    #[test]
    fn attribute_with_bracket_survives() {
        let html =
            "<p><a href=\"https://x.test/a?b=1&amp;c=2\" rel=\"noopener noreferrer\">link</a>.</p>";
        let out = annotate_snapshot_html(html);
        assert!(out.contains("data-block=\"0\""));
        assert!(out.contains("href=\"https://x.test/a?b=1&amp;c=2\""));
    }

    #[test]
    fn whitespace_between_blocks_is_kept() {
        let out = annotate_snapshot_html("<p>One.</p>\n<p>Two.</p>");
        assert!(out.contains("</p>\n<p data-block=\"1\">"));
    }
}
