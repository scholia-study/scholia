//! Shared drama-corpus parser: curated markdown → struct JSON, driven by a
//! [`Corpus`](crate::corpus::Corpus).
//!
//! Two modes, selected by whether the corpus has a reviewed layer:
//! - **Source** (two-layer): `md_modernized` (→ text/html) paired block-by-block
//!   with `md_reviewed` (→ original_text/original_html). Node labels come from
//!   the `NodeSpec` and are validated against the modernized front matter.
//! - **Translation** (single-layer): one layer (`md_modernized_translated` →
//!   text/html, no original). Node labels come from the file front matter (they
//!   are in the translation's language). The importer links it 1:1 to the source
//!   book by natural key, so the node/block/sentence shape must match.
//!
//! Each speech is the implicit run `speaker` + following `paragraph`/`verse`/
//! `stage` blocks until the next `speaker`/heading. Label blocks pair as a single
//! sentence; verse pairs line-by-line; prose pairs sentence-by-sentence (the two
//! layers must split into the same count). Speaker/stage/heading sentences are
//! non-clickable (`sentence_number = None`); only dialogue is numbered. A
//! missing/extra/misnamed file, front matter that doesn't match, a block-shape
//! divergence, or a prose sentence-parity mismatch is a hard error.

use std::collections::HashSet;
use std::fs;
use std::mem;
use std::path::Path;

use common::sentences::split_sentences_structural;
use text_struct::html::{md_to_html, md_to_plain};
use text_struct::model::*;

use crate::corpus::{Corpus, NodeSpec};
use crate::markers::{RawMarker, resolve_marker_to_sentence, strip_markers};

type Err = Box<dyn std::error::Error>;

struct FrontMatter {
    position: u32,
    label: String,
    depth: i16,
}

#[derive(Clone, Copy, PartialEq, Debug)]
enum BlockKind {
    Heading,
    Speaker,
    /// A scene/speaker-owned stage direction (`@stage (…)` or own-line `*(…)*`).
    Stage,
    /// A `- ` bullet run (the dramatis personae) → a single non-clickable
    /// `stage` block carrying a `<ul>`.
    List,
    /// Flush dialogue prose.
    Prose,
    /// `| ` verse / chant lines.
    Verse,
}

struct ParsedBlock {
    kind: BlockKind,
    lines: Vec<String>,
}

/// Parse a whole corpus into the struct-JSON `Output`.
pub fn build(corpus: &Corpus) -> Result<Output, Err> {
    let modernized_dir = Path::new(&corpus.modernized_dir);
    let reviewed_dir = corpus.reviewed_dir.as_deref().map(Path::new);
    let page_system = corpus
        .reference_systems
        .first()
        .map(|s| s.slug.as_str())
        .ok_or("corpus has no reference system for page markers")?;

    // Guard: every layer dir must contain exactly the canonical file set.
    let expected: HashSet<&str> = corpus.nodes.iter().map(|n| n.filename.as_str()).collect();
    let modernized_files = scan_dir(modernized_dir)?;
    check_file_set(&modernized_files, &expected, &corpus.modernized_dir)?;
    if let Some(rdir) = reviewed_dir {
        let reviewed_files = scan_dir(rdir)?;
        check_file_set(
            &reviewed_files,
            &expected,
            corpus.reviewed_dir.as_deref().unwrap(),
        )?;
    }

    let mut toc_nodes = Vec::with_capacity(corpus.nodes.len());
    let mut sentence_number = 1i32; // global per-book dialogue enumeration

    for (idx, spec) in corpus.nodes.iter().enumerate() {
        let sort_order = idx as i32;
        let (m_fm, m_blocks) = parse_file(modernized_dir, &spec.filename)?;
        // The node's display label always comes from the primary layer's front
        // matter: modernized spelling for the source, translated for the EN
        // edition — the markdown stays the single source of truth.
        validate_front_matter(&m_fm, spec, &spec.filename)?;
        let node_label = m_fm.label.clone();

        let reviewed_blocks = if let Some(rdir) = reviewed_dir {
            let (r_fm, r_blocks) = parse_file(rdir, &spec.filename)?;
            validate_front_matter(&r_fm, spec, &spec.filename)?;
            if m_blocks.len() != r_blocks.len() {
                return Err(format!(
                    "{}: block count mismatch — modernized {}, reviewed {}",
                    spec.filename,
                    m_blocks.len(),
                    r_blocks.len()
                )
                .into());
            }
            Some(r_blocks)
        } else {
            None
        };

        toc_nodes.push(build_node(
            spec,
            node_label,
            sort_order,
            &m_blocks,
            reviewed_blocks.as_deref(),
            &mut sentence_number,
            page_system,
        )?);
    }

    Ok(Output {
        book: corpus.book.clone(),
        reference_systems: corpus.reference_systems.clone(),
        toc_nodes,
    })
}

