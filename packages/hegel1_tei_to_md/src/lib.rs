//! DTA TEI P5 (`att.linguistic` edition) of Hegel's *Phänomenologie des Geistes*
//! (1807) → Scholia's curated `md_reviewed` layer.
//!
//! The layer is diplomatic: 1807 orthography as printed, long-s preserved,
//! `<sic>` readings kept, no editorial correction. One file per TOC node:
//!
//! ```text
//! ---
//! position: 4
//! label: "I. Die sinnliche Gewiſsheit; oder das Diese und das Meynen"
//! depth: 2
//! page_1807: 22
//! ---
//!
//! ## {{{ 22 }}} I. Die sinnliche Gewiſsheit; oder das Diese und das Meynen
//! ```
//!
//! A node's file holds only its `<div>`'s own direct `<p>`/`<lg>`/`<milestone>`
//! children; nested `<div>`s are separate nodes with their own files.
//!
//! Dropped: `<lb>` (with hyphen rejoin), `<fw>`, presentational `<hi>`.
//! Flattened to their text: `<w>`, `<s>`. `@norm` — the modernized reading —
//! is read as a *signal* for hyphen rejoining and is never emitted: this layer
//! carries 1807 surface forms only.

use std::cell::RefCell;
use std::collections::{BTreeSet, HashMap, HashSet};
use std::sync::LazyLock;

pub mod gw;
pub mod modernized;

use regex::Regex;
use roxmltree::{Node, NodeId};

/// `<lb/>` placeholder inside a not-yet-normalized string; resolved by [`normalize`].
const SENT: char = '\u{0}';
/// Marks a hyphen that a line break splits — it rejoins away.
const SOFT_HYPHEN: &str = "\u{2010}";

const SKIPPED_DIV_TYPES: [&str; 3] = ["contents", "imprint", "advertisement"];
/// The 1807 print has no heading over the Einleitung; the `<head/>` is empty.
const SUPPLIED_LABEL: &str = "Einleitung";
/// A `<milestone unit="section">` — the printed rule — becomes a separator block.
const SEPARATOR: &str = "---";
const PAGE_GAP_CONTEXT_CHARS: usize = 80;

// A soft hyphen and the line break after it rejoin into one word; a hyphen the
// rejoin discriminator left alone is the printed reading and survives, losing
// only the break. Either way a page marker moves past the word rather than
// splitting it.
const WS: &str = r"[ \t\r\n]*";
/// Any number of line breaks: a signature mark at the foot of the page splits
/// one word across two `<lb/>`s.
const BREAK: &str = r"[ \t\r\n]*(?:\x00[ \t\r\n]*)*";
const BREAKS: &str = r"[ \t\r\n]*(?:\x00[ \t\r\n]*)+";
const MARK: &str = r"(\{\{\{[^{}]*\}\}\})";

static RE_JOIN_BRACKET_MARK: LazyLock<Regex> =
    LazyLock::new(|| Regex::new(&format!(r"{SOFT_HYPHEN}(\]+){BREAK}{MARK}{WS}(\S+)")).unwrap());
static RE_JOIN_BRACKET: LazyLock<Regex> =
    LazyLock::new(|| Regex::new(&format!(r"{SOFT_HYPHEN}(\]+){BREAK}")).unwrap());
static RE_JOIN_MARK: LazyLock<Regex> =
    LazyLock::new(|| Regex::new(&format!(r"{SOFT_HYPHEN}{BREAK}{MARK}{WS}(\S+)")).unwrap());
static RE_JOIN_EMPH: LazyLock<Regex> =
    LazyLock::new(|| Regex::new(&format!(r"{SOFT_HYPHEN}([_*]+){BREAK}([^\s_*{{]+)")).unwrap());
static RE_JOIN: LazyLock<Regex> =
    LazyLock::new(|| Regex::new(&format!(r"{SOFT_HYPHEN}{BREAK}")).unwrap());
static RE_HARD_MARK: LazyLock<Regex> =
    LazyLock::new(|| Regex::new(&format!(r"-{BREAKS}{MARK}{WS}(\S+)")).unwrap());
static RE_HARD: LazyLock<Regex> = LazyLock::new(|| Regex::new(&format!(r"-{BREAKS}")).unwrap());
static RE_SENT: LazyLock<Regex> = LazyLock::new(|| Regex::new(&format!(r"{WS}\x00{WS}")).unwrap());
static RE_WS: LazyLock<Regex> = LazyLock::new(|| Regex::new(r"[ \t\r\n]+").unwrap());
/// A `_…_` antiqua span, matched to test whether its delimiters sit on word
/// boundaries. Escaped underscores never reach here, so pairs cannot straddle.
static RE_EMPHASIS: LazyLock<Regex> = LazyLock::new(|| Regex::new(r"_([^_]+)_").unwrap());

/// An erratum entry opens with the page it corrects: `S. 148. Z. 18. …`.
static RE_ERRATUM_PAGE: LazyLock<Regex> =
    LazyLock::new(|| Regex::new(r"^[*\s]*S[.,]\s*(\d+)").unwrap());

fn normalize(s: &str) -> String {
    // A supplied hyphen — `<supplied>-</supplied>` — is the editor's mark and
    // stays inside its brackets; only the break it spans goes away.
    let s = RE_JOIN_BRACKET_MARK
        .replace_all(s, "-${1}${3} ${2} ")
        .into_owned();
    let s = RE_JOIN_BRACKET.replace_all(&s, "-${1}").into_owned();
    let s = RE_JOIN_MARK.replace_all(&s, "${2} ${1} ").into_owned();
    // The split fell across an emphasis boundary: close the emphasis after the
    // rejoined word rather than mid-word, where markdown would not read it.
    let s = RE_JOIN_EMPH.replace_all(&s, "${2}${1}").into_owned();
    let s = RE_JOIN.replace_all(&s, "").into_owned();
    let s = RE_HARD_MARK.replace_all(&s, "-${2} ${1} ").into_owned();
    let s = RE_HARD.replace_all(&s, "-").into_owned();
    let s = RE_SENT.replace_all(&s, " ").into_owned();
    RE_WS.replace_all(&s, " ").trim().to_string()
}

