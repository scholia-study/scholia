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
//! layers must split into the same count). Speaker/heading sentences and the
//! dramatis-personae cast list are non-clickable (`sentence_number = None`);
//! dialogue and stage directions are numbered (quotable). Inline `*(…)*`
//! directions between sentences are peeled into their own numbered sentence;
//! mid-sentence ones stay woven in — except under `greek_splitter`, where no
//! such convention exists and no peeling happens. A missing/extra/misnamed
//! file, front matter that doesn't match, a block-shape divergence, or a prose
//! sentence-parity mismatch is a hard error.
//!
//! Footnotes (`[^marker]` inline refs + `[^marker]: text` definition blocks,
//! the same convention `md_prose_to_struct` uses) are lifted out of the block
//! stream, numbered sequentially across the whole book, and attached to the
//! sentence whose rendered HTML carries their `<sup>N</sup>` ref.

use std::collections::{HashMap, HashSet};
use std::fs;
use std::mem;
use std::path::Path;
use std::sync::LazyLock;

use common::sentences::{split_sentences_grc, split_sentences_structural};
use regex::Regex;
use text_struct::html::{md_to_html, md_to_plain};
use text_struct::model::*;

use crate::corpus::{Corpus, NodeSpec};
use crate::markers::{RawMarker, strip_markers};
use text_struct::parse::{
    FrontMatter, parse_front_matter, resolve_marker_to_sentence, scan_md_files, strip_indent,
};

type Err = Box<dyn std::error::Error>;

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
    /// A `[^marker]: text` footnote definition. Lifted out of the block stream
    /// before rendering — never reaches `build_node`.
    Footnote,
}

struct ParsedBlock {
    kind: BlockKind,
    lines: Vec<String>,
}

/// A collected footnote's raw content, keyed by its assigned book-global
/// number.
struct FootnoteContent {
    text: String,
    original_text: Option<String>,
}

/// A node's footnote registry: marker text → assigned number, and the
/// content behind each number. Empty for a corpus/node with no footnotes
/// (the common case — every builder function takes one unconditionally so
/// footnote support costs the no-footnote path nothing beyond a no-op regex
/// pass).
#[derive(Default)]
struct FootnoteLookup {
    marker_to_number: HashMap<String, i32>,
    by_number: HashMap<i32, FootnoteContent>,
}

/// Sentence splitter for the edition's language: `split_sentences_structural`
/// (paren-aware, drives stage-direction peeling) or, under `greek_splitter`,
/// `split_sentences_grc`.
type Splitter = fn(&str, &str) -> Vec<(String, String)>;

/// The genre knobs the block builders need, bundled so a new one doesn't grow
/// their argument lists.
struct BlockCtx<'a> {
    page_system: &'a str,
    subpage_letter_sort: bool,
    greek_splitter: bool,
    splitter: Splitter,
    footnotes: &'a FootnoteLookup,
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
    let splitter: Splitter = if corpus.greek_splitter {
        split_sentences_grc
    } else {
        split_sentences_structural
    };

    // Guard: every layer dir must contain exactly the canonical file set.
    let expected: HashSet<&str> = corpus.nodes.iter().map(|n| n.filename.as_str()).collect();
    let modernized_files: HashSet<String> = scan_md_files(modernized_dir)
        .map_err(|e| format!("cannot read {}: {e}", corpus.modernized_dir))?
        .into_iter()
        .collect();
    check_file_set(&modernized_files, &expected, &corpus.modernized_dir)?;
    if let Some(rdir) = reviewed_dir {
        let reviewed_files: HashSet<String> = scan_md_files(rdir)
            .map_err(|e| format!("cannot read {}: {e}", rdir.display()))?
            .into_iter()
            .collect();
        check_file_set(
            &reviewed_files,
            &expected,
            corpus.reviewed_dir.as_deref().unwrap(),
        )?;
    }

    let mut toc_nodes = Vec::with_capacity(corpus.nodes.len());
    let mut sentence_number = 1i32; // global per-book quotable-sentence count (dialogue + stage directions)
    let mut footnote_number = 0i32; // global per-book footnote count

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

        let footnotes = extract_footnotes(
            &spec.filename,
            &m_blocks,
            reviewed_blocks.as_deref(),
            &mut footnote_number,
        )?;
        let m_blocks: Vec<&ParsedBlock> = m_blocks
            .iter()
            .filter(|b| b.kind != BlockKind::Footnote)
            .collect();
        let reviewed_blocks: Option<Vec<&ParsedBlock>> = reviewed_blocks.as_ref().map(|rv| {
            rv.iter()
                .filter(|b| b.kind != BlockKind::Footnote)
                .collect()
        });

        let ctx = BlockCtx {
            page_system,
            subpage_letter_sort: corpus.subpage_letter_sort,
            greek_splitter: corpus.greek_splitter,
            splitter,
            footnotes: &footnotes,
        };

        toc_nodes.push(build_node(
            spec,
            node_label,
            sort_order,
            &m_blocks,
            reviewed_blocks.as_deref(),
            &mut sentence_number,
            &ctx,
        )?);
    }

    Ok(Output {
        book: corpus.book.clone(),
        reference_systems: corpus.reference_systems.clone(),
        toc_nodes,
    })
}

