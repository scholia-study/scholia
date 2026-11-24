//! Blocks → nodes: anchor every TOC entry at exactly one heading run, slice
//! the flow between anchors, and write each node's markdown.

use std::collections::{BTreeSet, HashMap};

use common::hegel4::toc::TocEntry;
use regex::Regex;

use crate::Result;
use crate::indent_runs::IndentRuns;
use crate::page_joins::{Join, PageJoins};
use crate::render::{PageState, Renderer, gw_marker};
use crate::source::{Item, Piece, Source};

#[derive(Debug, Clone)]
pub enum Block {
    Head {
        text: String,
        page: u32,
        marker: Option<u32>,
    },
    Para {
        remark: bool,
        text: String,
    },
}

/// Heading runs the print shows but the TOC deliberately leaves out: the
/// half-titles that precede "Zweiter Theil" and "Dritter Theil".
const EXCLUDED_HEAD_RUNS: &[&str] = &["ii naturphilosophie", "iii philosophie des geistes"];

pub fn build_blocks(
    source: &Source,
    renderer: &mut Renderer,
    joins: &PageJoins,
) -> Result<Vec<Block>> {
    let mut state = PageState::default();
    let mut blocks = Vec::new();
    let items = merge_split_paragraphs(&source.items, joins, &mut renderer.joined_paragraphs)?;
    for item in &items {
        match item {
            Item::Pb(page) => {
                state.page = *page;
                if let Some(lost) = state.pending.replace(*page) {
                    renderer.skipped_pages.push(lost);
                }
            }
            Item::Head(pieces) => blocks.push(Block::Head {
                text: heading_text(pieces)?,
                page: state.page,
                marker: state.pending.take(),
            }),
            Item::Para { remark, pieces } => {
                let text = renderer.render(pieces, &mut state)?;
                if text.is_empty() {
                    return Err(format!("empty paragraph on GW 20.{}", state.page));
                }
                blocks.push(Block::Para {
                    remark: *remark,
                    text,
                });
            }
        }
    }
    if !renderer.unresolved.is_empty() {
        return Err(renderer.unresolved.join("\n"));
    }
    if let Some(unused) = renderer
        .notes
        .keys()
        .find(|id| !renderer.consumed.contains(*id))
    {
        return Err(format!("note {unused} is never placed"));
    }
    Ok(blocks)
}

/// A page boundary where the XML closes a `<p>` and opens another is decided
/// by `page_joins.tsv`. The old shape test (the first part ends on a hyphen,
/// comma, semicolon or letter and the second begins lowercase) only detects:
/// a boundary where it fires and no row exists is an error, as is a row that
/// matches no split boundary. A joined continuation takes the first
/// paragraph's kind (the edition sometimes drops the small-type rendition on
/// it).
fn merge_split_paragraphs(
    items: &[Item],
    joins: &PageJoins,
    log: &mut Vec<String>,
) -> Result<Vec<Item>> {
    let mut out: Vec<Item> = Vec::new();
    let mut breaks: Vec<u32> = Vec::new();
    let mut seen: BTreeSet<u32> = BTreeSet::new();
    for item in items {
        match item {
            Item::Pb(page) => {
                breaks.push(*page);
                out.push(item.clone());
            }
            Item::Para { pieces, .. } => {
                let prev = out
                    .len()
                    .checked_sub(breaks.len() + 1)
                    .filter(|_| !breaks.is_empty());
                let split = prev.filter(|&i| matches!(out[i], Item::Para { .. }));
                let mut join_at = None;
                if let Some(i) = split {
                    let Item::Para {
                        pieces: prev_pieces,
                        ..
                    } = &out[i]
                    else {
                        unreachable!()
                    };
                    let page = breaks[0];
                    seen.insert(page);
                    let detected = last_char(prev_pieces)
                        .is_some_and(|c| c == '-' || c == ',' || c == ';' || c.is_alphabetic())
                        && first_char(pieces).is_some_and(char::is_lowercase);
                    match joins.get(page) {
                        Some(Join::Join) => join_at = Some(i),
                        Some(Join::Keep) => {}
                        None if detected => {
                            return Err(format!(
                                "paragraph split at GW 20.{page} looks like one paragraph (…{} | {}…); add a row to page_joins.tsv",
                                last_chars(prev_pieces, 24),
                                first_chars(pieces, 24)
                            ));
                        }
                        None => {}
                    }
                }
                if let Some(i) = join_at {
                    let tail = out.split_off(i + 1);
                    let Item::Para { pieces: merged, .. } = &mut out[i] else {
                        unreachable!()
                    };
                    log.push(format!(
                        "GW 20.{}: …{} | {}…",
                        breaks[0],
                        last_chars(merged, 24),
                        first_chars(pieces, 24)
                    ));
                    merged.push(Piece::Lb);
                    for pb in tail {
                        if let Item::Pb(page) = pb {
                            merged.push(Piece::Pb(page));
                        }
                    }
                    merged.extend(pieces.iter().cloned());
                } else {
                    out.push(item.clone());
                }
                breaks.clear();
            }
            Item::Head(_) => {
                breaks.clear();
                out.push(item.clone());
            }
        }
    }
    let stale = joins.stale(&seen);
    if !stale.is_empty() {
        return Err(format!(
            "page_joins.tsv rows match no split paragraph boundary: GW 20.{stale:?}"
        ));
    }
    Ok(out)
}

