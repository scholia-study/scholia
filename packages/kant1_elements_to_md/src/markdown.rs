use crate::model::{MdBlockType, MdTocNode};
use crate::toc;
pub use common::kant1::filenames::{filename, position_number};

/// Render a complete markdown file for one TOC node.
pub fn render_md(node: &MdTocNode) -> String {
    let mut out = String::new();

    // YAML front matter
    out.push_str("---\n");
    out.push_str(&format!("position: {}\n", position_number(node.flat_index)));
    out.push_str(&format!("label: \"{}\"\n", node.label.replace('"', "\\\"")));
    out.push_str(&format!("depth: {}\n", node.depth));
    out.push_str(&format!("aa_page: {}\n", node.aa_page));
    out.push_str("---\n\n");

    // Heading: always top-level # (depth is in front matter)
    out.push_str(&format!("# {}\n", node.label));

    // Blocks
    let mut prev_aa_page: Option<u16> = None;

    for block in &node.blocks {
        out.push('\n');

        match block.block_type {
            MdBlockType::Heading => {
                let text = insert_b_page_markers(&block.text, &block.b_page_anchors);
                let text = maybe_prepend_aa_page(text, block.aa_page, &mut prev_aa_page);
                out.push_str(&format!("## {}\n", text));
            }
            MdBlockType::Paragraph => {
                let text = insert_b_page_markers(&block.text, &block.b_page_anchors);
                let text = maybe_prepend_aa_page(text, block.aa_page, &mut prev_aa_page);
                out.push_str(&text);
                out.push('\n');
            }
        }
    }

    // Footnotes
    if !node.footnotes.is_empty() {
        out.push('\n');
        for footnote in &node.footnotes {
            out.push_str(&format!("[^{}]: {}\n", footnote.marker, footnote.text));
        }
    }

    out
}

/// Insert `{{ ref }}` B-page markers into text at b_page_anchor char offsets.
/// Inserts right-to-left to preserve earlier offsets.
fn insert_b_page_markers(text: &str, anchors: &[crate::stitch::BPageAnchor]) -> String {
    if anchors.is_empty() {
        return text.to_string();
    }

    // Sort by char_offset descending so we can insert right-to-left
    let mut sorted: Vec<_> = anchors.iter().collect();
    sorted.sort_by_key(|b| std::cmp::Reverse(b.char_offset));

    let mut result = text.to_string();
    for anchor in sorted {
        let marker = format!("{{{{ {} }}}} ", anchor.b_page);
        // Convert char offset to byte offset
        let byte_offset = char_to_byte_offset(&result, anchor.char_offset);
        if byte_offset <= result.len() {
            result.insert_str(byte_offset, &marker);
        }
    }
    result
}

/// Convert a char offset to a byte offset in a string.
fn char_to_byte_offset(s: &str, char_offset: usize) -> usize {
    s.char_indices()
        .nth(char_offset)
        .map(|(byte_idx, _)| byte_idx)
        .unwrap_or(s.len())
}

/// Prepend `{{{ N }}}` if aa_page differs from previous block.
fn maybe_prepend_aa_page(text: String, aa_page: u16, prev: &mut Option<u16>) -> String {
    if *prev != Some(aa_page) {
        *prev = Some(aa_page);
        format!("{{{{{{ {} }}}}}} {}", aa_page, text)
    } else {
        text
    }
}

/// Render `000_toc.md` — a table of contents with links to section files.
///
/// Each entry is indented by depth and links to the corresponding markdown file.
/// Entries that have a generated file get a clickable link; others are listed plain.
pub fn render_toc(emitted: &[&MdTocNode]) -> String {
    let flat = toc::flat_toc_entries();
    // Build a set of flat_indices that have emitted files
    let emitted_set: std::collections::HashMap<usize, &MdTocNode> =
        emitted.iter().map(|n| (n.flat_index, *n)).collect();

    let mut out = String::new();
    out.push_str("# Kritik der reinen Vernunft\n\n");
    out.push_str("Immanuel Kant — Akademie-Ausgabe Band III (B-Auflage 1787)\n\n");
    out.push_str("---\n\n");

    for &(idx, aa_page, depth, label, _) in &flat {
        let indent = "  ".repeat(depth.saturating_sub(1) as usize);
        if let Some(node) = emitted_set.get(&idx) {
            let fname = filename(node.flat_index, &node.label);
            out.push_str(&format!(
                "{}- [{}]({}) — AA {}\n",
                indent, label, fname, aa_page
            ));
        } else {
            out.push_str(&format!("{}- {} — AA {}\n", indent, label, aa_page));
        }
    }

    out
}

