//! EEBO-TCP A43998 (Hobbes, *Leviathan*, 1651) TEI → curated `md_reviewed`.
//!
//! Run-once pre-curation converter, diplomatic layer only (TCP normalized
//! long-s already; 1651 spelling is kept as transcribed). Conventions:
//!
//! - `<pb n>` → `{{{ N }}}` (orig1651 page markers, printed values verbatim —
//!   the 1651 misnumberings and duplicate runs are emitted as printed and
//!   listed in the anomaly report, never corrected here)
//! - `<note place="margin">` → ``{{$m `content`}}`` at the anchor position;
//!   the two literal in-text `*` anchor stars are stripped (the token position
//!   carries the anchor, `*` stays reserved for footnotes)
//! - `<hi>` → `_…_` (antiqua/italic); `rend="sup"` flattened + logged
//! - `<g ref="char:EOLhyphen"/>` → halves joined; other `<g>` → glyph text
//! - `<gap>` → its `<desc>` display form verbatim (`•…`, `〈◊〉`, …), inventoried
//!   in gaps.tsv — the modernized layer's fill-in worklist
//! - deep-nested `<list>` (the Chapter IX table of sciences) → `<figure>` with
//!   nested `<ul>`; shallow lists → `+ ` indented runs
//!
//! Output: one `NNN_slug.md` per TOC node (front matter: position, label,
//! depth, page_1651) + gaps.tsv / conversion_report.tsv under
//! assets/hobbes1/derived/.

use clap::Parser;
use regex::Regex;
use roxmltree::{Document, Node};
use std::fmt::Write as _;
use std::fs;
use std::path::PathBuf;
use std::sync::LazyLock;

const TEI: &str = "http://www.tei-c.org/ns/1.0";
/// Zero-width sentinel marking an EOL-hyphen join point; removed together
/// with any adjacent whitespace after collapsing.
const JOIN: &str = "\u{0}J\u{0}";

static WS_RE: LazyLock<Regex> = LazyLock::new(|| Regex::new(r"\s+").unwrap());
static JOIN_RE: LazyLock<Regex> = LazyLock::new(|| Regex::new(r"\s*\u{0}J\u{0}\s*").unwrap());
static CHAP_PREFIX_RE: LazyLock<Regex> =
    LazyLock::new(|| Regex::new(r"^CHAP\. [IVXLC]+\.\s*").unwrap());

#[derive(Parser)]
struct Cli {
    #[arg(long, default_value = "assets/hobbes1/raw/A43998.xml")]
    input: String,
    #[arg(long, default_value = "assets/hobbes1/curated/md_reviewed")]
    out_dir: String,
    #[arg(long, default_value = "assets/hobbes1/derived")]
    report_dir: String,
}

struct Conv {
    /// Last numbered page seen, in document order.
    current_page: Option<String>,
    /// (kind, file, page, detail) — conversion_report.tsv rows.
    report: Vec<(String, String, String, String)>,
    /// (file, page, reason, extent, display, preceding context) — gaps.tsv rows.
    gaps: Vec<(String, String, String, String, String, String)>,
    current_file: String,
    margin_notes: u32,
    stripped_stars: u32,
    unnumbered_pb: u32,
    /// TOC-label extraction renders heads a first time; suppress logging so
    /// the real render pass is the only one that reports.
    quiet: bool,
    /// Final marker value per `<pb>` node: the printed number, with `b`/`c`
    /// suffixed onto repeated occurrences (editorial ruling 2026-08-11 — the
    /// 1651 printer numbered five pages twice; the second count is suffixed
    /// so orig1651 citations stay unambiguous).
    pb_final: std::collections::HashMap<roxmltree::NodeId, String>,
}

impl Conv {
    /// The (possibly suffixed) marker value for a numbered `<pb>`; counts
    /// and skips unnumbered ones.
    fn pb_value(&mut self, pb: &Node) -> Option<String> {
        match self.pb_final.get(&pb.id()) {
            Some(v) => {
                self.current_page = Some(v.clone());
                Some(v.clone())
            }
            None => {
                self.unnumbered_pb += 1;
                None
            }
        }
    }
}