fn text_of(pieces: &[Piece]) -> String {
    pieces
        .iter()
        .filter_map(|p| match p {
            Piece::Text(t) => Some(t.as_str()),
            _ => None,
        })
        .collect::<String>()
        .split_whitespace()
        .collect::<Vec<_>>()
        .join(" ")
}

fn last_char(pieces: &[Piece]) -> Option<char> {
    text_of(pieces).chars().next_back()
}

fn first_char(pieces: &[Piece]) -> Option<char> {
    text_of(pieces).chars().next()
}

fn last_chars(pieces: &[Piece], n: usize) -> String {
    let t = text_of(pieces);
    t.chars()
        .skip(t.chars().count().saturating_sub(n))
        .collect()
}

fn first_chars(pieces: &[Piece], n: usize) -> String {
    text_of(pieces).chars().take(n).collect()
}

fn heading_text(pieces: &[Piece]) -> Result<String> {
    let mut text = String::new();
    for piece in pieces {
        match piece {
            Piece::Text(t) => text.push_str(&t.replace(['\n', '\r', '\t'], " ")),
            Piece::Lb => {
                if text.ends_with('-') {
                    return Err(format!("line-end hyphen inside heading {text:?}"));
                }
                text.push(' ');
            }
            Piece::Open | Piece::Close => {}
            Piece::Pb(_) | Piece::Note(_) => unreachable!("rejected at parse time"),
        }
    }
    Ok(text.split_whitespace().collect::<Vec<_>>().join(" "))
}