/// Lift `[^marker]: text` footnote-definition blocks out of `modern` (and, at
/// the same position, the paired `reviewed` layer — the two layers must
/// define footnotes at identical positions, exactly like every other block
/// kind), assigning each a book-global sequential number as it's encountered.
/// `footnote_number` threads the counter across nodes so numbering runs
/// continuously over the whole book, matching the global `sentence_number`
/// counter above it.
fn extract_footnotes(
    label: &str,
    modern: &[ParsedBlock],
    reviewed: Option<&[ParsedBlock]>,
    footnote_number: &mut i32,
) -> Result<FootnoteLookup, Err> {
    let mut lookup = FootnoteLookup::default();
    for (pos, mb) in modern.iter().enumerate() {
        let r = reviewed.and_then(|rv| rv.get(pos));
        let is_modern_fn = mb.kind == BlockKind::Footnote;
        let is_reviewed_fn = r.is_some_and(|r| r.kind == BlockKind::Footnote);
        if is_modern_fn != is_reviewed_fn {
            return Err(format!(
                "{label} block {pos}: footnote-definition mismatch between layers"
            )
            .into());
        }
        if !is_modern_fn {
            continue;
        }
        *footnote_number += 1;
        let n = *footnote_number;
        lookup.marker_to_number.insert(mb.lines[0].clone(), n);
        lookup.by_number.insert(
            n,
            FootnoteContent {
                text: mb.lines[1].clone(),
                original_text: r.map(|r| r.lines[1].clone()),
            },
        );
    }
    Ok(lookup)
}

