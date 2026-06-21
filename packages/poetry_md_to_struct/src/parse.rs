//! Shared verse-corpus parser: curated two-layer markdown (md_modernized +
//! md_reviewed) → struct JSON, driven by a [`Corpus`](crate::corpus::Corpus)
//! config. The two layers are parsed independently and paired (modern →
//! text/html, original → original_text/original_html): verse blocks pair
//! line-by-line, prose blocks (an Argument / The Verse) pair sentence-by-sentence
//! — the two layers must split into the same number of sentences. The corpus's
//! canonical node list is the guard — a missing, extra, or misnamed file, front
//! matter that doesn't match, a verse-line count that misses the expected total,
//! or a prose sentence-parity mismatch is a hard error.

use std::collections::HashSet;
use std::fs;
use std::path::Path;

use common::sentences::split_sentences_en;

use crate::corpus::{Corpus, NodeSpec};
use crate::html::{join_html, md_to_html, md_to_plain};
use crate::model::*;

type Err = Box<dyn std::error::Error>;

struct FrontMatter {
    position: u32,
    label: String,
    depth: i16,
}

#[derive(Clone, Copy, PartialEq)]
enum BlockKind {
    Heading,
    Verse,
    Prose,
}

struct ParsedBlock {
    kind: BlockKind,
    lines: Vec<String>,
}

/// Parse a whole corpus into the struct-JSON `Output`.
pub fn build(corpus: &Corpus) -> Result<Output, Err> {
    let modernized_dir = Path::new(&corpus.modernized_dir);
    let reviewed_dir = Path::new(&corpus.reviewed_dir);

    // Guard: both layer dirs must contain exactly the canonical file set.
    let expected_names: HashSet<&str> = corpus
        .nodes
        .iter()
        .filter_map(|n| n.content.as_ref().map(|c| c.filename.as_str()))
        .collect();
    let modernized_files = scan_dir(modernized_dir)?;
    let reviewed_files = scan_dir(reviewed_dir)?;
    for name in &expected_names {
        if !modernized_files.contains(*name) {
            return Err(format!("missing {name} in {}", corpus.modernized_dir).into());
        }
        if !reviewed_files.contains(*name) {
            return Err(format!("missing {name} in {}", corpus.reviewed_dir).into());
        }
    }
    for f in modernized_files.iter().chain(reviewed_files.iter()) {
        if !expected_names.contains(f.as_str()) {
            return Err(format!("unexpected curated file: {f}").into());
        }
    }

    let mut toc_nodes = Vec::with_capacity(corpus.nodes.len());
    let mut sentence_number = 1i32; // global per-book body enumeration

    for (idx, spec) in corpus.nodes.iter().enumerate() {
        let sort_order = idx as i32;
        match &spec.content {
            // Pure-navigation node (a Bible-shape work): no curated file.
            None => toc_nodes.push(TocNodeData {
                source_ref: spec.source_ref.clone(),
                slug: spec.slug.clone(),
                path: spec.path.clone(),
                sort_order,
                depth: spec.depth,
                label: spec.label.clone(),
                label_html: spec.label.clone(),
                parent_source_ref: spec.parent_source_ref.clone(),
                source: spec.source.clone(),
                content_blocks: vec![],
            }),
            Some(content) => {
                let (m_fm, m_blocks) = parse_file(modernized_dir, &content.filename, corpus)?;
                let (r_fm, r_blocks) = parse_file(reviewed_dir, &content.filename, corpus)?;
                validate_front_matter(&m_fm, spec, &content.filename, "modernized")?;
                validate_front_matter(&r_fm, spec, &content.filename, "reviewed")?;
                if m_blocks.len() != r_blocks.len() {
                    return Err(format!(
                        "{}: block count mismatch — modernized {}, reviewed {}",
                        content.filename,
                        m_blocks.len(),
                        r_blocks.len()
                    )
                    .into());
                }
                toc_nodes.push(build_node(
                    spec,
                    sort_order,
                    &m_blocks,
                    &r_blocks,
                    &mut sentence_number,
                )?);
            }
        }
    }

    Ok(Output {
        book: corpus.book.clone(),
        reference_systems: corpus.reference_systems.clone(),
        toc_nodes,
    })
}