fn collapse_ws(s: &str) -> String {
    RE_WS.replace_all(s, " ").trim().to_string()
}

fn escape_md(s: &str) -> String {
    let mut out = String::with_capacity(s.len());
    for ch in s.chars() {
        if matches!(ch, '_' | '*' | '[') {
            out.push('\\');
        }
        out.push(ch);
    }
    out
}

fn marker(page: &str) -> String {
    format!("{{{{{{ {page} }}}}}}")
}

fn prepend(marker: &str, text: &str) -> String {
    if marker.is_empty() {
        text.to_string()
    } else {
        format!("{marker} {text}")
    }
}

/// Emphasis delimiters must hug the text, so any leading/trailing whitespace or
/// pending line break moves outside the wrapper.
fn wrap(inner: &str, delim: &str) -> String {
    let pad = |c: char| c.is_whitespace() || c == SENT;
    let after_lead = inner.trim_start_matches(pad);
    let core = after_lead.trim_end_matches(pad);
    if core.is_empty() {
        return inner.to_string();
    }
    let lead = &inner[..inner.len() - after_lead.len()];
    let trail = &after_lead[core.len()..];
    format!("{lead}{delim}{core}{delim}{trail}")
}

/// Re-spell antiqua as `<i>…</i>` wherever a delimiter would land against a
/// word character.
///
/// The 1807 print italicises part of a word — `_Selbſt_bewuſstseyn`,
/// `welche_s_` — which DTA records by splitting the `<w>` around the `<hi>`.
/// CommonMark reads `_` as a delimiter only at a word boundary, so those spans
/// have to be authored as a tag. `md_prose_to_struct` renders both spellings to
/// the same `antiqua` span.
fn fix_intraword_antiqua(md: &str) -> String {
    let word = |c: char| c.is_alphanumeric();
    RE_EMPHASIS
        .replace_all(md, |caps: &regex::Captures| {
            let whole = caps.get(0).unwrap();
            let before = md[..whole.start()].chars().next_back();
            let after = md[whole.end()..].chars().next();
            if before.is_some_and(word) || after.is_some_and(word) {
                format!("<i>{}</i>", &caps[1])
            } else {
                whole.as_str().to_string()
            }
        })
        .into_owned()
}

/// The `<hi>` renditions that carry meaning in this print. Everything else
/// (`#in` initials, `#c` centring, `#et`, `#b`) is presentational: the wrapper
/// is dropped, the text kept, and the drop recorded in `dropped_markup.tsv`.
fn rendition_delim(rendition: Option<&str>) -> Option<&'static str> {
    match rendition {
        Some("#i") => Some("_"),
        Some("#g") => Some("***"),
        Some("#k") => Some("**"),
        _ => None,
    }
}

/// Markdown for the reader, plain text for a node label.
#[derive(Clone, Copy, PartialEq)]
enum Mode {
    Markdown,
    Plain,
}

/// The `<w>`s whose closing hyphen a line break split, so it must rejoin with
/// the fragment that follows.
///
/// DTA marks the split by giving the first `<w>` a `@norm` holding the whole
/// word and leaving the continuation fragment without one — the only signal in
/// the file that separates `Bewuſst-|seyn` from a printed `hin- und`. The
/// rejoin then concatenates the *surface* forms; `@norm` is never the text.
///
/// A line break between the halves is required as well: DTA also splits a `<w>`
/// where markup interrupts it, and there the continuation likewise carries no
/// `@norm` while the hyphen is a real compound hyphen (`ursprünglich-bestimmte`,
/// broken by an `<hi>`). Without this the rule would silently eat that hyphen.
pub fn rejoining_words(root: Node) -> HashSet<NodeId> {
    let mut rejoining = HashSet::new();
    let mut previous: Option<Node> = None;
    for n in root.descendants() {
        if !n.is_element() {
            continue;
        }
        if n.tag_name().name() != "w" {
            continue;
        }
        if let Some(p) = previous
            && n.attribute("norm").is_none()
            && all_text(p).trim_end().ends_with('-')
            && line_break_between(p, n)
        {
            rejoining.insert(p.id());
        }
        previous = Some(n);
    }
    rejoining
}

/// Whether a `<lb/>` separates two `<w>`s. The break sits between them
/// (`<w>Bewuſst-</w><lb/><w>seyn</w>`) or opens the continuation
/// (`<w>Fürſich-</w><w><lb/>seyns</w>`); both shapes occur.
fn line_break_between(first: Node, second: Node) -> bool {
    let is_lb = |n: &Node| n.is_element() && n.tag_name().name() == "lb";
    let separating = first
        .document()
        .root()
        .descendants()
        .skip_while(|n| n.id() != first.id())
        .take_while(|n| n.id() != second.id())
        .any(|n| is_lb(&n));
    separating || second.descendants().any(|n| is_lb(&n))
}

/// The page marker each node's heading carries, keyed by its `<div>`.
///
/// A `<pb>` outside any paragraph opens the next thing printed. When that thing
/// is a nested `<div>` — the usual shape, since chapters start on a fresh page —
/// the marker belongs to the child node's heading. A `<pb>` inside a paragraph
/// is emitted where it stands, so it never reaches a heading, and no page is
/// ever marked twice.
/// A `<pb n>` value as the page system uses it: DTA brackets around
/// supplied (unprinted) numbers are apparatus, and a misprinted number
/// carries its correction in brackets ("95[93]") — the corrected page keeps
/// the series monotonic. Recorded in `page_gaps.tsv` either way.
pub fn page_value(n: &str) -> String {
    let t = n.trim();
    if let Some(inner) = t.strip_prefix('[').and_then(|r| r.strip_suffix(']')) {
        return inner.to_string();
    }
    if let Some((_, rest)) = t.split_once('[')
        && let Some(corr) = rest.strip_suffix(']')
    {
        return corr.to_string();
    }
    t.to_string()
}

