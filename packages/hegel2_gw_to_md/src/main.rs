//! Digitale-Hegel-Edition JSON → curated hegel2 `md_reviewed` layer.
//!
//! Reads the three GW volume transcriptions (`assets/hegel2/raw/work-gw-*.js`)
//! and emits one markdown file per `common::hegel2::toc` node: the
//! Wissenschaft der Logik as GW 21 (1832 Seyn) + GW 11 pp. 233 ff. (1813
//! Wesen) + GW 12 pp. 5–253 (1816 Begriff), Nachlass Beilagen and printed
//! Inhaltsanzeigen excluded.
//!
//! Three curated tables steer the build, all refuse-to-emit on mismatch:
//! - `page_joins.tsv` — whether a paragraph continues across each page break
//!   (witness-derived: DTA for 1813/1816, the Werke text for 1832).
//! - `gw_dta_rulings.tsv` — adjudicated word-level differences against the
//!   DTA witnesses; `use_dta` rows are applied as anchored replacements.
//! - the TOC itself — every node must anchor at exactly one heading run on
//!   its own page.

use std::collections::HashMap;
use std::fs;
use std::path::{Path, PathBuf};

use clap::Parser;
use regex::Regex;

use common::hegel2::{filenames, meta, toc};

#[derive(Parser)]
#[command(about = "Convert hegeledition GW JSON to the hegel2 md_reviewed layer")]
struct Cli {
    /// Directory holding work-gw-21.js / work-gw-11.js / work-gw-12.js.
    #[arg(long, default_value = "assets/hegel2/raw")]
    input_dir: PathBuf,
    /// Which curated layer to emit.
    #[arg(long, default_value = "reviewed")]
    layer: String,
    /// Output layer directory (defaults per layer).
    #[arg(long)]
    out_dir: Option<PathBuf>,
    /// Curated tables directory (rulings, page joins).
    #[arg(long, default_value = "assets/hegel2/curated")]
    curated_dir: PathBuf,
    /// Reports directory.
    #[arg(long, default_value = "assets/hegel2/derived")]
    reports_dir: PathBuf,
}

// ---------------------------------------------------------------- raw model

#[derive(serde::Deserialize)]
struct Work {
    pages: Vec<RawPage>,
}

#[derive(serde::Deserialize)]
struct RawPage {
    #[serde(default)]
    page_kind: Option<String>,
    #[serde(default)]
    gw_page: Option<u32>,
    units: Vec<RawUnit>,
}

#[derive(serde::Deserialize)]
struct RawUnit {
    #[serde(rename = "type")]
    kind: String,
    #[serde(default)]
    text: String,
    #[serde(default)]
    footnote_id: Option<String>,
    #[serde(default)]
    part: Option<u32>,
}

fn load_work(path: &Path) -> Work {
    let s =
        fs::read_to_string(path).unwrap_or_else(|e| panic!("cannot read {}: {e}", path.display()));
    let start = s.find('{').expect("JSON start");
    let end = s.rfind('}').expect("JSON end");
    serde_json::from_str(&s[start..=end])
        .unwrap_or_else(|e| panic!("cannot parse {}: {e}", path.display()))
}

// -------------------------------------------------------------- block model

/// One flow item after page assembly. Headings keep their printed text (a
/// joined run of heading/subheading units); paragraphs carry inline
/// `{{{ vol.page }}}` markers where a page break fell inside them.
#[derive(Debug)]
enum Block {
    Heading { text: String, page: u32 },
    Para { text: String, page: u32 },
    Quote { lines: Vec<String>, page: u32 },
}

impl Block {
    fn page(&self) -> u32 {
        match self {
            Block::Heading { page, .. } | Block::Para { page, .. } | Block::Quote { page, .. } => {
                *page
            }
        }
    }
}

/// An authorial footnote: printed marker glyph dropped, parts joined.
struct Footnote {
    id: String,
    text: String,
    /// Page of the part-1 body (for reports only).
    page: u32,
}

const STOP_HEADINGS: [&str; 3] = ["BEILAGE", "ZUM ERKENNEN", "NOTIZEN ZUR VORREDE"];
const SKIP_KINDS: [&str; 5] = ["title", "title_page", "toc", "blank", "frontmatter"];

/// Set-off proposition lines the source types as heading units (the
/// judgment-form displays of the Urtheil chapter): emitted as indented
/// display lines, not headings.
const DISPLAY_LINES: [(u32, &str); 4] = [
    (61, "Das Einzelne ist Allgemein"),
    (63, "(Subject) (Prädicat)"),
    (63, "Das Einzelne ist allgemein"),
    (63, "Das Allgemeine ist einzeln"),
];

