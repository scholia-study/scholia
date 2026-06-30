use std::collections::HashMap;

use common::kant3::filenames::{position_number, slugify};
use common::kant3::toc_mod;
use common::sentences::{
    RUN_BREAK, split_sentences_en_forced, strip_forced_split_markers, strip_forced_splits,
    strip_forced_splits_keep_runs, take_run_marker,
};
use regex::Regex;
use std::sync::LazyLock;

use kant1_md_to_struct::figure::build_figure_block;
use kant1_md_to_struct::html::{FOOTNOTE_REF_RE, md_to_html, md_to_plain};
use kant1_md_to_struct::model::*;
use kant1_md_to_struct::parse::{
    MarkerKind, ParsedBlock, ParsedBlockType, RawMarker, strip_markers,
};
use kant1_md_to_struct::roman::roman_to_int;
use kant1_md_to_struct::separator::build_separator_block;

static SUP_NUMBER_RE: LazyLock<Regex> = LazyLock::new(|| Regex::new(r"<sup>(\d+)</sup>").unwrap());

/// A TOC entry with an owned (English) label. kant3 has no English TOC table —
/// labels come from the translated files' front matter — so unlike kant1 this
/// carries `String` rather than `&'static str`.
type EnEntry = (usize, u16, u16, String, Option<&'static str>);

/// Intermediate per-file parsed data (translation only, no original layer).
pub struct ParsedFile {
    pub flat_index: usize,
    pub blocks: Vec<ParsedBlock>,
    pub english_label: String,
}

struct FootnoteContent {
    text: String,
}

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

/// Build the English book output from parsed translation files. Structure
/// (depth/parent/aa_page) comes from the German TOC; labels and slugs come from
/// the translated files' front-matter English labels.
pub fn build_output(parsed_files: &[ParsedFile]) -> Output {
    let book = BookData {
        slug: "critique-of-the-power-of-judgment".to_string(),
        title: "Critique of the Power of Judgment".to_string(),
        author: "Immanuel Kant".to_string(),
        language: "en".to_string(),
        source: "Scholia Community Edition".to_string(),
        source_date: "2026".to_string(),
    };

    let reference_systems = vec![
        ReferenceSystemData {
            slug: "aa_v".to_string(),
            label: "Akademie-Ausgabe Band V".to_string(),
            ref_type: "block".to_string(),
            cite_priority: None,
            cite_template: Some("AA V {ref}".to_string()),
        },
        ReferenceSystemData {
            slug: "e1790".to_string(),
            label: "First Edition 1790".to_string(),
            ref_type: "inline".to_string(),
            cite_priority: None,
            cite_template: Some("E {ref}".to_string()),
        },
    ];

    // English flat entries: German structure + English labels (German label as
    // a fallback for any unmapped index, though normally all are present).
    let en_labels: HashMap<usize, &str> = parsed_files
        .iter()
        .map(|pf| (pf.flat_index, pf.english_label.as_str()))
        .collect();
    let en_entries: Vec<EnEntry> = toc_mod::flat_toc_entries()
        .into_iter()
        .map(|(idx, aa, depth, de_label, slug_override)| {
            let label = en_labels
                .get(&idx)
                .map(|s| s.to_string())
                .unwrap_or_else(|| de_label.to_string());
            (idx, aa, depth, label, slug_override)
        })
        .collect();

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
    let mut counters = Counters {
        paragraph: 1,
        sentence: 1,
        figure: 1,
    };
    let lookups = Lookups {
        marker_map: &marker_map,
        footnote_content: &footnote_content,
        number_to_key: &number_to_key,
    };

    let toc_nodes: Vec<TocNodeData> = parsed_files
        .iter()
        .map(|pf| {
            let (_, _, depth, label, slug_override) = &en_entries[pf.flat_index];
            let depth = *depth;
            let parent_source_ref = find_parent_source_ref(&en_entries, pf.flat_index, depth);
            let path = build_path(&en_entries, pf.flat_index, depth);

            let content_blocks: Vec<ContentBlockData> = pf
                .blocks
                .iter()
                .filter(|block| !matches!(&block.block_type, ParsedBlockType::Footnote { .. }))
                .enumerate()
                .map(|(block_pos, block)| {
                    build_block(block, block_pos, pf.flat_index, &mut counters, &lookups)
                })
                .collect();

            let plain_label = md_to_plain(label);
            let html_label = md_to_html(label);

            TocNodeData {
                source_ref: format!("{:03}", position_number(pf.flat_index)),
                // URL slug is hyphen-separated; the filename and ltree path
                // keep the underscore form from `slugify`.
                slug: slug_override
                    .map(|s| s.to_string())
                    .unwrap_or_else(|| slugify(&plain_label))
                    .replace('_', "-"),
                path,
                sort_order: position_number(pf.flat_index) as i32,
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

struct Counters {
    paragraph: i32,
    sentence: i32,
    figure: i32,
}

struct Lookups<'a> {
    marker_map: &'a HashMap<(usize, String), i32>,
    footnote_content: &'a HashMap<(usize, String), FootnoteContent>,
    number_to_key: &'a HashMap<i32, (usize, String)>,
}

fn build_block(
    block: &ParsedBlock,
    block_pos: usize,
    flat_index: usize,
    counters: &mut Counters,
    lookups: &Lookups<'_>,
) -> ContentBlockData {
    if let ParsedBlockType::Figure = &block.block_type {
        let n = counters.figure;
        counters.figure += 1;
        return build_figure_block(block, None, block_pos, flat_index, n, "Figure");
    }

    if let ParsedBlockType::Separator { dinkus } = &block.block_type {
        return build_separator_block(block_pos, *dinkus);
    }

    let (block_type_str, para_num) = match &block.block_type {
        ParsedBlockType::Heading => ("heading", None),
        ParsedBlockType::Paragraph => {
            let n = counters.paragraph;
            counters.paragraph += 1;
            ("paragraph", Some(n))
        }
        ParsedBlockType::Footnote { .. } => unreachable!("footnote blocks filtered out"),
        ParsedBlockType::Figure => unreachable!("figure blocks handled above"),
        ParsedBlockType::Separator { .. } => unreachable!("separator blocks handled above"),
    };

    let rewritten_text = rewrite_footnote_refs(&block.text, flat_index, lookups.marker_map);

    let (plain_no_markers, mut page_markers) = strip_markers(&md_to_plain(&rewritten_text));
    let (block_plain_tok, forced_splits) = strip_forced_splits_keep_runs(&plain_no_markers);
    let (html_no_markers, _) = strip_markers(&md_to_html(&rewritten_text));
    let block_html_tok = strip_forced_split_markers(&html_no_markers);
    let sentence_pairs =
        split_sentences_en_forced(&block_plain_tok, &block_html_tok, &forced_splits);

    for marker in &mut page_markers {
        let prefix: String = plain_no_markers.chars().take(marker.char_offset).collect();
        marker.char_offset -= prefix.matches("|||").count() * 3;
    }

    let block_plain = block_plain_tok.replace(RUN_BREAK, "");
    let block_html = block_html_tok.replace(RUN_BREAK, "");

    let mut sentences = Vec::new();
    let mut cumulative_chars: Vec<usize> = Vec::new();
    let mut offset: usize = 0;
    let mut current_segment: Option<i16> = None;
    let mut next_segment: i16 = 1;

    for (sent_pos, (sent_text, sent_html)) in sentence_pairs.iter().enumerate() {
        cumulative_chars.push(offset);
        let sent_char_count = sent_text.chars().count();
        offset += sent_char_count + 1;

        let sent_num = if block_type_str == "paragraph" {
            let n = counters.sentence;
            counters.sentence += 1;
            Some(n)
        } else {
            None
        };

        let footnotes: Vec<FootnoteData> = SUP_NUMBER_RE
            .captures_iter(sent_html)
            .filter_map(|caps| {
                let number: i32 = caps[1].parse().ok()?;
                let key = lookups.number_to_key.get(&number)?;
                let content = lookups.footnote_content.get(key)?;

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

        let (is_run_start, sent_text_clean) = take_run_marker(sent_text);
        if is_run_start {
            current_segment = Some(next_segment);
            next_segment += 1;
        }
        let (_, sent_html_clean) = take_run_marker(sent_html);

        sentences.push(SentenceData {
            position: sent_pos as i16,
            sentence_number: sent_num,
            segment: current_segment,
            text: sent_text_clean.to_string(),
            html: sent_html_clean.to_string(),
            original_text: None,
            original_html: None,
            page_markers: Vec::new(),
            footnotes,
        });
    }

    for marker in &page_markers {
        if sentences.is_empty() {
            continue;
        }
        let (sent_idx, mut char_offset_in_sentence) =
            resolve_marker_to_sentence(&sentence_pairs, &cumulative_chars, marker);
        if sentence_pairs[sent_idx].0.starts_with(RUN_BREAK) {
            char_offset_in_sentence = (char_offset_in_sentence - 1).max(0);
        }
        let (system_slug, sort_order) = match marker.kind {
            MarkerKind::Aa => ("aa_v", marker.value.parse::<i32>().unwrap_or(0)),
            MarkerKind::BEdition => {
                let sort = roman_to_int(&marker.value)
                    .map(|v| v as i32)
                    .or_else(|| marker.value.parse::<i32>().ok())
                    .unwrap_or(0);
                ("e1790", sort)
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
        figure_number: None,
        text: block_plain,
        html: block_html,
        original_text: None,
        original_html: None,
        sentences,
    }
}

fn find_parent_source_ref(
    flat_entries: &[EnEntry],
    current_idx: usize,
    current_depth: u16,
) -> Option<String> {
    if current_depth <= 1 {
        return None;
    }
    let target_depth = current_depth - 1;
    for i in (0..current_idx).rev() {
        let d = flat_entries[i].2;
        if d == target_depth {
            return Some(format!("{:03}", position_number(i)));
        }
        if d < target_depth {
            return None;
        }
    }
    None
}

fn entry_slug(entry: &EnEntry) -> String {
    entry
        .4
        .map(|s| s.to_string())
        .unwrap_or_else(|| slugify(&md_to_plain(&entry.3)))
}

fn build_path(flat_entries: &[EnEntry], current_idx: usize, current_depth: u16) -> String {
    let mut segments = vec![entry_slug(&flat_entries[current_idx])];
    let mut depth = current_depth;
    let mut idx = current_idx;

    while depth > 1 {
        let target = depth - 1;
        let mut found = false;
        for i in (0..idx).rev() {
            let d = flat_entries[i].2;
            if d == target {
                segments.push(entry_slug(&flat_entries[i]));
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
