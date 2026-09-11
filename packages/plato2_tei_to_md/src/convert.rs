//! Tree → markdown rendering. Two mutually recursive renderers plus a
//! turn-aware finisher, mirroring `plato1_tei_to_md::convert` (same milestone,
//! bracket, and quote-report handling) but built around speech turns instead
//! of reconstructed prose paragraphs:
//!
//! - [`render_block`] walks block-level content. `div`/`p`/`cit` are
//!   transparent wrappers; `l`/`add`/`del`/`gap`/`milestone` produce
//!   [`Piece`]s; `bibl` and `quote`/`q` have report side effects; `said`
//!   additionally opens a [`Piece::TurnStart`] (unless `rend="merge"`, which
//!   folds its content into the turn already open) and `label` is dropped
//!   entirely — Burnet's ΣΩ./ΚΑΛ. abbreviations never reach the markdown.
//! - [`render_inline`] walks the text-only content of `<l>`, `<add>` and
//!   `<del>` — the only elements confirmed to nest inside them (an ellipsis
//!   gap inside a verse line) — and returns a plain `String`.

use std::collections::HashMap;
use std::sync::LazyLock;

use regex::Regex;

use crate::xml::Node;

#[derive(Debug, Clone, PartialEq)]
pub enum Piece {
    Prose(String),
    /// One `<l>`, rendered but not yet whitespace-collapsed or `| `-prefixed.
    Line(String),
    /// An `ed="P" unit="para"` milestone: a paragraph break hint.
    ParaBreak,
    /// A genuine (non-merge) `<said>`: the speaker line a division renderer
    /// must emit before the pieces that follow, up to the next `TurnStart`.
    TurnStart {
        speaker: String,
        said_index: usize,
        section: Option<String>,
    },
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
    pub said_index: usize,
    last_report_idx: Option<usize>,
}

static WS_RE: LazyLock<Regex> = LazyLock::new(|| Regex::new(r"\s+").unwrap());

pub fn collapse_ws(s: &str) -> String {
    WS_RE.replace_all(s, " ").to_string()
}

/// `{{{ 447a }}}`, built by concatenation rather than `format!`'s brace
/// escaping so the literal token is easy to eyeball here.
pub fn marker_text(n: &str) -> String {
    let mut s = String::from("{{{ ");
    s.push_str(n);
    s.push_str(" }}}");
    s
}

/// The value inside a piece's text if it is exactly one marker token, e.g.
/// `marker_value("{{{ 447a }}}") == Some("447a")`.
static MARKER_RE: LazyLock<Regex> = LazyLock::new(|| Regex::new(r"\{\{\{[^}]*\}\}\}").unwrap());

pub fn marker_value(s: &str) -> Option<&str> {
    s.strip_prefix("{{{ ")?.strip_suffix(" }}}")
}

/// A line with every `{{{ ... }}}` token removed. Used by the self-check word
/// count, which has no interest in Stephanus markers.
pub fn marker_free(line: &str) -> String {
    MARKER_RE.replace_all(line, "").to_string()
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
    if attrs.get("reason").map(String::as_str) != Some("lost") {
        panic!(
            "unexpected <gap> reason: {:?} (only \"lost\" is handled)",
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

/// Text-only rendering for `<l>`, `<add>` and `<del>` content.
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
                other => panic!(
                    "unexpected inline element <{other}> nested inside add/del/l — the spec's \
                     exhaustive nesting check did not anticipate this"
                ),
            },
        }
    }
    out
}

fn record_quote_report(node: &Node, ctx: &mut Ctx, kind: QuoteKind) {
    let stephanus = ctx.current_stephanus.clone().unwrap_or_default();
    let greek_text = truncate80(collapse_ws(&plain_text(node)).trim());
    ctx.report.push(ReportRow {
        stephanus,
        kind,
        perseus_bibl: String::new(),
        greek_text,
    });
    ctx.last_report_idx = Some(ctx.report.len() - 1);
}