#[cfg(test)]
pub use common::kant1::filenames::slugify;

#[cfg(test)]
mod tests {
    use super::*;
    use crate::model::{MdBlock, MdBlockType, MdFootnote, MdTocNode};
    use crate::stitch::BPageAnchor;

    #[test]
    fn test_slugify() {
        assert_eq!(
            slugify("Vorrede zur zweiten Auflage"),
            "vorrede_zur_zweiten_auflage"
        );
        assert_eq!(slugify("Motto"), "motto");
        assert_eq!(slugify("§1"), "1");
        assert_eq!(
            slugify("I. Transscendentale Elementarlehre"),
            "i_transscendentale_elementarlehre"
        );
        assert_eq!(
            slugify("Die transscendentale Ästhetik"),
            "die_transscendentale_aesthetik"
        );
        assert_eq!(
            slugify("1. Hauptstück. Von dem Schematismus"),
            "1_hauptstueck_von_dem_schematismus"
        );
        assert_eq!(slugify("Grundsätze"), "grundsaetze");
    }

    #[test]
    fn test_filename() {
        assert_eq!(filename(0, "Motto"), "001_motto.md");
        assert_eq!(
            filename(2, "Vorrede zur zweiten Auflage"),
            "003_vorrede_zur_zweiten_auflage.md"
        );
    }

    #[test]
    fn test_render_simple() {
        let node = MdTocNode {
            flat_index: 0,
            label: "Motto".to_string(),
            aa_page: 2,
            depth: 1,
            blocks: vec![
                MdBlock {
                    block_type: MdBlockType::Heading,
                    text: "BACO DE VERULAMIO.".to_string(),
                    aa_page: 2,
                    b_page_anchors: vec![BPageAnchor {
                        b_page: "II".to_string(),
                        char_offset: 0,
                    }],
                },
                MdBlock {
                    block_type: MdBlockType::Paragraph,
                    text: "Some Latin text.".to_string(),
                    aa_page: 2,
                    b_page_anchors: vec![],
                },
            ],
            footnotes: vec![MdFootnote {
                marker: "1".to_string(),
                text: "Das Motto ist dem Titel entnommen.".to_string(),
            }],
        };
        let md = render_md(&node);
        assert!(md.contains("---\nposition: 1\n"));
        assert!(md.contains("# Motto\n"));
        assert!(md.contains("## {{{ 2 }}} {{ II }} BACO DE VERULAMIO.\n"));
        assert!(md.contains("Some Latin text.\n"));
        assert!(md.contains("[^1]: Das Motto ist dem Titel entnommen.\n"));
    }

    #[test]
    fn test_b_page_marker_insertion() {
        let anchors = vec![BPageAnchor {
            b_page: "XIV".to_string(),
            char_offset: 16,
        }];
        let result = insert_b_page_markers("Erster Teil des Satzes hier.", &anchors);
        assert_eq!(result, "Erster Teil des {{ XIV }} Satzes hier.");
    }

    #[test]
    fn test_aa_page_tracking() {
        let mut prev = None;
        let r1 = maybe_prepend_aa_page("Hello".to_string(), 7, &mut prev);
        assert_eq!(r1, "{{{ 7 }}} Hello");
        let r2 = maybe_prepend_aa_page("World".to_string(), 7, &mut prev);
        assert_eq!(r2, "World");
        let r3 = maybe_prepend_aa_page("New page".to_string(), 8, &mut prev);
        assert_eq!(r3, "{{{ 8 }}} New page");
    }
}
