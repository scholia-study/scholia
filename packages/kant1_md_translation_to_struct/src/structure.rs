use std::collections::HashMap;

use common::kant1::filenames::slugify;
use common::kant1::toc_en;
use common::sentences::{
    split_sentences_en_forced, strip_forced_split_markers, strip_forced_splits,
};
use regex::Regex;
use std::sync::LazyLock;

use kant1_md_to_struct::html::{md_to_html, md_to_plain, FOOTNOTE_REF_RE};
use kant1_md_to_struct::model::*;
use kant1_md_to_struct::parse::{MarkerKind, ParsedBlock, ParsedBlockType, RawMarker};
use kant1_md_to_struct::roman::roman_to_int;

/// Regex to find `<sup>NUMBER</sup>` in rendered HTML.
static SUP_NUMBER_RE: LazyLock<Regex> =
    LazyLock::new(|| Regex::new(r"<sup>(\d+)</sup>").unwrap());

/// Intermediate per-file parsed data (translation only, no original layer).
pub struct ParsedFile {
    pub flat_index: usize,
    pub blocks: Vec<ParsedBlock>,
}

/// Collected footnote content keyed by (flat_index, marker).
struct FootnoteContent {
    text: String,
}

/// Rewrite footnote references in raw markdown text: `[^*]` → `[^1]` etc.
fn rewrite_footnote_refs(
    text: &str,
    flat_index: usize,
    marker_map: &HashMap<(usize, String), i32>,
) -> String {
    FOOTNOTE_REF_RE
        .replace_all(text, |caps: &regex::Captures| {
            let marker = &caps[1];
            if let Some(&number) = marker_map.get(&(flat_index, marker.to_string())) {
                format!("[^{number}]")
            } else {
                caps[0].to_string()
            }
        })
        .into_owned()
}

/// Build the complete nested output from parsed translation files.
pub fn build_output(parsed_files: &[ParsedFile]) -> Output {
    let book = BookData {
        slug: "critique-of-pure-reason-b".to_string(),
        title: "Critique of Pure Reason".to_string(),
        author: "Immanuel Kant".to_string(),
        language: "en".to_string(),
        source: "Scholia Community Edition".to_string(),
        source_date: "2026".to_string(),
    };

    let reference_systems = vec![
        ReferenceSystemData {
            slug: "aa_iii".to_string(),
            label: "Akademie-Ausgabe Band III".to_string(),
            ref_type: "block".to_string(),
        },
        ReferenceSystemData {
            slug: "b_edition".to_string(),
            label: "B-Edition Page Number".to_string(),
            ref_type: "inline".to_string(),
        },
    ];

    // === Pass 1: Collect footnotes, assign global numbers ===
    let mut footnote_counter: i32 = 0;
    let mut marker_map: HashMap<(usize, String), i32> = HashMap::new();
    let mut footnote_content: HashMap<(usize, String), FootnoteContent> = HashMap::new();
    let mut number_to_key: HashMap<i32, (usize, String)> = HashMap::new();

    for pf in parsed_files {
        for block in &pf.blocks {
            if let ParsedBlockType::Footnote { marker } = &block.block_type {
                footnote_counter += 1;
                let key = (pf.flat_index, marker.clone());
                marker_map.insert(key.clone(), footnote_counter);
                number_to_key.insert(footnote_counter, key.clone());
                footnote_content.insert(
                    key,
                    FootnoteContent {
                        text: block.text.clone(),
                    },
                );
            }
        }
    }

    // === Pass 2: Build toc nodes, filtering out footnote blocks ===
    let flat_entries = toc_en::flat_toc_entries_en();
    let mut paragraph_counter: i32 = 1;
    let mut sentence_counter: i32 = 1;

    let toc_nodes: Vec<TocNodeData> = parsed_files
        .iter()
        .map(|pf| {
            let (_, _, depth, label) = flat_entries[pf.flat_index];
            let parent_source_ref = find_parent_source_ref(&flat_entries, pf.flat_index, depth);
            let path = build_path(&flat_entries, pf.flat_index, depth);

            let content_blocks: Vec<ContentBlockData> = pf
                .blocks
                .iter()
                .filter(|block| !matches!(&block.block_type, ParsedBlockType::Footnote { .. }))
                .enumerate()
                .map(|(block_pos, block)| {
                    build_block(
                        block,
                        block_pos,
                        pf.flat_index,
                        &mut paragraph_counter,
                        &mut sentence_counter,
                        &marker_map,
                        &footnote_content,
                        &number_to_key,
                    )
                })
                .collect();

            let plain_label = md_to_plain(label);
            let html_label = md_to_html(label);

            TocNodeData {
                source_ref: format!("{:03}", pf.flat_index + 1),
                slug: slugify(&plain_label),
                path,
                sort_order: pf.flat_index as i32 + 1,
                depth: depth as i16,
                label: plain_label,
                label_html: html_label,
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
    flat_index: usize,
    paragraph_counter: &mut i32,
    sentence_counter: &mut i32,
    marker_map: &HashMap<(usize, String), i32>,
    footnote_content: &HashMap<(usize, String), FootnoteContent>,
    number_to_key: &HashMap<i32, (usize, String)>,
) -> ContentBlockData {
    let (block_type_str, para_num) = match &block.block_type {
        ParsedBlockType::Heading => ("heading", None),
        ParsedBlockType::Paragraph => {
            let n = *paragraph_counter;
            *paragraph_counter += 1;
            ("paragraph", Some(n))
        }
        ParsedBlockType::Footnote { .. } => unreachable!("footnote blocks filtered out"),
    };

    // Rewrite footnote refs in raw text before conversion
    let rewritten_text = rewrite_footnote_refs(&block.text, flat_index, marker_map);

    let (block_plain, forced_splits) = strip_forced_splits(&md_to_plain(&rewritten_text));
    let block_html = strip_forced_split_markers(&md_to_html(&rewritten_text));
    let sentence_pairs = split_sentences_en_forced(&block_plain, &block_html, &forced_splits);

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

        // Scan sentence HTML for <sup>N</sup> to attach footnotes
        let footnotes: Vec<FootnoteData> = SUP_NUMBER_RE
            .captures_iter(sent_html)
            .filter_map(|caps| {
                let number: i32 = caps[1].parse().ok()?;
                let key = number_to_key.get(&number)?;
                let content = footnote_content.get(key)?;

                // Sentence-split the footnote body
                let (fn_plain, fn_forced) = strip_forced_splits(&md_to_plain(&content.text));
                let fn_html = strip_forced_split_markers(&md_to_html(&content.text));
                let fn_pairs = split_sentences_en_forced(&fn_plain, &fn_html, &fn_forced);

                let fn_sentences: Vec<FootnoteSentenceData> = fn_pairs
                    .iter()
                    .enumerate()
                    .map(|(pos, (ft, fh))| FootnoteSentenceData {
                        position: pos as i16,
                        sentence_number: None,
                        text: ft.clone(),
                        html: fh.clone(),
                        original_text: None,
                        original_html: None,
                    })
                    .collect();

                Some(FootnoteData {
                    number,
                    sentences: fn_sentences,
                })
            })
            .collect();

        sentences.push(SentenceData {
            position: sent_pos as i16,
            sentence_number: sent_num,
            text: sent_text.clone(),
            html: sent_html.clone(),
            original_text: None,
            original_html: None,
            page_markers: Vec::new(),
            footnotes,
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
        original_text: None,
        original_html: None,
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