fn build_node(
    spec: &NodeSpec,
    node_label: String,
    sort_order: i32,
    modern: &[ParsedBlock],
    reviewed: Option<&[ParsedBlock]>,
    sentence_number: &mut i32,
    page_system: &str,
) -> Result<TocNodeData, Err> {
    let label = &node_label;
    let mut content_blocks = Vec::with_capacity(modern.len());

    for (block_pos, mb) in modern.iter().enumerate() {
        let rb = reviewed.map(|rv| &rv[block_pos]);
        if let Some(rb) = rb
            && mb.kind != rb.kind
        {
            return Err(format!(
                "{label} block {block_pos}: block-kind mismatch — modernized {:?}, reviewed {:?}",
                mb.kind, rb.kind
            )
            .into());
        }
        let position = block_pos as i16;
        let r_first = rb.map(|r| r.lines[0].as_str());
        let r_lines = rb.map(|r| r.lines.as_slice());
        let block = match mb.kind {
            BlockKind::Heading => {
                label_block("heading", &mb.lines[0], r_first, position, page_system)
            }
            BlockKind::Speaker => {
                label_block("speaker", &mb.lines[0], r_first, position, page_system)
            }
            BlockKind::Stage => label_block("stage", &mb.lines[0], r_first, position, page_system),
            BlockKind::List => list_block(&mb.lines, r_lines, position),
            BlockKind::Verse => verse_block(
                label,
                block_pos,
                &mb.lines,
                r_lines,
                position,
                sentence_number,
                page_system,
            )?,
            BlockKind::Prose => prose_block(
                label,
                block_pos,
                &mb.lines,
                r_lines,
                position,
                sentence_number,
                page_system,
            )?,
        };
        content_blocks.push(block);
    }

    Ok(TocNodeData {
        source_ref: spec.source_ref.clone(),
        slug: spec.slug.clone(),
        path: spec.path.clone(),
        sort_order,
        depth: spec.depth,
        label: node_label.clone(),
        label_html: node_label,
        parent_source_ref: spec.parent_source_ref.clone(),
        source: None,
        content_blocks,
    })
}

/// `(stripped_plain, stripped_html)` for an optional reviewed raw string.
fn original_pair(r_raw: Option<&str>) -> (Option<String>, Option<String>) {
    match r_raw {
        Some(r) => (
            Some(strip_markers(&md_to_plain(r)).0),
            Some(strip_markers(&md_to_html(r)).0),
        ),
        None => (None, None),
    }
}

/// A single-sentence, non-clickable block (heading / speaker / stage). Page
/// markers ride through `md_to_plain`/`md_to_html` inert, so they're stripped
/// off the *rendered* text and their offsets land in plain-text coordinates.
fn label_block(
    block_type: &str,
    m_raw: &str,
    r_raw: Option<&str>,
    position: i16,
    page_system: &str,
) -> ContentBlockData {
    let (m_plain, m_markers) = strip_markers(&md_to_plain(m_raw));
    let (m_html, _) = strip_markers(&md_to_html(m_raw));
    let (orig_text, orig_html) = original_pair(r_raw);

    let page_markers = m_markers
        .iter()
        .map(|mk| page_marker(mk, mk.char_offset as i32, page_system))
        .collect();

    ContentBlockData {
        position,
        block_type: block_type.into(),
        paragraph_number: None,
        figure_number: None,
        text: m_plain.clone(),
        html: m_html.clone(),
        original_text: orig_text.clone(),
        original_html: orig_html.clone(),
        sentences: vec![SentenceData {
            position: 0,
            sentence_number: None,
            segment: None,
            indent: None,
            text: m_plain,
            html: m_html,
            original_text: orig_text,
            original_html: orig_html,
            page_markers,
        }],
    }
}