fn build_node(
    spec: &NodeSpec,
    node_label: String,
    sort_order: i32,
    modern: &[&ParsedBlock],
    reviewed: Option<&[&ParsedBlock]>,
    sentence_number: &mut i32,
    ctx: &BlockCtx,
) -> Result<TocNodeData, Err> {
    let label = &node_label;
    let mut content_blocks = Vec::with_capacity(modern.len());

    for (block_pos, mb) in modern.iter().enumerate() {
        let rb = reviewed.map(|rv| rv[block_pos]);
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
                label_block("heading", &mb.lines[0], r_first, position, ctx, None)
            }
            BlockKind::Speaker => {
                label_block("speaker", &mb.lines[0], r_first, position, ctx, None)
            }
            // A stage direction is authored dramatic text: quotable, so it gets
            // its own sentence_number (unlike the inert speaker/heading labels).
            BlockKind::Stage => {
                let n = *sentence_number;
                *sentence_number += 1;
                label_block("stage", &mb.lines[0], r_first, position, ctx, Some(n))
            }
            BlockKind::List => list_block(&mb.lines, r_lines, position),
            BlockKind::Verse => verse_block(
                label,
                block_pos,
                &mb.lines,
                r_lines,
                position,
                sentence_number,
                ctx,
            )?,
            BlockKind::Prose => prose_block(
                label,
                block_pos,
                &mb.lines,
                r_lines,
                position,
                sentence_number,
                ctx,
            )?,
            BlockKind::Footnote => unreachable!("footnote blocks filtered out before build_node"),
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

/// `[^marker]` footnote references — inert to `md_to_html`/`md_to_plain`
/// (neither touches `[`, `^`, or `]`), so a ref rides through rendering
/// intact and is resolved afterward, exactly like a page marker.
static FOOTNOTE_REF_RE: LazyLock<Regex> = LazyLock::new(|| Regex::new(r"\[\^([^\]]+)\]").unwrap());

/// Strip footnote refs from rendered plain text — the ref glyph carries no
/// reading-text content; only the HTML `<sup>` anchors a footnote.
fn strip_footnote_refs(plain: &str) -> String {
    FOOTNOTE_REF_RE.replace_all(plain, "").into_owned()
}

/// Rewrite a footnote ref to `<sup>N</sup>`, N being the marker's book-global
/// number. An unrecognized marker (no matching definition) is left as literal
/// text.
fn rewrite_footnote_refs_html(html: &str, marker_to_number: &HashMap<String, i32>) -> String {
    FOOTNOTE_REF_RE
        .replace_all(html, |caps: &regex::Captures| {
            match marker_to_number.get(&caps[1]) {
                Some(n) => format!("<sup>{n}</sup>"),
                None => caps[0].to_string(),
            }
        })
        .into_owned()
}

/// `text_struct::html::md_to_plain`, with footnote refs resolved away — the
/// printed reading text carries no ref glyph.
fn to_plain(raw: &str) -> String {
    strip_footnote_refs(&md_to_plain(raw))
}

/// `text_struct::html::md_to_html`, with footnote refs rewritten to
/// `<sup>N</sup>` per this node's marker→number assignment.
fn to_html(raw: &str, marker_to_number: &HashMap<String, i32>) -> String {
    rewrite_footnote_refs_html(&md_to_html(raw), marker_to_number)
}

/// `<sup>NUMBER</sup>` in rendered sentence HTML — the trace a footnote ref
/// leaves once resolved.
static SUP_NUMBER_RE: LazyLock<Regex> = LazyLock::new(|| Regex::new(r"<sup>(\d+)</sup>").unwrap());

/// Build one `FootnoteData` from a footnote's collected content: its body
/// (and, when the layer exists, its reviewed-layer body) split into sentences
/// with the same splitter the surrounding dialogue uses.
fn build_footnote_data(number: i32, content: &FootnoteContent, splitter: Splitter) -> FootnoteData {
    let fn_plain = md_to_plain(&content.text);
    let fn_html = md_to_html(&content.text);
    let fn_pairs = splitter(&fn_plain, &fn_html);

    let fn_orig_pairs: Option<Vec<(String, String)>> = content.original_text.as_ref().map(|orig| {
        let orig_plain = md_to_plain(orig);
        let orig_html = md_to_html(orig);
        splitter(&orig_plain, &orig_html)
    });

    let sentences = fn_pairs
        .iter()
        .enumerate()
        .map(|(pos, (text, html))| {
            let (original_text, original_html) = match &fn_orig_pairs {
                Some(op) => (
                    op.get(pos).map(|(t, _)| t.clone()),
                    op.get(pos).map(|(_, h)| h.clone()),
                ),
                None => (None, None),
            };
            FootnoteSentenceData {
                position: pos as i16,
                sentence_number: None,
                text: text.clone(),
                html: html.clone(),
                original_text,
                original_html,
            }
        })
        .collect();

    FootnoteData { number, sentences }
}

/// Scan a rendered sentence's HTML for footnote refs and build the
/// `FootnoteData` entries it carries.
fn attach_footnotes(
    sent_html: &str,
    footnotes: &FootnoteLookup,
    splitter: Splitter,
) -> Vec<FootnoteData> {
    SUP_NUMBER_RE
        .captures_iter(sent_html)
        .filter_map(|caps| {
            let number: i32 = caps[1].parse().ok()?;
            let content = footnotes.by_number.get(&number)?;
            Some(build_footnote_data(number, content, splitter))
        })
        .collect()
}

/// `(stripped_plain, stripped_html)` for an optional reviewed raw string.
fn original_pair(
    r_raw: Option<&str>,
    marker_to_number: &HashMap<String, i32>,
) -> (Option<String>, Option<String>) {
    match r_raw {
        Some(r) => (
            Some(strip_markers(&to_plain(r)).0),
            Some(strip_markers(&to_html(r, marker_to_number)).0),
        ),
        None => (None, None),
    }
}

/// A single-sentence label block (heading / speaker / stage). Page markers ride
/// through `md_to_plain`/`md_to_html` inert, so they're stripped off the
/// *rendered* text and their offsets land in plain-text coordinates. `num` sets
/// the sentence's `sentence_number`: `None` for the inert speaker/heading
/// apparatus, `Some(n)` for a stage direction (quotable dramatic text).
fn label_block(
    block_type: &str,
    m_raw: &str,
    r_raw: Option<&str>,
    position: i16,
    ctx: &BlockCtx,
    num: Option<i32>,
) -> ContentBlockData {
    let (m_plain, m_markers) = strip_markers(&to_plain(m_raw));
    let (m_html, _) = strip_markers(&to_html(m_raw, &ctx.footnotes.marker_to_number));
    let (orig_text, orig_html) = original_pair(r_raw, &ctx.footnotes.marker_to_number);

    let page_markers = m_markers
        .iter()
        .map(|mk| page_marker(ctx, mk, mk.char_offset as i32))
        .collect();
    let footnotes = attach_footnotes(&m_html, ctx.footnotes, ctx.splitter);

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
            sentence_number: num,
            segment: None,
            indent: None,
            text: m_plain,
            html: m_html,
            original_text: orig_text,
            original_html: orig_html,
            page_markers,
            footnotes,
            margin_notes: Vec::new(),
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
/// `stage` block holding a `<ul>`. (Cast lists carry no page markers or
/// footnotes.)
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
            footnotes: Vec::new(),
            margin_notes: Vec::new(),
        }],
    }
}