impl Conv {
    fn log(&mut self, kind: &str, detail: String) {
        if self.quiet {
            return;
        }
        self.report.push((
            kind.to_string(),
            self.current_file.clone(),
            self.current_page.clone().unwrap_or_default(),
            detail,
        ));
    }
}

fn is(n: &Node, tag: &str) -> bool {
    n.is_element() && n.tag_name().name() == tag && n.tag_name().namespace() == Some(TEI)
}

fn collapse(s: &str) -> String {
    let s = WS_RE.replace_all(s, " ");
    JOIN_RE.replace_all(&s, "").trim().to_string()
}

/// Inline renderer for mixed content (paragraphs, heads, note bodies, list
/// items). `in_figure` switches emphasis to literal `<i>` (figure HTML is
/// verbatim, never markdown-rendered) and suppresses margin-note tokens.
fn inline(conv: &mut Conv, node: Node, in_figure: bool) -> String {
    let mut out = String::new();
    for child in node.children() {
        if child.is_text() {
            out.push_str(child.text().unwrap_or(""));
            continue;
        }
        if !child.is_element() {
            continue;
        }
        match child.tag_name().name() {
            "g" => match child.attribute("ref") {
                Some("char:EOLhyphen") => out.push_str(JOIN),
                _ => out.push_str(child.text().unwrap_or("")),
            },
            "gap" => {
                let reason = child.attribute("reason").unwrap_or("");
                let extent = child.attribute("extent").unwrap_or("");
                let desc = child
                    .children()
                    .find(|c| is(c, "desc"))
                    .and_then(|d| d.text())
                    .unwrap_or("")
                    .to_string();
                let context = collapse(&out);
                let context = context
                    .chars()
                    .rev()
                    .take(50)
                    .collect::<String>()
                    .chars()
                    .rev()
                    .collect::<String>();
                if !conv.quiet {
                    conv.gaps.push((
                        conv.current_file.clone(),
                        conv.current_page.clone().unwrap_or_default(),
                        reason.to_string(),
                        extent.to_string(),
                        desc.clone(),
                        context,
                    ));
                }
                out.push_str(&desc);
            }
            "hi" => {
                let inner = inline(conv, child, in_figure);
                let trimmed = inner.trim();
                if trimmed.is_empty() {
                    continue;
                }
                if child.attribute("rend") == Some("sup") {
                    conv.log("sup_flattened", trimmed.to_string());
                    out.push_str(trimmed);
                } else if in_figure {
                    let _ = write!(out, "<i>{trimmed}</i>");
                } else {
                    let _ = write!(out, "_{trimmed}_");
                }
                // Edge whitespace moved outside the emphasis.
                if inner.ends_with(' ') {
                    out.push(' ');
                }
            }
            "note" => {
                let content = collapse(&inline(conv, child, false));
                if in_figure {
                    conv.log("note_in_figure_dropped", content);
                    continue;
                }
                assert!(
                    !content.contains('`') && !content.contains("{{"),
                    "margin note content unrepresentable: {content}"
                );
                conv.margin_notes += 1;
                let _ = write!(out, "{{{{$m `{content}`}}}}");
            }
            "pb" => {
                if let Some(n) = conv.pb_value(&child) {
                    let _ = write!(out, " {{{{{{ {n} }}}}}} ");
                }
            }
            "unclear" => {
                let inner = inline(conv, child, in_figure);
                conv.log("unclear", collapse(&inner));
                out.push_str(&inner);
            }
            // Lists are block-level; the block handlers render them. Skipping
            // here keeps an item's own text separable from its sub-list.
            "list" => {}
            // seg (drop caps), foreign, date, and anything else: transparent.
            _ => out.push_str(&inline(conv, child, in_figure)),
        }
    }
    out
}