fn build_node(
    spec: &NodeSpec,
    sort_order: i32,
    modern: &[ParsedBlock],
    reviewed: &[ParsedBlock],
    sentence_number: &mut i32,
) -> Result<TocNodeData, Err> {
    let label = &spec.label;
    let mut content_blocks = Vec::with_capacity(modern.len());
    let mut line_in_node = 0i32; // canonical line number, reset per node
    let mut verse_lines = 0usize;

    for (block_pos, (mb, rb)) in modern.iter().zip(reviewed.iter()).enumerate() {
        if mb.kind != rb.kind {
            return Err(
                format!("{label} block {block_pos}: block-kind mismatch between layers").into(),
            );
        }
        let position = block_pos as i16;
        match mb.kind {
            BlockKind::Heading => {
                let m_text = mb.lines[0].trim();
                let r_text = rb.lines[0].trim();
                content_blocks.push(ContentBlockData {
                    position,
                    block_type: "heading".into(),
                    paragraph_number: None,
                    figure_number: None,
                    text: md_to_plain(m_text),
                    html: md_to_html(m_text),
                    original_text: Some(md_to_plain(r_text)),
                    original_html: Some(md_to_html(r_text)),
                    sentences: vec![SentenceData {
                        position: 0,
                        sentence_number: None,
                        segment: None,
                        indent: None,
                        text: md_to_plain(m_text),
                        html: md_to_html(m_text),
                        original_text: Some(md_to_plain(r_text)),
                        original_html: Some(md_to_html(r_text)),
                        page_markers: vec![],
                    }],
                });
            }
            BlockKind::Verse => {
                if mb.lines.len() != rb.lines.len() {
                    return Err(format!(
                        "{label} block {block_pos}: verse line count mismatch — modernized {}, reviewed {}",
                        mb.lines.len(),
                        rb.lines.len()
                    )
                    .into());
                }
                let mut sentences = Vec::with_capacity(mb.lines.len());
                for (i, (m_raw, r_raw)) in mb.lines.iter().zip(rb.lines.iter()).enumerate() {
                    let (indent, m_text) = strip_indent(m_raw);
                    let r_text = r_raw.trim();
                    line_in_node += 1;
                    sentences.push(SentenceData {
                        position: i as i16,
                        sentence_number: Some(*sentence_number),
                        segment: None,
                        indent,
                        text: md_to_plain(&m_text),
                        html: md_to_html(&m_text),
                        original_text: Some(md_to_plain(r_text)),
                        original_html: Some(md_to_html(r_text)),
                        page_markers: vec![PageMarkerData {
                            system: "line".into(),
                            ref_value: line_in_node.to_string(),
                            sort_order: line_in_node - 1,
                            char_offset: 0,
                        }],
                    });
                    *sentence_number += 1;
                }
                verse_lines += mb.lines.len();
                let m_clean: Vec<String> = mb.lines.iter().map(|l| strip_indent(l).1).collect();
                let r_clean: Vec<String> = rb.lines.iter().map(|l| l.trim().to_string()).collect();
                content_blocks.push(ContentBlockData {
                    position,
                    block_type: "verse".into(),
                    paragraph_number: None,
                    figure_number: None,
                    text: m_clean.join("\n"),
                    html: join_html(&m_clean),
                    original_text: Some(r_clean.join("\n")),
                    original_html: Some(join_html(&r_clean)),
                    sentences,
                });
            }
            BlockKind::Prose => {
                // A prose paragraph (Argument / The Verse): join the wrapped
                // lines, sentence-split each layer, pair by index. The layers
                // must agree on sentence count — a mismatch means the curated
                // boundaries diverge and is a hard error (we never split/combine
                // in code; the content is reconciled instead).
                let m_join = mb
                    .lines
                    .iter()
                    .map(|l| l.trim())
                    .collect::<Vec<_>>()
                    .join(" ");
                let r_join = rb
                    .lines
                    .iter()
                    .map(|l| l.trim())
                    .collect::<Vec<_>>()
                    .join(" ");
                let (m_text, m_html) = (md_to_plain(&m_join), md_to_html(&m_join));
                let (r_text, r_html) = (md_to_plain(&r_join), md_to_html(&r_join));
                let m_sents = split_sentences_en(&m_text, &m_html);
                let r_sents = split_sentences_en(&r_text, &r_html);
                if m_sents.len() != r_sents.len() {
                    return Err(format!(
                        "{label} block {block_pos}: prose sentence parity mismatch — modernized {}, reviewed {} (reconcile the curated sentence boundaries)",
                        m_sents.len(),
                        r_sents.len()
                    )
                    .into());
                }
                let mut sentences = Vec::with_capacity(m_sents.len());
                for (i, ((mt, mh), (rt, rh))) in m_sents.iter().zip(r_sents.iter()).enumerate() {
                    sentences.push(SentenceData {
                        position: i as i16,
                        sentence_number: Some(*sentence_number),
                        segment: None,
                        indent: None,
                        text: mt.clone(),
                        html: mh.clone(),
                        original_text: Some(rt.clone()),
                        original_html: Some(rh.clone()),
                        page_markers: vec![],
                    });
                    *sentence_number += 1;
                }
                content_blocks.push(ContentBlockData {
                    position,
                    block_type: "paragraph".into(),
                    paragraph_number: None,
                    figure_number: None,
                    text: m_text,
                    html: m_html,
                    original_text: Some(r_text),
                    original_html: Some(r_html),
                    sentences,
                });
            }
        }
    }

    // Guard: verse-line count must match the canonical per-node total.
    if let Some(expected) = spec.content.as_ref().and_then(|c| c.expected_lines)
        && verse_lines != expected
    {
        return Err(
            format!("{label}: verse-line count {verse_lines} != expected {expected}").into(),
        );
    }

    Ok(TocNodeData {
        source_ref: spec.source_ref.clone(),
        slug: spec.slug.clone(),
        path: spec.path.clone(),
        sort_order,
        depth: spec.depth,
        label: label.clone(),
        label_html: label.clone(),
        parent_source_ref: spec.parent_source_ref.clone(),
        source: spec.source.clone(),
        content_blocks,
    })
}

