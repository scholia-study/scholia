use common::kant1::filenames::slugify;
use common::kant1::toc;
use common::sentences::split_sentences;

use crate::html::{md_to_html, md_to_plain};
use crate::model::*;
use crate::parse::{MarkerKind, ParsedBlock, ParsedBlockType, RawMarker};
use crate::roman::roman_to_int;

/// Intermediate per-file parsed data.
pub struct ParsedFile {
    pub flat_index: usize,
    pub blocks: Vec<ParsedBlock>,
}

/// Build the complete nested output from parsed files.
pub fn build_output(parsed_files: &[ParsedFile]) -> Output {
    let book = BookData {
        slug: "kant-krv".to_string(),
        title: "Kritik der reinen Vernunft".to_string(),
        author: "Immanuel Kant".to_string(),
        language: "de".to_string(),
        source: "Akademie-Ausgabe Band III".to_string(),
        source_date: "1787".to_string(),
    };

    let reference_systems = vec![
        ReferenceSystemData {
            slug: "aa_iii".to_string(),
            label: "Akademie-Ausgabe Band III".to_string(),
            ref_type: "block".to_string(),
        },
        ReferenceSystemData {
            slug: "b_edition".to_string(),
            label: "B-Auflage Seitenzahl".to_string(),
            ref_type: "inline".to_string(),
        },
    ];

    let flat_entries = toc::flat_toc_entries();
    let mut paragraph_counter: i32 = 1;
    let mut sentence_counter: i32 = 1;

    let toc_nodes: Vec<TocNodeData> = parsed_files
        .iter()
        .map(|pf| {
            let (_, _, depth, label) = flat_entries[pf.flat_index];
            let parent_source_ref = find_parent_source_ref(&flat_entries, pf.flat_index, depth);
            let path = build_path(&flat_entries, pf.flat_index, depth);

            let content_blocks = pf
                .blocks
                .iter()
                .enumerate()
                .map(|(block_pos, block)| {
                    build_block(
                        block,
                        block_pos,
                        &mut paragraph_counter,
                        &mut sentence_counter,
                    )
                })
                .collect();

            TocNodeData {
                source_ref: format!("{:03}", pf.flat_index + 1),
                slug: slugify(label),
                path,
                sort_order: pf.flat_index as i32 + 1,
                depth: depth as i16,
                label: label.to_string(),
                parent_source_ref,
                content_blocks,
            }
        })
        .collect();

    Output {
        book,
        reference_systems,
        toc_nodes,
    }
}

fn build_block(
    block: &ParsedBlock,
    block_pos: usize,
    paragraph_counter: &mut i32,
    sentence_counter: &mut i32,
) -> ContentBlockData {
    let (block_type_str, para_num) = match &block.block_type {
        ParsedBlockType::Heading => ("heading", None),
        ParsedBlockType::Paragraph => {
            let n = *paragraph_counter;
            *paragraph_counter += 1;
            ("paragraph", Some(n))
        }
        ParsedBlockType::Footnote { .. } => ("footnote", None),
    };

    let block_plain = md_to_plain(&block.text);
    let block_html = md_to_html(&block.text);
    let sentence_pairs = split_sentences(&block_plain, &block_html);

    // Build sentences with cumulative char tracking for marker resolution
    let mut sentences = Vec::new();
    let mut cumulative_chars: Vec<usize> = Vec::new();
    let mut offset: usize = 0;

    for (sent_pos, (sent_text, sent_html)) in sentence_pairs.iter().enumerate() {
        cumulative_chars.push(offset);
        let sent_char_count = sent_text.chars().count();
        offset += sent_char_count + 1; // +1 for space between sentences

        let sent_num = if block_type_str == "paragraph" {
            let n = *sentence_counter;
            *sentence_counter += 1;
            Some(n)
        } else {
            None
        };

        sentences.push(SentenceData {
            position: sent_pos as i16,
            sentence_number: sent_num,
            text: sent_text.clone(),
            html: sent_html.clone(),
            page_markers: Vec::new(),
        });
    }

    // Assign page markers to their sentences
    for marker in &block.markers {
        if sentences.is_empty() {
            continue;
        }

        let (sent_idx, char_offset_in_sentence) =
            resolve_marker_to_sentence(&sentence_pairs, &cumulative_chars, marker);

        let (system_slug, sort_order) = match marker.kind {
            MarkerKind::Aa => {
                let sort = marker.value.parse::<i32>().unwrap_or(0);
                ("aa_iii", sort)
            }
            MarkerKind::BEdition => {
                let sort = roman_to_int(&marker.value).map(|v| v as i32).unwrap_or(0);
                ("b_edition", sort)
            }
        };

        sentences[sent_idx].page_markers.push(PageMarkerData {
            system: system_slug.to_string(),
            ref_value: marker.value.clone(),
            sort_order,
            char_offset: char_offset_in_sentence,
        });
    }

    ContentBlockData {
        position: block_pos as i16,
        block_type: block_type_str.to_string(),
        paragraph_number: para_num,
        text: block_plain,
        html: block_html,
        sentences,
    }
}