/// The one footnote whose in-text `[^fn-…]` reference the digitization lost
/// (GW 11.351): the DTA witness anchors it directly after this phrase.
const MISSING_REFS: [(u8, &str, &str); 1] = [(11, "fn-351-1", "und ein Glück")];

struct VolumeFlow {
    volume: u8,
    blocks: Vec<Block>,
    footnotes: Vec<Footnote>,
}

fn marker(volume: u8, page: u32) -> String {
    format!("{{{{{{ {volume}.{page} }}}}}}")
}

/// Assemble one volume part into a block flow: page-kind filtering, stop
/// headings, heading-run joining, witness-ruled paragraph joins with inline
/// page markers, quote runs, footnote part joining.
fn build_flow(
    work: &Work,
    volume: u8,
    start_page: Option<u32>,
    joins: &HashMap<(u8, u32), bool>,
) -> VolumeFlow {
    let mut blocks: Vec<Block> = Vec::new();
    let mut footnotes: Vec<Footnote> = Vec::new();
    // marker text still owed to the flow: set at each new page, consumed by
    // the first main-flow content of that page.
    let mut pending_marker: Option<(u32, String)>;
    let mut stop = false;

    // One gw-12 body page (GW 12.28) lost its page number in the source
    // data; infer a missing number only when its neighbours pin it exactly.
    let mut numbered: Vec<(Option<u32>, &RawPage)> =
        work.pages.iter().map(|p| (p.gw_page, p)).collect();
    for i in 0..numbered.len() {
        if numbered[i].0.is_none() && numbered[i].1.page_kind.as_deref() == Some("body") {
            let prev = (i > 0).then(|| numbered[i - 1].0).flatten();
            let next = numbered.get(i + 1).and_then(|(n, _)| *n);
            match (prev, next) {
                (Some(p), Some(n)) if n == p + 2 => numbered[i].0 = Some(p + 1),
                _ => panic!("body page without an inferable gw_page (index {i})"),
            }
        }
    }

    for (gw_page, page) in numbered {
        if stop {
            break;
        }
        let Some(gw_page) = gw_page else { continue };
        if let Some(kind) = &page.page_kind
            && SKIP_KINDS.contains(&kind.as_str())
        {
            continue;
        }
        if let Some(start) = start_page
            && gw_page < start
        {
            continue;
        }
        pending_marker = Some((gw_page, marker(volume, gw_page)));

        for unit in &page.units {
            match unit.kind.as_str() {
                "heading" | "subheading" => {
                    let text = unit.text.trim();
                    if STOP_HEADINGS
                        .iter()
                        .any(|s| text.to_uppercase().starts_with(s))
                    {
                        stop = true;
                        break;
                    }
                    let text = match pending_marker.take() {
                        Some((_, m)) => format!("{m} {text}"),
                        None => text.to_string(),
                    };
                    blocks.push(Block::Heading {
                        text,
                        page: gw_page,
                    });
                }
                "spacer" => {}
                "paragraph" | "quote" => {
                    if unit.kind == "quote" {
                        let mut line = unit.text.trim().to_string();
                        if let Some((_, m)) = pending_marker.take() {
                            line = format!("{m} {line}");
                        }
                        match blocks.last_mut() {
                            Some(Block::Quote { lines, .. }) => lines.push(line),
                            _ => blocks.push(Block::Quote {
                                lines: vec![line],
                                page: gw_page,
                            }),
                        }
                        continue;
                    }
                    let text = unit.text.trim().to_string();
                    match pending_marker.take() {
                        Some((mpage, m)) => {
                            // first paragraph of the page: witness verdict
                            // decides whether it continues the previous one.
                            let join = joins.get(&(volume, mpage)).copied();
                            let continues = matches!(
                                (join, blocks.last()),
                                (Some(true), Some(Block::Para { .. }))
                            );
                            if continues {
                                let Some(Block::Para { text: prev, .. }) = blocks.last_mut() else {
                                    unreachable!()
                                };
                                // a hyphenated word split by the page break is
                                // rejoined, the marker moved to the word
                                // boundary before it; suspended hyphenation
                                // ("Denk- und") keeps its hyphen.
                                let suspended = ["und ", "oder ", "noch ", "als "]
                                    .iter()
                                    .any(|w| text.starts_with(w));
                                // an emphasis star may close right after the
                                // hyphen ("*ob-*" + "jective"): lift it off,
                                // rejoin, and re-close the star on the
                                // fragment so partial emphasis survives.
                                let (bare, restar) = match prev.strip_suffix("-*") {
                                    Some(b) => (format!("{b}-"), true),
                                    None => (prev.clone(), false),
                                };
                                if bare.ends_with('-') && !bare.ends_with("--") && !suspended {
                                    let cut = bare.rfind(char::is_whitespace).map_or(0, |i| i + 1);
                                    let mut frag = bare[cut..bare.len() - 1].to_string();
                                    if restar {
                                        frag.push('*');
                                    }
                                    prev.truncate(cut);
                                    prev.push_str(&m);
                                    prev.push(' ');
                                    prev.push_str(&frag);
                                    prev.push_str(&text);
                                } else {
                                    prev.push(' ');
                                    prev.push_str(&m);
                                    prev.push(' ');
                                    prev.push_str(&text);
                                }
                            } else {
                                blocks.push(Block::Para {
                                    text: format!("{m} {text}"),
                                    page: gw_page,
                                });
                            }
                        }
                        None => blocks.push(Block::Para {
                            text,
                            page: gw_page,
                        }),
                    }
                }
                "author_footnote" => {
                    let id = unit
                        .footnote_id
                        .clone()
                        .unwrap_or_else(|| panic!("footnote without id on GW {volume}.{gw_page}"));
                    let text = unit.text.trim().to_string();
                    if unit.part.unwrap_or(1) > 1 {
                        let fnote = footnotes
                            .iter_mut()
                            .rev()
                            .find(|f| f.id == id)
                            .unwrap_or_else(|| panic!("continuation of unknown footnote {id}"));
                        fnote.text.push(' ');
                        fnote.text.push_str(&text);
                    } else {
                        footnotes.push(Footnote {
                            id,
                            text,
                            page: gw_page,
                        });
                    }
                }
                other => panic!("unknown unit type {other:?} on GW {volume}.{gw_page}"),
            }
        }
    }
    VolumeFlow {
        volume,
        blocks,
        footnotes,
    }
}