pub fn heading_markers(root: Node) -> HashMap<NodeId, String> {
    let mut markers = HashMap::new();
    let mut pending: Vec<String> = Vec::new();
    for n in root.descendants().filter(Node::is_element) {
        match n.tag_name().name() {
            "pb" => {
                if let Some(p) = n.attribute("n")
                    && !in_paragraph(n)
                {
                    // a run of empty leaves collapses to the page the
                    // following content actually opens on
                    pending.clear();
                    pending.push(page_value(p));
                }
            }
            "div" if is_content_div(n) => {
                if !pending.is_empty() {
                    markers.insert(n.id(), pending.join(" "));
                    pending.clear();
                }
            }
            // The div's own first block consumes anything still pending, the
            // same way [`Conv::blocks`] prefixes it there.
            "p" | "lg" if n.parent().is_some_and(is_content_div) => pending.clear(),
            _ => {}
        }
    }
    markers
}

fn in_paragraph(n: Node) -> bool {
    n.ancestors()
        .any(|a| matches!(a.tag_name().name(), "p" | "lg" | "head"))
}

fn is_content_div(n: Node) -> bool {
    n.is_element() && n.tag_name().name() == "div" && !skipped_div(n)
}

fn skipped_div(n: Node) -> bool {
    n.attribute("type")
        .is_some_and(|t| SKIPPED_DIV_TYPES.contains(&t))
}

pub struct TocNode<'a, 'i> {
    pub div: Node<'a, 'i>,
    pub position: u32,
    pub depth: u8,
    /// The 1807 printed page the node opens on — Roman in the Vorrede, Arabic in
    /// the body, absent before the first numbered page. Front matter only: it is
    /// the page the node *starts on*, which is not the same as a page break
    /// falling at the node's start.
    pub page: Option<String>,
    /// The marker for the `##` line, present only when a numbered `<pb>` really
    /// does fall at this node's start.
    pub heading_page: Option<String>,
    pub label: String,
    pub slug: String,
}

impl TocNode<'_, '_> {
    pub fn file_name(&self) -> String {
        format!("{:03}_{}.md", self.position, self.slug)
    }

    /// The label as the given layer prints it. Filenames and slugs always
    /// derive from the reviewed label (the `label` field), so the two layers
    /// stay 1:1; only the displayed text re-spells.
    pub fn display_label(&self, conv: &Conv) -> String {
        if conv.layer == Layer::Reviewed {
            return self.label.clone();
        }
        child(self.div, "head")
            .map(|h| conv.label(h))
            .filter(|l| !l.is_empty())
            .unwrap_or_else(|| SUPPLIED_LABEL.to_string())
    }
}

pub fn slugify(label: &str) -> String {
    // the 1812 TEI writes umlauts as combining e (aͤ); fold before the
    // char-wise pass so they slug as ae/oe/ue like precomposed umlauts
    let label = label
        .replace("a\u{364}", "ä")
        .replace("o\u{364}", "ö")
        .replace("u\u{364}", "ü")
        .replace("A\u{364}", "Ä")
        .replace("O\u{364}", "Ö")
        .replace("U\u{364}", "Ü");
    let mut folded = String::new();
    for ch in label.chars().flat_map(char::to_lowercase) {
        match ch {
            'ſ' => folded.push('s'),
            'ä' => folded.push_str("ae"),
            'ö' => folded.push_str("oe"),
            'ü' => folded.push_str("ue"),
            'ß' => folded.push_str("ss"),
            c if c.is_ascii_alphanumeric() => folded.push(c),
            _ => folded.push('_'),
        }
    }
    let mut slug = String::with_capacity(folded.len());
    for ch in folded.chars() {
        if ch == '_' && slug.ends_with('_') {
            continue;
        }
        slug.push(ch);
    }
    slug.trim_matches('_').to_string()
}

/// Every content `<div>` in document order, with the page open when it starts.
/// Front matter, contents and the publisher's leaves are not nodes.
///
/// `promote` lifts named divs (matched by slugified head) one level up,
/// subtree and all — an editorial re-levelling of the TEI's nesting where
/// the printed table of contents disagrees with it (the 1812 "Erstes Buch"
/// sits inside the Logik division in the DTA markup but at top level in
/// the print's own contents).
pub fn toc_nodes<'a, 'i>(
    root: Node<'a, 'i>,
    conv: &Conv,
    promote: &[String],
) -> Vec<TocNode<'a, 'i>> {
    let headings = heading_markers(root);
    let mut page: Option<String> = None;
    let mut nodes = Vec::new();
    let mut taken: HashSet<String> = HashSet::new();
    let mut promoted: HashSet<NodeId> = HashSet::new();
    for n in root.descendants().filter(Node::is_element) {
        match n.tag_name().name() {
            "pb" => {
                if let Some(p) = n.attribute("n") {
                    page = Some(page_value(p));
                }
            }
            "div" if is_content_div(n) => {
                let label = child(n, "head")
                    .map(|h| conv.label(h))
                    .filter(|l| !l.is_empty())
                    .unwrap_or_else(|| SUPPLIED_LABEL.to_string());
                if promote.iter().any(|p| slugify(p) == slugify(&label)) {
                    promoted.insert(n.id());
                }
                // repeated labels (the 1812 Anmerkungen) get a counter
                // suffix so node slugs stay unique per book
                let base = slugify(&label);
                let mut slug = base.clone();
                let mut k = 1;
                while !taken.insert(slug.clone()) {
                    k += 1;
                    slug = format!("{base}_{k}");
                }
                let lift = n.ancestors().filter(|a| promoted.contains(&a.id())).count() as u8;
                let depth = n
                    .ancestors()
                    .filter(|a| a.is_element() && a.tag_name().name() == "div")
                    .count() as u8;
                assert!(depth > lift, "promotion would lift {label:?} past the root");
                nodes.push(TocNode {
                    div: n,
                    position: nodes.len() as u32 + 1,
                    depth: depth - lift,
                    page: page.clone(),
                    heading_page: headings.get(&n.id()).cloned(),
                    slug,
                    label,
                });
            }
            _ => {}
        }
    }
    assert_eq!(
        promoted.len(),
        promote.len(),
        "not every --promote-head matched a div"
    );
    nodes
}

/// A printed rule the file does not keep, so a reviewer can put it back.
pub struct DroppedSeparator {
    pub div: NodeId,
    pub index: usize,
    pub before: String,
    pub after: String,
}

