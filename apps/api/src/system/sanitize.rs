//! HTML sanitization for user-authored content.
//!
//! Article markdown is rendered to HTML server-side
//! (`writing::articles::db::render_article_markdown`) and stored; article
//! quotations carry a client-supplied `html` field. Both are rendered in the
//! browser with `html-react-parser`, which performs no sanitization — so every
//! byte we persist must already be safe. These allowlist cleaners are the
//! single chokepoint enforcing that.

use std::collections::{HashMap, HashSet};
use std::sync::OnceLock;

use ammonia::Builder;

/// Cleaner for rendered article bodies. Starts from ammonia's default
/// allowlist (which already drops `<script>`, event-handler attributes,
/// `<iframe>`, `style`, and unsafe URL schemes) and widens it for the three
/// constructs the renderer injects: quotation embeds
/// (`div.quotation-embed` / `div.article-quotation-embed` plus their inert
/// `data-*` attributes), inline citations (`span.citation`), and the generated
/// bibliography (`section.bibliography`).
fn article_cleaner() -> &'static Builder<'static> {
    static CLEANER: OnceLock<Builder<'static>> = OnceLock::new();
    CLEANER.get_or_init(|| {
        let classes: HashMap<&str, HashSet<&str>> = HashMap::from([
            (
                "div",
                HashSet::from(["quotation-embed", "article-quotation-embed"]),
            ),
            ("span", HashSet::from(["citation"])),
            ("section", HashSet::from(["bibliography"])),
        ]);

        let mut b = Builder::default();
        b.add_tags(["section"]);
        b.add_generic_attribute_prefixes(["data-"]);
        b.url_schemes(["http", "https", "mailto"].into_iter().collect());
        b.allowed_classes(classes);

        b
    })
}

/// Sanitize rendered article-body HTML. Strips scripts, event handlers, and
/// unsafe URL schemes while preserving the embed/citation/bibliography markup
/// the frontend renderer depends on.
pub fn clean_article_html(html: &str) -> String {
    article_cleaner().clean(html).to_string()
}

/// Sanitize an inline HTML snippet (the article-quotation `html` field). Uses
/// ammonia's default allowlist — quotations need only basic inline formatting,
/// and the defaults already drop scripts, event handlers, and `<iframe>`.
pub fn clean_inline_html(html: &str) -> String {
    ammonia::clean(html)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn strips_script_and_contents() {
        let out = clean_article_html("<p>hi</p><script>alert(1)</script>");
        assert!(out.contains("<p>hi</p>"));
        assert!(!out.contains("alert"));
        assert!(!out.contains("<script"));
    }

    #[test]
    fn strips_event_handler_attributes() {
        let out = clean_article_html(r#"<img src="x" onerror="alert(1)">"#);
        assert!(out.contains("<img"));
        assert!(!out.contains("onerror"));
        assert!(!out.contains("alert"));
    }

    #[test]
    fn neutralizes_javascript_urls() {
        let out = clean_article_html(r#"<a href="javascript:alert(1)">x</a>"#);
        assert!(!out.contains("javascript:"));
    }

    #[test]
    fn preserves_quotation_embed() {
        let input = r#"<div class="quotation-embed" data-quotation-book="kjv" data-quotation-start="1"></div>"#;
        let out = clean_article_html(input);
        assert!(out.contains(r#"class="quotation-embed""#));
        assert!(out.contains(r#"data-quotation-book="kjv""#));
        assert!(out.contains(r#"data-quotation-start="1""#));
    }

    #[test]
    fn preserves_article_quotation_embed() {
        let input =
            r#"<div class="article-quotation-embed" data-article-quotation-id="abc"></div>"#;
        let out = clean_article_html(input);
        assert!(out.contains(r#"class="article-quotation-embed""#));
        assert!(out.contains(r#"data-article-quotation-id="abc""#));
    }

    #[test]
    fn preserves_citation_and_bibliography() {
        let input = concat!(
            r#"<span class="citation">(Kant 1781)</span>"#,
            r#"<section class="bibliography"><h2>Bibliography</h2>"#,
            r#"<ul><li><em>Kritik</em>.</li></ul></section>"#,
        );
        let out = clean_article_html(input);
        assert!(out.contains(r#"<span class="citation">"#));
        assert!(out.contains(r#"<section class="bibliography">"#));
        assert!(out.contains("<h2>Bibliography</h2>"));
        assert!(out.contains("<em>Kritik</em>"));
    }

    #[test]
    fn strips_injected_handler_on_embed_div() {
        // Mirrors the latent attribute-injection via the ::article-quotation
        // `#id` shorthand (regex accepts `"`), which could emit a stray
        // attribute after the data-* one. The id survives; the handler dies.
        let input = r#"<div class="article-quotation-embed" data-article-quotation-id="abc" onmouseover="alert(1)"></div>"#;
        let out = clean_article_html(input);
        assert!(out.contains(r#"data-article-quotation-id="abc""#));
        assert!(!out.contains("onmouseover"));
        assert!(!out.contains("alert"));
    }

    #[test]
    fn drops_disallowed_class_values() {
        // Only the renderer's own class names are allowed; an attacker-chosen
        // class (e.g. to hijack app styling) is dropped.
        let out = clean_article_html(r#"<div class="evil">x</div>"#);
        assert!(!out.contains("evil"));
    }

    #[test]
    fn inline_cleaner_strips_script() {
        let out = clean_inline_html("<b>quote</b><script>steal()</script>");
        assert!(out.contains("<b>quote</b>"));
        assert!(!out.contains("steal"));
    }
}