// ----------------------------------------------------------------- rulings

struct Rule {
    id: String,
    volume: u8,
    gw_page: u32,
    gw_text: String,
    dta_text: String,
    ctx_before: String,
    ctx_after: String,
}

fn load_rulings(path: &Path) -> Vec<Rule> {
    let mut out = Vec::new();
    let content =
        fs::read_to_string(path).unwrap_or_else(|e| panic!("cannot read {}: {e}", path.display()));
    for line in content.lines().skip(1) {
        let f: Vec<&str> = line.split('\t').collect();
        assert_eq!(f.len(), 11, "malformed rulings row: {line}");
        if f[6] != "use_dta" {
            continue;
        }
        out.push(Rule {
            id: f[0].to_string(),
            volume: f[1].parse().unwrap(),
            gw_page: f[2].parse().unwrap(),
            gw_text: f[4].to_string(),
            dta_text: f[5].to_string(),
            ctx_before: f[8].to_string(),
            ctx_after: f[9].to_string(),
        });
    }
    out
}

/// modernize_rulings.tsv: word surface → modernized spelling.
fn load_modernize(path: &Path) -> HashMap<String, String> {
    let mut out = HashMap::new();
    let content =
        fs::read_to_string(path).unwrap_or_else(|e| panic!("cannot read {}: {e}", path.display()));
    for line in content.lines().skip(1) {
        let f: Vec<&str> = line.split('\t').collect();
        assert!(f.len() >= 2, "malformed modernize row: {line}");
        out.insert(f[0].to_string(), f[1].to_string());
    }
    out
}

/// Replace each word core per the modernize table; everything else
/// (punctuation, markers, emphasis markup) passes through untouched.
fn modernize_text(text: &str, map: &HashMap<String, String>, missing: &mut Vec<String>) -> String {
    let word_re = Regex::new(r"\p{L}+").unwrap();
    word_re
        .replace_all(text, |caps: &regex::Captures| {
            let w = caps.get(0).unwrap().as_str();
            match map.get(w) {
                Some(r) => r.clone(),
                None => {
                    if w.contains("ey") || w.contains("Ey") || w.contains('ſ') {
                        missing.push(w.to_string());
                    }
                    w.to_string()
                }
            }
        })
        .into_owned()
}

fn load_joins(path: &Path) -> HashMap<(u8, u32), bool> {
    let mut out = HashMap::new();
    let content =
        fs::read_to_string(path).unwrap_or_else(|e| panic!("cannot read {}: {e}", path.display()));
    for line in content.lines().skip(1) {
        let f: Vec<&str> = line.split('\t').collect();
        assert!(f.len() >= 4, "malformed joins row: {line}");
        assert_ne!(f[2], "review", "unresolved page join: {line}");
        out.insert(
            (f[0].parse().unwrap(), f[1].parse().unwrap()),
            f[2] == "join",
        );
    }
    out
}