/// Which curated layer the converter emits: `Reviewed` is the diplomatic
/// text, `Modernized` re-spells every token through the rule table and the
/// decision table and takes `<corr>` over `<sic>`.
#[derive(Clone, Copy, PartialEq, Eq)]
pub enum Layer {
    Reviewed,
    Modernized,
}

/// Stateful converter, collecting any element name the mapping doesn't cover —
/// a signal that the source has markup this layer silently loses.
pub struct Conv {
    layer: Layer,
    rejoining: HashSet<NodeId>,
    replacements: HashMap<NodeId, modernized::Replacement>,
    pub unknown: RefCell<BTreeSet<String>>,
    pub dropped_separators: RefCell<Vec<DroppedSeparator>>,
    /// Author footnotes collected while rendering the current div's blocks:
    /// (per-file number, rendered text). Drained after each paragraph.
    pending_notes: RefCell<Vec<(u32, String)>>,
    note_counter: RefCell<u32>,
}

impl Conv {
    pub fn new(rejoining: HashSet<NodeId>) -> Self {
        Self {
            layer: Layer::Reviewed,
            rejoining,
            replacements: HashMap::new(),
            unknown: RefCell::new(BTreeSet::new()),
            dropped_separators: RefCell::new(Vec::new()),
            pending_notes: RefCell::new(Vec::new()),
            note_counter: RefCell::new(0),
        }
    }

    /// The modernized-layer converter; `replacements` comes from
    /// [`modernized::replacements`] and already carries every decision.
    pub fn modernized(
        rejoining: HashSet<NodeId>,
        replacements: HashMap<NodeId, modernized::Replacement>,
    ) -> Self {
        Self {
            layer: Layer::Modernized,
            rejoining,
            replacements,
            unknown: RefCell::new(BTreeSet::new()),
            dropped_separators: RefCell::new(Vec::new()),
            pending_notes: RefCell::new(Vec::new()),
            note_counter: RefCell::new(0),
        }
    }

    /// Serialize mixed content, with [`SENT`] standing in for `<lb/>` until
    /// [`normalize`] resolves the line breaks. `in_word` is true inside a `<w>`,
    /// where a line break splits one word rather than separating two.
    fn inline(&self, el: Node, mode: Mode, in_word: bool) -> String {
        let mut out = String::new();
        for ch in el.children() {
            if ch.is_text() {
                let text = ch.text().unwrap_or("");
                match mode {
                    Mode::Markdown => out.push_str(&escape_md(text)),
                    Mode::Plain => out.push_str(text),
                }
                continue;
            }
            if !ch.is_element() {
                continue;
            }
            match ch.tag_name().name() {
                "lb" => {
                    if in_word && out.ends_with('-') {
                        out.pop();
                        out.push_str(SOFT_HYPHEN);
                    }
                    out.push(SENT);
                }
                "pb" => {
                    if let (Mode::Markdown, Some(n)) = (mode, ch.attribute("n")) {
                        out.push(' ');
                        out.push_str(&marker(&page_value(n)));
                        out.push(' ');
                    }
                }
                "fw" | "milestone" => {}
                "w" => {
                    if let Some(r) = self.replacements.get(&ch.id()) {
                        match mode {
                            Mode::Markdown => out.push_str(&escape_md(&r.text)),
                            Mode::Plain => out.push_str(&r.text),
                        }
                        if r.soft_joined {
                            out.push_str(SOFT_HYPHEN);
                        }
                        continue;
                    }
                    let mut word = self.inline(ch, mode, true);
                    if self.rejoining.contains(&ch.id()) {
                        soften_trailing_hyphen(&mut word);
                    }
                    out.push_str(&word);
                }
                "s" | "seg" | "sic" | "corr" => out.push_str(&self.inline(ch, mode, in_word)),
                "hi" => {
                    let inner = self.inline(ch, mode, in_word);
                    match (mode, rendition_delim(ch.attribute("rendition"))) {
                        (Mode::Markdown, Some(delim)) => out.push_str(&wrap(&inner, delim)),
                        _ => out.push_str(&inner),
                    }
                }
                "choice" => {
                    let branch = match self.layer {
                        Layer::Reviewed => "sic",
                        Layer::Modernized => "corr",
                    };
                    match child(ch, branch) {
                        Some(taken) => out.push_str(&self.inline(taken, mode, in_word)),
                        None => {
                            self.unknown
                                .borrow_mut()
                                .insert(format!("choice-without-{branch}"));
                        }
                    }
                }
                "supplied" => {
                    let inner = self.inline(ch, mode, in_word);
                    match mode {
                        Mode::Markdown => out.push_str(&format!("[{inner}]")),
                        Mode::Plain => out.push_str(&inner),
                    }
                }
                // TeX fractions in the ratio discussions (the 1812 prints
                // 2/7 as a stacked fraction) — rendered as plain a/b.
                "formula" => {
                    let tex = text_of(ch);
                    out.push_str(&render_formula(&tex));
                }
                // A char-level lacuna in the damaged copy; the DTA supplies
                // restored readings around it, the gap itself shows as […].
                "gap" => {
                    if mode == Mode::Markdown {
                        out.push_str("[…]");
                    }
                }
                // Author footnote: a per-file [^N] reference at the anchor;
                // the definition is emitted after the enclosing paragraph.
                "note" => {
                    if mode == Mode::Markdown {
                        let mut counter = self.note_counter.borrow_mut();
                        *counter += 1;
                        let k = *counter;
                        drop(counter);
                        let text = normalize(&self.note_text(ch));
                        self.pending_notes.borrow_mut().push((k, text));
                        out.push_str(&format!("[^{k}]"));
                    }
                }
                other => {
                    self.unknown.borrow_mut().insert(other.to_string());
                    out.push_str(&self.inline(ch, mode, in_word));
                }
            }
        }
        out
    }

