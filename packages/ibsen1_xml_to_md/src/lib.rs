//! TEI (Henrik Ibsens Skrifter schema) → curated drama-Markdown conversion.
//!
//! Faithfully extracts the printed text of a HIS drama into Scholia's curated
//! `md_reviewed` form (see `docs`/auto-memory `project_ibsen1_ingest`). The
//! convention:
//!
//! ```text
//! ## HEAD              act / part / cast heading (faithful to print)
//! @ NAME *(opener)*S   speech opener; S = literal separator (. | : | none)
//! flush lines          prose dialogue (one line per paragraph; reader reflows)
//! | line               verse dialogue (one <l> each)
//! @stage (…)           scene-level stage direction (a <div> child)
//! *(…)* on its own line speaker-owned stage direction (a <hisSp> child)
//! *(…)* / *word* inline inline stage direction / emphasis
//! {{{ N }}}            1873 printed-page marker (from <fw type=page>)
//! ```
//!
//! Dropped: `lb` (with hyphen rejoin), `pb`/`fw` (consumed into page markers),
//! `synopticViewPtr`, `figure`. Flattened to their text: `stageRole`, `speaker`,
//! `sic`. The page marker is placed immediately before the first printed thing
//! on the new page; header-less pages get no marker (reviewers add those).

use std::cell::RefCell;
use std::collections::{BTreeSet, HashMap};
use std::sync::LazyLock;

use regex::Regex;
use roxmltree::{Node, NodeId};

const XML_NS: &str = "http://www.w3.org/XML/1998/namespace";
/// `<lb/>` placeholder inside a not-yet-normalized string; resolved by [`normalize`].
const SENT: char = '\u{0}';
const SOFT_HYPHEN: &str = "\u{2010}";

// ---------------------------------------------------------------------------
// Whitespace / hyphen normalization (HIS hyphen policy, see the edition's
// <change> log): soft hyphen U+2010 marks a line-break split that rejoins to one
// word; hard hyphen U+002D is a genuine hyphen kept verbatim (it may also fall
// at a line break). A page break inside a hyphenated word is moved to after the
// rejoined word so a marker never splits a word.
// ---------------------------------------------------------------------------
const WS: &str = r"[ \t\r\n]*";
const MARK: &str = r"(\{\{\{[^{}]*\}\}\})";

static RE_SOFT_MARK: LazyLock<Regex> =
    LazyLock::new(|| Regex::new(&format!(r"{SOFT_HYPHEN}{WS}\x00{WS}{MARK}{WS}(\S+)")).unwrap());
static RE_HARD_MARK: LazyLock<Regex> =
    LazyLock::new(|| Regex::new(&format!(r"-{WS}\x00{WS}{MARK}{WS}(\S+)")).unwrap());
static RE_SOFT: LazyLock<Regex> =
    LazyLock::new(|| Regex::new(&format!(r"{SOFT_HYPHEN}{WS}\x00{WS}")).unwrap());
static RE_HARD: LazyLock<Regex> = LazyLock::new(|| Regex::new(&format!(r"-{WS}\x00{WS}")).unwrap());
static RE_SENT: LazyLock<Regex> = LazyLock::new(|| Regex::new(&format!(r"{WS}\x00{WS}")).unwrap());
static RE_WS: LazyLock<Regex> = LazyLock::new(|| Regex::new(r"[ \t\r\n]+").unwrap());

fn normalize(s: &str) -> String {
    let s = RE_SOFT_MARK.replace_all(s, "${2} ${1} ").into_owned(); // mid-word break + marker
    let s = RE_HARD_MARK.replace_all(&s, "-${2} ${1} ").into_owned();
    let s = RE_SOFT.replace_all(&s, "").into_owned(); // soft break: drop hyphen, no space
    let s = RE_HARD.replace_all(&s, "-").into_owned(); // hard break: keep hyphen, no space
    let s = RE_SENT.replace_all(&s, " ").into_owned(); // any other break → space
    RE_WS.replace_all(&s, " ").trim().to_string()
}

fn collapse_ws(s: &str) -> String {
    RE_WS.replace_all(s, " ").into_owned()
}

// ---------------------------------------------------------------------------
// Page markers: each <pb> → the page number printed on the page it opens (the
// next <fw type=page> before the following <pb>). Header-less pages get no entry
// so no number is fabricated.
// ---------------------------------------------------------------------------
pub fn compute_pb_pages(root: Node) -> HashMap<NodeId, u32> {
    #[derive(Clone, Copy)]
    enum Tok {
        Pb(NodeId),
        Fw(Option<u32>),
    }
    let mut seq = Vec::new();
    for n in root.descendants() {
        if !n.is_element() {
            continue;
        }
        match n.tag_name().name() {
            "pb" => seq.push(Tok::Pb(n.id())),
            "fw" if n.attribute("type") == Some("page") => {
                seq.push(Tok::Fw(all_text(n).trim().parse::<u32>().ok()));
            }
            _ => {}
        }
    }
    let mut map = HashMap::new();
    for i in 0..seq.len() {
        let Tok::Pb(id) = seq[i] else { continue };
        for tok in &seq[i + 1..] {
            match tok {
                Tok::Pb(_) => break,
                Tok::Fw(Some(p)) => {
                    map.insert(id, *p);
                    break;
                }
                Tok::Fw(None) => {} // skip non-numeric header, keep scanning
            }
        }
    }
    map
}