/// Word-level view of a text: emphasis stars and page markers dropped,
/// punctuation trimmed at word edges, each word mapped to the byte range of
/// its trimmed core in the original. Rulings are expressed in token form, so
/// matching must be blind to punctuation attachment and emphasis markup.
struct Words {
    words: Vec<String>,
    spans: Vec<(usize, usize)>,
}

fn word_view(text: &str) -> Words {
    let marker_re = Regex::new(r"\{\{\{[^}]*\}\}\}").unwrap();
    let mut skip = vec![false; text.len()];
    for m in marker_re.find_iter(text) {
        for flag in skip.iter_mut().take(m.end()).skip(m.start()) {
            *flag = true;
        }
    }
    let mut words = Vec::new();
    let mut spans = Vec::new();
    let mut cur = String::new();
    let mut cur_start = 0usize;
    let mut cur_end = 0usize;
    let mut flush = |w: &mut String, start: usize, end: usize| {
        if w.is_empty() {
            return;
        }
        words.push(std::mem::take(w));
        spans.push((start, end));
    };
    let mut in_word = false;
    for (i, ch) in text.char_indices() {
        let boundary = ch.is_whitespace() || skip[i] || ch == '*';
        if boundary {
            if in_word {
                flush(&mut cur, cur_start, cur_end);
                in_word = false;
            }
            continue;
        }
        if !in_word {
            cur_start = i;
            in_word = true;
        }
        cur.push(ch);
        cur_end = i + ch.len_utf8();
    }
    if in_word {
        flush(&mut cur, cur_start, cur_end);
    }
    // trim punctuation at the edges of each word, adjusting spans
    for (w, span) in words.iter_mut().zip(spans.iter_mut()) {
        let mut start_off = 0;
        for ch in w.chars() {
            if ch.is_alphanumeric() {
                break;
            }
            start_off += ch.len_utf8();
        }
        let mut end_off = 0;
        for ch in w.chars().rev() {
            if ch.is_alphanumeric() {
                break;
            }
            end_off += ch.len_utf8();
        }
        if start_off + end_off < w.len() {
            span.0 += start_off;
            span.1 -= end_off;
            *w = w[start_off..w.len() - end_off].to_string();
        }
    }
    let keep: Vec<bool> = words
        .iter()
        .map(|w| w.chars().any(|c| c.is_alphanumeric()))
        .collect();
    let words = words
        .into_iter()
        .zip(&keep)
        .filter(|(_, k)| **k)
        .map(|(w, _)| w)
        .collect();
    let spans = spans
        .into_iter()
        .zip(&keep)
        .filter(|(_, k)| **k)
        .map(|(s, _)| s)
        .collect();
    Words { words, spans }
}

fn pattern_words(s: &str) -> Vec<String> {
    word_view(s).words
}

/// Apply one `use_dta` rule. The ruled words are located framed with as much
/// surrounding context as it takes to pin exactly one occurrence near the
/// ruled page; context spanning a block boundary falls back to a
/// boundary-anchored search.
fn apply_rule(flow: &mut VolumeFlow, rule: &Rule) -> Result<(), String> {
    let needle = pattern_words(&rule.gw_text);

    for ctx_words in [2usize, 3, 4, 6] {
        let cb_all = pattern_words(&rule.ctx_before);
        let cb = &cb_all[cb_all.len().saturating_sub(ctx_words)..];
        let ca_all = pattern_words(&rule.ctx_after);
        let ca = &ca_all[..ctx_words.min(ca_all.len())];
        let mut pattern: Vec<String> = Vec::new();
        pattern.extend_from_slice(cb);
        pattern.extend_from_slice(&needle);
        pattern.extend_from_slice(ca);
        if pattern.is_empty() {
            return Err(format!("{}: empty pattern", rule.id));
        }

        enum Site {
            Block(usize),
            Note(usize),
        }
        let mut hits: Vec<(Site, usize)> = Vec::new();
        for (bi, block) in flow.blocks.iter().enumerate() {
            if block.page().abs_diff(rule.gw_page) > 2 {
                continue;
            }
            let Block::Para { text, .. } = block else {
                continue;
            };
            let wv = word_view(text);
            for at in find_seq(&wv.words, &pattern) {
                hits.push((Site::Block(bi), at));
            }
        }
        for (fi, f) in flow.footnotes.iter().enumerate() {
            if f.page.abs_diff(rule.gw_page) > 2 {
                continue;
            }
            let wv = word_view(&f.text);
            for at in find_seq(&wv.words, &pattern) {
                hits.push((Site::Note(fi), at));
            }
        }
        if hits.len() > 1 {
            continue;
        }
        let Some((site, at)) = hits.pop() else {
            if ctx_words == 6 {
                return apply_rule_cross_block(flow, rule);
            }
            continue;
        };

        let n_start = at + cb.len();
        let edit = |text: &str| -> Result<String, String> {
            let wv = word_view(text);
            if needle.is_empty() {
                let ins = wv.spans[n_start].0;
                return Ok(format!(
                    "{}{} {}",
                    &text[..ins],
                    rule.dta_text,
                    &text[ins..]
                ));
            }
            let start = wv.spans[n_start].0;
            let end = wv.spans[n_start + needle.len() - 1].1;
            if text[start..end].contains("{{{") {
                return Err(format!("{}: ruled span straddles a page marker", rule.id));
            }
            let replacement = pattern_words(&rule.dta_text).join(" ");
            Ok(format!("{}{}{}", &text[..start], replacement, &text[end..]))
        };
        match site {
            Site::Block(bi) => {
                let Block::Para { text, .. } = &mut flow.blocks[bi] else {
                    unreachable!()
                };
                *text = edit(text)?;
            }
            Site::Note(fi) => {
                flow.footnotes[fi].text = edit(&flow.footnotes[fi].text)?;
            }
        }
        return Ok(());
    }
    Err(format!(
        "{}: still ambiguous with 6 context words near GW {}.{}",
        rule.id, rule.volume, rule.gw_page
    ))
}