    /// A footnote's rendered body: inner paragraphs joined with a space, the
    /// printed marker glyph (the `n` attribute's `*)`) not repeated.
    fn note_text(&self, note: Node) -> String {
        let mut parts = Vec::new();
        for ch in note.children().filter(Node::is_element) {
            if ch.tag_name().name() == "p" {
                parts.push(self.inline(ch, Mode::Markdown, false));
            }
        }
        if parts.is_empty() {
            return self.inline(note, Mode::Markdown, false);
        }
        parts.join(" ")
    }

    /// Head text as printed — hyphens rejoined exactly as in the body, markup
    /// flattened, trailing full stop dropped. This is the node's label, so it
    /// carries no emphasis and no page marker.
    pub fn label(&self, head: Node) -> String {
        let text = normalize(&self.inline(head, Mode::Plain, false));
        text.strip_suffix('.')
            .unwrap_or(&text)
            .trim_end()
            .to_string()
    }

    /// The div's own direct children as markdown blocks. Nested `<div>`s are
    /// skipped: each is a node of its own. A `<pb>` between blocks rides the
    /// front of the next one, so a marker never stands alone.
    pub fn blocks(&self, div: Node) -> Vec<String> {
        *self.note_counter.borrow_mut() = 0;
        self.pending_notes.borrow_mut().clear();
        let mut emitted: Vec<Emitted> = Vec::new();
        let mut pending = String::new();
        for (index, ch) in div.children().filter(Node::is_element).enumerate() {
            match ch.tag_name().name() {
                "head" | "div" | "lb" | "fw" => {}
                "pb" => {
                    if let Some(n) = ch.attribute("n") {
                        pending = marker(&page_value(n));
                    }
                }
                "milestone" => emitted.push(Emitted::Separator(DroppedSeparator {
                    div: div.id(),
                    index: index + 1,
                    before: sibling_name(ch.prev_sibling_element()),
                    after: sibling_name(ch.next_sibling_element()),
                })),
                // The errata notice is apparatus like the list it introduces;
                // both stay out of the layer, the notice via dropped_markup.
                "p" if is_errata_notice(ch) => {}
                "p" => {
                    let body =
                        fix_intraword_antiqua(&normalize(&self.inline(ch, Mode::Markdown, false)));
                    if body.is_empty() {
                        continue;
                    }
                    emitted.push(Emitted::Block(prepend(&pending, &body)));
                    pending.clear();
                    for (k, text) in self.pending_notes.borrow_mut().drain(..) {
                        emitted.push(Emitted::Block(format!("[^{k}]: {text}")));
                    }
                }
                "lg" => {
                    let mut lines = Vec::new();
                    for l in children_named(ch, "l") {
                        let line = normalize(&self.inline(l, Mode::Markdown, false));
                        lines.push(format!("| {}", prepend(&pending, &line)));
                        pending.clear();
                    }
                    if !lines.is_empty() {
                        emitted.push(Emitted::Block(lines.join("\n")));
                    }
                }
                // The three "Verbesserungen" errata lists. They are apparatus
                // rather than reading text, so they stay out of the layer — but
                // they are 1807 source data, and every item is recorded in
                // `errata_1807.tsv`, so this is not a silent drop.
                "list" => {}
                other => {
                    self.unknown.borrow_mut().insert(other.to_string());
                }
            }
        }
        self.prune_separators(emitted)
    }

    /// The printed rules sit between a div's children, and a parent div's
    /// children are mostly the child divs that become files of their own — so
    /// most of its milestones separate nothing here. Only a rule with text on
    /// both sides survives; the rest go to `dropped_separators.tsv`.
    fn prune_separators(&self, emitted: Vec<Emitted>) -> Vec<String> {
        let mut blocks: Vec<String> = Vec::new();
        let mut held: Option<DroppedSeparator> = None;
        let mut dropped = self.dropped_separators.borrow_mut();
        for item in emitted {
            match item {
                Emitted::Separator(site) => {
                    if blocks.is_empty() {
                        dropped.push(site);
                    } else if let Some(previous) = held.replace(site) {
                        dropped.push(previous);
                    }
                }
                Emitted::Block(block) => {
                    if held.take().is_some() {
                        blocks.push(SEPARATOR.to_string());
                    }
                    blocks.push(block);
                }
            }
        }
        if let Some(site) = held.take() {
            dropped.push(site);
        }
        blocks
    }
}

enum Emitted {
    Block(String),
    Separator(DroppedSeparator),
}

/// The printer's note introducing a "Verbesserungen" `<list>` — the reader
/// is asked to apply the corrections before reading. It sits directly before
/// its list (only line/page furniture between), which no reading-text
/// paragraph in this print does.
fn is_errata_notice(p: Node) -> bool {
    let mut sibling = p.next_sibling_element();
    while let Some(s) = sibling {
        match s.tag_name().name() {
            "lb" | "pb" | "fw" => sibling = s.next_sibling_element(),
            other => return other == "list",
        }
    }
    false
}

fn sibling_name(node: Option<Node>) -> String {
    node.map(|n| n.tag_name().name().to_string())
        .unwrap_or_default()
}

fn soften_trailing_hyphen(word: &mut String) {
    let end = word.trim_end().len();
    if word[..end].ends_with('-') {
        word.replace_range(end - 1..end, SOFT_HYPHEN);
    }
}

pub fn frontmatter(
    page_key: &str,
    position: u32,
    label: &str,
    depth: u8,
    page: Option<&str>,
) -> String {
    let mut out = format!("---\nposition: {position}\nlabel: \"{label}\"\ndepth: {depth}\n");
    if let Some(p) = page {
        // Roman numerals are not YAML numbers; quote them so the value round-trips.
        if p.chars().all(|c| c.is_ascii_digit()) {
            out.push_str(&format!("{page_key}: {p}\n"));
        } else {
            out.push_str(&format!("{page_key}: \"{p}\"\n"));
        }
    }
    out.push_str("---\n");
    out
}

pub fn heading(label: &str, page: Option<&str>) -> String {
    match page {
        Some(p) => format!("## {} {label}", marker(p)),
        None => format!("## {label}"),
    }
}

/// `\frac{a}{b}` → `a/b`; anything else keeps its TeX source verbatim.
fn render_formula(tex: &str) -> String {
    let tex = tex.trim();
    if let Some(rest) = tex.strip_prefix("\\frac{")
        && let Some((a, brest)) = rest.split_once('}')
        && let Some(b) = brest.strip_prefix('{').and_then(|r| r.strip_suffix('}'))
    {
        return format!("{a}/{b}");
    }
    tex.to_string()
}