/// Find parent's source_ref: nearest preceding entry with depth - 1.
fn find_parent_source_ref(
    flat_entries: &[(usize, u16, u16, &str)],
    current_idx: usize,
    current_depth: u16,
) -> Option<String> {
    if current_depth <= 1 {
        return None;
    }
    let target_depth = current_depth - 1;
    for i in (0..current_idx).rev() {
        let (_, _, d, _) = flat_entries[i];
        if d == target_depth {
            return Some(format!("{:03}", i + 1));
        }
        if d < target_depth {
            return None;
        }
    }
    None
}

/// Build an ltree path from slugs of ancestors.
fn build_path(
    flat_entries: &[(usize, u16, u16, &str)],
    current_idx: usize,
    current_depth: u16,
) -> String {
    let mut segments = vec![slugify(flat_entries[current_idx].3)];
    let mut depth = current_depth;
    let mut idx = current_idx;

    while depth > 1 {
        let target = depth - 1;
        let mut found = false;
        for i in (0..idx).rev() {
            let (_, _, d, label) = flat_entries[i];
            if d == target {
                segments.push(slugify(label));
                idx = i;
                depth = target;
                found = true;
                break;
            }
            if d < target {
                break;
            }
        }
        if !found {
            break;
        }
    }

    segments.reverse();
    segments.join(".")
}

/// Resolve which sentence a marker belongs to, returning (sentence_index, char_offset_in_sentence).
fn resolve_marker_to_sentence(
    sentence_pairs: &[(String, String)],
    cumulative_chars: &[usize],
    marker: &RawMarker,
) -> (usize, i32) {
    for i in (0..sentence_pairs.len()).rev() {
        if marker.char_offset >= cumulative_chars[i] {
            let offset_in_sentence = (marker.char_offset - cumulative_chars[i]) as i32;
            return (i, offset_in_sentence);
        }
    }
    (0, 0)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::parse::{MarkerKind, RawMarker};

    #[test]
    fn test_resolve_marker_single_sentence() {
        let pairs = vec![("Hello world".to_string(), "Hello world".to_string())];
        let cumulative = vec![0];
        let marker = RawMarker {
            kind: MarkerKind::BEdition,
            value: "VII".to_string(),
            char_offset: 6,
        };
        let (idx, offset) = resolve_marker_to_sentence(&pairs, &cumulative, &marker);
        assert_eq!(idx, 0);
        assert_eq!(offset, 6);
    }

    #[test]
    fn test_resolve_marker_second_sentence() {
        let pairs = vec![
            ("First sentence.".to_string(), "First sentence.".to_string()),
            ("Second sentence.".to_string(), "Second sentence.".to_string()),
        ];
        let cumulative = vec![0, 16]; // 15 chars + 1 space
        let marker = RawMarker {
            kind: MarkerKind::BEdition,
            value: "X".to_string(),
            char_offset: 18,
        };
        let (idx, offset) = resolve_marker_to_sentence(&pairs, &cumulative, &marker);
        assert_eq!(idx, 1);
        assert_eq!(offset, 2);
    }

    #[test]
    fn test_build_path_depth1() {
        let entries = toc::flat_toc_entries();
        let path = build_path(&entries, 0, 1);
        assert_eq!(path, "motto");
    }
}