/// Renders a `<said>`'s children, dropping `<label>` entirely and returning
/// the pieces its speaker's speech should carry, in order: the `TurnStart`
/// (when it isn't a `rend="merge"` continuation) first, then whatever came
/// before `<label>` (typically just the milestones that open this turn's own
/// section), then whatever follows it.
///
/// The section captured on `TurnStart` is deliberately read only after the
/// pre-label prefix is rendered: that is "the last section milestone between
/// `<said>` and `<label>`, if any; otherwise the last one seen before
/// `<said>`" (`ctx.current_stephanus` already holds the latter when we start,
/// and rendering the prefix updates it in place if the former applies).
fn render_said(attrs: &HashMap<String, String>, children: &[Node], ctx: &mut Ctx) -> Vec<Piece> {
    let said_index = ctx.said_index;
    ctx.said_index += 1;
    let is_merge = attrs.get("rend").map(String::as_str) == Some("merge");
    let speaker = attrs
        .get("who")
        .map(|w| w.trim_start_matches('#').to_string())
        .unwrap_or_else(|| panic!("<said> missing who attribute"));

    let label_pos = children
        .iter()
        .position(|c| matches!(c, Node::Element { name, .. } if name == "label"));
    let (prefix, rest) = match label_pos {
        Some(i) => (&children[..i], &children[i + 1..]),
        None => (children, &children[children.len()..]),
    };

    let prefix_pieces = render_block(prefix, ctx);
    let section = ctx.current_stephanus.clone();

    let mut out = Vec::new();
    if !is_merge {
        out.push(Piece::TurnStart {
            speaker,
            said_index,
            section,
        });
    }
    out.extend(prefix_pieces);
    out.extend(render_block(rest, ctx));
    out
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
                "p" | "cit" => out.extend(render_block(children, ctx)),
                "said" => out.extend(render_said(attrs, children, ctx)),
                "label" => unreachable!(
                    "render_said strips <label> before calling render_block on its siblings"
                ),
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

/// Every `TurnStart` in a piece stream, in document order.
pub fn find_turns(pieces: &[Piece]) -> Vec<(usize, &str, usize, Option<&str>)> {
    pieces
        .iter()
        .enumerate()
        .filter_map(|(i, p)| match p {
            Piece::TurnStart {
                speaker,
                said_index,
                section,
            } => Some((i, speaker.as_str(), *said_index, section.as_deref())),
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
                lines.push(format!("| {s}"));
                lines.push(String::new());
            }
            Piece::ParaBreak => unreachable!("ParaBreak must be split out before finalizing"),
            Piece::TurnStart { .. } => {
                unreachable!("TurnStart must be split into its own speech before finalizing")
            }
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
/// the end of a speech, the one before), so every marker rides on a block
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

/// Splits a speech's piece stream on [`Piece::ParaBreak`] and renders each
/// paragraph, dropping any that end up empty (a paragraph hint with no
/// actual content between it and its neighbours). Must not be handed a
/// stream containing [`Piece::TurnStart`] — split those out with
/// [`render_speeches`] first.
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

/// Renders one division's span of pieces into `@ Speaker` blocks. The span
/// must open on a [`Piece::TurnStart`] — every division boundary lands there
/// by construction, since divisions never split a speech.
pub fn render_speeches(pieces: &[Piece]) -> String {
    let mut speeches: Vec<(&str, Vec<Piece>)> = Vec::new();
    for p in pieces {
        match p {
            Piece::TurnStart { speaker, .. } => speeches.push((speaker.as_str(), Vec::new())),
            other => speeches
                .last_mut()
                .expect("division span must open on a TurnStart")
                .1
                .push(other.clone()),
        }
    }
    speeches
        .into_iter()
        .map(|(speaker, content)| {
            let paragraphs = render_paragraphs(&content);
            format!("@ {speaker}\n\n{}", paragraphs.join("\n\n"))
        })
        .collect::<Vec<_>>()
        .join("\n\n")
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
    fn lost_gap_becomes_unicode_ellipsis() {
        let mut ctx = Ctx::default();
        let pieces = render_block(&nodes(r#"<gap reason="lost"/>"#), &mut ctx);
        assert_eq!(pieces, vec![Piece::Prose("…".to_string())]);
    }

    #[test]
    #[should_panic(expected = "unexpected <gap> reason")]
    fn non_lost_gap_panics() {
        let mut ctx = Ctx::default();
        render_block(&nodes(r#"<gap reason="illegible"/>"#), &mut ctx);
    }

    #[test]
    fn section_milestone_becomes_marker_and_updates_stephanus() {
        let mut ctx = Ctx::default();
        let pieces = render_block(
            &nodes(r#"<milestone unit="section" resp="Stephanus" n="447a"/>πολέμου"#),
            &mut ctx,
        );
        assert_eq!(
            pieces,
            vec![
                Piece::Prose("{{{ 447a }}}".to_string()),
                Piece::Prose("πολέμου".to_string())
            ]
        );
        assert_eq!(ctx.current_stephanus.as_deref(), Some("447a"));
    }

    #[test]
    fn page_milestone_is_dropped() {
        let mut ctx = Ctx::default();
        let pieces = render_block(
            &nodes(r#"<milestone unit="page" resp="Stephanus" n="447"/>"#),
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
        assert_eq!(
            render_paragraphs(&pieces),
            vec!["a".to_string(), "b".to_string()]
        );
    }

    #[test]
    fn verse_lines_become_pipe_prefixed_and_stay_in_one_paragraph() {
        let mut ctx = Ctx::default();
        let pieces = render_block(
            &nodes(r#"<quote type="verse"> <l>νόμος ὁ πάντων</l> <l>βασιλεύς</l> </quote> tail"#),
            &mut ctx,
        );
        let paragraphs = render_paragraphs(&pieces);
        assert_eq!(paragraphs.len(), 1);
        assert_eq!(paragraphs[0], "| νόμος ὁ πάντων\n| βασιλεύς\ntail");
    }

    #[test]
    fn trailing_lost_gap_on_a_verse_line() {
        let mut ctx = Ctx::default();
        let pieces = render_block(
            &nodes(
                r#"<quote type="verse"><l>τὸ κατθανεῖν δὲ ζῆν; <gap reason="lost"/></l></quote>"#,
            ),
            &mut ctx,
        );
        let paragraphs = render_paragraphs(&pieces);
        assert_eq!(paragraphs[0], "| τὸ κατθανεῖν δὲ ζῆν; …");
    }

    #[test]
    fn quote_and_bibl_produce_one_report_row() {
        let mut ctx = Ctx {
            current_stephanus: Some("484b".to_string()),
            ..Default::default()
        };
        let pieces = render_block(
            &nodes(r#"<cit><quote type="verse"><l>a</l></quote> <bibl>Pind. fr. 169</bibl></cit>"#),
            &mut ctx,
        );
        assert_eq!(ctx.report.len(), 1);
        assert_eq!(ctx.report[0].stephanus, "484b");
        assert_eq!(ctx.report[0].kind, QuoteKind::Verse);
        assert_eq!(ctx.report[0].perseus_bibl, "Pind. fr. 169");
        assert_eq!(ctx.report[0].greek_text, "a");
        assert!(
            !pieces
                .iter()
                .any(|p| matches!(p, Piece::Prose(s) if s.contains("Pind")))
        );
    }

    #[test]
    fn quote_without_bibl_has_empty_perseus_bibl() {
        let mut ctx = Ctx::default();
        render_block(
            &nodes(r#"<quote type="paraphrase">λαμπρός</quote>"#),
            &mut ctx,
        );
        assert_eq!(ctx.report.len(), 1);
        assert_eq!(ctx.report[0].kind, QuoteKind::Prose);
        assert_eq!(ctx.report[0].perseus_bibl, "");
    }

    #[test]
    fn q_element_reports_as_q() {
        let mut ctx = Ctx::default();
        render_block(&nodes(r#"<q>τίς ἡ ἀριθμητικὴ τέχνη;</q>"#), &mut ctx);
        assert_eq!(ctx.report.len(), 1);
        assert_eq!(ctx.report[0].kind, QuoteKind::Q);
    }

    #[test]
    #[should_panic(expected = "unexpected <div> shape")]
    fn unexpected_div_shape_panics() {
        let mut ctx = Ctx::default();
        render_block(&nodes(r#"<div type="mystery">x</div>"#), &mut ctx);
    }

    #[test]
    fn genuine_said_emits_turn_start_and_drops_label() {
        let mut ctx = Ctx::default();
        let pieces = render_block(
            &nodes(r##"<said who="#Καλλίκλης"><label>ΚΑΛ.</label> πολέμου.</said>"##),
            &mut ctx,
        );
        assert_eq!(
            pieces,
            vec![
                Piece::TurnStart {
                    speaker: "Καλλίκλης".to_string(),
                    said_index: 0,
                    section: None,
                },
                Piece::Prose(" πολέμου.".to_string()),
            ]
        );
    }

    #[test]
    fn said_captures_section_opened_before_its_own_label() {
        let mut ctx = Ctx::default();
        let pieces = render_block(
            &nodes(
                r##"<said who="#Σωκράτης"><milestone unit="section" resp="Stephanus" n="458a"/><label>ΣΩ.</label> ἐγὼ οὖν.</said>"##,
            ),
            &mut ctx,
        );
        let Piece::TurnStart { section, .. } = &pieces[0] else {
            panic!("expected TurnStart first");
        };
        assert_eq!(section.as_deref(), Some("458a"));
        // the section marker itself still renders, right after the speaker line.
        assert_eq!(pieces[1], Piece::Prose("{{{ 458a }}}".to_string()));
    }

    #[test]
    fn said_with_no_leading_milestone_inherits_the_section_already_in_force() {
        let mut ctx = Ctx {
            current_stephanus: Some("447a".to_string()),
            ..Default::default()
        };
        let pieces = render_block(
            &nodes(r##"<said who="#Σωκράτης"><label>ΣΩ.</label> ἀλλʼ ἦ.</said>"##),
            &mut ctx,
        );
        let Piece::TurnStart { section, .. } = &pieces[0] else {
            panic!("expected TurnStart first");
        };
        assert_eq!(section.as_deref(), Some("447a"));
    }

    #[test]
    fn merge_continuation_emits_no_second_turn_start() {
        let mut ctx = Ctx::default();
        let pieces = render_block(
            &nodes(
                r##"<said who="#Γοργίας"><label>ΓΟΡ.</label> πρῶτον.</said><said who="#Γοργίας" rend="merge"><label>ΓΟΡ.</label> δεύτερον.</said>"##,
            ),
            &mut ctx,
        );
        let turn_starts = pieces
            .iter()
            .filter(|p| matches!(p, Piece::TurnStart { .. }))
            .count();
        assert_eq!(turn_starts, 1);
        assert_eq!(ctx.said_index, 2, "both <said> elements still counted");
    }

    #[test]
    fn said_index_counts_every_said_including_merges() {
        let mut ctx = Ctx::default();
        render_block(
            &nodes(
                r##"<said who="#Σωκράτης"><label>ΣΩ.</label> a.</said><said who="#Πῶλος"><label>ΠΩΛ.</label> b.</said>"##,
            ),
            &mut ctx,
        );
        assert_eq!(ctx.said_index, 2);
    }

    #[test]
    fn render_speeches_produces_speaker_blocks_separated_by_blank_lines() {
        let mut ctx = Ctx::default();
        let pieces = render_block(
            &nodes(
                r##"<said who="#Σωκράτης"><label>ΣΩ.</label> πρῶτον.</said><said who="#Πῶλος"><label>ΠΩΛ.</label> δεύτερον.</said>"##,
            ),
            &mut ctx,
        );
        assert_eq!(
            render_speeches(&pieces),
            "@ Σωκράτης\n\nπρῶτον.\n\n@ Πῶλος\n\nδεύτερον."
        );
    }

    #[test]
    fn render_speeches_keeps_multi_paragraph_turns_under_one_speaker_line() {
        let mut ctx = Ctx::default();
        let pieces = render_block(
            &nodes(
                r##"<said who="#Σωκράτης"><label>ΣΩ.</label> a.<milestone ed="P" unit="para"/>b.</said>"##,
            ),
            &mut ctx,
        );
        assert_eq!(render_speeches(&pieces), "@ Σωκράτης\n\na.\n\nb.");
    }

    #[test]
    fn find_turns_reports_speaker_index_and_section() {
        let mut ctx = Ctx::default();
        let pieces = render_block(
            &nodes(
                r##"<said who="#Καλλίκλης"><milestone unit="section" resp="Stephanus" n="447a"/><label>ΚΑΛ.</label> a.</said>"##,
            ),
            &mut ctx,
        );
        let turns = find_turns(&pieces);
        assert_eq!(turns, vec![(0, "Καλλίκλης", 0, Some("447a"))]);
    }
}

#[cfg(test)]
mod orphan_marker_tests {
    use super::*;

    #[test]
    fn marker_only_paragraph_is_folded_into_the_next() {
        let out = attach_orphan_markers(vec!["{{{ 461b }}}".to_string(), "λέγε δή.".to_string()]);
        assert_eq!(out, vec!["{{{ 461b }}} λέγε δή."]);
    }

    #[test]
    fn trailing_orphan_attaches_to_the_previous_paragraph() {
        let out = attach_orphan_markers(vec!["ναί.".to_string(), "{{{ 461c }}}".to_string()]);
        assert_eq!(out, vec!["ναί. {{{ 461c }}}"]);
    }

    #[test]
    fn is_marker_only_distinguishes_text_from_bare_markers() {
        assert!(is_marker_only("{{{ 447a }}}"));
        assert!(!is_marker_only("{{{ 447a }}} πολέμου"));
        assert!(!is_marker_only("πολέμου"));
    }
}