pub fn render(blocks: &[String]) -> String {
    blocks.join("\n\n")
}

pub struct SicCorr {
    pub file: String,
    pub page: String,
    pub sic: String,
    pub corr: String,
}

pub struct DroppedMarkup {
    pub file: String,
    pub page: String,
    pub element: String,
    pub rendition: String,
    pub text: String,
}

pub struct PageGap {
    pub facs: String,
    pub preceding_page: String,
    pub context: String,
}

pub struct Erratum {
    pub file: String,
    pub page: String,
    pub item: String,
}

#[derive(Default)]
pub struct Reports {
    pub sic_corr: Vec<SicCorr>,
    pub dropped: Vec<DroppedMarkup>,
    pub page_gaps: Vec<PageGap>,
    pub errata: Vec<Erratum>,
}

/// One document-order pass recording everything the markdown does not carry:
/// the `<corr>` readings the diplomatic layer declines, the presentational
/// markup it flattens, the unnumbered leaves that get no page marker, and the
/// 1807 errata lists, which are source data for the later modernization pass.
/// `owners` maps a node `<div>` to the file it becomes, so each row says where
/// in the curated layer to look.
pub fn collect_reports(root: Node, owners: &HashMap<NodeId, String>) -> Reports {
    let mut reports = Reports::default();
    let mut page = String::new();
    for n in root.descendants() {
        if n.is_text() {
            let text = n.text().unwrap_or("");
            for gap in reports.page_gaps.iter_mut() {
                if gap.context.chars().count() < PAGE_GAP_CONTEXT_CHARS {
                    gap.context.push_str(text);
                }
            }
            continue;
        }
        if !n.is_element() {
            continue;
        }
        match n.tag_name().name() {
            "pb" => match n.attribute("n") {
                Some(p) => page = p.to_string(),
                None => reports.page_gaps.push(PageGap {
                    facs: n.attribute("facs").unwrap_or("").to_string(),
                    preceding_page: page.clone(),
                    context: String::new(),
                }),
            },
            "choice" => reports.sic_corr.push(SicCorr {
                file: owner_file(n, owners),
                page: page.clone(),
                sic: child(n, "sic").map(text_of).unwrap_or_default(),
                corr: child(n, "corr").map(text_of).unwrap_or_default(),
            }),
            "hi" if rendition_delim(n.attribute("rendition")).is_none() => {
                reports.dropped.push(DroppedMarkup {
                    file: owner_file(n, owners),
                    page: page.clone(),
                    element: "hi".to_string(),
                    rendition: n.attribute("rendition").unwrap_or("").to_string(),
                    text: text_of(n),
                })
            }
            "p" if is_errata_notice(n) => reports.dropped.push(DroppedMarkup {
                file: owner_file(n, owners),
                page: page.clone(),
                element: "p".to_string(),
                rendition: "errata-notice".to_string(),
                text: text_of(n),
            }),
            "fw" => reports.dropped.push(DroppedMarkup {
                file: owner_file(n, owners),
                page: page.clone(),
                element: "fw".to_string(),
                rendition: n.attribute("type").unwrap_or("").to_string(),
                text: text_of(n),
            }),
            "list" => {
                let file = owner_file(n, owners);
                if file.is_empty() {
                    continue; // the printed contents and the publisher's ads are not nodes
                }
                for item in n.descendants().filter(|d| is_named(*d, "item")) {
                    let text = text_of(item);
                    reports.errata.push(Erratum {
                        file: file.clone(),
                        page: RE_ERRATUM_PAGE
                            .captures(&text)
                            .map(|c| c[1].to_string())
                            .unwrap_or_default(),
                        item: text,
                    });
                }
            }
            _ => {}
        }
    }
    for gap in reports.page_gaps.iter_mut() {
        gap.context = collapse_ws(&gap.context)
            .chars()
            .take(PAGE_GAP_CONTEXT_CHARS)
            .collect();
    }
    reports
}

fn owner_file(n: Node, owners: &HashMap<NodeId, String>) -> String {
    n.ancestors()
        .find_map(|a| owners.get(&a.id()))
        .cloned()
        .unwrap_or_default()
}

fn is_named(n: Node, name: &str) -> bool {
    n.is_element() && n.tag_name().name() == name
}

fn text_of(n: Node) -> String {
    collapse_ws(&all_text(n))
}

fn all_text(node: Node) -> String {
    node.descendants()
        .filter(|d| d.is_text())
        .filter_map(|d| d.text())
        .collect()
}

fn child<'a, 'i>(node: Node<'a, 'i>, name: &str) -> Option<Node<'a, 'i>> {
    node.children().find(|n| is_named(*n, name))
}

fn children_named<'a, 'i>(
    node: Node<'a, 'i>,
    name: &'static str,
) -> impl Iterator<Item = Node<'a, 'i>> {
    node.children().filter(move |n| is_named(*n, name))
}

#[cfg(test)]
mod tests {
    use super::*;

    const TEI_OPEN: &str = r#"<TEI xmlns="http://www.tei-c.org/ns/1.0">"#;