/// Build a `<ul>` from `- ` items: `(plain_joined, html)`.
fn build_ul(items: &[String]) -> (String, String) {
    let mut lis = String::new();
    let mut plains = Vec::with_capacity(items.len());
    for it in items {
        let (clean, _) = strip_markers(it);
        lis.push_str(&format!("<li>{}</li>", md_to_html(&clean)));
        plains.push(md_to_plain(&clean));
    }
    (plains.join("\n"), format!("<ul>{lis}</ul>"))
}

/// The dramatis personae: a `- ` bullet run rendered as one non-clickable
/// `stage` block holding a `<ul>`. (Cast lists carry no page markers.)
fn list_block(m_items: &[String], r_items: Option<&[String]>, position: i16) -> ContentBlockData {
    let (m_plain, m_html) = build_ul(m_items);
    let (orig_text, orig_html) = match r_items {
        Some(r) => {
            let (p, h) = build_ul(r);
            (Some(p), Some(h))
        }
        None => (None, None),
    };

    ContentBlockData {
        position,
        block_type: "stage".into(),
        paragraph_number: None,
        figure_number: None,
        text: m_plain.clone(),
        html: m_html.clone(),
        original_text: orig_text.clone(),
        original_html: orig_html.clone(),
        sentences: vec![SentenceData {
            position: 0,
            sentence_number: None,
            segment: None,
            indent: None,
            text: m_plain,
            html: m_html,
            original_text: orig_text,
            original_html: orig_html,
            page_markers: vec![],
        }],
    }
}

/// A prose speech: join the lines, sentence-split each layer, pair by index.
fn prose_block(
    label: &str,
    block_pos: usize,
    m_lines: &[String],
    r_lines: Option<&[String]>,
    position: i16,
    sentence_number: &mut i32,
    page_system: &str,
) -> Result<ContentBlockData, Err> {
    let m_join = join_trimmed(m_lines);
    let (m_plain, m_markers) = strip_markers(&md_to_plain(&m_join));
    let (m_html, _) = strip_markers(&md_to_html(&m_join));
    let m_sents = split_sentences_structural(&m_plain, &m_html);

    // Optional reviewed layer, split + parity-checked against the modernized.
    let reviewed = match r_lines {
        Some(rl) => {
            let r_join = join_trimmed(rl);
            let (r_plain, _) = strip_markers(&md_to_plain(&r_join));
            let (r_html, _) = strip_markers(&md_to_html(&r_join));
            let r_sents = split_sentences_structural(&r_plain, &r_html);
            if m_sents.len() != r_sents.len() {
                return Err(format!(
                    "{label} block {block_pos}: prose sentence parity mismatch — modernized {}, reviewed {} (reconcile the curated sentence boundaries)\n  MOD: {m_plain}\n  REV: {r_plain}",
                    m_sents.len(),
                    r_sents.len()
                )
                .into());
            }
            Some((r_plain, r_html, r_sents))
        }
        None => None,
    };

    let mut sentences = Vec::with_capacity(m_sents.len());
    let mut cumulative = Vec::with_capacity(m_sents.len());
    let mut offset = 0usize;
    for (i, (mt, mh)) in m_sents.iter().enumerate() {
        cumulative.push(offset);
        offset += mt.chars().count() + 1; // +1 for the space between sentences
        let (ot, oh) = match &reviewed {
            Some((_, _, rs)) => (Some(rs[i].0.clone()), Some(rs[i].1.clone())),
            None => (None, None),
        };
        sentences.push(SentenceData {
            position: i as i16,
            sentence_number: Some(*sentence_number),
            segment: None,
            indent: None,
            text: mt.clone(),
            html: mh.clone(),
            original_text: ot,
            original_html: oh,
            page_markers: vec![],
        });
        *sentence_number += 1;
    }
    for mk in &m_markers {
        let (idx, off) = resolve_marker_to_sentence(&cumulative, mk.char_offset);
        sentences[idx]
            .page_markers
            .push(page_marker(mk, off, page_system));
    }

    Ok(ContentBlockData {
        position,
        block_type: "paragraph".into(),
        paragraph_number: None,
        figure_number: None,
        text: m_plain,
        html: m_html,
        original_text: reviewed.as_ref().map(|(p, _, _)| p.clone()),
        original_html: reviewed.as_ref().map(|(_, h, _)| h.clone()),
        sentences,
    })
}