/// Leading whitespace → indent level (2 spaces per level), and the de-indented
/// line. Flush lines yield `(None, line)`.
fn strip_indent(line: &str) -> (Option<i16>, String) {
    let trimmed = line.trim_start();
    let spaces = line.len() - trimmed.len();
    let indent = (spaces >= 2).then_some((spaces / 2) as i16);
    (indent, trimmed.trim_end().to_string())
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

fn parse_file(
    dir: &Path,
    fname: &str,
    corpus: &Corpus,
) -> Result<(FrontMatter, Vec<ParsedBlock>), Err> {
    let path = dir.join(fname);
    let content =
        fs::read_to_string(&path).map_err(|e| format!("cannot read {}: {e}", path.display()))?;
    let (fm, body) = parse_front_matter(&content)
        .ok_or_else(|| format!("no front matter in {}", path.display()))?;
    Ok((fm, parse_blocks(body, &corpus.prose_headings)))
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

/// Split a body into blocks: a `## …` line is its own heading block; runs of
/// other non-blank lines (separated by blank lines) are verse stanzas. A run
/// immediately following a heading listed in `prose_headings` is reclassified as
/// a prose paragraph.
fn parse_blocks(body: &str, prose_headings: &[String]) -> Vec<ParsedBlock> {
    let mut blocks = Vec::new();
    let mut cur: Vec<String> = Vec::new();
    for line in body.lines() {
        let trimmed = line.trim();
        if trimmed.is_empty() {
            flush_verse(&mut blocks, &mut cur);
        } else if let Some(heading) = trimmed.strip_prefix("## ") {
            flush_verse(&mut blocks, &mut cur);
            blocks.push(ParsedBlock {
                kind: BlockKind::Heading,
                lines: vec![heading.trim().to_string()],
            });
        } else {
            cur.push(line.to_string());
        }
    }
    flush_verse(&mut blocks, &mut cur);

    // Reclassify: the block right after a prose-introducing heading is prose.
    for i in 0..blocks.len() {
        let is_prose_heading = blocks[i].kind == BlockKind::Heading
            && prose_headings.iter().any(|h| h == &blocks[i].lines[0]);
        if is_prose_heading
            && let Some(next) = blocks.get_mut(i + 1)
            && next.kind == BlockKind::Verse
        {
            next.kind = BlockKind::Prose;
        }
    }
    blocks
}

fn flush_verse(blocks: &mut Vec<ParsedBlock>, cur: &mut Vec<String>) {
    if !cur.is_empty() {
        blocks.push(ParsedBlock {
            kind: BlockKind::Verse,
            lines: std::mem::take(cur),
        });
    }
}

fn validate_front_matter(
    fm: &FrontMatter,
    spec: &NodeSpec,
    fname: &str,
    layer: &str,
) -> Result<(), Err> {
    let expected_position = spec
        .content
        .as_ref()
        .map(|c| c.expected_position)
        .unwrap_or(0);
    if fm.position != expected_position {
        return Err(format!(
            "{fname} ({layer}): position {} != expected {expected_position}",
            fm.position
        )
        .into());
    }
    if fm.label != spec.label {
        return Err(format!(
            "{fname} ({layer}): label {:?} != expected {:?}",
            fm.label, spec.label
        )
        .into());
    }
    if fm.depth != spec.depth {
        return Err(format!(
            "{fname} ({layer}): depth {} != expected {}",
            fm.depth, spec.depth
        )
        .into());
    }
    Ok(())
}