fn find_seq(haystack: &[String], pattern: &[String]) -> Vec<usize> {
    if pattern.is_empty() || haystack.len() < pattern.len() {
        return Vec::new();
    }
    (0..=haystack.len() - pattern.len())
        .filter(|&i| haystack[i..i + pattern.len()] == *pattern)
        .collect()
}

/// Fallback for sites whose context straddles a block boundary: the context
/// before ends one paragraph, the context after opens the following block. A
/// short insertion (a dropped paragraph number) prefixes the next block.
fn apply_rule_cross_block(flow: &mut VolumeFlow, rule: &Rule) -> Result<(), String> {
    let needle = pattern_words(&rule.gw_text);
    if !needle.is_empty() {
        return Err(format!(
            "{}: no in-block match and cross-block replacement unsupported near GW {}.{}",
            rule.id, rule.volume, rule.gw_page
        ));
    }
    let cb_all = pattern_words(&rule.ctx_before);
    let cb = &cb_all[cb_all.len().saturating_sub(3)..];
    let ca_all = pattern_words(&rule.ctx_after);
    let ca = &ca_all[..3.min(ca_all.len())];

    // anchor on the block that OPENS with the after-context; the before-
    // context may sit blocks away (across intervening headings).
    let mut sites: Vec<usize> = Vec::new();
    for (bi, block) in flow.blocks.iter().enumerate() {
        if block.page().abs_diff(rule.gw_page) > 2 {
            continue;
        }
        let Block::Para { text: b, .. } = block else {
            continue;
        };
        let wb = word_view(b);
        if wb.words.len() >= ca.len() && wb.words[..ca.len()] == *ca {
            sites.push(bi);
        }
    }
    if sites.len() > 1 {
        // disambiguate by the before-context: the nearest preceding
        // paragraph (headings skipped) must end with it.
        sites.retain(|&bi| {
            flow.blocks[..bi]
                .iter()
                .rev()
                .find_map(|b| match b {
                    Block::Para { text, .. } => Some(word_view(text)),
                    _ => None,
                })
                .is_some_and(|wa| {
                    wa.words.len() >= cb.len() && wa.words[wa.words.len() - cb.len()..] == *cb
                })
        });
    }
    if sites.len() != 1 {
        return Err(format!(
            "{}: cross-block fallback found {} sites near GW {}.{}",
            rule.id,
            sites.len(),
            rule.volume,
            rule.gw_page
        ));
    }
    let bi = sites[0];
    let Block::Para { text, .. } = &mut flow.blocks[bi] else {
        unreachable!()
    };
    let wv = word_view(text);
    let ins = wv.spans[0].0;
    text.insert_str(ins, &format!("{} ", rule.dta_text));
    Ok(())
}

// ------------------------------------------------------------ node assembly

fn norm_match(s: &str) -> String {
    let mut out = String::new();
    for ch in s.chars() {
        match ch {
            'ä' | 'Ä' => out.push_str("ae"),
            'ö' | 'Ö' => out.push_str("oe"),
            'ü' | 'Ü' => out.push_str("ue"),
            'ß' => out.push_str("ss"),
            c if c.is_alphanumeric() => out.extend(c.to_lowercase()),
            _ => out.push(' '),
        }
    }
    out.split_whitespace().collect::<Vec<_>>().join(" ")
}