/// A `| ` verse run (hymn / chant): one numbered sentence per line.
fn verse_block(
    label: &str,
    block_pos: usize,
    m_lines: &[String],
    r_lines: Option<&[String]>,
    position: i16,
    sentence_number: &mut i32,
    page_system: &str,
) -> Result<ContentBlockData, Err> {
    if let Some(rl) = r_lines
        && m_lines.len() != rl.len()
    {
        return Err(format!(
            "{label} block {block_pos}: verse line count mismatch — modernized {}, reviewed {}",
            m_lines.len(),
            rl.len()
        )
        .into());
    }

    let mut sentences = Vec::with_capacity(m_lines.len());
    let mut m_htmls = Vec::with_capacity(m_lines.len());
    let mut m_plains = Vec::with_capacity(m_lines.len());
    for (i, m_raw) in m_lines.iter().enumerate() {
        let (indent, m_line) = strip_indent(m_raw);
        let (m_plain, m_markers) = strip_markers(&md_to_plain(&m_line));
        let (m_html, _) = strip_markers(&md_to_html(&m_line));
        let (orig_text, orig_html) = original_pair(r_lines.map(|rl| rl[i].trim()));

        let page_markers = m_markers
            .iter()
            .map(|mk| page_marker(mk, mk.char_offset as i32, page_system))
            .collect();
        m_plains.push(m_plain.clone());
        m_htmls.push(m_html.clone());
        sentences.push(SentenceData {
            position: i as i16,
            sentence_number: Some(*sentence_number),
            segment: None,
            indent,
            text: m_plain,
            html: m_html,
            original_text: orig_text,
            original_html: orig_html,
            page_markers,
        });
        *sentence_number += 1;
    }

    let (orig_text, orig_html) = match r_lines {
        Some(rl) => {
            let plains: Vec<String> = rl.iter().map(|l| md_to_plain(l.trim())).collect();
            let htmls: Vec<String> = rl
                .iter()
                .map(|l| strip_markers(&md_to_html(l.trim())).0)
                .collect();
            (Some(plains.join("\n")), Some(htmls.join("<br>\n")))
        }
        None => (None, None),
    };
    Ok(ContentBlockData {
        position,
        block_type: "verse".into(),
        paragraph_number: None,
        figure_number: None,
        text: m_plains.join("\n"),
        html: m_htmls.join("<br>\n"),
        original_text: orig_text,
        original_html: orig_html,
        sentences,
    })
}

fn page_marker(m: &RawMarker, char_offset: i32, system: &str) -> PageMarkerData {
    PageMarkerData {
        system: system.into(),
        ref_value: m.value.clone(),
        sort_order: m.value.parse::<i32>().unwrap_or(0),
        char_offset,
    }
}

fn join_trimmed(lines: &[String]) -> String {
    lines.iter().map(|l| l.trim()).collect::<Vec<_>>().join(" ")
}

/// Leading whitespace → indent level (2 spaces per level), and the de-indented
/// line. Flush lines yield `(None, line)`.
fn strip_indent(line: &str) -> (Option<i16>, String) {
    let trimmed = line.trim_start();
    let spaces = line.len() - trimmed.len();
    let indent = (spaces >= 2).then_some((spaces / 2) as i16);
    (indent, trimmed.trim_end().to_string())
}

fn check_file_set(found: &HashSet<String>, expected: &HashSet<&str>, dir: &str) -> Result<(), Err> {
    for name in expected {
        if !found.contains(*name) {
            return Err(format!("missing {name} in {dir}").into());
        }
    }
    for f in found {
        if !expected.contains(f.as_str()) {
            return Err(format!("unexpected curated file: {f} in {dir}").into());
        }
    }
    Ok(())
}