/// A paragraph-level string, cleaned: whitespace collapsed, EOL-hyphen halves
/// joined, literal anchor stars stripped.
fn para_text(conv: &mut Conv, node: Node, in_figure: bool) -> String {
    let raw = inline(conv, node, in_figure);
    let mut text = collapse(&raw);
    while let Some(pos) = text.find('*') {
        conv.stripped_stars += 1;
        conv.log(
            "anchor_star_stripped",
            text[..pos]
                .chars()
                .rev()
                .take(40)
                .collect::<String>()
                .chars()
                .rev()
                .collect(),
        );
        text.remove(pos);
    }
    collapse(&text)
}

fn list_depth(list: Node) -> usize {
    1 + list
        .descendants()
        .filter(|d| is(d, "list") && d.id() != list.id())
        .map(|d| {
            let mut depth = 0;
            let mut cur = d;
            while let Some(p) = cur.parent() {
                if is(&p, "list") {
                    depth += 1;
                }
                cur = p;
            }
            depth
        })
        .max()
        .unwrap_or(0)
}

/// Shallow list → `+ ` indented-run lines; a `<head>` inside the list becomes
/// its own preceding paragraph; `<label>` glues onto its following item.
fn render_run_list(conv: &mut Conv, list: Node, blocks: &mut Vec<String>) {
    let mut pending_label: Option<String> = None;
    let mut lines: Vec<String> = Vec::new();
    for child in list.children().filter(|c| c.is_element()) {
        match child.tag_name().name() {
            "head" => blocks.push(para_text(conv, child, false)),
            "label" => pending_label = Some(para_text(conv, child, false)),
            "item" => {
                let item = para_text(conv, child, false);
                let line = match pending_label.take() {
                    Some(l) => format!("{l} {item}"),
                    None => item,
                };
                if !line.is_empty() {
                    lines.push(format!("+ {line}"));
                }
                // A sub-list inside a shallow item flushes the run so far and
                // renders as its own run block right after.
                for sub in child.children().filter(|c| is(c, "list")) {
                    conv.log("nested_run_list", String::new());
                    if !lines.is_empty() {
                        blocks.push(lines.join("\n"));
                        lines.clear();
                    }
                    render_run_list(conv, sub, blocks);
                }
            }
            "pb" => {
                if let Some(n) = conv.pb_value(&child) {
                    lines.push(format!("+ {{{{{{ {n} }}}}}}"));
                }
            }
            "list" => render_run_list(conv, child, blocks),
            other => conv.log("list_child_dropped", other.to_string()),
        }
    }
    if !lines.is_empty() {
        blocks.push(lines.join("\n"));
    }
}

/// Deep-nested list (the Chapter IX table of sciences) → verbatim `<figure>`
/// HTML with nested `<ul>`. Page markers ride inside and are lifted onto the
/// anchor sentence by the struct parser.
fn render_figure_list(conv: &mut Conv, list: Node, caption: &str) -> String {
    fn ul(conv: &mut Conv, list: Node, out: &mut String) {
        out.push_str("<ul>");
        for child in list.children().filter(|c| c.is_element()) {
            match child.tag_name().name() {
                "item" => {
                    out.push_str("<li>");
                    // Direct inline content first, then any nested list.
                    let text = para_text(conv, child, true);
                    out.push_str(&text);
                    for sub in child.children().filter(|c| is(c, "list")) {
                        ul(conv, sub, out);
                    }
                    out.push_str("</li>");
                }
                "pb" => {
                    if let Some(n) = conv.pb_value(&child) {
                        let _ = write!(out, " {{{{{{ {n} }}}}}} ");
                    }
                }
                "label" | "head" => {
                    let t = para_text(conv, child, true);
                    let _ = write!(out, "<li>{t}</li>");
                }
                other => conv.log("figure_list_child_dropped", other.to_string()),
            }
        }
        out.push_str("</ul>");
    }
    let mut html = String::new();
    html.push_str("<figure>\n  <figcaption>");
    html.push_str(caption);
    html.push_str("</figcaption>\n  ");
    ul(conv, list, &mut html);
    html.push_str("\n</figure>");
    html
}