fn strip_markers_for_match(text: &str) -> String {
    Regex::new(r"\{\{\{[^}]*\}\}\}")
        .unwrap()
        .replace_all(text, " ")
        .into_owned()
}

struct Node {
    flat_index: usize,
    heading_marker: Option<String>,
    blocks: Vec<Block>,
}

/// Anchor every TOC entry at a heading run and slice the flow.
fn assemble_nodes(flows: Vec<VolumeFlow>) -> (Vec<Node>, Vec<Footnote>, Vec<String>) {
    let entries = toc::entries();
    let mut all_blocks: Vec<(u8, Block)> = Vec::new();
    let mut all_footnotes: Vec<Footnote> = Vec::new();
    for flow in flows {
        for b in flow.blocks {
            all_blocks.push((flow.volume, b));
        }
        all_footnotes.extend(flow.footnotes);
    }

    let mut nodes: Vec<Node> = Vec::new();
    let mut anchor_report: Vec<String> = Vec::new();
    let cursor = 0usize;
    let marker_re = Regex::new(r"^\{\{\{[^}]*\}\}\}\s*").unwrap();

    for (i, entry) in entries.iter().enumerate() {
        let Some(page_ref) = entry.page else {
            // synthesized Theil container: heading-only, consumes nothing.
            nodes.push(Node {
                flat_index: i,
                heading_marker: None,
                blocks: Vec::new(),
            });
            continue;
        };
        let (vol_s, page_s) = page_ref.split_once('.').expect("vol.page");
        let vol: u8 = vol_s.parse().unwrap();
        let page: u32 = page_s.parse().unwrap();
        // One anchor override: the Wesen Buch's printed body heading reads
        // "Zweytes Buch. Das Wesen" while its label follows the 1813
        // half-title.
        let want = match page_ref {
            "11.241" => "zweytes buch das wesen".to_string(),
            _ => norm_match(entry.label),
        };
        let prefix_only = false;

        // A printed heading may span several consecutive heading units
        // ("ERSTES BUCH." + "DIE LEHRE VOM SEYN."): match the label against
        // runs of 1..=4 units, smallest run first.
        let mut found: Option<(usize, usize)> = None; // (start, run_len)
        'search: for bi in cursor..all_blocks.len() {
            let (bvol, Block::Heading { page: bpage, .. }) = &all_blocks[bi] else {
                continue;
            };
            if *bvol != vol || bpage.abs_diff(page) > 1 {
                continue;
            }
            let mut joined = String::new();
            for k in 0..4usize {
                let Some((rvol, Block::Heading { text, page: rpage })) = all_blocks.get(bi + k)
                else {
                    break;
                };
                if *rvol != vol || rpage.abs_diff(page) > 1 {
                    break;
                }
                if !joined.is_empty() {
                    joined.push(' ');
                }
                joined.push_str(&norm_match(&strip_markers_for_match(text)));
                if joined == want || (prefix_only && joined.starts_with(&want)) {
                    found = Some((bi, k + 1));
                    break 'search;
                }
            }
        }
        let (bi, run_len) = found.unwrap_or_else(|| {
            panic!(
                "cannot anchor TOC entry {} ({:?}) at GW {}.{}",
                entry.position, entry.label, vol, page
            )
        });
        // blocks since the previous anchor belong to the previous node
        let tail: Vec<Block> = all_blocks
            .splice(cursor..bi, std::iter::empty())
            .map(|(_, b)| b)
            .collect();
        if let Some(prev) = nodes.last_mut() {
            prev.blocks.extend(tail);
        } else {
            assert!(tail.is_empty(), "content before the first TOC anchor");
        }
        // consume the anchor run; keep the first marker found in it
        let mut heading_marker: Option<String> = None;
        let mut printed_parts: Vec<String> = Vec::new();
        for (_, b) in all_blocks.splice(cursor..cursor + run_len, std::iter::empty()) {
            let Block::Heading { text, .. } = b else {
                unreachable!()
            };
            match marker_re.find(&text) {
                Some(m) => {
                    if heading_marker.is_none() {
                        heading_marker = Some(text[m.range()].trim().to_string());
                    }
                    printed_parts.push(text[m.end()..].to_string());
                }
                None => printed_parts.push(text),
            }
        }
        let printed = printed_parts.join(" ");
        if norm_match(&printed) != norm_match(entry.label) {
            anchor_report.push(format!(
                "{}\t{}\t{}\t{}",
                entry.position, page_ref, printed, entry.label
            ));
        }
        nodes.push(Node {
            flat_index: i,
            heading_marker,
            blocks: Vec::new(),
        });
    }
    // the tail of the flow belongs to the last node
    let tail: Vec<Block> = all_blocks.drain(..).map(|(_, b)| b).collect();
    nodes.last_mut().expect("nodes").blocks.extend(tail);

    (nodes, all_footnotes, anchor_report)
}