fn scan_dir(dir: &Path) -> Result<HashSet<String>, Err> {
    let mut out = HashSet::new();
    for entry in fs::read_dir(dir).map_err(|e| format!("cannot read {}: {e}", dir.display()))? {
        let name = entry?.file_name().to_string_lossy().to_string();
        if name.ends_with(".md") && name != "000_toc.md" {
            out.insert(name);
        }
    }
    Ok(out)
}

fn parse_file(dir: &Path, fname: &str) -> Result<(FrontMatter, Vec<ParsedBlock>), Err> {
    let path = dir.join(fname);
    let content =
        fs::read_to_string(&path).map_err(|e| format!("cannot read {}: {e}", path.display()))?;
    let (fm, body) = parse_front_matter(&content)
        .ok_or_else(|| format!("no front matter in {}", path.display()))?;
    Ok((fm, parse_blocks(body)))
}

fn parse_front_matter(content: &str) -> Option<(FrontMatter, &str)> {
    let rest = content.trim_start_matches('\u{feff}').strip_prefix("---")?;
    let close = rest.find("\n---")?;
    let fm_text = &rest[..close];
    let body = rest[close + "\n---".len()..]
        .trim_start_matches('\n')
        .trim_end_matches('\n');

    let (mut position, mut label, mut depth) = (None, None, None);
    for line in fm_text.lines() {
        let line = line.trim();
        if let Some(v) = line.strip_prefix("position:") {
            position = v.trim().parse().ok();
        } else if let Some(v) = line.strip_prefix("label:") {
            label = Some(v.trim().trim_matches('"').to_string());
        } else if let Some(v) = line.strip_prefix("depth:") {
            depth = v.trim().parse().ok();
        }
    }
    Some((
        FrontMatter {
            position: position?,
            label: label?,
            depth: depth?,
        },
        body,
    ))
}

/// Tokenise a body into drama blocks (see module docs for the markup).
fn parse_blocks(body: &str) -> Vec<ParsedBlock> {
    let mut blocks = Vec::new();
    let mut cur: Vec<String> = Vec::new();
    let mut cur_kind: Option<BlockKind> = None;

    for line in body.lines() {
        let t = line.trim();
        if t.is_empty() {
            flush(&mut blocks, &mut cur, &mut cur_kind);
        } else if let Some(h) = t.strip_prefix("## ") {
            push_single(&mut blocks, &mut cur, &mut cur_kind, BlockKind::Heading, h);
        } else if let Some(s) = t.strip_prefix("@stage") {
            push_single(
                &mut blocks,
                &mut cur,
                &mut cur_kind,
                BlockKind::Stage,
                s.trim_start(),
            );
        } else if let Some(sp) = t.strip_prefix("@ ") {
            push_single(&mut blocks, &mut cur, &mut cur_kind, BlockKind::Speaker, sp);
        } else if t.starts_with("*(") {
            push_single(&mut blocks, &mut cur, &mut cur_kind, BlockKind::Stage, t);
        } else if let Some(v) = verse_content(t) {
            accumulate(&mut blocks, &mut cur, &mut cur_kind, BlockKind::Verse, v);
        } else if let Some(li) = t.strip_prefix("- ") {
            accumulate(&mut blocks, &mut cur, &mut cur_kind, BlockKind::List, li);
        } else {
            accumulate(&mut blocks, &mut cur, &mut cur_kind, BlockKind::Prose, t);
        }
    }
    flush(&mut blocks, &mut cur, &mut cur_kind);
    blocks
}

/// `| line` → its content; a lone `|` → an empty line.
fn verse_content(t: &str) -> Option<&str> {
    t.strip_prefix("| ")
        .or(if t == "|" { Some("") } else { None })
}

fn push_single(
    blocks: &mut Vec<ParsedBlock>,
    cur: &mut Vec<String>,
    cur_kind: &mut Option<BlockKind>,
    kind: BlockKind,
    content: &str,
) {
    flush(blocks, cur, cur_kind);
    blocks.push(ParsedBlock {
        kind,
        lines: vec![content.to_string()],
    });
}

fn accumulate(
    blocks: &mut Vec<ParsedBlock>,
    cur: &mut Vec<String>,
    cur_kind: &mut Option<BlockKind>,
    kind: BlockKind,
    content: &str,
) {
    if *cur_kind != Some(kind) {
        flush(blocks, cur, cur_kind);
        *cur_kind = Some(kind);
    }
    cur.push(content.to_string());
}

