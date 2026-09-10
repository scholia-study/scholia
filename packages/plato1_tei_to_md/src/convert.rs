//! Tree → markdown rendering. Two mutually recursive renderers:
//!
//! - [`render_block`] walks block-level content (div/p/said/quote/q/cit are
//!   all transparent wrappers; l/add/del/gap/milestone produce [`Piece`]s;
//!   bibl and quote/q have report side effects).
//! - [`render_inline`] walks the text-only content of `<l>`, `<add>`,
//!   `<del>` and `<sic>` — the only elements confirmed to nest inside them
//!   (a bare word restored via `<add>` inside a `<l>`, a paragraph-hint
//!   milestone inside an `<add>`) — and returns a plain `String`. Anything
//!   else found there is a shape the spec didn't anticipate, so it panics
//!   naming it rather than guessing.

use std::collections::HashMap;
use std::sync::LazyLock;

use regex::Regex;

use crate::xml::Node;

#[derive(Debug, Clone, PartialEq)]
pub enum Piece {
    Prose(String),
    /// One `<l>`, rendered but not yet whitespace-collapsed or `+ `-prefixed.
    Line(String),
    /// An `ed="P" unit="para"` milestone: a paragraph break hint.
    ParaBreak,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum QuoteKind {
    Verse,
    Prose,
    Q,
}

impl QuoteKind {
    pub fn as_str(&self) -> &'static str {
        match self {
            QuoteKind::Verse => "verse",
            QuoteKind::Prose => "prose",
            QuoteKind::Q => "q",
        }
    }
}

#[derive(Debug, Clone)]
pub struct ReportRow {
    pub stephanus: String,
    pub kind: QuoteKind,
    pub perseus_bibl: String,
    pub greek_text: String,
}

#[derive(Default)]
pub struct Ctx {
    pub current_stephanus: Option<String>,
    pub report: Vec<ReportRow>,
    last_report_idx: Option<usize>,
}

static WS_RE: LazyLock<Regex> = LazyLock::new(|| Regex::new(r"\s+").unwrap());

pub fn collapse_ws(s: &str) -> String {
    WS_RE.replace_all(s, " ").to_string()
}

/// `{{{ 327a }}}`, built by concatenation rather than `format!`'s brace
/// escaping so the literal token is easy to eyeball here.
pub fn marker_text(n: &str) -> String {
    let mut s = String::from("{{{ ");
    s.push_str(n);
    s.push_str(" }}}");
    s
}

/// The value inside a piece's text if it is exactly one marker token, e.g.
/// `marker_value("{{{ 327a }}}") == Some("327a")`.
static MARKER_RE: std::sync::LazyLock<regex::Regex> =
    std::sync::LazyLock::new(|| regex::Regex::new(r"\{\{\{[^}]*\}\}\}").unwrap());

pub fn marker_value(s: &str) -> Option<&str> {
    s.strip_prefix("{{{ ")?.strip_suffix(" }}}")
}

enum MilestoneKind {
    Page,
    Para,
    Section(String),
}

fn milestone_kind(attrs: &HashMap<String, String>) -> MilestoneKind {
    let unit = attrs.get("unit").map(String::as_str);
    let resp = attrs.get("resp").map(String::as_str);
    let ed = attrs.get("ed").map(String::as_str);
    match (unit, resp, ed) {
        (Some("page"), _, _) => MilestoneKind::Page,
        (Some("para"), _, Some("P")) => MilestoneKind::Para,
        (Some("section"), Some("Stephanus"), _) => {
            let n = attrs
                .get("n")
                .unwrap_or_else(|| panic!("section milestone missing n attribute"));
            MilestoneKind::Section(n.clone())
        }
        other => panic!("unexpected <milestone> shape: unit/resp/ed = {other:?}"),
    }
}

fn validate_gap(attrs: &HashMap<String, String>) {
    if attrs.get("reason").map(String::as_str) != Some("ellipsis") {
        panic!(
            "unexpected <gap> reason: {:?} (only \"ellipsis\" is handled)",
            attrs.get("reason")
        );
    }
}

fn validate_div_shape(attrs: &HashMap<String, String>) {
    let t = attrs.get("type").map(String::as_str);
    let subtype = attrs.get("subtype").map(String::as_str);
    let resp = attrs.get("resp").map(String::as_str);
    match (t, subtype, resp) {
        (Some("edition"), None, None) => {}
        (Some("textpart"), Some("book"), None) => {}
        (Some("textpart"), Some("section"), Some("perseus")) => {}
        other => panic!("unexpected <div> shape: type/subtype/resp = {other:?}"),
    }
}