    fn doc_of(xml: &str) -> roxmltree::Document<'static> {
        let wrapped: &'static str = Box::leak(format!("{TEI_OPEN}{xml}</TEI>").into_boxed_str());
        roxmltree::Document::parse(wrapped).unwrap()
    }

    fn conv_of(doc: &roxmltree::Document) -> Conv {
        Conv::new(rejoining_words(doc.root_element()))
    }

    fn para(xml: &str) -> String {
        let doc = doc_of(xml);
        let p = child(doc.root_element(), "p").unwrap();
        normalize(&conv_of(&doc).inline(p, Mode::Markdown, false))
    }

    fn div_blocks(xml: &str) -> Vec<String> {
        let doc = doc_of(xml);
        let div = child(doc.root_element(), "div").unwrap();
        conv_of(&doc).blocks(div)
    }

    #[test]
    fn hyphen_inside_a_word_rejoins() {
        assert_eq!(
            para("<p><w>Be-<lb/>\nwuſstseyns</w> <w>iſt</w></p>"),
            "Bewuſstseyns iſt"
        );
    }

    #[test]
    fn a_hyphen_across_words_rejoins_when_the_continuation_has_no_norm() {
        assert_eq!(
            para(concat!(
                r#"<p><w norm="Bewußtsein">Bewuſst-</w><lb/>"#,
                "\n<w>seyn</w></p>",
            )),
            "Bewuſstseyn"
        );
    }

    #[test]
    fn a_hyphen_across_words_stays_when_the_continuation_has_a_norm() {
        assert_eq!(
            para(concat!(
                r#"<p><w norm="hin-">hin-</w> <w norm="und">und</w> "#,
                r#"<w norm="hergehen">hergehen</w></p>"#,
            )),
            "hin- und hergehen"
        );
    }

    #[test]
    fn the_rejoin_concatenates_surface_forms_never_the_norm() {
        assert_eq!(
            para(concat!(
                r#"<p><w norm="Bewusstseins">Bewuſst-</w><lb/>"#,
                "\n<w>ſeyns</w></p>",
            )),
            "Bewuſstſeyns"
        );
    }

    #[test]
    fn a_page_marker_never_splits_a_rejoined_word() {
        assert_eq!(
            para(r#"<p><w>Be-<lb/><pb n="300"/>wuſstseyns</w> <w>iſt</w></p>"#),
            "Bewuſstseyns {{{ 300 }}} iſt"
        );
        assert_eq!(
            para(concat!(
                r#"<p><w norm="Kopf-">Kopf-</w><lb/>"#,
                "\n",
                r#"<pb n="271"/>"#,
                "\n",
                r#"<w norm="Kitzel">Kitzel</w></p>"#,
            )),
            "Kopf-Kitzel {{{ 271 }}}"
        );
    }

    #[test]
    fn an_unnumbered_page_break_emits_nothing() {
        assert_eq!(
            para(r##"<p><w>eins</w> <pb facs="#f0015"/> <w>zwey</w></p>"##),
            "eins zwey"
        );
    }

    #[test]
    fn hi_renditions_map_to_their_markup() {
        assert_eq!(
            para(concat!(
                r##"<p><hi rendition="#i"><w>kursiv</w></hi> "##,
                r##"<hi rendition="#g"><w>geſperrt</w></hi> "##,
                r##"<hi rendition="#k"><w>kapitälchen</w></hi> "##,
                r##"<hi rendition="#in"><w>I</w></hi> "##,
                r##"<hi rendition="#c"><w>zentriert</w></hi></p>"##,
            )),
            "_kursiv_ ***geſperrt*** **kapitälchen** I zentriert"
        );
    }

    #[test]
    fn emphasis_delimiters_hug_their_text() {
        assert_eq!(
            para("<p><w>a</w> <hi rendition=\"#i\"><w>b</w><lb/>\n</hi><w>c</w></p>"),
            "a _b_ c"
        );
    }

    #[test]
    fn a_rejoin_across_an_emphasis_boundary_closes_after_the_word() {
        assert_eq!(
            para(concat!(
                r##"<p><hi rendition="#i"><w norm="Fürsichsein">Fürſich-</w></hi>"##,
                "<w><lb/>\nseyns</w></p>",
            )),
            "_Fürſichseyns_"
        );
    }

    /// Markup can split a `<w>` with no line break involved, leaving the
    /// continuation without a `@norm` exactly as a split word does. The hyphen
    /// there is a real compound hyphen and must survive.
    #[test]
    fn a_compound_split_by_markup_keeps_its_hyphen() {
        assert_eq!(
            para(concat!(
                r##"<p><w norm="ursprünglich-bestimmte">ursprünglich-</w>"##,
                r##"<hi rendition="#i"><w>bestimmte</w></hi></p>"##,
            )),
            "ursprünglich-_bestimmte_"
        );
    }

    #[test]
    fn a_supplied_hyphen_keeps_its_brackets_and_still_joins() {
        assert_eq!(
            para(concat!(
                r#"<p><w norm="Sprache">Spra</w><supplied><w>-</w></supplied>"#,
                "<w><lb/>\nche</w></p>",
            )),
            "Spra[-]che"
        );
    }

    #[test]
    fn choice_keeps_the_sic_reading() {
        assert_eq!(
            para("<p><choice><sic>Einſieht</sic><corr><s><w>Einſicht</w></s></corr></choice></p>"),
            "Einſieht"
        );
    }

    #[test]
    fn supplied_text_is_bracketed() {
        assert_eq!(
            para("<p><w>ſ</w><supplied><s><w>e</w></s></supplied><w>ine</w></p>"),
            "ſ[e]ine"
        );
    }

    #[test]
    fn a_milestone_becomes_a_separator() {
        assert_eq!(
            div_blocks(concat!(
                "<div><head>H.</head><p><w>eins</w></p>",
                r##"<milestone unit="section" rendition="#hr"/>"##,
                "<p><w>zwey</w></p></div>",
            )),
            vec!["eins".to_string(), "---".to_string(), "zwey".to_string()]
        );
    }

    /// The paragraph introducing a "Verbesserungen" list is apparatus like
    /// the list: both drop, and the rule before them prunes with nothing to
    /// separate.
    #[test]
    fn the_errata_notice_drops_with_its_list() {
        assert_eq!(
            div_blocks(concat!(
                "<div><head>H.</head><p><w>Text</w></p>",
                r##"<milestone unit="section" rendition="#hr"/>"##,
                "<p><w>Nachſtehende</w> <w>Druckfehler</w></p>",
                "<list><item><w>S. 5</w></item></list></div>",
            )),
            vec!["Text".to_string()]
        );
    }

    #[test]
    fn separators_that_separate_nothing_are_pruned_and_logged() {
        let rule = r##"<milestone unit="section" rendition="#hr"/>"##;
        let xml = format!("<div><head>H.</head>{rule}<p><w>eins</w></p>{rule}{rule}</div>");
        let doc = doc_of(&xml);
        let conv = conv_of(&doc);
        let blocks = conv.blocks(child(doc.root_element(), "div").unwrap());
        assert_eq!(blocks, vec!["eins".to_string()]);
        let dropped = conv.dropped_separators.borrow();
        assert_eq!(dropped.len(), 3);
        assert_eq!(dropped[0].before, "head");
        assert_eq!(dropped[0].after, "p");
    }

    #[test]
    fn a_page_break_between_blocks_opens_the_next_one() {
        assert_eq!(
            div_blocks(concat!(
                "<div><head>H.</head><p><w>eins</w></p>",
                r#"<pb n="42"/><p><w>zwey</w></p></div>"#,
            )),
            vec!["eins".to_string(), "{{{ 42 }}} zwey".to_string()]
        );
    }

    #[test]
    fn a_page_break_before_a_child_div_marks_the_child_heading() {
        let doc = doc_of(concat!(
            "<text><div><head>Parent.</head><p><w>eins</w></p>",
            r#"<pb n="42"/><div><head>Kind.</head><p><w>zwey</w></p></div></div></text>"#,
        ));
        let conv = conv_of(&doc);
        let nodes = toc_nodes(doc.root_element(), &conv, &[]);
        assert_eq!(nodes.len(), 2);
        assert_eq!(nodes[0].heading_page, None);
        assert_eq!(nodes[1].heading_page.as_deref(), Some("42"));
        // The parent's own file must not repeat it.
        assert_eq!(conv.blocks(nodes[0].div), vec!["eins".to_string()]);
    }

    #[test]
    fn verse_lines_are_pipe_prefixed() {
        assert_eq!(
            div_blocks(concat!(
                r#"<div><head>H.</head><pb n="299"/><lg type="poem">"#,
                "<l><w>Es</w> <w>verachtet</w></l><lb/>\n<l><w>Verſtand</w></l></lg></div>",
            )),
            vec!["| {{{ 299 }}} Es verachtet\n| Verſtand".to_string()]
        );
    }

    #[test]
    fn markdown_metacharacters_are_escaped() {
        assert_eq!(
            para("<p><w>a_b</w> <w>c*d</w> <w>e[f</w></p>"),
            r"a\_b c\*d e\[f"
        );
    }

    #[test]
    fn unhandled_elements_are_collected() {
        let doc = doc_of("<div><head>H.</head><table><row>x</row></table></div>");
        let conv = conv_of(&doc);
        conv.blocks(child(doc.root_element(), "div").unwrap());
        assert_eq!(
            conv.unknown.borrow().iter().cloned().collect::<Vec<_>>(),
            vec!["table".to_string()]
        );
    }

    /// The 1807 print italicises part of a word; `_` cannot express that, so
    /// those spans become `<i>` tags while word-bounded ones keep `_`.
    #[test]
    fn intraword_antiqua_becomes_an_italic_tag() {
        assert_eq!(
            fix_intraword_antiqua("_Selbſt_bewuſstseyn und _an_ und _fürsich_seyenden"),
            "<i>Selbſt</i>bewuſstseyn und _an_ und <i>fürsich</i>seyenden"
        );
        assert_eq!(fix_intraword_antiqua("welche_s_ ein"), "welche<i>s</i> ein");
        assert_eq!(fix_intraword_antiqua("_Wissen_ des"), "_Wissen_ des");
        assert_eq!(
            fix_intraword_antiqua("_wahrhaffteste;_ denn"),
            "_wahrhaffteste;_ denn"
        );
    }

    /// Errata lists are routed to `errata_1807.tsv`, so they must not also be
    /// reported as unhandled — that would read as a silent drop.
    #[test]
    fn errata_lists_are_not_reported_as_unhandled() {
        let doc = doc_of("<div><head>H.</head><list><item>S. 5. lies x</item></list></div>");
        let conv = conv_of(&doc);
        conv.blocks(child(doc.root_element(), "div").unwrap());
        assert!(conv.unknown.borrow().is_empty());
    }

    #[test]
    fn head_labels_rejoin_hyphens_and_drop_the_trailing_stop() {
        let doc = doc_of(concat!(
            "<head><w>b.</w><lb/>\n<w>Das</w> <w>Gesetz</w> <w>des</w> ",
            "<w>Herzens</w>, <w>und</w> <w>der</w> ",
            r#"<w norm="Wahnsinn">Wahn-</w><lb/>"#,
            "\n<w>ſinn</w><w>.</w></head>",
        ));
        let head = child(doc.root_element(), "head").unwrap();
        assert_eq!(
            conv_of(&doc).label(head),
            "b. Das Gesetz des Herzens, und der Wahnſinn"
        );
    }

    #[test]
    fn labels_carry_no_emphasis_or_markers() {
        let doc = doc_of(concat!(
            "<head><w>B.</w><lb/>\n",
            r##"<hi rendition="#g"><w>Die</w> <w>Kunst-Religion</w></hi>"##,
            "<w>.</w></head>",
        ));
        let head = child(doc.root_element(), "head").unwrap();
        assert_eq!(conv_of(&doc).label(head), "B. Die Kunst-Religion");
    }

    #[test]
    fn slugs_fold_the_1807_orthography() {
        assert_eq!(
            slugify("IV. Die Wahrheit der Gewiſsheit seiner selbst"),
            "iv_die_wahrheit_der_gewissheit_seiner_selbst"
        );
        assert_eq!(
            slugify("a. Das geistige Thierreich und der Betrug, oder die Sache selbs’"),
            "a_das_geistige_thierreich_und_der_betrug_oder_die_sache_selbs"
        );
        assert_eq!(
            slugify("III. Die abſolute Freyheit"),
            "iii_die_absolute_freyheit"
        );
    }

    #[test]
    fn front_matter_quotes_roman_pages_only() {
        assert!(
            frontmatter("page_1807", 2, "Erster Theil", 1, Some("XCI"))
                .contains("page_1807: \"XCI\"\n")
        );
        assert!(frontmatter("page_1807", 4, "I.", 2, Some("22")).contains("page_1807: 22\n"));
        assert!(!frontmatter("page_1807", 1, "Vorrede", 1, None).contains("page_1807"));
    }
}
