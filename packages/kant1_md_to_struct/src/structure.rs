use std::collections::HashMap;

use common::kant1::filenames::{position_number, slugify};
use common::kant1::toc_mod;
use common::sentences::{
    RUN_BREAK, split_sentences_forced, strip_forced_split_markers, strip_forced_splits,
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
use kant1_md_to_struct::separator::build_separator_block;

/// Regex to find `<sup>NUMBER</sup>` in rendered HTML (only footnote refs produce these).
static SUP_NUMBER_RE: LazyLock<Regex> = LazyLock::new(|| Regex::new(r"<sup>(\d+)</sup>").unwrap());

/// Intermediate per-file parsed data.
pub struct ParsedFile {
    pub flat_index: usize,
    /// Modernized blocks (primary text/html).
    pub blocks: Vec<ParsedBlock>,
    /// Reviewed blocks (original_text/original_html).
    pub original_blocks: Vec<ParsedBlock>,
}

/// Collected footnote content (modernized + original text) keyed by (flat_index, marker).
struct FootnoteContent {
    modernized_text: String,
    original_text: String,
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
                // Not a known footnote marker — leave unchanged
                caps[0].to_string()
            }
        })
        .into_owned()
}

/// Build the complete nested output from parsed files.
pub fn build_output(parsed_files: &[ParsedFile]) -> Output {
    let book = BookData {
        slug: "kritik-der-reinen-vernunft-b".to_string(),
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
            // Citation-capable but not the default — Kant cites by sentence.
            cite_priority: None,
            cite_template: Some("AA III {ref}".to_string()),
        },
        ReferenceSystemData {
            slug: "b_edition".to_string(),
            label: "B-Auflage Seitenzahl".to_string(),
            ref_type: "inline".to_string(),
            cite_priority: None,
            cite_template: Some("B {ref}".to_string()),
        },
    ];

    // === Pass 1: Collect footnotes, assign global numbers ===
    let mut footnote_counter: i32 = 0;
    // (flat_index, marker_string) → global number
    let mut marker_map: HashMap<(usize, String), i32> = HashMap::new();
    // (flat_index, marker_string) → footnote content
    let mut footnote_content: HashMap<(usize, String), FootnoteContent> = HashMap::new();
    // global_number → (flat_index, marker_string) for reverse lookup
    let mut number_to_key: HashMap<i32, (usize, String)> = HashMap::new();

    for pf in parsed_files {
        for (block, orig_block) in pf.blocks.iter().zip(pf.original_blocks.iter()) {
            if let ParsedBlockType::Footnote { marker } = &block.block_type {
                footnote_counter += 1;
                let key = (pf.flat_index, marker.clone());
                marker_map.insert(key.clone(), footnote_counter);
                number_to_key.insert(footnote_counter, key.clone());
                footnote_content.insert(
                    key,
                    FootnoteContent {
                        modernized_text: block.text.clone(),
                        original_text: orig_block.text.clone(),
                    },
                );
            }
        }
    }

    // === Pass 2: Build toc nodes, filtering out footnote blocks ===
    let flat_entries = toc_mod::flat_toc_entries();
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
            let (_, _, depth, label, slug_override) = flat_entries[pf.flat_index];
            let parent_source_ref = find_parent_source_ref(&flat_entries, pf.flat_index, depth);
            let path = build_path(&flat_entries, pf.flat_index, depth);

            let content_blocks: Vec<ContentBlockData> = pf
                .blocks
                .iter()
                .zip(pf.original_blocks.iter())
                .filter_map(|(block, original_block)| {
                    // Skip footnote blocks — they are now attached to sentences
                    if matches!(&block.block_type, ParsedBlockType::Footnote { .. }) {
                        return None;
                    }
                    Some((block, original_block))
                })
                .enumerate()
                .map(|(block_pos, (block, original_block))| {
                    build_block(
                        block,
                        original_block,
                        block_pos,
                        pf.flat_index,
                        &mut counters,
                        &lookups,
                    )
                })
                .collect();

            let plain_label = md_to_plain(label);
            let html_label = md_to_html(label);

            TocNodeData {
                source_ref: format!("{:03}", position_number(pf.flat_index)),
                slug: slug_override
                    .map(|s| s.to_string())
                    .unwrap_or_else(|| slugify(&plain_label)),
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
    original_block: &ParsedBlock,
    block_pos: usize,
    flat_index: usize,
    counters: &mut Counters,
    lookups: &Lookups<'_>,
) -> ContentBlockData {
    if let ParsedBlockType::Figure = &block.block_type {
        let n = counters.figure;
        counters.figure += 1;
        return build_figure_block(
            block,
            Some(original_block),
            block_pos,
            flat_index,
            n,
            "Abbildung",
        );
    }

    // Separators carry no content — no sentences, no markers, no footnote
    // rewriting — so they bypass the prose pipeline like figures do.
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

    // Rewrite footnote refs in raw text before conversion
    let rewritten_text = rewrite_footnote_refs(&block.text, flat_index, lookups.marker_map);
    let rewritten_orig =
        rewrite_footnote_refs(&original_block.text, flat_index, lookups.marker_map);

    // Modernized (primary). Page markers ride through md_to_plain/md_to_html
    // inert (no markdown chars), so we strip them off the RENDERED text rather
    // than the raw markdown — that way each marker's recorded offset is already
    // in plain-text coordinates (the space the sentence offsets live in).
    // RUN_BREAK sentinels (from `+ ` run markers) are kept through splitting so
    // each indented run's first sentence can be detected and tagged below;
    // they're stripped from the stored text.
    let (plain_no_markers, mut page_markers) = strip_markers(&md_to_plain(&rewritten_text));
    let (block_plain_tok, forced_splits) = strip_forced_splits_keep_runs(&plain_no_markers);
    let (html_no_markers, _) = strip_markers(&md_to_html(&rewritten_text));
    let block_html_tok = strip_forced_split_markers(&html_no_markers);
    let sentence_pairs = split_sentences_forced(&block_plain_tok, &block_html_tok, &forced_splits);

    // `strip_forced_splits_keep_runs` deleted each `|||` (3 chars) from the
    // plain text; shift any marker that sat after one back into block_plain_tok
    // coordinates. (RUN_BREAK is kept, so it needs no adjustment.)
    for marker in &mut page_markers {
        let prefix: String = plain_no_markers.chars().take(marker.char_offset).collect();
        marker.char_offset -= prefix.matches("|||").count() * 3;
    }

    // Original (reviewed) — split at its OWN run/forced positions so the
    // sentinels land correctly despite spelling differences from the
    // modernized text. Markers are stripped (offsets unused — page markers are
    // positioned by the modernized text). The sentence-count check below guards
    // alignment.
    let (orig_plain_no_markers, _) = strip_markers(&md_to_plain(&rewritten_orig));
    let (orig_plain_tok, orig_forced) = strip_forced_splits_keep_runs(&orig_plain_no_markers);
    let (orig_html_no_markers, _) = strip_markers(&md_to_html(&rewritten_orig));
    let orig_html_tok = strip_forced_split_markers(&orig_html_no_markers);
    let orig_sentence_pairs = split_sentences_forced(&orig_plain_tok, &orig_html_tok, &orig_forced);

    // Stored block text/html carry no sentinels.
    let block_plain = block_plain_tok.replace(RUN_BREAK, "");
    let block_html = block_html_tok.replace(RUN_BREAK, "");
    let orig_plain = orig_plain_tok.replace(RUN_BREAK, "");
    let orig_html = orig_html_tok.replace(RUN_BREAK, "");

    // Validate sentence counts match
    if sentence_pairs.len() != orig_sentence_pairs.len() {
        panic!(
            "Sentence count mismatch in file index {}, block {} ({}): modernized has {} sentences, reviewed has {}",
            position_number(flat_index),
            block_pos,
            block_type_str,
            sentence_pairs.len(),
            orig_sentence_pairs.len(),
        );
    }

    // Build sentences with cumulative char tracking for marker resolution.
    // `current_segment` carries the active indented-run index forward across
    // a run's sentences; a leading RUN_BREAK opens the next run.
    let mut sentences = Vec::new();
    let mut cumulative_chars: Vec<usize> = Vec::new();
    let mut offset: usize = 0;
    let mut current_segment: Option<i16> = None;
    let mut next_segment: i16 = 1;

    for (sent_pos, ((sent_text, sent_html), (orig_sent_text, orig_sent_html))) in sentence_pairs
        .iter()
        .zip(orig_sentence_pairs.iter())
        .enumerate()
    {
        cumulative_chars.push(offset);
        let sent_char_count = sent_text.chars().count();
        offset += sent_char_count + 1; // +1 for space between sentences

        let sent_num = if block_type_str == "paragraph" {
            let n = counters.sentence;
            counters.sentence += 1;
            Some(n)
        } else {
            None
        };

        // Scan sentence HTML for <sup>N</sup> to attach footnotes
        let footnotes: Vec<FootnoteData> = SUP_NUMBER_RE
            .captures_iter(sent_html)
            .filter_map(|caps| {
                let number: i32 = caps[1].parse().ok()?;
                let key = lookups.number_to_key.get(&number)?;
                let content = lookups.footnote_content.get(key)?;

                // Sentence-split the footnote body
                let (fn_plain, fn_forced) =
                    strip_forced_splits(&md_to_plain(&content.modernized_text));
                let fn_html = strip_forced_split_markers(&md_to_html(&content.modernized_text));
                let fn_pairs = split_sentences_forced(&fn_plain, &fn_html, &fn_forced);

                let (fn_orig_plain, _) = strip_forced_splits(&md_to_plain(&content.original_text));
                let fn_orig_html = strip_forced_split_markers(&md_to_html(&content.original_text));
                let fn_orig_pairs =
                    split_sentences_forced(&fn_orig_plain, &fn_orig_html, &fn_forced);

                let fn_sentences: Vec<FootnoteSentenceData> = fn_pairs
                    .iter()
                    .zip(fn_orig_pairs.iter())
                    .enumerate()
                    .map(|(pos, ((ft, fh), (fot, foh)))| FootnoteSentenceData {
                        position: pos as i16,
                        sentence_number: None,
                        text: ft.clone(),
                        html: fh.clone(),
                        original_text: Some(fot.clone()),
                        original_html: Some(foh.clone()),
                    })
                    .collect();

                Some(FootnoteData {
                    number,
                    sentences: fn_sentences,
                })
            })
            .collect();

        // A leading RUN_BREAK marks the first sentence of an indented run.
        // Tagging is driven by the modernized text; the sentinel is stripped
        // from all four stored strings.
        let (is_run_start, sent_text_clean) = take_run_marker(sent_text);
        if is_run_start {
            current_segment = Some(next_segment);
            next_segment += 1;
        }
        let (_, sent_html_clean) = take_run_marker(sent_html);
        let (_, orig_text_clean) = take_run_marker(orig_sent_text);
        let (_, orig_html_clean) = take_run_marker(orig_sent_html);

        sentences.push(SentenceData {
            position: sent_pos as i16,
            sentence_number: sent_num,
            segment: current_segment,
            text: sent_text_clean.to_string(),
            html: sent_html_clean.to_string(),
            original_text: Some(orig_text_clean.to_string()),
            original_html: Some(orig_html_clean.to_string()),
            page_markers: Vec::new(),
            footnotes,
        });
    }

    // Assign page markers to their sentences (based on modernized text positions)
    for marker in &page_markers {
        if sentences.is_empty() {
            continue;
        }

        let (sent_idx, mut char_offset_in_sentence) =
            resolve_marker_to_sentence(&sentence_pairs, &cumulative_chars, marker);

        // The offset is measured against block_plain_tok, where a run's first
        // sentence still carries its leading RUN_BREAK sentinel; the stored text
        // has it stripped (take_run_marker), so shift markers in that sentence
        // back by the one sentinel char.
        if sentence_pairs[sent_idx].0.starts_with(RUN_BREAK) {
            char_offset_in_sentence = (char_offset_in_sentence - 1).max(0);
        }

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
        figure_number: None,
        text: block_plain,
        html: block_html,
        original_text: Some(orig_plain),
        original_html: Some(orig_html),
        sentences,
    }
}