/// Concatenates every descendant text node verbatim, ignoring markup. Used
/// only to build the quotations report's `greek_text` preview, never for
/// body output.
pub fn plain_text(node: &Node) -> String {
    match node {
        Node::Text(s) => s.clone(),
        Node::Element { children, .. } => children.iter().map(plain_text).collect(),
    }
}

fn truncate80(s: &str) -> String {
    s.chars().take(80).collect()
}

/// Text-only rendering for `<l>`, `<add>`, `<del>` and `<sic>` content.
pub fn render_inline(nodes: &[Node], ctx: &mut Ctx) -> String {
    let mut out = String::new();
    for node in nodes {
        match node {
            Node::Text(s) => out.push_str(s),
            Node::Element {
                name,
                attrs,
                children,
            } => match name.as_str() {
                "milestone" => match milestone_kind(attrs) {
                    MilestoneKind::Page | MilestoneKind::Para => {}
                    MilestoneKind::Section(n) => {
                        ctx.current_stephanus = Some(n.clone());
                        out.push_str(&marker_text(&n));
                    }
                },
                "gap" => {
                    validate_gap(attrs);
                    out.push('…');
                }
                "add" => {
                    out.push('⟨');
                    out.push_str(&render_inline(children, ctx));
                    out.push('⟩');
                }
                "del" => {
                    out.push('[');
                    out.push_str(&render_inline(children, ctx));
                    out.push(']');
                }
                "sic" => out.push_str(&render_inline(children, ctx)),
                other => panic!(
                    "unexpected inline element <{other}> nested inside add/del/sic/l — the spec's \
                     exhaustive nesting check did not anticipate this"
                ),
            },
        }
    }
    out
}

fn record_quote_report(node: &Node, ctx: &mut Ctx, kind: QuoteKind) {
    let stephanus = ctx.current_stephanus.clone().unwrap_or_default();
    let greek_text = truncate80(&collapse_ws(&plain_text(node)).trim().to_string());
    ctx.report.push(ReportRow {
        stephanus,
        kind,
        perseus_bibl: String::new(),
        greek_text,
    });
    ctx.last_report_idx = Some(ctx.report.len() - 1);
}

/// Block-level rendering for the direct content of `<body>` (and everything
/// beneath it). Returns the flattened sequence of [`Piece`]s for the whole
/// span passed in.
pub fn render_block(nodes: &[Node], ctx: &mut Ctx) -> Vec<Piece> {
    let mut out = Vec::new();
    for node in nodes {
        match node {
            Node::Text(s) => out.push(Piece::Prose(s.clone())),
            Node::Element {
                name,
                attrs,
                children,
            } => match name.as_str() {
                "div" => {
                    validate_div_shape(attrs);
                    out.extend(render_block(children, ctx));
                }
                "p" | "said" | "cit" => out.extend(render_block(children, ctx)),
                "quote" => {
                    let content = render_block(children, ctx);
                    let is_verse = content.iter().any(|p| matches!(p, Piece::Line(_)));
                    record_quote_report(
                        node,
                        ctx,
                        if is_verse {
                            QuoteKind::Verse
                        } else {
                            QuoteKind::Prose
                        },
                    );
                    out.extend(content);
                }
                "q" => {
                    let content = render_block(children, ctx);
                    record_quote_report(node, ctx, QuoteKind::Q);
                    out.extend(content);
                }
                "bibl" => {
                    if let Some(idx) = ctx.last_report_idx {
                        ctx.report[idx].perseus_bibl =
                            collapse_ws(&plain_text(node)).trim().to_string();
                    }
                }
                "l" => out.push(Piece::Line(render_inline(children, ctx))),
                "add" => out.push(Piece::Prose(format!("⟨{}⟩", render_inline(children, ctx)))),
                "del" => out.push(Piece::Prose(format!("[{}]", render_inline(children, ctx)))),
                "sic" => out.push(Piece::Prose(render_inline(children, ctx))),
                "gap" => {
                    validate_gap(attrs);
                    out.push(Piece::Prose("…".to_string()));
                }
                "milestone" => match milestone_kind(attrs) {
                    MilestoneKind::Page => {}
                    MilestoneKind::Para => out.push(Piece::ParaBreak),
                    MilestoneKind::Section(n) => {
                        ctx.current_stephanus = Some(n.clone());
                        out.push(Piece::Prose(marker_text(&n)));
                    }
                },
                other => unreachable!(
                    "xml::parse_body already restricts elements to ALLOWED_ELEMENTS; <{other}> \
                     has no render_block arm"
                ),
            },
        }
    }
    out
}