/// Render a div's block-level children into markdown blocks. `heading` names
/// the child that becomes the `## ` file heading (the div's own head).
fn render_div(conv: &mut Conv, div: Node, blocks: &mut Vec<String>, chapter_head: &str) {
    let mut pending_markers = String::new();
    for child in div.children().filter(|c| c.is_element()) {
        match child.tag_name().name() {
            "pb" => {
                if let Some(n) = conv.pb_value(&child) {
                    let _ = write!(pending_markers, "{{{{{{ {n} }}}}}} ");
                }
            }
            "head" => {
                let text = para_text(conv, child, false);
                blocks.push(format!("## {pending_markers}{text}"));
                pending_markers.clear();
            }
            "p" => {
                let text = para_text(conv, child, false);
                if text.is_empty() && pending_markers.is_empty() {
                    continue;
                }
                if text.is_empty() {
                    conv.log("empty_paragraph_dropped", String::new());
                    continue;
                }
                blocks.push(format!("{pending_markers}{text}"));
                pending_markers.clear();
            }
            "opener" | "closer" | "salute" | "signed" | "dateline" | "trailer" => {
                let text = para_text(conv, child, false);
                if !text.is_empty() {
                    blocks.push(format!("{pending_markers}{text}"));
                    pending_markers.clear();
                }
            }
            "list" => {
                if !pending_markers.is_empty() {
                    blocks.push(pending_markers.trim_end().to_string());
                    pending_markers.clear();
                }
                if list_depth(child) > 2 {
                    let caption = CHAP_PREFIX_RE.replace(chapter_head, "").to_string();
                    conv.log("deep_list_as_figure", format!("caption: {caption}"));
                    blocks.push(render_figure_list(conv, child, &caption));
                } else {
                    render_run_list(conv, child, blocks);
                }
            }
            "lg" => {
                let lines: Vec<String> = child
                    .children()
                    .filter(|c| is(c, "l"))
                    .map(|l| format!("+ {}", para_text(conv, l, false)))
                    .collect();
                conv.log("verse_as_runs", format!("{} lines", lines.len()));
                blocks.push(lines.join("\n"));
            }
            "figure" => {
                let ps: Vec<Node> = child.children().filter(|c| is(c, "p")).collect();
                if ps.is_empty() {
                    conv.log("empty_figure_dropped", String::new());
                    continue;
                }
                conv.log(
                    "figure_flattened_to_paragraphs",
                    format!("{} paragraphs", ps.len()),
                );
                for p in ps {
                    let text = para_text(conv, p, false);
                    if !text.is_empty() {
                        blocks.push(format!("{pending_markers}{text}"));
                        pending_markers.clear();
                    }
                }
            }
            "div" => render_div(conv, child, blocks, chapter_head),
            other => conv.log("div_child_dropped", other.to_string()),
        }
    }
    if !pending_markers.is_empty() {
        blocks.push(pending_markers.trim_end().to_string());
    }
}

/// TOC labels are editorial metadata (reader TOC, node slugs, citations), so
/// unlike the diplomatic heading lines they get readable resolved forms:
/// gap displays and OCR artifacts in the three affected chapter heads are
/// resolved by ruling (2026-08-11), 1651 spelling kept.
const LABEL_RULINGS: &[(&str, &str)] = &[
    ("concernîng", "concerning"),
    ("Mis•…ry", "Misery"),
    ("s•…cond", "second"),
    ("RÉDEMPTION", "REDEMPTION"),
];

fn apply_label_rulings(conv: &mut Conv, label: &str) -> String {
    let mut out = label.to_string();
    for (from, to) in LABEL_RULINGS {
        if out.contains(from) {
            // Reported unconditionally — TOC assembly runs quiet, but a
            // ruling firing is exactly what the report is for.
            conv.report.push((
                "label_ruling_applied".to_string(),
                String::new(),
                String::new(),
                format!("{from} → {to}"),
            ));
            out = out.replace(from, to);
        }
    }
    out
}