/// Stateful converter: holds the page-number map and collects any unexpected
/// element names encountered (a signal that the source has markup the convention
/// doesn't yet handle).
pub struct Conv {
    pb_page: HashMap<NodeId, u32>,
    pub unknown: RefCell<BTreeSet<String>>,
}

impl Conv {
    pub fn new(pb_page: HashMap<NodeId, u32>) -> Self {
        Self {
            pb_page,
            unknown: RefCell::new(BTreeSet::new()),
        }
    }

    fn marker_for(&self, pb: Node) -> String {
        match self.pb_page.get(&pb.id()) {
            Some(p) => format!("{{{{{{ {p} }}}}}}"),
            None => String::new(),
        }
    }

    /// Serialize an element's mixed content to markdown, with [`SENT`] standing in
    /// for `<lb/>` until [`normalize`] resolves line breaks. Text and element
    /// children are visited in document order, so both leading text and inter-
    /// element "tails" are covered.
    fn inline(&self, el: Node) -> String {
        let mut out = String::new();
        for ch in el.children() {
            if ch.is_text() {
                out.push_str(ch.text().unwrap_or(""));
                continue;
            }
            if !ch.is_element() {
                continue;
            }
            match ch.tag_name().name() {
                "lb" => out.push(SENT),
                "synopticViewPtr" | "fw" => {}
                "pb" => {
                    let m = self.marker_for(ch);
                    if !m.is_empty() {
                        out.push(' ');
                        out.push_str(&m);
                        out.push(' ');
                    }
                }
                "emph" | "hi" | "hisStage" => {
                    let inner = self.inline(ch);
                    out.push('*');
                    out.push_str(inner.trim());
                    out.push('*');
                }
                "stageRole" | "speaker" | "sic" => out.push_str(&self.inline(ch)),
                other => {
                    self.unknown.borrow_mut().insert(other.to_string());
                    out.push_str(&self.inline(ch));
                }
            }
        }
        out
    }

    fn txt(&self, el: Node) -> String {
        normalize(&self.inline(el))
    }

    /// A `<hisSp>` → output blocks. The opener (speaker + opener-stage +
    /// separator) leads; the first prose paragraph and any verse lines ride
    /// contiguously with it, later paragraphs are their own blocks. A `<hisStage>`
    /// child is speaker-owned → `*(…)*` on its own line (stays in the turn);
    /// scene-level stages are emitted by [`Conv::convert_act`]. `lead` is a page
    /// marker for a break that opens this speech (rides the `@ NAME` line).
    pub fn convert_speech(&self, sp: Node, lead: &str) -> Vec<Vec<String>> {
        let opener = child(sp, "spOpener").expect("hisSp without spOpener");
        let mut head = vec![format!("@ {}", prepend(lead, &self.txt(opener)))];
        let mut tail: Vec<Vec<String>> = Vec::new();
        let mut started_body = false;
        let mut pending = String::new();
        for ch in sp.children() {
            if !ch.is_element() {
                continue;
            }
            match ch.tag_name().name() {
                "spOpener" | "lb" | "fw" | "synopticViewPtr" | "figure" => {}
                "pb" => pending = self.marker_for(ch),
                "l" => {
                    head.push(format!("| {}", prepend(&pending, &self.txt(ch))));
                    pending.clear();
                    started_body = true;
                }
                "p" => {
                    let body = self.txt(ch);
                    if body.is_empty() {
                        continue; // empty paragraph: keep pending for the next block
                    }
                    let line = prepend(&pending, &body);
                    pending.clear();
                    if !started_body && head.len() == 1 {
                        head.push(line); // first prose paragraph: contiguous with speaker
                    } else {
                        tail.push(vec![line]); // later paragraph: own block
                    }
                    started_body = true;
                }
                "hisStage" => {
                    tail.push(vec![prepend(&pending, &format!("*{}*", self.txt(ch)))]);
                    pending.clear();
                    started_body = true;
                }
                other => {
                    self.unknown.borrow_mut().insert(format!("sp>{other}"));
                }
            }
        }
        let mut blocks = vec![head];
        blocks.extend(tail);
        blocks
    }

    pub fn convert_act(&self, div: Node) -> Vec<Vec<String>> {
        let mut blocks = Vec::new();
        let mut pending = String::new(); // page marker for a between-blocks break
        for ch in div.children() {
            if !ch.is_element() {
                continue;
            }
            match ch.tag_name().name() {
                "pb" => pending = self.marker_for(ch),
                "lb" | "fw" | "synopticViewPtr" | "figure" => {}
                "head" => {
                    blocks.push(vec![format!("## {}", prepend(&pending, &self.txt(ch)))]);
                    pending.clear();
                }
                "hisStage" => {
                    blocks.push(vec![format!("@stage {}", prepend(&pending, &self.txt(ch)))]);
                    pending.clear();
                }
                "hisSp" => {
                    blocks.extend(self.convert_speech(ch, &pending));
                    pending.clear();
                }
                other => {
                    self.unknown.borrow_mut().insert(format!("act>{other}"));
                }
            }
        }
        blocks
    }