pub fn norm_match(s: &str) -> String {
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

pub fn paragraph_number(head: &str) -> Option<u32> {
    let re = Regex::new(r"^§\.\s*(\d+)\.$").unwrap();
    re.captures(head).and_then(|c| c[1].parse().ok())
}

#[derive(Debug)]
pub struct NodeOut {
    pub file_name: String,
    pub content: String,
    pub footnotes: usize,
    pub sections: Vec<u32>,
    pub remark_fences: usize,
    pub queued_sections: Vec<(u32, &'static str)>,
    pub moved_headings: usize,
    indented_sections: Vec<u32>,
}

pub fn assemble(
    blocks: Vec<Block>,
    entries: &[TocEntry],
    definitions: &HashMap<String, String>,
    indents: &IndentRuns,
    file_name: impl Fn(&TocEntry) -> String,
) -> Result<Vec<NodeOut>> {
    let anchors = anchor_entries(&blocks, entries)?;
    if let Some(first) = anchors.first()
        && first.0 > 0
    {
        return Err(format!(
            "{} block(s) precede the first node ({:?} ...)",
            first.0,
            describe(&blocks[0])
        ));
    }
    let mut nodes = Vec::new();
    let mut last_section = 0u32;
    let mut open_section: (Option<u32>, usize) = (None, 0);
    let mut moved: Vec<Block> = Vec::new();
    let mut moved_marker: Option<u32> = None;
    for (i, entry) in entries.iter().enumerate() {
        let (start, run) = anchors[i];
        let end = anchors.get(i + 1).map_or(blocks.len(), |a| a.0);
        let Block::Head { marker, .. } = &blocks[start] else {
            unreachable!()
        };
        if marker.is_some() && moved_marker.is_some() {
            return Err(format!(
                "{:?} starts a page, and so does the § heading moved into it",
                entry.label
            ));
        }
        let mut node = NodeWriter::new(entry, marker.or(moved_marker.take()), indents);
        (node.section, node.remark_runs_in_section) = open_section;
        let carried = std::mem::take(&mut moved);
        node.moved_headings = carried.len();
        let mut body: Vec<&Block> = carried.iter().collect();
        body.extend(&blocks[start + run..end]);
        let mut trailing = body.len();
        while trailing > 0 && is_section_head(body[trailing - 1]) {
            trailing -= 1;
        }
        if trailing < body.len() {
            if i + 1 == entries.len() {
                return Err(format!(
                    "{:?} ends on a § heading with no text",
                    entry.label
                ));
            }
            moved = body.split_off(trailing).into_iter().cloned().collect();
            for block in &mut moved {
                let Block::Head { marker, .. } = block else {
                    unreachable!()
                };
                if let Some(page) = marker.take() {
                    if moved_marker.replace(page).is_some() {
                        return Err(format!(
                            "two moved § headings start pages in {:?}",
                            entry.label
                        ));
                    }
                }
            }
        }
        node.write_body(&body, &mut last_section)?;
        open_section = (node.section, node.remark_runs_in_section);
        nodes.push(node.finish(file_name(entry), definitions)?);
    }
    let used: Vec<u32> = nodes
        .iter()
        .flat_map(|n| n.indented_sections.iter().copied())
        .collect();
    let stale = indents.stale(&used);
    if !stale.is_empty() {
        return Err(format!(
            "indent_runs.tsv rows match no run of main-text paragraphs: § {stale:?}"
        ));
    }
    Ok(nodes)
}

fn is_section_head(block: &Block) -> bool {
    matches!(block, Block::Head { text, .. } if paragraph_number(text).is_some())
}

fn describe(block: &Block) -> String {
    match block {
        Block::Head { text, page, .. } => format!("head {text:?} on GW 20.{page}"),
        Block::Para { text, .. } => format!("paragraph {:.40?}", text),
    }
}

fn anchor_entries(blocks: &[Block], entries: &[TocEntry]) -> Result<Vec<(usize, usize)>> {
    let mut anchors: Vec<(usize, usize)> = Vec::new();
    let mut cursor = 0usize;
    for entry in entries {
        let page: u32 = entry
            .page
            .strip_prefix("20.")
            .and_then(|p| p.parse().ok())
            .ok_or_else(|| format!("bad TOC page {:?}", entry.page))?;
        let want = norm_match(entry.label);
        let mut found: Vec<(usize, usize)> = Vec::new();
        for bi in cursor..blocks.len() {
            let Block::Head { page: hp, .. } = &blocks[bi] else {
                continue;
            };
            if *hp != page {
                continue;
            }
            let mut joined = String::new();
            for k in 0..4usize {
                let Some(Block::Head { text, page: rp, .. }) = blocks.get(bi + k) else {
                    break;
                };
                if *rp != page {
                    break;
                }
                if !joined.is_empty() {
                    joined.push(' ');
                }
                joined.push_str(&norm_match(text));
                if joined == want {
                    found.push((bi, k + 1));
                    break;
                }
            }
        }
        match found.as_slice() {
            [one] => {
                anchors.push(*one);
                cursor = one.0 + one.1;
            }
            [] => {
                return Err(format!(
                    "cannot anchor TOC entry {} ({:?}) at GW 20.{page}",
                    entry.position, entry.label
                ));
            }
            many => {
                return Err(format!(
                    "TOC entry {} ({:?}) anchors at {} heading runs on GW 20.{page}",
                    entry.position,
                    entry.label,
                    many.len()
                ));
            }
        }
    }
    Ok(anchors)
}

struct NodeWriter<'e> {
    entry: &'e TocEntry,
    blocks: Vec<String>,
    heading_marker: Option<u32>,
    fence_open: bool,
    fences: usize,
    remark_runs_in_section: usize,
    section: Option<u32>,
    sections: Vec<u32>,
    queued: Vec<(u32, &'static str)>,
    excluded_run: String,
    indents: &'e IndentRuns,
    main_in_section: usize,
    indent_pending: Vec<String>,
    indented_sections: Vec<u32>,
    moved_headings: usize,
}

impl<'e> NodeWriter<'e> {
    fn new(entry: &'e TocEntry, heading_marker: Option<u32>, indents: &'e IndentRuns) -> Self {
        NodeWriter {
            entry,
            blocks: Vec::new(),
            heading_marker,
            fence_open: false,
            fences: 0,
            remark_runs_in_section: 0,
            section: None,
            sections: Vec::new(),
            queued: Vec::new(),
            excluded_run: String::new(),
            indents,
            main_in_section: 0,
            indent_pending: Vec::new(),
            indented_sections: Vec::new(),
            moved_headings: 0,
        }
    }

    fn close_fence(&mut self) {
        if self.fence_open {
            if let Some(last) = self.blocks.last_mut() {
                last.push_str("\n:::");
            }
            self.fence_open = false;
        }
    }

    fn write_body(&mut self, body: &[&Block], last_section: &mut u32) -> Result<()> {
        let mut main_after_remark = false;
        for block in body {
            match block {
                Block::Head { text, marker, .. } => {
                    if let Some(n) = paragraph_number(text) {
                        if !self.indent_pending.is_empty() {
                            return Err(format!("§ {n} starts inside an indent run"));
                        }
                        self.close_fence();
                        self.note_section_end(main_after_remark);
                        main_after_remark = false;
                        if n != *last_section + 1 {
                            return Err(format!(
                                "§ numbering breaks: {n} follows {last_section} in {:?}",
                                self.entry.label
                            ));
                        }
                        *last_section = n;
                        self.section = Some(n);
                        self.sections.push(n);
                        self.remark_runs_in_section = 0;
                        self.main_in_section = 0;
                        let gw = marker.map(gw_marker).unwrap_or_default();
                        self.blocks.push(format!("## {gw}{{{{ {n} }}}} §. {n}."));
                        continue;
                    }
                    self.excluded_run = if self.excluded_run.is_empty() {
                        norm_match(text)
                    } else {
                        format!("{} {}", self.excluded_run, norm_match(text))
                    };
                    let known = EXCLUDED_HEAD_RUNS.iter().any(|r| *r == self.excluded_run);
                    let prefix = EXCLUDED_HEAD_RUNS
                        .iter()
                        .any(|r| r.starts_with(&self.excluded_run));
                    if known {
                        self.excluded_run.clear();
                    } else if !prefix {
                        return Err(format!(
                            "heading {text:?} inside {:?} is neither a § nor an excluded half-title",
                            self.entry.label
                        ));
                    }
                }
                Block::Para {
                    remark: false,
                    text,
                } => {
                    if self.fence_open || self.remark_runs_in_section > 0 {
                        main_after_remark = true;
                    }
                    self.close_fence();
                    self.main_in_section += 1;
                    let range = self.section.and_then(|n| self.indents.get(n));
                    match range {
                        Some((first, last)) if (first..=last).contains(&self.main_in_section) => {
                            self.indent_pending.push(format!("+ {text}"));
                            if self.main_in_section == last {
                                self.indented_sections.push(self.section.unwrap());
                                let run = std::mem::take(&mut self.indent_pending).join("\n");
                                self.blocks.push(run);
                            }
                        }
                        _ => self.blocks.push(text.clone()),
                    }
                }
                Block::Para { remark: true, text } => {
                    let Some(n) = self.section else {
                        return Err(format!(
                            "Remark outside any § in {:?}: {text:.40}",
                            self.entry.label
                        ));
                    };
                    if !self.indent_pending.is_empty() {
                        return Err(format!("§ {n}: a Remark interrupts an indent run"));
                    }
                    let mut text = text.clone();
                    if !self.fence_open {
                        self.fence_open = true;
                        self.fences += 1;
                        self.remark_runs_in_section += 1;
                        let run = self.remark_runs_in_section;
                        if run > 9 {
                            return Err(format!("§ {n} has more than nine Remark runs"));
                        }
                        let suffix = if run == 1 {
                            "A".to_string()
                        } else {
                            format!("A{run}")
                        };
                        text = insert_after_gw_markers(&text, &format!("{{{{ {n} {suffix} }}}} "));
                        text = format!("::: remark\n{text}");
                    }
                    self.blocks.push(text);
                }
            }
        }
        self.close_fence();
        if !self.indent_pending.is_empty() {
            return Err(format!("indent run left open in {:?}", self.entry.label));
        }
        self.note_section_end(main_after_remark);
        if !self.excluded_run.is_empty() {
            return Err(format!("dangling excluded heading {:?}", self.excluded_run));
        }
        Ok(())
    }

    fn note_section_end(&mut self, main_after_remark: bool) {
        let Some(n) = self.section else { return };
        if main_after_remark {
            self.queued.push((n, "main text resumes after a Remark"));
        }
    }

    fn finish(self, file_name: String, definitions: &HashMap<String, String>) -> Result<NodeOut> {
        let entry = self.entry;
        let heading = match self.heading_marker {
            Some(p) => format!("## {}{}", gw_marker(p), entry.label),
            None => format!("## {}", entry.label),
        };
        let mut body = self.blocks.join("\n\n");

        let call = Regex::new(r"\[\^@([^\]]+)\]").unwrap();
        let mut order: Vec<String> = Vec::new();
        body = call
            .replace_all(&body, |c: &regex::Captures| {
                let id = c[1].to_string();
                let n = match order.iter().position(|x| *x == id) {
                    Some(i) => i + 1,
                    None => {
                        order.push(id);
                        order.len()
                    }
                };
                format!("[^{n}]")
            })
            .into_owned();

        let mut content = format!(
            "---\nposition: {}\nlabel: \"{}\"\ndepth: {}\npage_gw: \"{}\"\n---\n\n{heading}\n",
            entry.position, entry.label, entry.depth, entry.page
        );
        if !body.is_empty() {
            content.push('\n');
            content.push_str(&body);
            content.push('\n');
        }
        for (i, id) in order.iter().enumerate() {
            let def = definitions
                .get(id)
                .ok_or_else(|| format!("footnote {id} has no definition"))?;
            content.push_str(&format!("\n[^{}]: {def}\n", i + 1));
        }
        Ok(NodeOut {
            file_name,
            content,
            footnotes: order.len(),
            sections: self.sections,
            remark_fences: self.fences,
            queued_sections: self.queued,
            moved_headings: self.moved_headings,
            indented_sections: self.indented_sections,
        })
    }
}

fn insert_after_gw_markers(text: &str, marker: &str) -> String {
    let re = Regex::new(r"^(?:\{\{\{[^}]*\}\}\} )*").unwrap();
    let end = re.find(text).map_or(0, |m| m.end());
    format!("{}{marker}{}", &text[..end], &text[end..])
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::hyphen::{Oracle, Rulings};
    use crate::source;

    static TOC: &[TocEntry] = &[
        TocEntry {
            position: 1,
            depth: 1,
            page: "20.5",
            slug: "a_erste_stellung",
            label: "A. Erste Stellung",
        },
        TocEntry {
            position: 2,
            depth: 1,
            page: "20.7",
            slug: "b_zweite",
            label: "B. Zweite",
        },
    ];

    fn run(inner: &str) -> Result<Vec<NodeOut>> {
        let xml = format!(
            "<TEI xmlns=\"http://www.tei-c.org/ns/1.0\"><text><body>{inner}</body></text></TEI>"
        );
        let src = source::parse(&xml)?;
        let oracle = Oracle::default();
        let rulings = Rulings::default();
        let mut r = Renderer::new(&oracle, &rulings, &src.notes);
        let blocks = build_blocks(&src, &mut r, &PageJoins::default())?;
        assemble(blocks, TOC, &r.definitions, &IndentRuns::default(), |e| {
            format!("{:03}.md", e.position)
        })
    }

    #[test]
    fn split_heads_join_into_one_anchor_and_the_marker_lands_on_the_heading() {
        let nodes = run(
            "<pb n=\"5\"/><div><head>A.</head></div><div><head>ERSTE STELLUNG</head></div>\
             <div><head>§. 1.</head><p>Haupt.</p></div>\
             <pb n=\"7\"/><div><head>B.</head></div><div><head>ZWEITE</head></div>",
        )
        .unwrap();
        assert!(
            nodes[0]
                .content
                .contains("## {{{ 20.5 }}} A. Erste Stellung\n\n## {{ 1 }} §. 1.\n\nHaupt.")
        );
        assert!(nodes[1].content.contains("## {{{ 20.7 }}} B. Zweite\n"));
        assert_eq!(nodes[0].sections, [1]);
    }

    #[test]
    fn a_page_start_before_a_section_heading_puts_both_markers_on_it() {
        let nodes = run("<pb n=\"5\"/><div><head>A. ERSTE STELLUNG</head></div>\
             <pb n=\"6\"/><div><head>§. 1.</head><p>x</p></div>\
             <pb n=\"7\"/><div><head>B. ZWEITE</head></div>")
        .unwrap();
        assert!(
            nodes[0]
                .content
                .contains("## {{{ 20.6 }}} {{ 1 }} §. 1.\n\nx")
        );
    }

    #[test]
    fn remark_runs_become_fences_opened_by_the_a_marker() {
        let nodes = run("<pb n=\"5\"/><div><head>A. ERSTE STELLUNG</head></div>\
             <div><head>§. 1.</head><p>Haupt.</p>\
             <p rendition=\"#et #smaller\">Erste.</p><pb n=\"6\"/>\
             <p rendition=\"#et #smaller\">Zweite.</p></div>\
             <pb n=\"7\"/><div><head>B. ZWEITE</head></div>")
        .unwrap();
        assert!(
            nodes[0]
                .content
                .contains("Haupt.\n\n::: remark\n{{ 1 A }} Erste.\n\n{{{ 20.6 }}} Zweite.\n:::\n"),
            "{}",
            nodes[0].content
        );
        assert_eq!(nodes[0].remark_fences, 1);
        assert!(nodes[0].queued_sections.is_empty());
    }

    #[test]
    fn a_second_remark_run_is_fenced_and_marked_a2() {
        let nodes = run("<pb n=\"5\"/><div><head>A. ERSTE STELLUNG</head></div>\
             <div><head>§. 1.</head><p>Haupt.</p>\
             <p rendition=\"#et #smaller\">Erste.</p><p>Wieder Haupt.</p>\
             <p rendition=\"#et #smaller\">Zweite.</p></div>\
             <pb n=\"7\"/><div><head>B. ZWEITE</head></div>")
        .unwrap();
        let c = &nodes[0].content;
        assert_eq!(c.matches("{{ 1 A }}").count(), 1);
        assert_eq!(c.matches("{{ 1 A2 }}").count(), 1);
        assert_eq!(nodes[0].remark_fences, 2);
        assert_eq!(nodes[0].queued_sections.len(), 1);
    }

    #[test]
    fn sections_must_be_numbered_in_sequence() {
        let nodes = run("<pb n=\"5\"/><div><head>A. ERSTE STELLUNG</head></div>\
             <div><head>§. 2.</head><p>x</p></div>\
             <pb n=\"7\"/><div><head>B. ZWEITE</head></div>");
        assert!(nodes.unwrap_err().contains("numbering"));
    }

    #[test]
    fn an_unanchored_toc_entry_is_refused() {
        let nodes = run("<pb n=\"5\"/><div><head>A. ERSTE STELLUNG</head></div>");
        assert!(nodes.unwrap_err().contains("cannot anchor"));
    }

    #[test]
    fn footnotes_are_renumbered_per_node_with_definitions_at_the_end() {
        let nodes = run(
            "<pb n=\"5\"/><div><head>A. ERSTE STELLUNG</head></div>\
             <div><p>Satz<note place=\"foot\" xml:id=\"zz\">Text</note> mehr<note place=\"foot\" xml:id=\"yy\">Andere</note>.</p></div>\
             <pb n=\"7\"/><div><head>B. ZWEITE</head></div>",
        )
        .unwrap();
        assert!(nodes[0].content.contains("Satz[^1] mehr[^2]."));
        assert!(nodes[0].content.ends_with("\n[^1]: Text\n\n[^2]: Andere\n"));
        assert_eq!(nodes[0].footnotes, 2);
    }

    #[test]
    fn an_undecided_split_boundary_is_refused_and_a_stale_row_too() {
        let xml = "<pb n=\"5\"/><div><head>A. ERSTE STELLUNG</head></div>\
             <div><head>§. 1.</head><p>Haupt, und</p><pb n=\"6\"/><p>weiter.</p></div>\
             <pb n=\"7\"/><div><head>B. ZWEITE</head></div>";
        assert!(run(xml).is_err());
        let src = source::parse(&format!(
            "<TEI xmlns=\"http://www.tei-c.org/ns/1.0\"><text><body>{xml}</body></text></TEI>"
        ))
        .unwrap();
        let log = &mut Vec::new();
        let joined = PageJoins::parse("6\tjoin\tcertain\t\n").unwrap();
        assert_eq!(
            merge_split_paragraphs(&src.items, &joined, log)
                .unwrap()
                .len(),
            6
        );
        let stale = PageJoins::parse("6\tjoin\tcertain\t\n9\tkeep\tuncertain\t\n").unwrap();
        assert!(merge_split_paragraphs(&src.items, &stale, log).is_err());
    }

    fn run_with(inner: &str, indents: &IndentRuns) -> Result<Vec<NodeOut>> {
        let xml = format!(
            "<TEI xmlns=\"http://www.tei-c.org/ns/1.0\"><text><body>{inner}</body></text></TEI>"
        );
        let src = source::parse(&xml)?;
        let (oracle, rulings) = (Oracle::default(), Rulings::default());
        let mut r = Renderer::new(&oracle, &rulings, &src.notes);
        let blocks = build_blocks(&src, &mut r, &PageJoins::default())?;
        assemble(blocks, TOC, &r.definitions, indents, |e| {
            format!("{:03}.md", e.position)
        })
    }

    #[test]
    fn an_indent_run_joins_main_paragraphs_as_plus_lines() {
        let indents = IndentRuns::parse("1\t2\t3\t\n").unwrap();
        let nodes = run_with(
            "<pb n=\"5\"/><div><head>A. ERSTE STELLUNG</head></div>\
             <div><head>§. 1.</head><p>Haupt.</p><p>1) Eins.</p><pb n=\"6\"/><p>2) Zwei.</p></div>\
             <pb n=\"7\"/><div><head>B. ZWEITE</head></div>",
            &indents,
        )
        .unwrap();
        assert!(
            nodes[0]
                .content
                .contains("Haupt.\n\n+ 1) Eins.\n+ {{{ 20.6 }}} 2) Zwei.\n"),
            "{}",
            nodes[0].content
        );
    }

    #[test]
    fn an_indent_run_that_matches_nothing_is_refused() {
        let indents = IndentRuns::parse("9\t2\t3\t\n").unwrap();
        let err = run_with(
            "<pb n=\"5\"/><div><head>A. ERSTE STELLUNG</head></div>\
             <div><head>§. 1.</head><p>Haupt.</p></div>\
             <pb n=\"7\"/><div><head>B. ZWEITE</head></div>",
            &indents,
        );
        assert!(err.is_err());
    }

    #[test]
    fn a_trailing_section_heading_moves_to_the_node_it_heads() {
        let nodes = run("<pb n=\"5\"/><div><head>A. ERSTE STELLUNG</head></div>\
             <div><head>§. 1.</head><p>Text.</p></div><div><head>§. 2.</head></div>\
             <pb n=\"7\"/><div><head>B. ZWEITE</head></div><p>Zwei.</p>")
        .unwrap();
        assert!(!nodes[0].content.contains("§. 2."));
        assert!(
            nodes[1]
                .content
                .contains("B. Zweite\n\n## {{ 2 }} §. 2.\n\nZwei.")
        );
        assert_eq!(nodes[1].moved_headings, 1);
    }

    #[test]
    fn a_moved_heading_hands_its_page_marker_to_the_node_heading() {
        let nodes = run("<pb n=\"5\"/><div><head>A. ERSTE STELLUNG</head></div>\
             <div><head>§. 1.</head><p>Text.</p></div><pb n=\"7\"/><div><head>§. 2.</head></div>\
             <div><head>B. ZWEITE</head></div><p>Zwei.</p>")
        .unwrap();
        assert!(
            nodes[1]
                .content
                .contains("## {{{ 20.7 }}} B. Zweite\n\n## {{ 2 }} §. 2.")
        );
    }
}