// ------------------------------------------------------------------ output

/// `*span*` → `_span_`, or `<i>span</i>` when the span abuts a letter (an
/// intraword emphasis markdown underscores cannot express).
fn convert_emphasis(text: &str) -> String {
    let re = Regex::new(r"\*([^*\s][^*]*?)\*").unwrap();
    let mut out = String::new();
    let mut last = 0;
    for m in re.captures_iter(text) {
        let whole = m.get(0).unwrap();
        let inner = m.get(1).unwrap().as_str();
        out.push_str(&text[last..whole.start()]);
        let before = text[..whole.start()].chars().next_back();
        let after = text[whole.end()..].chars().next();
        let intraword = before.is_some_and(|c| c.is_alphanumeric())
            || after.is_some_and(|c| c.is_alphanumeric());
        if intraword {
            out.push_str(&format!("<i>{inner}</i>"));
        } else {
            out.push_str(&format!("_{inner}_"));
        }
        last = whole.end();
    }
    out.push_str(&text[last..]);
    out
}

fn main() {
    let cli = Cli::parse();
    let modernized = match cli.layer.as_str() {
        "reviewed" => false,
        "modernized" => true,
        other => panic!("unknown layer {other:?} (expected reviewed | modernized)"),
    };
    let out_dir = cli.out_dir.clone().unwrap_or_else(|| {
        PathBuf::from(if modernized {
            meta::MODERNIZED_DIR
        } else {
            meta::REVIEWED_DIR
        })
    });
    let mod_map =
        modernized.then(|| load_modernize(&cli.curated_dir.join("modernize_rulings.tsv")));

    let joins = load_joins(&cli.curated_dir.join("page_joins.tsv"));
    let rules = load_rulings(&cli.curated_dir.join("gw_dta_rulings.tsv"));

    let w21 = load_work(&cli.input_dir.join("work-gw-21.js"));
    let w11 = load_work(&cli.input_dir.join("work-gw-11.js"));
    let w12 = load_work(&cli.input_dir.join("work-gw-12.js"));

    let mut flow21 = build_flow(&w21, 21, None, &joins);
    let mut flow11 = build_flow(&w11, 11, Some(233), &joins);
    let mut flow12 = build_flow(&w12, 12, None, &joins);

    // the lost footnote reference (see MISSING_REFS)
    for (vol, id, anchor) in MISSING_REFS {
        let flow = match vol {
            11 => &mut flow11,
            _ => unreachable!(),
        };
        let mut placed = false;
        for block in &mut flow.blocks {
            if let Block::Para { text, .. } = block
                && let Some(pos) = text.find(anchor)
            {
                text.insert_str(pos + anchor.len(), &format!("[^{id}]"));
                placed = true;
                break;
            }
        }
        assert!(placed, "missing-ref anchor {anchor:?} not found for {id}");
    }

    let mut applied = 0;
    let mut failures: Vec<String> = Vec::new();
    for rule in &rules {
        let flow = match rule.volume {
            21 => &mut flow21,
            11 => &mut flow11,
            12 => &mut flow12,
            v => panic!("rule {} names unknown volume {v}", rule.id),
        };
        match apply_rule(flow, rule) {
            Ok(()) => applied += 1,
            Err(e) => failures.push(e),
        }
    }
    if !failures.is_empty() {
        for f in &failures {
            eprintln!("RULING FAILED: {f}");
        }
        panic!("{} ruling(s) failed to apply", failures.len());
    }

    let (mut nodes, footnotes, anchor_report) = assemble_nodes(vec![flow21, flow11, flow12]);

    // attach footnotes to the node/block containing their reference and
    // rewrite refs to per-file sequence numbers
    let ref_re = Regex::new(r"\[\^(fn-[a-z0-9-]+)\]").unwrap();
    let mut fn_by_id: HashMap<String, &Footnote> = HashMap::new();
    for f in &footnotes {
        fn_by_id.insert(f.id.clone(), f);
    }
    let mut consumed: Vec<String> = Vec::new();
    let entries = toc::entries();

    fs::create_dir_all(&out_dir).expect("create out dir");
    fs::create_dir_all(&cli.reports_dir).expect("create reports dir");
    let mut leftover_headings: Vec<String> = Vec::new();

    let mut missing_modern: Vec<String> = Vec::new();
    for node in &mut nodes {
        let entry = &entries[node.flat_index];
        let label = if modernized {
            common::hegel2::toc_mod::label(node.flat_index)
        } else {
            entry.label
        };
        let render = |text: &str, missing: &mut Vec<String>| -> String {
            let text = convert_emphasis(text);
            match &mod_map {
                Some(map) => modernize_text(&text, map, missing),
                None => text,
            }
        };
        let mut counter = 0u32;
        let mut out = String::new();
        out.push_str("---\n");
        out.push_str(&format!("position: {}\n", entry.position));
        out.push_str(&format!("label: \"{}\"\n", label.replace('"', "\\\"")));
        out.push_str(&format!("depth: {}\n", entry.depth));
        if let Some(p) = entry.page {
            out.push_str(&format!("page_gw: \"{p}\"\n"));
        }
        out.push_str("---\n\n");
        match &node.heading_marker {
            Some(m) => out.push_str(&format!("## {m} {}\n", label)),
            None => out.push_str(&format!("## {}\n", label)),
        }

        let mut prev_display = false;
        for block in &node.blocks {
            out.push('\n');
            match block {
                Block::Heading { text, page } => {
                    let display = DISPLAY_LINES
                        .iter()
                        .any(|(dp, prefix)| dp == page && text.starts_with(prefix));
                    if display {
                        // consecutive display lines fold into one block: drop
                        // the blank line the loop just opened
                        if prev_display && out.ends_with("\n\n") {
                            out.pop();
                        }
                        out.push_str(&format!("+ {}\n", render(text, &mut missing_modern)));
                        prev_display = true;
                        continue;
                    }
                    leftover_headings.push(format!("{}\t{}\t{}", entry.position, page, text));
                    out.push_str(&format!("## {}\n", render(text, &mut missing_modern)));
                }
                Block::Para { text, .. } => {
                    prev_display = false;
                    let mut para = render(text, &mut missing_modern);
                    let mut defs: Vec<(u32, String)> = Vec::new();
                    while let Some(c) = ref_re.captures(&para) {
                        let id = c.get(1).unwrap().as_str().to_string();
                        let f = fn_by_id
                            .get(&id)
                            .unwrap_or_else(|| panic!("ref to unknown footnote {id}"));
                        counter += 1;
                        let range = c.get(0).unwrap().range();
                        para.replace_range(range, &format!("[^{counter}]"));
                        defs.push((counter, render(f.text.trim(), &mut missing_modern)));
                        consumed.push(id);
                    }
                    out.push_str(&para);
                    out.push('\n');
                    for (n, text) in defs {
                        out.push_str(&format!("\n[^{n}]: {text}\n"));
                    }
                }
                Block::Quote { lines, .. } => {
                    prev_display = false;
                    for line in lines {
                        out.push_str(&format!("+ {}\n", render(line, &mut missing_modern)));
                    }
                }
            }
        }
        let filename = filenames::filename(node.flat_index);
        fs::write(out_dir.join(&filename), out)
            .unwrap_or_else(|e| panic!("cannot write {filename}: {e}"));
    }
    if !missing_modern.is_empty() {
        missing_modern.sort();
        missing_modern.dedup();
        panic!("modernized layer: untabled pre-reform surfaces: {missing_modern:?}");
    }

    let unconsumed: Vec<&str> = footnotes
        .iter()
        .filter(|f| !consumed.contains(&f.id))
        .map(|f| f.id.as_str())
        .collect();
    assert!(
        unconsumed.is_empty(),
        "footnotes without references: {unconsumed:?}"
    );

    fs::write(
        cli.reports_dir.join("anchor_report.tsv"),
        format!(
            "position\tpage\tprinted_heading\tlabel\n{}\n",
            anchor_report.join("\n")
        ),
    )
    .expect("write anchor report");
    fs::write(
        cli.reports_dir.join("leftover_headings.tsv"),
        format!("position\tpage\ttext\n{}\n", leftover_headings.join("\n")),
    )
    .expect("write leftover headings report");

    eprintln!(
        "hegel2 md_{}: {} files, {} rulings applied, {} footnotes, \
         {} in-file headings, {} anchor-wording notes",
        if modernized { "modernized" } else { "reviewed" },
        nodes.len(),
        applied,
        consumed.len(),
        leftover_headings.len(),
        anchor_report.len()
    );

    // reference the meta module so book identity stays wired to this crate
    let _ = meta::BOOK_SLUG;
}
