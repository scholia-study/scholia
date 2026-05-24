//! Shared construction of `figure` content blocks.
//!
//! A figure is a diagram-like insertion (e.g. Kant's table of judgments)
//! authored as verbatim `<figure>` HTML. Unlike prose blocks it bypasses the
//! markdown → sentence pipeline: the HTML is preserved as-is on the block, and
//! a single anchor sentence carries the `<figcaption>` label so the figure is
//! selectable, quotable, alignable across translations, and can hold page
//! markers. The anchor gets `sentence_number = None`, keeping it out of the
//! body-text enumeration (the same treatment headings and footnotes receive).
//!
//! Both ingestion pipelines build figures identically; the only difference is
//! whether a reviewed/original layer exists (German has one, the translation
//! does not), expressed via the `reviewed` argument.

use crate::model::{ContentBlockData, PageMarkerData, SentenceData};
use crate::parse::{MarkerKind, ParsedBlock, RawMarker, figure_caption, strip_html_tags};
use crate::roman::roman_to_int;

/// Map a raw page marker to its DB-ready form, resolving the reference-system
/// slug and a numeric sort order.
pub fn marker_to_page_marker(marker: &RawMarker) -> PageMarkerData {
    let (system, sort_order) = match marker.kind {
        MarkerKind::Aa => ("aa_iii", marker.value.parse::<i32>().unwrap_or(0)),
        MarkerKind::BEdition => (
            "b_edition",
            roman_to_int(&marker.value).map(|v| v as i32).unwrap_or(0),
        ),
    };
    PageMarkerData {
        system: system.to_string(),
        ref_value: marker.value.clone(),
        sort_order,
        // A figure has no meaningful intra-unit position, so every marker
        // attaches to the anchor at offset 0.
        char_offset: 0,
    }
}

/// Build a `figure` content block from the primary (modernized/translation)
/// figure and, when present, the reviewed/original figure.
///
/// Panics if either layer lacks a `<figcaption>` — every figure must carry a
/// label, and a missing one is an authoring error worth failing the build for.
pub fn build_figure_block(
    primary: &ParsedBlock,
    reviewed: Option<&ParsedBlock>,
    block_pos: usize,
    flat_index: usize,
    figure_number: i32,
) -> ContentBlockData {
    let caption = figure_caption(&primary.text).unwrap_or_else(|| {
        panic!(
            "file index {}, block {}: <figure> has no <figcaption>",
            flat_index + 1,
            block_pos,
        )
    });

    let orig_caption = reviewed.map(|r| {
        figure_caption(&r.text).unwrap_or_else(|| {
            panic!(
                "file index {}, block {}: reviewed <figure> has no <figcaption>",
                flat_index + 1,
                block_pos,
            )
        })
    });

    let page_markers = primary.markers.iter().map(marker_to_page_marker).collect();

    let anchor = SentenceData {
        position: 0,
        sentence_number: None,
        text: caption.clone(),
        html: caption,
        original_text: orig_caption.clone(),
        original_html: orig_caption,
        page_markers,
        footnotes: Vec::new(),
    };

    ContentBlockData {
        position: block_pos as i16,
        block_type: "figure".to_string(),
        paragraph_number: None,
        figure_number: Some(figure_number),
        text: strip_html_tags(&primary.text),
        html: primary.text.clone(),
        original_text: reviewed.map(|r| strip_html_tags(&r.text)),
        original_html: reviewed.map(|r| r.text.clone()),
        sentences: vec![anchor],
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::parse::ParsedBlockType;

    fn figure(text: &str) -> ParsedBlock {
        ParsedBlock {
            block_type: ParsedBlockType::Figure,
            text: text.to_string(),
            markers: Vec::new(),
        }
    }

    #[test]
    fn dual_layer_figure_has_one_anchor_with_both_captions() {
        let primary =
            figure("<figure><figcaption>Table of Judgments</figcaption><table></table></figure>");
        let reviewed =
            figure("<figure><figcaption>Tafel der Urtheile</figcaption><table></table></figure>");
        let block = build_figure_block(&primary, Some(&reviewed), 2, 28, 3);

        assert_eq!(block.block_type, "figure");
        assert_eq!(block.position, 2);
        assert_eq!(block.figure_number, Some(3));
        assert!(block.html.contains("<table>"));
        assert_eq!(block.sentences.len(), 1);

        let anchor = &block.sentences[0];
        assert_eq!(anchor.position, 0);
        assert_eq!(anchor.sentence_number, None);
        assert_eq!(anchor.text, "Table of Judgments");
        assert_eq!(anchor.original_text.as_deref(), Some("Tafel der Urtheile"));
    }

    #[test]
    fn single_layer_figure_has_no_original() {
        let primary = figure("<figure><figcaption>Table of Judgments</figcaption></figure>");
        let block = build_figure_block(&primary, None, 0, 0, 1);
        assert_eq!(block.figure_number, Some(1));
        assert_eq!(block.original_html, None);
        assert_eq!(block.sentences[0].original_text, None);
    }
}