/// All `{{{ ... }}}` markers in a rendered piece stream, in document order.
pub fn find_markers(pieces: &[Piece]) -> Vec<String> {
    pieces
        .iter()
        .filter_map(|p| match p {
            Piece::Prose(s) => marker_value(s).map(str::to_string),
            _ => None,
        })
        .collect()
}

fn finalize_paragraph(pieces: &[Piece]) -> String {
    let mut lines: Vec<String> = vec![String::new()];
    for piece in pieces {
        match piece {
            Piece::Prose(s) => lines.last_mut().unwrap().push_str(s),
            Piece::Line(s) => {
                lines.push(format!("+ {s}"));
                lines.push(String::new());
            }
            Piece::ParaBreak => unreachable!("ParaBreak must be split out before finalizing"),
        }
    }
    lines
        .iter()
        .map(|l| collapse_ws(l).trim().to_string())
        .filter(|l| !l.is_empty())
        .collect::<Vec<_>>()
        .join("\n")
}

/// True if a rendered paragraph carries nothing but `{{{ ... }}}` tokens.
/// Such a paragraph must never stand alone: the parser strips markers off the
/// block text, leaving an empty block with no sentence for the marker to
/// resolve to, and the marker is silently dropped from the import.
fn is_marker_only(paragraph: &str) -> bool {
    let stripped = MARKER_RE.replace_all(paragraph, "");
    stripped.trim().is_empty()
}

/// Fold marker-only paragraphs into the paragraph that follows them (or, at
/// the end of a division, the one before), so every marker rides on a block
/// that actually has sentences.
fn attach_orphan_markers(paragraphs: Vec<String>) -> Vec<String> {
    let mut out: Vec<String> = Vec::new();
    let mut pending = String::new();
    for p in paragraphs {
        if is_marker_only(&p) {
            pending.push_str(p.trim());
            pending.push(' ');
        } else if pending.is_empty() {
            out.push(p);
        } else {
            out.push(format!("{pending}{p}"));
            pending.clear();
        }
    }
    if !pending.is_empty() {
        let tail = pending.trim_end().to_string();
        match out.last_mut() {
            Some(last) => {
                last.push(' ');
                last.push_str(&tail);
            }
            None => out.push(tail),
        }
    }
    out
}