/// Forced sentence splits in the diplomatic layer (`|||`, the shared curated
/// convention): where a gap display or an unresolved `▪` hides a sentence
/// boundary that the modernized layer's resolution makes real, the reviewed
/// text must split at the same spot for layer parity. Applied at emit so
/// regeneration is stable; each must fire exactly once corpus-wide.
const FORCED_SPLIT_RULINGS: &[(&str, &str)] = &[
    (". •…ut at this day", ". |||•…ut at this day"),
    (". •…r wee", ". |||•…r wee"),
    (
        "Spirituall▪ Seeing then they had",
        "Spirituall▪ |||Seeing then they had",
    ),
    ("sustain it▪ And therefore", "sustain it▪ |||And therefore"),
    ("_Past▪_ So there is", "_Past▪_ |||So there is"),
    (
        "made capitall▪ On the contrary",
        "made capitall▪ |||On the contrary",
    ),
];

struct TocNode<'a> {
    label: String,
    supplied: bool,
    depth: u16,
    div: Node<'a, 'a>,
}

fn head_text(conv: &mut Conv, div: Node) -> Option<String> {
    div.children()
        .find(|c| is(c, "head"))
        .map(|h| para_text(conv, h, false))
}

fn slugify(label: &str) -> String {
    let mut s = String::new();
    let mut last_us = true;
    for ch in label.chars() {
        let ch = match ch {
            '&' => {
                if !last_us {
                    s.push('_');
                }
                s.push_str("and");
                last_us = false;
                continue;
            }
            c if c.is_ascii_alphanumeric() => c.to_ascii_lowercase(),
            _ => {
                if !last_us {
                    s.push('_');
                    last_us = true;
                }
                continue;
            }
        };
        s.push(ch);
        last_us = false;
    }
    let s = s.trim_matches('_').to_string();
    // Keep filenames sane for the very long chapter heads.
    if s.len() > 70 {
        let cut = s[..70].rfind('_').unwrap_or(70);
        s[..cut].to_string()
    } else {
        s
    }
}

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let cli = Cli::parse();
    let xml = fs::read_to_string(&cli.input)?;
    let doc = Document::parse_with_options(
        &xml,
        roxmltree::ParsingOptions {
            allow_dtd: true,
            ..Default::default()
        },
    )?;

    let mut conv = Conv {
        current_page: None,
        report: Vec::new(),
        gaps: Vec::new(),
        current_file: String::new(),
        margin_notes: 0,
        stripped_stars: 0,
        unnumbered_pb: 0,
        quiet: false,
        pb_final: std::collections::HashMap::new(),
    };

    conv.quiet = true;

    let text = doc
        .descendants()
        .find(|n| is(n, "text"))
        .ok_or("no <text>")?;
    let front = text.children().find(|n| is(n, "front")).ok_or("no front")?;
    let body = text.children().find(|n| is(n, "body")).ok_or("no body")?;
    let back = text.children().find(|n| is(n, "back")).ok_or("no back")?;

    // Pre-pass: resolve every numbered <pb> to its final marker value,
    // suffixing repeated printed numbers (b, c, …) in document order.
    {
        let mut seen: std::collections::HashMap<String, u32> = std::collections::HashMap::new();
        for pb in text.descendants().filter(|n| is(n, "pb")) {
            let Some(n) = pb.attribute("n") else { continue };
            let count = seen.entry(n.to_string()).or_insert(0);
            *count += 1;
            let value = if *count == 1 {
                n.to_string()
            } else {
                let suffix = (b'a' + (*count - 1) as u8) as char;
                let v = format!("{n}{suffix}");
                conv.report.push((
                    "page_value_suffixed".to_string(),
                    String::new(),
                    v.clone(),
                    format!("printed {n}, occurrence {count} → {v}"),
                ));
                v
            };
            conv.pb_final.insert(pb.id(), value);
        }
    }

    // --- Assemble the TOC: front divs, introduction, parts + chapters, R&C.
    let mut nodes: Vec<TocNode> = Vec::new();
    let supplied_front_label = |t: &str| match t {
        "engraved_title_page" => Some("Engraved Title Page"),
        "title_page" => Some("Title Page"),
        "coat_of_arms" => Some("Coat of Arms"),
        _ => None,
    };
    for div in front.children().filter(|n| is(n, "div")) {
        let typ = div.attribute("type").unwrap_or("");
        // Editorial exclusions: the "coat of arms" is a 1701 Essex ownership
        // bookplate in the scanned copy, not part of the 1651 work (ruling
        // 2026-08-11); the printed Contents and Errata are print apparatus
        // with no reading value — Scholia generates its own TOC, and the
        // errata's corrections live on in the modernized layer (ruling
        // 2026-08-13).
        if matches!(typ, "coat_of_arms" | "table_of_contents" | "errata") {
            conv.quiet = false;
            conv.log("front_div_excluded", typ.into());
            conv.quiet = true;
            continue;
        }
        let (label, supplied) = match supplied_front_label(typ) {
            Some(l) => (l.to_string(), true),
            None => {
                let raw = head_text(&mut conv, div).unwrap_or_else(|| typ.to_string());
                (apply_label_rulings(&mut conv, &raw), false)
            }
        };
        nodes.push(TocNode {
            label,
            supplied,
            depth: 1,
            div,
        });
    }
    for div in body.children().filter(|n| is(n, "div")) {
        let raw = head_text(&mut conv, div).expect("body div head");
        let label = apply_label_rulings(&mut conv, &raw);
        nodes.push(TocNode {
            label,
            supplied: false,
            depth: 1,
            div,
        });
        if div.attribute("type") == Some("part") {
            for ch in div.children().filter(|n| is(n, "div")) {
                let raw = head_text(&mut conv, ch).expect("chapter head");
                let label = apply_label_rulings(&mut conv, &raw);
                nodes.push(TocNode {
                    label,
                    supplied: false,
                    depth: 2,
                    div: ch,
                });
            }
        }
    }
    for div in back.children().filter(|n| is(n, "div")) {
        let raw = head_text(&mut conv, div).expect("back div head");
        let label = apply_label_rulings(&mut conv, &raw);
        nodes.push(TocNode {
            label,
            supplied: false,
            depth: 1,
            div,
        });
    }

    conv.quiet = false;
    conv.margin_notes = 0;
    conv.stripped_stars = 0;
    conv.unnumbered_pb = 0;

    // --- Render each node to NNN_slug.md (part divs render only their own
    // head + any direct content; chapter divs are separate nodes).
    fs::create_dir_all(&cli.out_dir)?;
    fs::create_dir_all(&cli.report_dir)?;
    conv.current_page = None;
    let mut written = 0u32;
    let mut forced_split_hits = 0u32;
    for (i, node) in nodes.iter().enumerate() {
        let position = i + 1;
        let slug = slugify(&node.label);
        let filename = format!("{position:03}_{slug}.md");
        conv.current_file = filename.clone();
        if node.supplied {
            conv.log("supplied_label", node.label.clone());
        }

        // Leading pbs before any content set the node's start page.
        let mut entry_page = conv.current_page.clone();
        for child in node.div.children().filter(|c| c.is_element()) {
            if is(&child, "pb") {
                if let Some(v) = conv.pb_final.get(&child.id()) {
                    entry_page = Some(v.clone());
                }
                continue;
            }
            break;
        }

        let mut blocks: Vec<String> = Vec::new();
        let chapter_head = node.label.clone();
        if node.depth == 1 && node.div.attribute("type") == Some("part") {
            // Part node: its own head only; chapters are separate nodes.
            let mut part_blocks: Vec<String> = Vec::new();
            let shallow_children: Vec<Node> = node
                .div
                .children()
                .filter(|c| c.is_element() && !is(c, "div"))
                .collect();
            let mut pending = String::new();
            for child in shallow_children {
                if is(&child, "pb") {
                    if let Some(n) = conv.pb_value(&child) {
                        let _ = write!(pending, "{{{{{{ {n} }}}}}} ");
                    }
                } else if is(&child, "head") {
                    let text = para_text(&mut conv, child, false);
                    part_blocks.push(format!("## {pending}{text}"));
                    pending.clear();
                } else {
                    let text = para_text(&mut conv, child, false);
                    if !text.is_empty() {
                        part_blocks.push(format!("{pending}{text}"));
                        pending.clear();
                    }
                }
            }
            if !pending.is_empty() {
                part_blocks.push(pending.trim_end().to_string());
            }
            blocks = part_blocks;
        } else {
            render_div(&mut conv, node.div, &mut blocks, &chapter_head);
        }

        let mut out = String::new();
        out.push_str("---\n");
        let _ = writeln!(out, "position: {position}");
        let _ = writeln!(out, "label: \"{}\"", node.label.replace('"', "\\\""));
        let _ = writeln!(out, "depth: {}", node.depth);
        if let Some(p) = &entry_page {
            let _ = writeln!(out, "page_1651: \"{p}\"");
        }
        out.push_str("---\n\n");
        // A node without its own printed head gets a supplied `## ` heading
        // from the label, so every file opens with a heading block.
        if !blocks.iter().any(|b| b.starts_with("## ")) {
            let _ = writeln!(out, "## {}\n", node.label);
        }
        out.push_str(&blocks.join("\n\n"));
        out.push('\n');
        for (from, to) in FORCED_SPLIT_RULINGS {
            if out.contains(from) {
                out = out.replace(from, to);
                forced_split_hits += 1;
            }
        }
        fs::write(PathBuf::from(&cli.out_dir).join(&filename), out)?;
        written += 1;
    }

    // --- Pagination anomalies: scan the emitted files in order and report
    // every non-consecutive step in the printed page numbers (misnumberings,
    // skips, and the duplicate runs of the 1651 printing).
    let marker_re = Regex::new(r"\{\{\{ (\d+) \}\}\}").unwrap();
    let mut entries = fs::read_dir(&cli.out_dir)?
        .map(|e| e.map(|e| e.path()))
        .collect::<Result<Vec<_>, _>>()?;
    entries.sort();
    let mut prev: Option<i64> = None;
    for path in &entries {
        let content = fs::read_to_string(path)?;
        let file = path.file_name().unwrap().to_string_lossy().to_string();
        for cap in marker_re.captures_iter(&content) {
            let n: i64 = cap[1].parse().unwrap();
            if let Some(p) = prev
                && n != p + 1
            {
                conv.report.push((
                    "pagination_anomaly".to_string(),
                    file.clone(),
                    n.to_string(),
                    format!("printed sequence jumps {p} → {n}"),
                ));
            }
            prev = Some(n);
        }
    }

    // --- Reports ------------------------------------------------------------
    let mut gaps_tsv = String::from("file\tpage\treason\textent\tdisplay\tpreceding_context\n");
    for (file, page, reason, extent, display, ctx) in &conv.gaps {
        let _ = writeln!(
            gaps_tsv,
            "{file}\t{page}\t{reason}\t{extent}\t{display}\t{ctx}"
        );
    }
    fs::write(PathBuf::from(&cli.report_dir).join("gaps.tsv"), gaps_tsv)?;

    let mut report_tsv = String::from("kind\tfile\tpage\tdetail\n");
    for (kind, file, page, detail) in &conv.report {
        let _ = writeln!(report_tsv, "{kind}\t{file}\t{page}\t{detail}");
    }
    fs::write(
        PathBuf::from(&cli.report_dir).join("conversion_report.tsv"),
        report_tsv,
    )?;

    eprintln!("=== hobbes1_tei_to_md ===");
    eprintln!("  files written:   {written}");
    eprintln!("  margin notes:    {}", conv.margin_notes);
    eprintln!("  gaps:            {}", conv.gaps.len());
    eprintln!("  stripped stars:  {}", conv.stripped_stars);
    eprintln!("  unnumbered pbs:  {}", conv.unnumbered_pb);
    eprintln!("  report rows:     {}", conv.report.len());
    assert_eq!(
        forced_split_hits as usize,
        FORCED_SPLIT_RULINGS.len(),
        "every forced-split ruling must fire exactly once"
    );
    eprintln!("  forced splits:   {forced_split_hits}");
    Ok(())
}