/// Find parent's source_ref: nearest preceding entry with depth - 1.
fn find_parent_source_ref(
    flat_entries: &[(usize, u16, u16, &str, Option<&str>)],
    current_idx: usize,
    current_depth: u16,
) -> Option<String> {
    if current_depth <= 1 {
        return None;
    }
    let target_depth = current_depth - 1;
    for i in (0..current_idx).rev() {
        let (_, _, d, _, _) = flat_entries[i];
        if d == target_depth {
            return Some(format!("{:03}", position_number(i)));
        }
        if d < target_depth {
            return None;
        }
    }
    None
}

/// Derive slug from a TOC entry, using override if present.
fn entry_slug(entry: &(usize, u16, u16, &str, Option<&str>)) -> String {
    entry
        .4
        .map(|s| s.to_string())
        .unwrap_or_else(|| slugify(&md_to_plain(entry.3)))
}

/// Build an ltree path from slugs of ancestors.
fn build_path(
    flat_entries: &[(usize, u16, u16, &str, Option<&str>)],
    current_idx: usize,
    current_depth: u16,
) -> String {
    let mut segments = vec![entry_slug(&flat_entries[current_idx])];
    let mut depth = current_depth;
    let mut idx = current_idx;

    while depth > 1 {
        let target = depth - 1;
        let mut found = false;
        for i in (0..idx).rev() {
            let (_, _, d, _, _) = flat_entries[i];
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

use kant1_md_to_struct::roman::roman_to_int;

#[cfg(test)]
mod tests {
    use super::*;
    use kant1_md_to_struct::parse::{MarkerKind, RawMarker};

    #[test]
    fn marker_offset_is_plain_coordinate_inside_emphasis() {
        // A page marker sitting inside a Sperrdruck span must be recorded at its
        // plain-text offset — not shifted by the `***` syntax that md_to_plain
        // removes. Regression test for the markdown/plain coordinate bug.
        let (plain, markers) = strip_markers(&md_to_plain("x ***a {{ 5 }} b*** y"));
        assert_eq!(plain, "x a b y");
        assert_eq!(markers.len(), 1);
        assert_eq!(markers[0].value, "5");
        // "x a " is 4 chars → marker sits before "b", not at 7 (the *** drift).
        assert_eq!(markers[0].char_offset, 4);
    }

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
            (
                "Second sentence.".to_string(),
                "Second sentence.".to_string(),
            ),
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
    fn test_build_block_separator_is_contentless_and_skips_counters() {
        use kant1_md_to_struct::parse::{ParsedBlock, ParsedBlockType};

        let sep = ParsedBlock {
            block_type: ParsedBlockType::Separator { dinkus: true },
            text: String::new(),
            markers: Vec::new(),
        };

        let marker_map = HashMap::new();
        let footnote_content = HashMap::new();
        let number_to_key = HashMap::new();
        let lookups = Lookups {
            marker_map: &marker_map,
            footnote_content: &footnote_content,
            number_to_key: &number_to_key,
        };
        let mut counters = Counters {
            paragraph: 1,
            sentence: 1,
            figure: 1,
        };

        let block = build_block(&sep, &sep, 4, 0, &mut counters, &lookups);

        assert_eq!(block.block_type, "separator");
        assert_eq!(block.position, 4);
        assert!(block.sentences.is_empty());
        assert_eq!(block.paragraph_number, None);
        assert_eq!(block.figure_number, None);
        assert!(block.html.contains("dinkus"));
        // A divider must not consume a paragraph/sentence/figure number, or
        // numbering would jump across the break.
        assert_eq!(counters.paragraph, 1);
        assert_eq!(counters.sentence, 1);
        assert_eq!(counters.figure, 1);
    }

    #[test]
    fn test_build_block_tags_indented_runs() {
        use kant1_md_to_struct::parse::{ParsedBlock, ParsedBlockType};

        // Intro flow, then two `+ ` runs (as parse_blocks would emit them).
        let text = format!(
            "Intro flow here. {RUN_BREAK}1) First item is here. {RUN_BREAK}2) Second item here."
        );
        let block = ParsedBlock {
            block_type: ParsedBlockType::Paragraph,
            text,
            markers: Vec::new(),
        };

        let marker_map = HashMap::new();
        let footnote_content = HashMap::new();
        let number_to_key = HashMap::new();
        let lookups = Lookups {
            marker_map: &marker_map,
            footnote_content: &footnote_content,
            number_to_key: &number_to_key,
        };
        let mut counters = Counters {
            paragraph: 1,
            sentence: 1,
            figure: 1,
        };

        // Pass the same block as modernized + original (identical run structure).
        let out = build_block(&block, &block, 0, 0, &mut counters, &lookups);

        assert_eq!(out.block_type, "paragraph");
        assert_eq!(out.paragraph_number, Some(1)); // one true paragraph, one number
        assert_eq!(out.sentences.len(), 3);

        // Intro is normal flow; each `+ ` item is its own indented run.
        assert_eq!(out.sentences[0].segment, None);
        assert_eq!(out.sentences[1].segment, Some(1));
        assert_eq!(out.sentences[2].segment, Some(2));

        // Sentence numbers stay continuous across the whole paragraph.
        assert_eq!(out.sentences[0].sentence_number, Some(1));
        assert_eq!(out.sentences[2].sentence_number, Some(3));

        // The sentinel never reaches stored text/html.
        assert!(out.sentences[1].text.starts_with("1)"));
        assert!(!out.sentences.iter().any(|s| s.text.contains(RUN_BREAK)));
        assert!(!out.sentences.iter().any(|s| s.html.contains(RUN_BREAK)));
        assert!(!out.text.contains(RUN_BREAK));
        assert!(!out.html.contains(RUN_BREAK));
    }

    #[test]
    fn test_build_block_nihil_enumeration_end_to_end() {
        use kant1_md_to_struct::parse::parse_blocks;

        // The nihil passage shape: intro flow, then four `+ ` runs carrying
        // page markers ({{ N }}), Sperrdruck (***x***) and antiqua (_x_).
        let body = "\
Weil die Kategorien fortgehen.
+ 1) {{ 347 }} Den Begriffen ist ***Keines*** entgegengesetzt (_ens rationis_).
+ 2) Realität ist ***Etwas***, Negation ist ***Nichts*** (_nihil privativum_).
+ 3) Die bloße Form der Anschauung (_ens imaginarium_).
+ 4) Der Gegenstand {{ 348 }} ist Nichts (_nihil negativum_).
";
        let blocks = parse_blocks(body);
        assert_eq!(blocks.len(), 1, "must stay a single paragraph block");
        let block = &blocks[0];

        let marker_map = HashMap::new();
        let footnote_content = HashMap::new();
        let number_to_key = HashMap::new();
        let lookups = Lookups {
            marker_map: &marker_map,
            footnote_content: &footnote_content,
            number_to_key: &number_to_key,
        };
        let mut counters = Counters {
            paragraph: 1,
            sentence: 1,
            figure: 1,
        };

        let out = build_block(block, block, 0, 0, &mut counters, &lookups);

        // Intro + four items, each item its own indented run.
        assert_eq!(out.sentences.len(), 5);
        let segs: Vec<_> = out.sentences.iter().map(|s| s.segment).collect();
        assert_eq!(segs, vec![None, Some(1), Some(2), Some(3), Some(4)]);

        // Markup survives inside a run; the sentinel never does.
        assert!(out.sentences[1].html.contains("sperrdruck"));
        assert!(out.sentences[1].html.contains("antiqua"));
        assert!(out.sentences[1].text.starts_with("1)"));
        assert!(!out.sentences.iter().any(|s| s.text.contains(RUN_BREAK)));
        assert!(!out.sentences.iter().any(|s| s.html.contains(RUN_BREAK)));

        // Page markers land on the runs that carry them.
        assert!(
            out.sentences[1]
                .page_markers
                .iter()
                .any(|m| m.ref_value == "347")
        );
        assert!(
            out.sentences[4]
                .page_markers
                .iter()
                .any(|m| m.ref_value == "348")
        );
    }

    #[test]
    fn test_build_path_depth1() {
        let entries = toc_mod::flat_toc_entries();
        let path = build_path(&entries, 0, 1);
        assert_eq!(path, "motto");
    }

    #[test]
    fn test_rewrite_footnote_refs() {
        let mut marker_map = HashMap::new();
        marker_map.insert((0, "*".to_string()), 1);
        marker_map.insert((0, "**".to_string()), 2);

        let result = rewrite_footnote_refs("text[^*] more[^**] end", 0, &marker_map);
        assert_eq!(result, "text[^1] more[^2] end");
    }

    #[test]
    fn test_rewrite_footnote_refs_unknown_marker() {
        let marker_map = HashMap::new();
        let result = rewrite_footnote_refs("text[^*] end", 0, &marker_map);
        assert_eq!(result, "text[^*] end");
    }

    #[test]
    fn test_label_plain_and_html() {
        use kant1_md_to_struct::html::{md_to_html, md_to_plain};

        // Plain label — no formatting
        let label = "Vorrede zur zweiten Auflage";
        assert_eq!(md_to_plain(label), "Vorrede zur zweiten Auflage");
        assert_eq!(md_to_html(label), "Vorrede zur zweiten Auflage");

        // Label with italic
        let label = "Von den _Ideen_ überhaupt";
        assert_eq!(md_to_plain(label), "Von den Ideen überhaupt");
        assert_eq!(
            md_to_html(label),
            "Von den <span class=\"antiqua\">Ideen</span> überhaupt"
        );

        // Label with bold
        let label = "**Einleitung** zur Kritik";
        assert_eq!(md_to_plain(label), "Einleitung zur Kritik");
        assert_eq!(md_to_html(label), "<b>Einleitung</b> zur Kritik");

        // Label with Sperrdruck
        let label = "***Ästhetik*** und Logik";
        assert_eq!(md_to_plain(label), "Ästhetik und Logik");
        assert_eq!(
            md_to_html(label),
            "<span class=\"sperrdruck\">Ästhetik</span> und Logik"
        );
    }
}
