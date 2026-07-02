//! Build the struct-JSON `Output` tree from parsed annotated-prose markdown,
//! for both edition kinds:
//!
//! - **Source** (two-layer): modernized blocks (→ text/html) paired with
//!   reviewed blocks (→ original_text/original_html), split with the German
//!   sentence splitter.
//! - **Translation** (single-layer): translated blocks only, split with the
//!   English splitter; node labels come from the corpus's English TOC (kant1)
//!   or the translated files' front matter (kant3).
//!
//! Footnotes are collected in a first pass (global numbering), attached to the
//! sentence whose rendered HTML carries their `<sup>N</sup>` ref, and filtered
//! out of the block stream. Page markers are stripped from the rendered text
//! and resolved to the sentence they fall in.

use common::sentences::{
    RUN_BREAK, split_sentences_en_forced, split_sentences_forced, strip_forced_split_markers,
    strip_forced_splits, strip_forced_splits_keep_runs, take_run_marker,
};
use regex::Regex;
use std::collections::HashMap;
use std::sync::LazyLock;
use text_struct::parse::resolve_marker_to_sentence;

use crate::corpus::Corpus;
use crate::figure::build_figure_block;
use crate::html::{FOOTNOTE_REF_RE, md_to_html, md_to_plain};
use crate::model::*;
use crate::parse::{MarkerKind, ParsedBlock, ParsedBlockType, strip_markers};
use crate::roman::roman_to_int;
use crate::separator::build_separator_block;

/// Regex to find `<sup>NUMBER</sup>` in rendered HTML (only footnote refs produce these).
static SUP_NUMBER_RE: LazyLock<Regex> = LazyLock::new(|| Regex::new(r"<sup>(\d+)</sup>").unwrap());