fn flush(blocks: &mut Vec<ParsedBlock>, cur: &mut Vec<String>, cur_kind: &mut Option<BlockKind>) {
    if let Some(kind) = cur_kind.take()
        && !cur.is_empty()
    {
        blocks.push(ParsedBlock {
            kind,
            lines: mem::take(cur),
        });
    }
    cur.clear();
}

fn validate_front_matter(fm: &FrontMatter, spec: &NodeSpec, fname: &str) -> Result<(), Err> {
    if fm.position != spec.expected_position {
        return Err(format!(
            "{fname}: position {} != expected {}",
            fm.position, spec.expected_position
        )
        .into());
    }
    if fm.depth != spec.depth {
        return Err(format!("{fname}: depth {} != expected {}", fm.depth, spec.depth).into());
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    fn kinds(body: &str) -> Vec<BlockKind> {
        parse_blocks(body).iter().map(|b| b.kind).collect()
    }

    #[test]
    fn tokenises_a_speech_with_stage_and_verse() {
        let body = "## FØRSTE HANDLING.\n\n@stage (Påskenatt.)\n\n@ Lovsang *(i kirken)*.\n| Linje en\n| Linje to\n\n@ Soldaten.\nVet ikke. Han kommer snart.\n\n*(han går.)*";
        assert_eq!(
            kinds(body),
            vec![
                BlockKind::Heading,
                BlockKind::Stage,
                BlockKind::Speaker,
                BlockKind::Verse,
                BlockKind::Speaker,
                BlockKind::Prose,
                BlockKind::Stage,
            ]
        );
    }

    #[test]
    fn tokenises_cast_list() {
        let body = "## DE OPPTREDENDE:\n\n- Keiser Konstanzios.\n- Fyrstinne Helena, *søster.*\n\n@stage (Tiden …)";
        assert_eq!(
            kinds(body),
            vec![BlockKind::Heading, BlockKind::List, BlockKind::Stage]
        );
    }

    #[test]
    fn speaker_only_line_is_its_own_block() {
        // A silent action: a speaker line with no spoken body.
        let body = "@ Fyrst Julian *(folder hendene)*.\n\n@ Keiserinne Eusebia.\nFrykt ikke!";
        assert_eq!(
            kinds(body),
            vec![BlockKind::Speaker, BlockKind::Speaker, BlockKind::Prose]
        );
    }

    #[test]
    fn list_block_two_layer_builds_ul() {
        let b = list_block(
            &["Keiser Konstanzios.".into(), "Helena, *søster.*".into()],
            Some(&["Kejser Konstanzios.".into(), "Helena, *søster.*".into()]),
            0,
        );
        assert_eq!(b.block_type, "stage");
        assert_eq!(
            b.html,
            "<ul><li>Keiser Konstanzios.</li><li>Helena, <i>søster.</i></li></ul>"
        );
        assert_eq!(
            b.original_html.as_deref(),
            Some("<ul><li>Kejser Konstanzios.</li><li>Helena, <i>søster.</i></li></ul>")
        );
        assert!(b.sentences[0].sentence_number.is_none());
    }

    #[test]
    fn list_block_single_layer_has_no_original() {
        let b = list_block(&["Emperor Constantius.".into()], None, 0);
        assert_eq!(b.html, "<ul><li>Emperor Constantius.</li></ul>");
        assert_eq!(b.original_html, None);
        assert_eq!(b.sentences[0].original_html, None);
    }

    #[test]
    fn prose_block_single_layer_numbers_and_omits_original() {
        let mut sn = 1;
        let b = prose_block(
            "Act",
            0,
            &["Take that. And that.".into()],
            None,
            0,
            &mut sn,
            "1873",
        )
        .unwrap();
        assert_eq!(b.block_type, "paragraph");
        assert_eq!(b.sentences.len(), 2);
        assert_eq!(b.sentences[0].sentence_number, Some(1));
        assert_eq!(b.sentences[1].sentence_number, Some(2));
        assert_eq!(b.original_text, None);
        assert_eq!(b.sentences[0].original_text, None);
        assert_eq!(sn, 3);
    }
}