    pub fn convert_cast(&self, castlist: Node, setel: Option<Node>) -> Vec<Vec<String>> {
        let head = child(castlist, "head").expect("castList without head");
        let mut blocks = vec![vec![format!("## {}", self.txt(head))]];
        let mut items = Vec::new();
        for ci in children_named(castlist, "castItem") {
            let item = match child(ci, "role") {
                Some(role) => {
                    let mut item = self.txt(role);
                    match child(ci, "roleDesc") {
                        // "<role>, <roleDesc>desc.</roleDesc>" → "role, *desc.*"
                        Some(rd) => {
                            item.push_str(&collapse_ws(tail(role)));
                            item.push_str(&format!("*{}*", self.txt(rd)));
                        }
                        // "<role>." → "role." (the separator is role's tail)
                        None => item.push_str(tail(role).trim()),
                    }
                    item
                }
                None => self.txt(ci), // type="list" grouped extras
            };
            items.push(format!("- {item}"));
        }
        blocks.push(items);
        if let Some(p) = setel.and_then(|se| child(se, "p")) {
            blocks.push(vec![format!("@stage {}", self.txt(p))]);
        }
        blocks
    }

    pub fn convert_titlepage(&self, part: Node) -> Vec<Vec<String>> {
        let tp =
            descendant_attr(part, "titlePage", "type", "part").expect("part without titlePage");
        let dt = child(tp, "docTitle").expect("titlePage without docTitle");
        let mut main = String::new();
        let mut desc: Option<String> = None;
        for tpart in children_named(dt, "titlePart") {
            match tpart.attribute("type") {
                Some("main") => main = self.txt(tpart),
                Some("desc") => desc = Some(self.txt(tpart)),
                _ => {}
            }
        }
        let mut blocks = vec![vec![format!("## {main}")]];
        if let Some(d) = desc.filter(|d| !d.is_empty()) {
            blocks.push(vec![d]);
        }
        blocks
    }
}

// ---------------------------------------------------------------------------
// Rendering
// ---------------------------------------------------------------------------
fn prepend(marker: &str, text: &str) -> String {
    if marker.is_empty() {
        text.to_string()
    } else {
        format!("{marker} {text}")
    }
}

pub fn frontmatter(position: u32, label: &str, depth: u8) -> String {
    format!("---\nposition: {position}\nlabel: \"{label}\"\ndepth: {depth}\n---\n")
}

/// Blocks → markdown body: lines within a block join with `\n`, blocks with `\n\n`.
pub fn render(blocks: &[Vec<String>]) -> String {
    blocks
        .iter()
        .map(|b| b.join("\n"))
        .collect::<Vec<_>>()
        .join("\n\n")
}

// ---------------------------------------------------------------------------
// Node lookup helpers
// ---------------------------------------------------------------------------
fn all_text(node: Node) -> String {
    node.descendants()
        .filter(|d| d.is_text())
        .filter_map(|d| d.text())
        .collect()
}

/// First `<text xml:id=…>` descendant — the two part wrappers (KG1, KG2).
pub fn get_text<'a, 'i>(root: Node<'a, 'i>, xid: &str) -> Option<Node<'a, 'i>> {
    root.descendants().find(|n| {
        n.is_element() && n.tag_name().name() == "text" && n.attribute((XML_NS, "id")) == Some(xid)
    })
}

/// First descendant element with the given local name.
pub fn descendant<'a, 'i>(node: Node<'a, 'i>, name: &str) -> Option<Node<'a, 'i>> {
    node.descendants()
        .find(|n| n.is_element() && n.tag_name().name() == name)
}

fn descendant_attr<'a, 'i>(
    node: Node<'a, 'i>,
    name: &str,
    attr: &str,
    val: &str,
) -> Option<Node<'a, 'i>> {
    node.descendants()
        .find(|n| n.is_element() && n.tag_name().name() == name && n.attribute(attr) == Some(val))
}

fn child<'a, 'i>(node: Node<'a, 'i>, name: &str) -> Option<Node<'a, 'i>> {
    node.children()
        .find(|n| n.is_element() && n.tag_name().name() == name)
}

fn children_named<'a, 'i>(
    node: Node<'a, 'i>,
    name: &'static str,
) -> impl Iterator<Item = Node<'a, 'i>> {
    node.children()
        .filter(move |n| n.is_element() && n.tag_name().name() == name)
}

/// The text immediately following an element (its ElementTree-style "tail").
fn tail<'a>(node: Node<'a, '_>) -> &'a str {
    match node.next_sibling() {
        Some(n) if n.is_text() => n.text().unwrap_or(""),
        _ => "",
    }
}