/// A TOC entry with an owned label: (flat_index, aa_page, depth, label,
/// slug_override). Source mode borrows from the static TOC tables; translation
/// mode may carry labels captured from front matter, so the label is owned.
pub type Entry = (usize, u16, u16, String, Option<&'static str>);

/// Intermediate per-file parsed data.
pub struct ParsedFile {
    pub flat_index: usize,
    /// Primary-layer blocks (modernized for a source edition, translated for a
    /// translation edition) → text/html.
    pub blocks: Vec<ParsedBlock>,
    /// Reviewed-layer blocks → original_text/original_html; `None` for a
    /// translation edition (single-layer).
    pub original_blocks: Option<Vec<ParsedBlock>>,
    /// English label captured from the file's front matter — only used by a
    /// translation edition without an English TOC (kant3).
    pub english_label: Option<String>,
}

/// Collected footnote content keyed by (flat_index, marker).
struct FootnoteContent {
    text: String,
    original_text: Option<String>,
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

/// Forced-split-aware sentence splitter: (plain, html, forced positions) →
/// (text, html) pairs. `split_sentences_forced` (German) or
/// `split_sentences_en_forced` (English), picked per edition.
type Splitter = fn(&str, &str, &[usize]) -> Vec<(String, String)>;

/// The genre knobs `build_block` needs, extracted from the corpus + mode so
/// the block builder stays testable without a full corpus.
struct BlockCtx<'a> {
    /// Sentence splitter for the edition's language (German for source,
    /// English for translation).
    splitter: Splitter,
    figure_label: &'a str,
    aa_system_slug: &'a str,
    edition_system_slug: &'a str,
    edition_sort_arabic_fallback: bool,
}

/// Build the complete nested output from parsed files.
pub fn build_output(corpus: &Corpus, translation: bool, parsed_files: &[ParsedFile]) -> Output {
    // === Pass 1: Collect footnotes, assign global numbers ===
    let mut footnote_counter: i32 = 0;
    let mut marker_map: HashMap<(usize, String), i32> = HashMap::new();
    let mut footnote_content: HashMap<(usize, String), FootnoteContent> = HashMap::new();
    let mut number_to_key: HashMap<i32, (usize, String)> = HashMap::new();

    for pf in parsed_files {
        for (i, block) in pf.blocks.iter().enumerate() {
            if let ParsedBlockType::Footnote { marker } = &block.block_type {
                footnote_counter += 1;
                let key = (pf.flat_index, marker.clone());
                marker_map.insert(key.clone(), footnote_counter);
                number_to_key.insert(footnote_counter, key.clone());
                footnote_content.insert(
                    key,
                    FootnoteContent {
                        text: block.text.clone(),
                        original_text: pf.original_blocks.as_ref().map(|ob| ob[i].text.clone()),
                    },
                );
            }
        }
    }

    // === Pass 2: Build toc nodes, filtering out footnote blocks ===
    // Node labels: modernized TOC for a source edition; English TOC (kant1) or
    // captured front-matter labels over the German structure (kant3) for a
    // translation edition.
    let entries: Vec<Entry> = if !translation {
        to_owned_entries(&corpus.toc_modernized)
    } else if let Some(en) = &corpus.toc_en {
        to_owned_entries(&en.entries)
    } else {
        let en_labels: HashMap<usize, &str> = parsed_files
            .iter()
            .filter_map(|pf| {
                pf.english_label
                    .as_ref()
                    .map(|l| (pf.flat_index, l.as_str()))
            })
            .collect();
        corpus
            .toc_modernized
            .iter()
            .map(|&(idx, aa, depth, de_label, slug_override)| {
                let label = en_labels
                    .get(&idx)
                    .map(|s| s.to_string())
                    .unwrap_or_else(|| de_label.to_string());
                (idx, aa, depth, label, slug_override)
            })
            .collect()
    };

    let ctx = BlockCtx {
        splitter: if translation {
            split_sentences_en_forced
        } else {
            split_sentences_forced
        },
        figure_label: corpus.figure_label,
        aa_system_slug: corpus.aa_system_slug,
        edition_system_slug: corpus.edition_system_slug,
        edition_sort_arabic_fallback: corpus.edition_sort_arabic_fallback,
    };
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
    let position_number = corpus.position_number;

    let toc_nodes: Vec<TocNodeData> = parsed_files
        .iter()
        .map(|pf| {
            let (_, _, depth, label, slug_override) = &entries[pf.flat_index];
            let depth = *depth;
            let parent_source_ref =
                find_parent_source_ref(&entries, pf.flat_index, depth, position_number);
            let path = build_path(&entries, pf.flat_index, depth, corpus.slugify);

            let content_blocks: Vec<ContentBlockData> = pf
                .blocks
                .iter()
                .enumerate()
                .filter(|(_, block)| {
                    // Skip footnote blocks — they are now attached to sentences
                    !matches!(&block.block_type, ParsedBlockType::Footnote { .. })
                })
                .enumerate()
                .map(|(block_pos, (i, block))| {
                    let original_block = pf.original_blocks.as_ref().map(|ob| &ob[i]);
                    build_block(
                        block,
                        original_block,
                        block_pos,
                        pf.flat_index,
                        &mut counters,
                        &lookups,
                        &ctx,
                    )
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
                    .unwrap_or_else(|| (corpus.slugify)(&plain_label))
                    .replace('_', "-"),
                path,
                sort_order: position_number(pf.flat_index) as i32,
                depth: depth as i16,
                label: plain_label,
                label_html: html_label,
                parent_source_ref,
                source: None,
                content_blocks,
            }
        })
        .collect();

    Output {
        book: corpus.book.clone(),
        reference_systems: corpus.reference_systems.clone(),
        toc_nodes,
    }
}

fn to_owned_entries(entries: &[crate::corpus::FlatEntry]) -> Vec<Entry> {
    entries
        .iter()
        .map(|&(idx, aa, depth, label, slug_override)| {
            (idx, aa, depth, label.to_string(), slug_override)
        })
        .collect()
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
    original_block: Option<&ParsedBlock>,
    block_pos: usize,
    flat_index: usize,
    counters: &mut Counters,
    lookups: &Lookups<'_>,
    ctx: &BlockCtx<'_>,
) -> ContentBlockData {
    if let ParsedBlockType::Figure = &block.block_type {
        let n = counters.figure;
        counters.figure += 1;
        return build_figure_block(
            block,
            original_block,
            block_pos,
            flat_index,
            n,
            ctx.figure_label,
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

    // Primary layer. Page markers ride through md_to_plain/md_to_html inert
    // (no markdown chars), so we strip them off the RENDERED text rather than
    // the raw markdown — that way each marker's recorded offset is already in
    // plain-text coordinates (the space the sentence offsets live in).
    // RUN_BREAK sentinels (from `+ ` run markers) are kept through splitting so
    // each indented run's first sentence can be detected and tagged below;
    // they're stripped from the stored text.
    let (plain_no_markers, mut page_markers) = strip_markers(&md_to_plain(&rewritten_text));
    let (block_plain_tok, forced_splits) = strip_forced_splits_keep_runs(&plain_no_markers);
    let (html_no_markers, _) = strip_markers(&md_to_html(&rewritten_text));
    let block_html_tok = strip_forced_split_markers(&html_no_markers);
    let sentence_pairs = (ctx.splitter)(&block_plain_tok, &block_html_tok, &forced_splits);

    // `strip_forced_splits_keep_runs` deleted each `|||` (3 chars) from the
    // plain text; shift any marker that sat after one back into block_plain_tok
    // coordinates. (RUN_BREAK is kept, so it needs no adjustment.)
    for marker in &mut page_markers {
        let prefix: String = plain_no_markers.chars().take(marker.char_offset).collect();
        marker.char_offset -= prefix.matches("|||").count() * 3;
    }

    // Original (reviewed) layer, when present — split at its OWN run/forced
    // positions so the sentinels land correctly despite spelling differences
    // from the modernized text. Markers are stripped (offsets unused — page
    // markers are positioned by the primary text). The sentence-count check
    // below guards alignment.
    let orig = original_block.map(|ob| {
        let rewritten_orig = rewrite_footnote_refs(&ob.text, flat_index, lookups.marker_map);
        let (orig_plain_no_markers, _) = strip_markers(&md_to_plain(&rewritten_orig));
        let (orig_plain_tok, orig_forced) = strip_forced_splits_keep_runs(&orig_plain_no_markers);
        let (orig_html_no_markers, _) = strip_markers(&md_to_html(&rewritten_orig));
        let orig_html_tok = strip_forced_split_markers(&orig_html_no_markers);
        let orig_sentence_pairs = (ctx.splitter)(&orig_plain_tok, &orig_html_tok, &orig_forced);
        (orig_plain_tok, orig_html_tok, orig_sentence_pairs)
    });

    // Stored block text/html carry no sentinels.
    let block_plain = block_plain_tok.replace(RUN_BREAK, "");
    let block_html = block_html_tok.replace(RUN_BREAK, "");
    let (orig_plain, orig_html) = match &orig {
        Some((p, h, _)) => (
            Some(p.replace(RUN_BREAK, "")),
            Some(h.replace(RUN_BREAK, "")),
        ),
        None => (None, None),
    };

    // Validate sentence counts match across the two layers
    if let Some((_, _, orig_pairs)) = &orig
        && sentence_pairs.len() != orig_pairs.len()
    {
        panic!(
            "Sentence count mismatch in file index {}, block {} ({}): modernized has {} sentences, reviewed has {}",
            flat_index,
            block_pos,
            block_type_str,
            sentence_pairs.len(),
            orig_pairs.len(),
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

    for (sent_pos, (sent_text, sent_html)) in sentence_pairs.iter().enumerate() {
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

                // Sentence-split the footnote body. A reviewed layer splits at
                // the PRIMARY layer's forced positions (the layers pair
                // sentence-for-sentence).
                let (fn_plain, fn_forced) = strip_forced_splits(&md_to_plain(&content.text));
                let fn_html = strip_forced_split_markers(&md_to_html(&content.text));
                let fn_pairs = (ctx.splitter)(&fn_plain, &fn_html, &fn_forced);

                let fn_orig_pairs: Option<Vec<(String, String)>> =
                    content.original_text.as_ref().map(|orig_text| {
                        let (fn_orig_plain, _) = strip_forced_splits(&md_to_plain(orig_text));
                        let fn_orig_html = strip_forced_split_markers(&md_to_html(orig_text));
                        (ctx.splitter)(&fn_orig_plain, &fn_orig_html, &fn_forced)
                    });

                let fn_sentences: Vec<FootnoteSentenceData> = fn_pairs
                    .iter()
                    .enumerate()
                    .map(|(pos, (ft, fh))| {
                        let (fot, foh) = match &fn_orig_pairs {
                            Some(op) => (
                                op.get(pos).map(|(t, _)| t.clone()),
                                op.get(pos).map(|(_, h)| h.clone()),
                            ),
                            None => (None, None),
                        };
                        FootnoteSentenceData {
                            position: pos as i16,
                            sentence_number: None,
                            text: ft.clone(),
                            html: fh.clone(),
                            original_text: fot,
                            original_html: foh,
                        }
                    })
                    .collect();

                Some(FootnoteData {
                    number,
                    sentences: fn_sentences,
                })
            })
            .collect();

        // A leading RUN_BREAK marks the first sentence of an indented run.
        // Tagging is driven by the primary text; the sentinel is stripped
        // from all stored strings.
        let (is_run_start, sent_text_clean) = take_run_marker(sent_text);
        if is_run_start {
            current_segment = Some(next_segment);
            next_segment += 1;
        }
        let (_, sent_html_clean) = take_run_marker(sent_html);
        let (orig_text_clean, orig_html_clean) = match &orig {
            Some((_, _, orig_pairs)) => {
                let (ot, oh) = (&orig_pairs[sent_pos].0, &orig_pairs[sent_pos].1);
                (
                    Some(take_run_marker(ot).1.to_string()),
                    Some(take_run_marker(oh).1.to_string()),
                )
            }
            None => (None, None),
        };

        sentences.push(SentenceData {
            position: sent_pos as i16,
            sentence_number: sent_num,
            segment: current_segment,
            indent: None,
            text: sent_text_clean.to_string(),
            html: sent_html_clean.to_string(),
            original_text: orig_text_clean,
            original_html: orig_html_clean,
            page_markers: Vec::new(),
            footnotes,
        });
    }

    // Assign page markers to their sentences (based on primary text positions)
    for marker in &page_markers {
        if sentences.is_empty() {
            continue;
        }

        let (sent_idx, mut char_offset_in_sentence) =
            resolve_marker_to_sentence(&cumulative_chars, marker.char_offset);

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
                (ctx.aa_system_slug, sort)
            }
            MarkerKind::BEdition => {
                let sort = roman_to_int(&marker.value)
                    .map(|v| v as i32)
                    .or_else(|| {
                        // e.g. the 1790 first edition paginates its preface in
                        // Roman numerals and its body in Arabic.
                        ctx.edition_sort_arabic_fallback
                            .then(|| marker.value.parse::<i32>().ok())
                            .flatten()
                    })
                    .unwrap_or(0);
                (ctx.edition_system_slug, sort)
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
        original_text: orig_plain,
        original_html: orig_html,
        sentences,
    }
}

/// Find parent's source_ref: nearest preceding entry with depth - 1.
fn find_parent_source_ref(
    entries: &[Entry],
    current_idx: usize,
    current_depth: u16,
    position_number: fn(usize) -> usize,
) -> Option<String> {
    if current_depth <= 1 {
        return None;
    }
    let target_depth = current_depth - 1;
    for i in (0..current_idx).rev() {
        let (_, _, d, _, _) = &entries[i];
        if *d == target_depth {
            return Some(format!("{:03}", position_number(i)));
        }
        if *d < target_depth {
            return None;
        }
    }
    None
}

/// Derive slug from a TOC entry, using override if present.
fn entry_slug(entry: &Entry, slugify: fn(&str) -> String) -> String {
    entry
        .4
        .map(|s| s.to_string())
        .unwrap_or_else(|| slugify(&md_to_plain(&entry.3)))
}

/// Build an ltree path from slugs of ancestors.
fn build_path(
    entries: &[Entry],
    current_idx: usize,
    current_depth: u16,
    slugify: fn(&str) -> String,
) -> String {
    let mut segments = vec![entry_slug(&entries[current_idx], slugify)];
    let mut depth = current_depth;
    let mut idx = current_idx;

    while depth > 1 {
        let target = depth - 1;
        let mut found = false;
        for i in (0..idx).rev() {
            let (_, _, d, _, _) = &entries[i];
            if *d == target {
                segments.push(entry_slug(&entries[i], slugify));
                idx = i;
                depth = target;
                found = true;
                break;
            }
            if *d < target {
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

#[cfg(test)]
mod tests {
    use super::*;

    fn test_ctx() -> BlockCtx<'static> {
        BlockCtx {
            splitter: split_sentences_forced,
            figure_label: "Abbildung",
            aa_system_slug: "aa_iii",
            edition_system_slug: "b_edition",
            edition_sort_arabic_fallback: false,
        }
    }

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
    fn test_build_block_separator_is_contentless_and_skips_counters() {
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

        let block = build_block(&sep, Some(&sep), 4, 0, &mut counters, &lookups, &test_ctx());

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
    fn test_build_block_single_layer_has_no_original() {
        let block = ParsedBlock {
            block_type: ParsedBlockType::Paragraph,
            text: "One sentence here.".to_string(),
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

        let out = build_block(&block, None, 0, 0, &mut counters, &lookups, &test_ctx());
        assert_eq!(out.original_text, None);
        assert_eq!(out.original_html, None);
        assert_eq!(out.sentences.len(), 1);
        assert_eq!(out.sentences[0].original_text, None);
    }

    #[test]
    fn test_build_block_tags_indented_runs() {
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
        let out = build_block(
            &block,
            Some(&block),
            0,
            0,
            &mut counters,
            &lookups,
            &test_ctx(),
        );

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
        use crate::parse::parse_blocks;

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

        let out = build_block(
            block,
            Some(block),
            0,
            0,
            &mut counters,
            &lookups,
            &test_ctx(),
        );

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
        let entries = to_owned_entries(&common::kant1::toc_mod::flat_toc_entries());
        let path = build_path(&entries, 0, 1, common::kant1::filenames::slugify);
        assert_eq!(path, "motto");
    }

    #[test]
    fn test_build_path_section_nests_under_ancestors() {
        // § 1 (index 16, depth 5) sits under 1. Moment / Erstes Buch /
        // Erster Abschnitt / Erster Teil.
        let entries = to_owned_entries(&common::kant3::toc_mod::flat_toc_entries());
        let path = build_path(&entries, 16, 5, common::kant3::filenames::slugify);
        assert_eq!(
            path,
            "erster_teil_kritik_der_aesthetischen_urteilskraft.\
erster_abschnitt_analytik_der_aesthetischen_urteilskraft.\
erstes_buch_analytik_des_schoenen.\
1_moment_des_geschmacksurteils_der_qualitaet_nach.\
1_das_geschmacksurteil_ist_aesthetisch"
        );
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
        use crate::html::{md_to_html, md_to_plain};

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