/// Wrap parenthetical emphasis runs (`*(…)*` → `<i>(…)</i>`) in a `stage` class
/// so the reader mutes stage directions without also muting ordinary emphasis
/// (`*word*` → `<i>word</i>`, left untouched). Matches an `<i>` whose content is
/// wholly `(…)` with no nested tags.
static STAGE_I_RE: LazyLock<Regex> = LazyLock::new(|| Regex::new(r"<i>(\([^<]*\))</i>").unwrap());

fn tag_stage_directions(html: &str) -> String {
    STAGE_I_RE
        .replace_all(html, r#"<i class="stage">$1</i>"#)
        .into_owned()
}

const STAGE_OPEN: &str = "<i class=\"stage\">";

/// Split any sentence that *opens* with a stage direction into a standalone
/// direction sentence + the remaining dialogue. A between-sentence direction
/// (`…stake. (draws aside.) Oh…`) lands at the head of the following sentence
/// after structural splitting; peeling it off stops it from riding along when
/// that dialogue line is selected. A mid-sentence direction never opens a
/// sentence, so it stays woven in. Both edition layers peel identically (their
/// `*(…)*` markers are parallel), preserving sentence parity.
fn peel_directions(sents: Vec<(String, String)>) -> Vec<(String, String)> {
    let mut out = Vec::with_capacity(sents.len());
    for (text, html) in sents {
        peel_one(text.trim(), html.trim(), &mut out);
    }
    out
}

fn peel_one(text: &str, html: &str, out: &mut Vec<(String, String)>) {
    if html.starts_with(STAGE_OPEN)
        && let (Some(pe), Some(he)) = (leading_paren_end(text), leading_i_end(html))
    {
        out.push((text[..pe].to_string(), html[..he].to_string()));
        let rest_t = text[pe..].trim();
        let rest_h = html[he..].trim();
        if !rest_t.is_empty() {
            peel_one(rest_t, rest_h, out);
        }
        return;
    }
    out.push((text.to_string(), html.to_string()));
}

/// Byte offset just past the `)` that closes a leading `(` at depth 0.
fn leading_paren_end(text: &str) -> Option<usize> {
    if !text.starts_with('(') {
        return None;
    }
    let mut depth = 0i32;
    for (i, c) in text.char_indices() {
        match c {
            '(' => depth += 1,
            ')' => {
                depth -= 1;
                if depth == 0 {
                    return Some(i + c.len_utf8());
                }
            }
            _ => {}
        }
    }
    None
}

/// Byte offset just past the first `</i>`.
fn leading_i_end(html: &str) -> Option<usize> {
    html.find("</i>").map(|i| i + "</i>".len())
}

/// A prose speech: join the lines, sentence-split each layer, pair by index.
fn prose_block(
    label: &str,
    block_pos: usize,
    m_lines: &[String],
    r_lines: Option<&[String]>,
    position: i16,
    sentence_number: &mut i32,
    ctx: &BlockCtx,
) -> Result<ContentBlockData, Err> {
    let m_join = join_trimmed(m_lines);
    let (m_plain, m_markers) = strip_markers(&to_plain(&m_join));
    let (m_html_raw, _) = strip_markers(&to_html(&m_join, &ctx.footnotes.marker_to_number));
    let m_html = tag_stage_directions(&m_html_raw);
    let m_pairs = (ctx.splitter)(&m_plain, &m_html);
    let m_sents = if ctx.greek_splitter {
        m_pairs
    } else {
        peel_directions(m_pairs)
    };

    // Optional reviewed layer, split + parity-checked against the modernized.
    let reviewed = match r_lines {
        Some(rl) => {
            let r_join = join_trimmed(rl);
            let (r_plain, _) = strip_markers(&to_plain(&r_join));
            let (r_html_raw, _) = strip_markers(&to_html(&r_join, &ctx.footnotes.marker_to_number));
            let r_html = tag_stage_directions(&r_html_raw);
            let r_pairs = (ctx.splitter)(&r_plain, &r_html);
            let r_sents = if ctx.greek_splitter {
                r_pairs
            } else {
                peel_directions(r_pairs)
            };
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
        let footnotes = attach_footnotes(mh, ctx.footnotes, ctx.splitter);
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
            footnotes,
            margin_notes: Vec::new(),
        });
        *sentence_number += 1;
    }
    for mk in &m_markers {
        let (idx, off) = resolve_marker_to_sentence(&cumulative, mk.char_offset);
        sentences[idx].page_markers.push(page_marker(ctx, mk, off));
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
    ctx: &BlockCtx,
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
        let (m_plain, m_markers) = strip_markers(&to_plain(&m_line));
        let (m_html, _) = strip_markers(&to_html(&m_line, &ctx.footnotes.marker_to_number));
        let (orig_text, orig_html) = original_pair(
            r_lines.map(|rl| rl[i].trim()),
            &ctx.footnotes.marker_to_number,
        );

        let page_markers = m_markers
            .iter()
            .map(|mk| page_marker(ctx, mk, mk.char_offset as i32))
            .collect();
        let footnotes = attach_footnotes(&m_html, ctx.footnotes, ctx.splitter);
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
            footnotes,
            margin_notes: Vec::new(),
        });
        *sentence_number += 1;
    }

    let (orig_text, orig_html) = match r_lines {
        Some(rl) => {
            let plains: Vec<String> = rl.iter().map(|l| to_plain(l.trim())).collect();
            let htmls: Vec<String> = rl
                .iter()
                .map(|l| strip_markers(&to_html(l.trim(), &ctx.footnotes.marker_to_number)).0)
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

/// Sort order for a Stephanus-style marker (`447a`) treated as a sub-page
/// address: `447a` → 4470 … `447e` → 4474, `448` → 4480 — so sections stay
/// strictly ordered within and across pages. Mirrors
/// `md_prose_to_struct::roman::block_sort_order_subpage`, narrowed to drama's
/// digits[+letter] marker form (no Roman/dotted/venue variants).
fn subpage_sort_order(value: &str) -> i32 {
    let digits = value.trim_end_matches(|c: char| c.is_ascii_lowercase());
    let letters = &value[digits.len()..];
    let page: i32 = digits.parse().unwrap_or(0);
    match letters.chars().next() {
        Some(ch) if letters.len() == 1 => page * 10 + (ch as i32 - 'a' as i32),
        _ => page * 10,
    }
}

fn page_marker(ctx: &BlockCtx, m: &RawMarker, char_offset: i32) -> PageMarkerData {
    PageMarkerData {
        system: ctx.page_system.into(),
        ref_value: m.value.clone(),
        sort_order: if ctx.subpage_letter_sort {
            subpage_sort_order(&m.value)
        } else {
            m.value.parse::<i32>().unwrap_or(0)
        },
        char_offset,
    }
}

fn join_trimmed(lines: &[String]) -> String {
    lines.iter().map(|l| l.trim()).collect::<Vec<_>>().join(" ")
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

fn parse_file(dir: &Path, fname: &str) -> Result<(FrontMatter, Vec<ParsedBlock>), Err> {
    let path = dir.join(fname);
    let content =
        fs::read_to_string(&path).map_err(|e| format!("cannot read {}: {e}", path.display()))?;
    let (fm, body) = parse_front_matter(&content)
        .ok_or_else(|| format!("no front matter in {}", path.display()))?;
    let body = body.trim_start_matches('\n').trim_end_matches('\n');
    Ok((fm, parse_blocks(body)))
}

/// Tokenise a body into drama blocks (see module docs for the markup).
fn parse_blocks(body: &str) -> Vec<ParsedBlock> {
    let mut blocks = Vec::new();
    let mut cur: Vec<String> = Vec::new();
    let mut cur_kind: Option<BlockKind> = None;

    let lines: Vec<&str> = body.lines().collect();
    let mut i = 0;
    while i < lines.len() {
        let t = lines[i].trim();
        if t.is_empty() {
            flush(&mut blocks, &mut cur, &mut cur_kind);
            i += 1;
        } else if let Some((marker, first)) = try_parse_footnote_start(t) {
            flush(&mut blocks, &mut cur, &mut cur_kind);
            let mut text = first.to_string();
            i += 1;
            while i < lines.len() {
                let next = lines[i].trim();
                if next.is_empty() || is_block_start(next) {
                    break;
                }
                text.push(' ');
                text.push_str(next);
                i += 1;
            }
            blocks.push(ParsedBlock {
                kind: BlockKind::Footnote,
                lines: vec![marker.to_string(), text],
            });
        } else if let Some(h) = t.strip_prefix("## ") {
            push_single(&mut blocks, &mut cur, &mut cur_kind, BlockKind::Heading, h);
            i += 1;
        } else if let Some(s) = t.strip_prefix("@stage") {
            push_single(
                &mut blocks,
                &mut cur,
                &mut cur_kind,
                BlockKind::Stage,
                s.trim_start(),
            );
            i += 1;
        } else if let Some(sp) = t.strip_prefix("@ ") {
            push_single(&mut blocks, &mut cur, &mut cur_kind, BlockKind::Speaker, sp);
            i += 1;
        } else if t.starts_with("*(") {
            push_single(&mut blocks, &mut cur, &mut cur_kind, BlockKind::Stage, t);
            i += 1;
        } else if let Some(v) = verse_content(t) {
            accumulate(&mut blocks, &mut cur, &mut cur_kind, BlockKind::Verse, v);
            i += 1;
        } else if let Some(li) = t.strip_prefix("- ") {
            accumulate(&mut blocks, &mut cur, &mut cur_kind, BlockKind::List, li);
            i += 1;
        } else {
            accumulate(&mut blocks, &mut cur, &mut cur_kind, BlockKind::Prose, t);
            i += 1;
        }
    }
    flush(&mut blocks, &mut cur, &mut cur_kind);
    blocks
}

/// Whether a (trimmed) line opens a new non-prose block — used to stop a
/// footnote definition's continuation lines from swallowing the next block.
fn is_block_start(line: &str) -> bool {
    line.starts_with("## ")
        || line.starts_with("@stage")
        || line.starts_with("@ ")
        || line.starts_with("*(")
        || verse_content(line).is_some()
        || line.starts_with("- ")
        || try_parse_footnote_start(line).is_some()
}

/// `| line` → its content; a lone `|` → an empty line.
fn verse_content(t: &str) -> Option<&str> {
    t.strip_prefix("| ")
        .or(if t == "|" { Some("") } else { None })
}

/// Try to parse a footnote-definition start: `[^marker]: text`.
fn try_parse_footnote_start(line: &str) -> Option<(&str, &str)> {
    let rest = line.strip_prefix("[^")?;
    let end_bracket = rest.find("]:")?;
    let marker = &rest[..end_bracket];
    let text = rest[end_bracket + 2..].trim();
    Some((marker, text))
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

    fn test_ctx<'a>(page_system: &'a str, footnotes: &'a FootnoteLookup) -> BlockCtx<'a> {
        BlockCtx {
            page_system,
            subpage_letter_sort: false,
            greek_splitter: false,
            splitter: split_sentences_structural,
            footnotes,
        }
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
    fn tokenises_footnote_definition_and_continuation() {
        let body = "He spoke well.\n\n[^1]: Cited from the *Iliad* 2.100,\nsecond line of the citation.\n\n@ Next.\nMore dialogue.";
        let blocks = parse_blocks(body);
        assert_eq!(
            blocks.iter().map(|b| b.kind).collect::<Vec<_>>(),
            vec![
                BlockKind::Prose,
                BlockKind::Footnote,
                BlockKind::Speaker,
                BlockKind::Prose,
            ]
        );
        assert_eq!(blocks[1].lines[0], "1");
        assert_eq!(
            blocks[1].lines[1],
            "Cited from the *Iliad* 2.100, second line of the citation."
        );
    }

    /// A Greek edition's footnotes are English citations, and the Greek
    /// splitter has no abbreviation filter — it breaks on any `. ` — so an
    /// abbreviated citation silently becomes two footnote sentences. The
    /// expanded form DEC-17 mandates is therefore also a mechanical
    /// requirement, not only a style one. `11.569` is safe: the splitter
    /// requires whitespace after the terminator.
    #[test]
    fn greek_splitter_breaks_abbreviated_citations_but_not_expanded_ones() {
        let one = |raw: &str| {
            build_footnote_data(
                1,
                &FootnoteContent {
                    text: raw.to_string(),
                    original_text: None,
                },
                split_sentences_grc,
            )
            .sentences
            .len()
        };
        assert_eq!(one("Homer, *Odyssey* 11.569."), 1);
        assert_eq!(one("Pindar, fragment 169 Snell-Maehler."), 1);
        assert_eq!(one("Euripides, *Antiope*, fragment 638 Nauck."), 1);
        // Two, not three: the boundary after `fr.` is dropped because what
        // follows it holds no letter, which is the splitter's guard against
        // emitting an empty quotable unit.
        assert_eq!(
            one("Pind. fr. 169."),
            2,
            "abbreviations split — do not use them"
        );
    }

    #[test]
    fn footnote_continuation_stops_at_next_block() {
        let body = "[^1]: A short note.\n@ Speaker.\nDialogue.";
        let blocks = parse_blocks(body);
        assert_eq!(blocks[0].kind, BlockKind::Footnote);
        assert_eq!(blocks[0].lines[1], "A short note.");
        assert_eq!(blocks[1].kind, BlockKind::Speaker);
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
        let footnotes = FootnoteLookup::default();
        let ctx = test_ctx("1873", &footnotes);
        let mut sn = 1;
        let b = prose_block(
            "Act",
            0,
            &["Take that. And that.".into()],
            None,
            0,
            &mut sn,
            &ctx,
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

    #[test]
    fn tag_stage_directions_spares_emphasis() {
        // Emphasis stays a bare <i>; only wholly-parenthesized runs get the class.
        assert_eq!(tag_stage_directions("<i>word</i>"), "<i>word</i>");
        assert_eq!(
            tag_stage_directions("<i>(dir)</i>"),
            "<i class=\"stage\">(dir)</i>"
        );
        assert_eq!(
            tag_stage_directions("a <i>one</i> b <i>(x y)</i>"),
            "a <i>one</i> b <i class=\"stage\">(x y)</i>"
        );
    }

    #[test]
    fn label_block_stage_is_numbered_others_inert() {
        let footnotes = FootnoteLookup::default();
        let ctx = test_ctx("1873", &footnotes);
        let stage = label_block("stage", "(Easter night.)", None, 0, &ctx, Some(7));
        assert_eq!(stage.block_type, "stage");
        assert_eq!(stage.sentences[0].sentence_number, Some(7));
        let head = label_block("heading", "ACT ONE", None, 0, &ctx, None);
        assert_eq!(head.sentences[0].sentence_number, None);
    }

    #[test]
    fn prose_block_isolates_between_sentence_direction() {
        let footnotes = FootnoteLookup::default();
        let ctx = test_ctx("1873", &footnotes);
        let mut sn = 1;
        let b = prose_block(
            "Act",
            0,
            &["He shall pay at the stake. *(draws aside.)* Oh, let us hold.".into()],
            None,
            0,
            &mut sn,
            &ctx,
        )
        .unwrap();
        assert_eq!(b.sentences.len(), 3);
        // Dialogue on either side stays clean and numbered.
        assert_eq!(b.sentences[0].text, "He shall pay at the stake.");
        assert_eq!(b.sentences[2].text, "Oh, let us hold.");
        // The direction is its own numbered, muted sentence.
        assert_eq!(b.sentences[1].text, "(draws aside.)");
        assert_eq!(b.sentences[1].html, "<i class=\"stage\">(draws aside.)</i>");
        assert_eq!(b.sentences[1].sentence_number, Some(2));
        assert_eq!(sn, 4);
    }

    #[test]
    fn prose_block_keeps_mid_sentence_direction_inline() {
        let footnotes = FootnoteLookup::default();
        let ctx = test_ctx("1873", &footnotes);
        let mut sn = 1;
        let b = prose_block(
            "Act",
            0,
            &["This man *(points toward Maximus)* is here.".into()],
            None,
            0,
            &mut sn,
            &ctx,
        )
        .unwrap();
        assert_eq!(b.sentences.len(), 1);
        assert_eq!(
            b.sentences[0].html,
            "This man <i class=\"stage\">(points toward Maximus)</i> is here."
        );
        assert_eq!(b.sentences[0].sentence_number, Some(1));
    }

    #[test]
    fn prose_block_keeps_multi_sentence_direction_whole() {
        // A direction carrying its own sentence punctuation stays one unit.
        let footnotes = FootnoteLookup::default();
        let ctx = test_ctx("1873", &footnotes);
        let mut sn = 1;
        let b = prose_block(
            "Act",
            0,
            &["Look. *(he rises. He walks.)* Now go.".into()],
            None,
            0,
            &mut sn,
            &ctx,
        )
        .unwrap();
        assert_eq!(b.sentences.len(), 3);
        assert_eq!(b.sentences[1].text, "(he rises. He walks.)");
        assert_eq!(
            b.sentences[1].html,
            "<i class=\"stage\">(he rises. He walks.)</i>"
        );
    }

    #[test]
    fn prose_block_two_layer_isolation_keeps_parity_and_original() {
        let footnotes = FootnoteLookup::default();
        let ctx = test_ctx("1873", &footnotes);
        let mut sn = 1;
        let b = prose_block(
            "Act",
            0,
            &["Stop. *(he turns.)* Come here.".into()],
            Some(&["Stopp. *(han vender seg.)* Kom hit.".into()]),
            0,
            &mut sn,
            &ctx,
        )
        .unwrap();
        assert_eq!(b.sentences.len(), 3);
        assert_eq!(b.sentences[1].text, "(he turns.)");
        assert_eq!(
            b.sentences[1].original_text.as_deref(),
            Some("(han vender seg.)")
        );
        assert_eq!(
            b.sentences[1].original_html.as_deref(),
            Some("<i class=\"stage\">(han vender seg.)</i>")
        );
        assert_eq!(b.sentences[1].sentence_number, Some(2));
    }

    #[test]
    fn stephanus_subpage_sort_order_is_strictly_ascending() {
        assert_eq!(subpage_sort_order("447a"), 4470);
        assert_eq!(subpage_sort_order("447e"), 4474);
        assert_eq!(subpage_sort_order("448a"), 4480);
        assert_eq!(subpage_sort_order("448"), 4480);
    }

    #[test]
    fn page_marker_sort_order_respects_subpage_flag() {
        let footnotes = FootnoteLookup::default();
        let m = RawMarker {
            value: "447a".into(),
            char_offset: 0,
        };
        let mut ctx = test_ctx("gorgias", &footnotes);
        ctx.subpage_letter_sort = true;
        assert_eq!(page_marker(&ctx, &m, 0).sort_order, 4470);

        ctx.subpage_letter_sort = false;
        // Off: today's exact behaviour — parse::<i32>() fails on "447a", so
        // every Stephanus marker would collide at 0. That's precisely why
        // the flag exists; ibsen1 never sets it.
        assert_eq!(page_marker(&ctx, &m, 0).sort_order, 0);

        let digits_only = RawMarker {
            value: "12".into(),
            char_offset: 0,
        };
        assert_eq!(page_marker(&ctx, &digits_only, 0).sort_order, 12);
    }

    #[test]
    fn extract_footnotes_assigns_global_numbers_and_pairs_layers() {
        let modern = parse_blocks("Said the poet.[^*]\n\n[^*]: Iliad 2.100.");
        let reviewed = parse_blocks("Sagt av dikteren.[^*]\n\n[^*]: Iliad 2.100 (norsk).");
        let mut n = 0;
        let lookup = extract_footnotes("test", &modern, Some(&reviewed), &mut n).unwrap();
        assert_eq!(n, 1);
        assert_eq!(lookup.marker_to_number.get("*"), Some(&1));
        let content = lookup.by_number.get(&1).unwrap();
        assert_eq!(content.text, "Iliad 2.100.");
        assert_eq!(
            content.original_text.as_deref(),
            Some("Iliad 2.100 (norsk).")
        );
    }

    #[test]
    fn extract_footnotes_errors_on_layer_mismatch() {
        let modern = parse_blocks("Said the poet.[^*]\n\n[^*]: Iliad 2.100.");
        let reviewed = parse_blocks("Sagt av dikteren.[^*]");
        let mut n = 0;
        assert!(extract_footnotes("test", &modern, Some(&reviewed), &mut n).is_err());
    }

    #[test]
    fn prose_block_attaches_footnote_to_sentence() {
        let mut footnotes = FootnoteLookup::default();
        footnotes.marker_to_number.insert("1".into(), 1);
        footnotes.by_number.insert(
            1,
            FootnoteContent {
                text: "Iliad 2.100.".into(),
                original_text: None,
            },
        );
        let ctx = test_ctx("gorgias", &footnotes);
        let mut sn = 1;
        let b = prose_block(
            "Act",
            0,
            &["As the poet says.[^1]".into()],
            None,
            0,
            &mut sn,
            &ctx,
        )
        .unwrap();
        assert_eq!(b.sentences.len(), 1);
        assert_eq!(b.sentences[0].text, "As the poet says.");
        assert_eq!(b.sentences[0].html, "As the poet says.<sup>1</sup>");
        assert_eq!(b.sentences[0].footnotes.len(), 1);
        assert_eq!(b.sentences[0].footnotes[0].number, 1);
        assert_eq!(
            b.sentences[0].footnotes[0].sentences[0].text,
            "Iliad 2.100."
        );
    }

    #[test]
    fn prose_block_without_footnote_defs_is_unaffected() {
        let footnotes = FootnoteLookup::default();
        let ctx = test_ctx("1873", &footnotes);
        let mut sn = 1;
        let b = prose_block(
            "Act",
            0,
            &["Take that. And that.".into()],
            None,
            0,
            &mut sn,
            &ctx,
        )
        .unwrap();
        assert!(b.sentences.iter().all(|s| s.footnotes.is_empty()));
    }

    #[test]
    fn prose_block_greek_splitter_skips_direction_peeling() {
        let footnotes = FootnoteLookup::default();
        let mut ctx = test_ctx("s", &footnotes);
        ctx.greek_splitter = true;
        ctx.splitter = split_sentences_grc;
        let mut sn = 1;
        let b = prose_block(
            "Act",
            0,
            &["He shall pay at the stake. *(draws aside.)* Oh, let us hold.".into()],
            None,
            0,
            &mut sn,
            &ctx,
        )
        .unwrap();
        // Unlike the structural path (see
        // prose_block_isolates_between_sentence_direction), the Greek path
        // performs no direction peeling: the direction never stands alone as
        // its own `<i class="stage">…</i>`-only sentence.
        assert!(
            b.sentences
                .iter()
                .all(|s| s.html != "<i class=\"stage\">(draws aside.)</i>")
        );
    }
}