/// Splits a division's piece stream on [`Piece::ParaBreak`] and renders each
/// paragraph, dropping any that end up empty (a paragraph hint with no
/// actual content between it and its neighbours).
pub fn render_paragraphs(pieces: &[Piece]) -> Vec<String> {
    let mut paragraphs = Vec::new();
    let mut current: Vec<Piece> = Vec::new();
    for p in pieces {
        if matches!(p, Piece::ParaBreak) {
            if !current.is_empty() {
                let text = finalize_paragraph(&current);
                if !text.is_empty() {
                    paragraphs.push(text);
                }
                current.clear();
            }
        } else {
            current.push(p.clone());
        }
    }
    if !current.is_empty() {
        let text = finalize_paragraph(&current);
        if !text.is_empty() {
            paragraphs.push(text);
        }
    }
    attach_orphan_markers(paragraphs)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::xml::parse_body;

    fn nodes(xml: &str) -> Vec<Node> {
        parse_body(&format!("<body>{xml}</body>")).unwrap()
    }

    #[test]
    fn add_wraps_in_angle_brackets() {
        let mut ctx = Ctx::default();
        let pieces = render_block(&nodes("<add>τα</add>"), &mut ctx);
        assert_eq!(pieces, vec![Piece::Prose("⟨τα⟩".to_string())]);
    }

    #[test]
    fn del_wraps_in_square_brackets() {
        let mut ctx = Ctx::default();
        let pieces = render_block(&nodes("<del>ἂν</del>"), &mut ctx);
        assert_eq!(pieces, vec![Piece::Prose("[ἂν]".to_string())]);
    }

    #[test]
    fn sic_is_verbatim() {
        let mut ctx = Ctx::default();
        let pieces = render_block(&nodes("<sic>οἴεται</sic>"), &mut ctx);
        assert_eq!(pieces, vec![Piece::Prose("οἴεται".to_string())]);
    }

    #[test]
    fn ellipsis_gap_becomes_unicode_ellipsis() {
        let mut ctx = Ctx::default();
        let pieces = render_block(&nodes(r#"<gap reason="ellipsis"/>"#), &mut ctx);
        assert_eq!(pieces, vec![Piece::Prose("…".to_string())]);
    }

    #[test]
    #[should_panic(expected = "unexpected <gap> reason")]
    fn non_ellipsis_gap_panics() {
        let mut ctx = Ctx::default();
        render_block(&nodes(r#"<gap reason="illegible"/>"#), &mut ctx);
    }

    #[test]
    fn add_mid_word_has_no_inserted_space() {
        // Real corpus shape: ὄν<add>τα</add> — the restored letters complete
        // the previous word with zero source whitespace between them.
        let mut ctx = Ctx::default();
        let pieces = render_block(&nodes("ὄν<add>τα</add> περιελόμενον"), &mut ctx);
        let text = finalize_paragraph(&pieces);
        assert_eq!(text, "ὄν⟨τα⟩ περιελόμενον");
    }

    #[test]
    fn section_milestone_becomes_marker_and_updates_stephanus() {
        let mut ctx = Ctx::default();
        let pieces = render_block(
            &nodes(r#"<milestone unit="section" resp="Stephanus" n="327a"/>κατέβην"#),
            &mut ctx,
        );
        assert_eq!(
            pieces,
            vec![
                Piece::Prose("{{{ 327a }}}".to_string()),
                Piece::Prose("κατέβην".to_string())
            ]
        );
        assert_eq!(ctx.current_stephanus.as_deref(), Some("327a"));
    }

    #[test]
    fn page_milestone_is_dropped() {
        let mut ctx = Ctx::default();
        let pieces = render_block(
            &nodes(r#"<milestone unit="page" resp="Stephanus" n="327"/>"#),
            &mut ctx,
        );
        assert!(pieces.is_empty());
    }

    #[test]
    fn para_milestone_becomes_para_break() {
        let mut ctx = Ctx::default();
        let pieces = render_block(&nodes(r#"a<milestone ed="P" unit="para"/>b"#), &mut ctx);
        assert_eq!(
            pieces,
            vec![
                Piece::Prose("a".to_string()),
                Piece::ParaBreak,
                Piece::Prose("b".to_string())
            ]
        );
        let paragraphs = render_paragraphs(&pieces);
        assert_eq!(paragraphs, vec!["a".to_string(), "b".to_string()]);
    }

    #[test]
    fn para_break_inside_add_is_suppressed_not_a_block_boundary() {
        // Real corpus shape: <add>ἀρκέσει;\n<milestone ed="P" unit="para"/>ναί.</add>
        let mut ctx = Ctx::default();
        let pieces = render_block(
            &nodes(r#"<add>ἀρκέσει;<milestone ed="P" unit="para"/>ναί.</add>"#),
            &mut ctx,
        );
        assert_eq!(pieces, vec![Piece::Prose("⟨ἀρκέσει;ναί.⟩".to_string())]);
    }

    #[test]
    fn verse_lines_become_plus_prefixed_and_stay_in_one_paragraph() {
        let mut ctx = Ctx::default();
        let pieces = render_block(
            &nodes(r#"<quote type="verse"> <l>γλυκεῖά οἱ</l> <l>ἀτάλλοισα</l> </quote> tail"#),
            &mut ctx,
        );
        let paragraphs = render_paragraphs(&pieces);
        assert_eq!(paragraphs.len(), 1);
        assert_eq!(paragraphs[0], "+ γλυκεῖά οἱ\n+ ἀτάλλοισα\ntail");
    }

    #[test]
    fn leading_ellipsis_gap_on_a_verse_line() {
        let mut ctx = Ctx::default();
        let pieces = render_block(
            &nodes(r#"<quote type="verse"><l><gap reason="ellipsis"/> ἢ βασιλῆος</l></quote>"#),
            &mut ctx,
        );
        let paragraphs = render_paragraphs(&pieces);
        assert_eq!(paragraphs[0], "+ … ἢ βασιλῆος");
    }

    #[test]
    fn quote_and_bibl_produce_one_report_row() {
        let mut ctx = Ctx::default();
        ctx.current_stephanus = Some("331e".to_string());
        let pieces = render_block(
            &nodes(
                r#"<cit><quote type="verse"><l>a</l></quote> <bibl>Pindar Frag. 214</bibl></cit>"#,
            ),
            &mut ctx,
        );
        assert_eq!(ctx.report.len(), 1);
        assert_eq!(ctx.report[0].stephanus, "331e");
        assert_eq!(ctx.report[0].kind, QuoteKind::Verse);
        assert_eq!(ctx.report[0].perseus_bibl, "Pindar Frag. 214");
        assert_eq!(ctx.report[0].greek_text, "a");
        // bibl never enters the body
        assert!(
            !pieces
                .iter()
                .any(|p| matches!(p, Piece::Prose(s) if s.contains("Pindar")))
        );
    }

    #[test]
    fn quote_without_bibl_has_empty_perseus_bibl() {
        let mut ctx = Ctx::default();
        render_block(
            &nodes(r#"<quote type="prose">ἐπὶ γήραος οὐδῷ</quote>"#),
            &mut ctx,
        );
        assert_eq!(ctx.report.len(), 1);
        assert_eq!(ctx.report[0].kind, QuoteKind::Prose);
        assert_eq!(ctx.report[0].perseus_bibl, "");
    }

    #[test]
    fn q_element_reports_as_q_and_never_borrows_a_bibl() {
        let mut ctx = Ctx::default();
        render_block(&nodes(r#"<q>πῶς,</q> ἔφη"#), &mut ctx);
        assert_eq!(ctx.report.len(), 1);
        assert_eq!(ctx.report[0].kind, QuoteKind::Q);
    }

    #[test]
    fn div_wrappers_are_transparent_and_keep_text() {
        let mut ctx = Ctx::default();
        let pieces = render_block(
            &nodes(
                r##"<div type="textpart" subtype="section" resp="perseus" n="327"><p><said who="#S">hi</said></p></div>"##,
            ),
            &mut ctx,
        );
        assert_eq!(pieces, vec![Piece::Prose("hi".to_string())]);
    }

    #[test]
    #[should_panic(expected = "unexpected <div> shape")]
    fn unexpected_div_shape_panics() {
        let mut ctx = Ctx::default();
        render_block(&nodes(r#"<div type="mystery">x</div>"#), &mut ctx);
    }

    #[test]
    fn no_stray_double_brace_from_marker_padding() {
        let mut ctx = Ctx::default();
        let pieces = render_block(
            &nodes(
                r#"a <milestone unit="section" resp="Stephanus" n="327b"/> b <milestone unit="section" resp="Stephanus" n="327c"/> c"#,
            ),
            &mut ctx,
        );
        let paragraphs = render_paragraphs(&pieces);
        let joined = paragraphs.join("\n\n");
        for window in joined.as_bytes().windows(2) {
            if window == b"{{" {
                let idx = joined.find("{{").unwrap();
                assert!(
                    joined[idx..].starts_with("{{{"),
                    "found a bare {{ in {joined}"
                );
            }
        }
    }
}

#[cfg(test)]
mod orphan_marker_tests {
    use super::*;

    #[test]
    fn marker_only_paragraph_is_folded_into_the_next() {
        let out = attach_orphan_markers(vec![
            "{{{ 331e }}}".to_string(),
            "λέγε δή, εἶπον ἐγώ.".to_string(),
        ]);
        assert_eq!(out, vec!["{{{ 331e }}} λέγε δή, εἶπον ἐγώ."]);
    }

    #[test]
    fn consecutive_orphans_all_ride_the_next_paragraph() {
        let out = attach_orphan_markers(vec![
            "{{{ 331e }}}".to_string(),
            "{{{ 332a }}}".to_string(),
            "ναί.".to_string(),
        ]);
        assert_eq!(out, vec!["{{{ 331e }}} {{{ 332a }}} ναί."]);
    }

    /// A trailing orphan has no successor, so it must attach backwards rather
    /// than be dropped.
    #[test]
    fn trailing_orphan_attaches_to_the_previous_paragraph() {
        let out = attach_orphan_markers(vec!["ναί.".to_string(), "{{{ 332a }}}".to_string()]);
        assert_eq!(out, vec!["ναί. {{{ 332a }}}"]);
    }

    #[test]
    fn ordinary_paragraphs_are_untouched() {
        let ps = vec!["πρῶτον.".to_string(), "{{{ 332a }}} δεύτερον.".to_string()];
        assert_eq!(attach_orphan_markers(ps.clone()), ps);
    }

    #[test]
    fn is_marker_only_distinguishes_text_from_bare_markers() {
        assert!(is_marker_only("{{{ 327a }}}"));
        assert!(is_marker_only("  {{{ 327a }}}  {{{ 327b }}} "));
        assert!(!is_marker_only("{{{ 327a }}} κατέβην"));
        assert!(!is_marker_only("κατέβην"));
    }
}
