//! Wikisource ProofreadPage wikitext → peirce1 curated markdown.
//!
//! Extraction only: everything emitted is present in the transcription or in
//! the article's own `<pages …/>` declaration.
//!
//! Printed page numbers advance with the scan pages from the paper's declared
//! opening page — an article occupies a contiguous run, so this is derivation
//! from the source, not guesswork. Many transcriptions also record a `{{rh}}`
//! running head; where one exists it is checked against the derived number and
//! a disagreement aborts the conversion. Across all fourteen papers every
//! recorded head agreed, which is what licenses the derivation for the pages
//! whose head the transcriber left out.

use regex::Regex;
use std::sync::LazyLock;

use crate::papers::Paper;

/// Footnote marks, following the convention the other corpora use: one more
/// asterisk per note, in document order. The printings restart their own marks
/// (*, †, ‡) on every page, which a per-file key cannot express — and the
/// importer renumbers footnotes book-globally anyway, so the marker is only an
/// identifier within its file.
fn mark(index: usize) -> String {
    "*".repeat(index + 1)
}

static NOINCLUDE: LazyLock<Regex> =
    LazyLock::new(|| Regex::new(r"(?s)<noinclude>.*?</noinclude>").unwrap());
// Template names are case-insensitive in wikitext; both {{rh}} and {{RH}}
// occur across these volumes.
static RUNNING_HEAD: LazyLock<Regex> =
    LazyLock::new(|| Regex::new(r"(?i)\{\{rh\|([^|]*)\|[^|]*\|([^|}]*)\}\}").unwrap());
static HWS: LazyLock<Regex> =
    LazyLock::new(|| Regex::new(r"\{\{hws\|[^|}]*\|([^}]*)\}\}").unwrap());
static HWE: LazyLock<Regex> =
    LazyLock::new(|| Regex::new(r"\{\{hwe\|[^|}]*\|([^}]*)\}\}").unwrap());
static REF: LazyLock<Regex> = LazyLock::new(|| Regex::new(r"(?s)<ref[^>]*>(.*?)</ref>").unwrap());
static CHROME: LazyLock<Regex> = LazyLock::new(|| {
    Regex::new(
        r"\{\{(?:rh|anchor|nop|Dhr|smallrefs|PSM rule|PSMPage\w*|PSMLayout\w+|PD-old|reflist)(?:\|[^{}]*)?\}\}",
    )
    .unwrap()
});
// NB: these patterns are assembled by concatenation rather than written as one
// multi-line literal. A trailing backslash inside a Rust *raw* string is a
// literal backslash, not a line continuation, so a wrapped alternation silently
// breaks at the fold.
static MASTHEAD: LazyLock<Regex> = LazyLock::new(|| {
    Regex::new(
        &[
            r"\{\{Pt\|[^}]*\}\}",
            r"\{\{c\|\{\{fs\d+\|\{\{sc\|By C\. S\. PEIRCE[^}]*\}\}\}\}\}\}",
            r"\{\{c\|\{\{fs\d+\|ASSISTANT[^}]*\}\}\}\}",
            r"\{\{c\|(?:FIRST|SECOND|THIRD|FOURTH|FIFTH|SIXTH) PAPER[^}]*\}\}",
        ]
        .join("|"),
    )
    .unwrap()
});
static BOLD: LazyLock<Regex> = LazyLock::new(|| Regex::new(r"(?s)'''(.+?)'''").unwrap());
static ITALIC: LazyLock<Regex> = LazyLock::new(|| Regex::new(r"(?s)''(.+?)''").unwrap());
static PIPED_LINK: LazyLock<Regex> =
    LazyLock::new(|| Regex::new(r"\[\[[^\]|]*\|([^\]]*)\]\]").unwrap());
static PLAIN_LINK: LazyLock<Regex> = LazyLock::new(|| Regex::new(r"\[\[([^\]]*)\]\]").unwrap());
static TAGS: LazyLock<Regex> = LazyLock::new(|| {
    Regex::new(r"</?(?:section|references|pagequality|noinclude)[^>]*/?>").unwrap()
});
static BLANKS: LazyLock<Regex> = LazyLock::new(|| Regex::new(r"\n{3,}").unwrap());

/// The printed page number, read off the running head. Verso carries it left,
/// recto right; the paper's opening page has no head at all.
fn printed_page(raw: &str) -> Option<String> {
    let c = RUNNING_HEAD.captures(raw)?;
    for i in [1, 2] {
        let v = c.get(i).map_or("", |m| m.as_str()).trim();
        if !v.is_empty() {
            return Some(v.to_string());
        }
    }
    None
}

/// Keep only the named section, for a scan page shared with a neighbouring
/// article. A section with no explicit end runs to the foot of the page.
fn take_section(raw: &str, name: &str) -> String {
    let begin = format!("<section begin={name} />");
    let begin_alt = format!("<section begin=\"{name}\" />");
    let start = raw
        .find(&begin)
        .map(|i| i + begin.len())
        .or_else(|| raw.find(&begin_alt).map(|i| i + begin_alt.len()));
    let Some(start) = start else {
        return raw.to_string();
    };
    let rest = &raw[start..];
    let end = rest
        .find(&format!("<section end={name} />"))
        .or_else(|| rest.find(&format!("<section end=\"{name}\" />")))
        .unwrap_or(rest.len());
    rest[..end].to_string()
}

/// `{{hws|com|complain}}` / `{{hwe|plain|complain}}` split one word across a
/// page break. Emit the whole word on the earlier page and drop the remainder.
fn rejoin_hyphenation(pages: &mut [String]) {
    for i in 0..pages.len() {
        let page = pages[i].clone();
        let Some(c) = HWS.captures(&page) else {
            continue;
        };
        let whole = c.get(1).map_or("", |m| m.as_str()).to_string();
        pages[i] = HWS.replace(&page, whole.as_str()).into_owned();
        if let Some(next) = pages.get_mut(i + 1) {
            *next = HWE.replace(next, "").into_owned();
        }
    }
}

// {{frac|1|4}}, {{sfrac|1|2}}, {{over|81|100}} — a stacked fraction. Peirce's
// probabilities read perfectly well inline, which is how modern editions set
// them, so they become "81/100" rather than markup.
static FRACTION: LazyLock<Regex> =
    LazyLock::new(|| Regex::new(r"(?i)\{\{(?:frac|sfrac|over)\|([^|{}]*)\|([^|{}]*)\}\}").unwrap());
// {{overline|V}} — logic notation. A combining overline keeps it text.
static OVERLINE: LazyLock<Regex> =
    LazyLock::new(|| Regex::new(r"(?i)\{\{overline\|([^{}]*)\}\}").unwrap());
static SUBSUP: LazyLock<Regex> =
    LazyLock::new(|| Regex::new(r"(?i)\{\{(sub|sup)\|([^{}]*)\}\}").unwrap());
// Some transcriptions set exponents as literal tags rather than templates.
static SUBSUP_TAG: LazyLock<Regex> =
    LazyLock::new(|| Regex::new(r"(?i)<(sub|sup)>([^<]*)</(?:sub|sup)>").unwrap());

/// Exponents and indices become Unicode characters, never `<sup>`/`<sub>`.
///
/// A footnote reference is *rendered* as `<sup>…</sup>` further down the
/// pipeline, so a literal `<sup>` here is read back as a footnote reference:
/// Peirce's "2⁵" would silently acquire a footnote numbered 5.
fn super_subscript(kind: &str, body: &str) -> String {
    let table: &[(char, char)] = if kind.eq_ignore_ascii_case("sup") {
        &[
            ('0', '⁰'),
            ('1', '¹'),
            ('2', '²'),
            ('3', '³'),
            ('4', '⁴'),
            ('5', '⁵'),
            ('6', '⁶'),
            ('7', '⁷'),
            ('8', '⁸'),
            ('9', '⁹'),
            ('+', '⁺'),
            ('-', '⁻'),
            ('n', 'ⁿ'),
            ('i', 'ⁱ'),
        ]
    } else {
        &[
            ('0', '₀'),
            ('1', '₁'),
            ('2', '₂'),
            ('3', '₃'),
            ('4', '₄'),
            ('5', '₅'),
            ('6', '₆'),
            ('7', '₇'),
            ('8', '₈'),
            ('9', '₉'),
            ('+', '₊'),
            ('-', '₋'),
        ]
    };
    let mapped: Option<String> = body
        .trim()
        .chars()
        .map(|c| table.iter().find(|(from, _)| *from == c).map(|(_, to)| *to))
        .collect();
    match mapped {
        Some(s) if !s.is_empty() => s,
        // Anything Unicode cannot carry stays legible as an ASCII exponent.
        _ => format!("^({})", body.trim()),
    }
}
// {{dent|0|2em|text}} — indentation; the text is the third argument.
static DENT: LazyLock<Regex> =
    LazyLock::new(|| Regex::new(r"(?i)\{\{dent\|[^|{}]*\|[^|{}]*\|([^{}]*)\}\}").unwrap());
// Wrappers whose effect is purely typographic — size, case, alignment,
// indentation. The content is the text; the wrapper is presentation.
const WRAPPERS: &[&str] = &[
    r"fs\d+",
    "sc",
    "asc",
    "smallcaps",
    "c",
    "center",
    "block center",
    "float-center",
    "left",
    "right",
    "larger",
    "x-larger",
    "xx-larger",
    "smaller",
    "smaller block",
    "x-larger block",
    "hi",
    "Pt",
    "di",
    "dropcap",
    "float-left",
    "float-right",
];
static WRAPPER: LazyLock<Regex> = LazyLock::new(|| {
    let pattern = format!(
        r"(?is)\{{\{{(?:{})\|([^{{}}]*?)\|?\}}\}}",
        WRAPPERS.join("|")
    );
    Regex::new(&pattern).unwrap()
});
// {{Left margin|4em|text}} — like {{dent}}, the text is the last argument.
static LEFT_MARGIN: LazyLock<Regex> =
    LazyLock::new(|| Regex::new(r"(?is)\{\{left margin\|[^|{}]*\|([^{}]*)\}\}").unwrap());
// {{SIC|posession|possession}} — the printing's error and its correction. This
// is the diplomatic layer, so the error stands.
static SIC: LazyLock<Regex> =
    LazyLock::new(|| Regex::new(r"(?i)\{\{sic\|([^|{}]*)\|[^{}]*\}\}").unwrap());
static ELLIPSIS: LazyLock<Regex> = LazyLock::new(|| Regex::new(r"\{\{\.\.\.\}\}").unwrap());
// The transcribers set Peirce's "therefore" sign as an image.
static THEREFORE: LazyLock<Regex> =
    LazyLock::new(|| Regex::new(r"\[\[File:ThreeDots\.svg[^\]]*\]\]").unwrap());
// A page marker may already sit at the head of the line a table opens or closes
// on, so neither delimiter can be anchored to the raw line start.
static WIKITABLE: LazyLock<Regex> = LazyLock::new(|| {
    Regex::new(r"(?sm)^(?:\{\{\{[^{}]*\}\}\} )?\{\|.*?^(?:\{\{\{[^{}]*\}\}\} )?\|\}").unwrap()
});
static MARKER: LazyLock<Regex> = LazyLock::new(|| Regex::new(r"\{\{\{[^{}]*\}\}\}").unwrap());
// {{FIS|file=…|caption=…}} — a floating image. There is no image pipeline here,
// so the printing's own caption is kept and the plate itself is not reproduced.
static FIS: LazyLock<Regex> =
    LazyLock::new(|| Regex::new(r"(?is)\{\{FIS\s*\|([^{}]*)\}\}").unwrap());
static TS: LazyLock<Regex> = LazyLock::new(|| Regex::new(r"\{\{ts\|[^{}]*\}\}").unwrap());
// Cells are split on `||` before this runs, so a lone `|` is the boundary
// between a cell's attributes and its content.
static CELL_ATTRS: LazyLock<Regex> =
    LazyLock::new(|| Regex::new(r#"^\s*((?:\w+="?[^"|]*"?\s*)+)\|"#).unwrap());
static SPAN_ATTR: LazyLock<Regex> =
    LazyLock::new(|| Regex::new(r#"(rowspan|colspan)="?(\d+)"?"#).unwrap());
static POEM: LazyLock<Regex> = LazyLock::new(|| Regex::new(r"(?s)<poem>(.*?)</poem>").unwrap());
static FAMILYTREE: LazyLock<Regex> = LazyLock::new(|| {
    Regex::new(r"(?s)\{\{familytree/start\}\}(.*?)\{\{familytree/end\}\}").unwrap()
});
static FT_ROW: LazyLock<Regex> =
    LazyLock::new(|| Regex::new(r"(?s)\{\{familytree\|border=0\|(.*?)\}\}").unwrap());

/// `{{familytree}}` draws a tree with box-art. There is no drawing capability
/// here, and the thing drawn is a classification, so it becomes the nested list
/// it actually is.
///
/// Nesting comes from the grid: labels carry a column index, and each label
/// attaches to the nearest label on the level above it by column distance.
fn familytree_to_figure(src: &str) -> String {
    let mut levels: Vec<Vec<(usize, String)>> = Vec::new();
    for row in FT_ROW.captures_iter(src) {
        let raw = &row[1];
        let (grid, defs) = match raw.find('=') {
            Some(_) => {
                let mut parts = raw.split('|').collect::<Vec<_>>();
                let split = parts
                    .iter()
                    .position(|p| p.contains('='))
                    .unwrap_or(parts.len());
                let defs = parts.split_off(split);
                (parts, defs)
            }
            None => (raw.split('|').collect::<Vec<_>>(), Vec::new()),
        };
        let joined_defs = defs.join("|");
        let mut labels: std::collections::HashMap<&str, String> = Default::default();
        for d in joined_defs.split('|') {
            if let Some((k, v)) = d.split_once('=') {
                labels
                    .entry(k.trim())
                    .or_default()
                    .push_str(v.split_whitespace().collect::<Vec<_>>().join(" ").trim());
            }
        }
        let row_labels: Vec<(usize, String)> = grid
            .iter()
            .enumerate()
            .filter_map(|(col, key)| labels.get(key.trim()).map(|l| (col, l.trim().to_string())))
            .filter(|(_, l)| !l.is_empty())
            .collect();
        if !row_labels.is_empty() {
            levels.push(row_labels);
        }
    }

    // Children attach to the nearest node on the level above, by column.
    let mut html = String::from("<figure>\n");
    fn render(
        levels: &[Vec<(usize, String)>],
        depth: usize,
        parent_col: Option<usize>,
        out: &mut String,
    ) {
        let Some(level) = levels.get(depth) else {
            return;
        };
        let mine: Vec<&(usize, String)> = level
            .iter()
            .filter(|(col, _)| match (parent_col, depth) {
                (None, _) => true,
                (Some(p), _) => nearest_above(levels, depth, *col) == Some(p),
            })
            .collect();
        if mine.is_empty() {
            return;
        }
        out.push_str("  <ul>\n");
        for (col, label) in mine {
            out.push_str(&format!("    <li>{label}\n"));
            render(levels, depth + 1, Some(*col), out);
            out.push_str("    </li>\n");
        }
        out.push_str("  </ul>\n");
    }
    render(&levels, 0, None, &mut html);
    html.push_str("  <figcaption>Diagram</figcaption>\n</figure>");
    html
}

/// The column of the nearest label on the level above `depth`.
fn nearest_above(levels: &[Vec<(usize, String)>], depth: usize, col: usize) -> Option<usize> {
    if depth == 0 {
        return None;
    }
    levels
        .get(depth - 1)?
        .iter()
        .min_by_key(|(c, _)| c.abs_diff(col))
        .map(|(c, _)| *c)
}

/// `<math>` is used here only for ordinary algebra, which reads perfectly well
/// as text. There is no formula renderer in the reader, so a LaTeX blob would
/// otherwise surface raw.
static TEX_SQRT: LazyLock<Regex> =
    LazyLock::new(|| Regex::new(r"\\sqrt\{([^{}]*(?:\{[^{}]*\}[^{}]*)*)\}").unwrap());
static TEX_FRAC: LazyLock<Regex> =
    LazyLock::new(|| Regex::new(r"\\[dt]?frac\{([^{}]*)\}\{([^{}]*)\}").unwrap());
static TEX_SUP: LazyLock<Regex> = LazyLock::new(|| Regex::new(r"\^\{([^{}]*)\}").unwrap());
static TEX_SUB: LazyLock<Regex> = LazyLock::new(|| Regex::new(r"_\{([^{}]*)\}").unwrap());

fn math_to_text(src: &str) -> String {
    let mut s = TEX_SQRT.replace_all(src.trim(), "√($1)").into_owned();
    // Fractions nest one level in these papers; three passes unwind them.
    for _ in 0..3 {
        s = TEX_FRAC.replace_all(&s, "$1/$2").into_owned();
    }
    s = TEX_SUP.replace_all(&s, "^($1)").into_owned();
    s = TEX_SUB.replace_all(&s, "_($1)").into_owned();
    s.replace('\\', "")
        .split_whitespace()
        .collect::<Vec<_>>()
        .join(" ")
}

/// One wikitable → a `<figure>` holding an HTML table.
///
/// The caption is the table's own leading label when it has one ("Fig. 1.",
/// "Table II.", "Deduction.") — a lone short cell on the opening row. Where the
/// printing gives no label the caption is left empty rather than invented, so
/// the reader shows only the automatic "Figure N." number.
fn wikitable_to_figure(src: &str) -> String {
    // Markers are re-emitted ahead of the figure by the caller.
    let body = MARKER.replace_all(src, "");
    let body = TS.replace_all(&body, "");
    let mut rows: Vec<Vec<(String, String)>> = Vec::new();
    let mut current: Vec<(String, String)> = Vec::new();

    for line in body.lines() {
        let line = line.trim_end();
        if line.starts_with("{|") {
            continue;
        }
        if line.starts_with("|}") {
            break;
        }
        if line.starts_with("|-") {
            if !current.is_empty() {
                rows.push(std::mem::take(&mut current));
            }
            continue;
        }
        if let Some(rest) = line.strip_prefix('|').or_else(|| line.strip_prefix('!')) {
            for raw_cell in rest.split("||") {
                let mut attrs = String::new();
                let mut text = raw_cell.to_string();
                if let Some(c) = CELL_ATTRS.captures(raw_cell) {
                    let head = c.get(1).map_or("", |m| m.as_str());
                    for s in SPAN_ATTR.captures_iter(head) {
                        attrs.push_str(&format!(" {}=\"{}\"", &s[1], &s[2]));
                    }
                    text = raw_cell[c.get(0).unwrap().end()..].to_string();
                }
                // A figure's markup is used verbatim, bypassing the markdown
                // pipeline, so emphasis inside a cell must already be HTML.
                let text = BOLD.replace_all(text.trim(), "<b>$1</b>").into_owned();
                let text = ITALIC.replace_all(&text, "<i>$1</i>").into_owned();
                current.push((attrs, text));
            }
        } else if let Some(last) = current.last_mut() {
            // A cell whose content runs onto the next line.
            last.1.push(' ');
            last.1.push_str(line.trim());
        }
    }
    if !current.is_empty() {
        rows.push(current);
    }

    // A figure's caption is its anchor sentence, so the parser requires a
    // non-empty one. Where the printing labels the table ("Fig. 1.", "Table
    // II.", "Deduction.") that label is the caption. Where it does not, the
    // caption falls back to the bare word "Table" — the one piece of editorial
    // wording in this corpus, chosen because it describes the object's form
    // without interpreting its content.
    let mut caption = String::from("Table");
    if let Some(first) = rows.first()
        && first.len() == 1
        && first[0].1.len() <= 30
        && rows.len() > 1
    {
        caption = first[0].1.clone();
        rows.remove(0);
    }

    let mut html = String::from("<figure>\n  <table>\n");
    for row in rows {
        html.push_str("    <tr>");
        for (attrs, text) in row {
            if text.is_empty() {
                continue;
            }
            html.push_str(&format!("<td{attrs}>{text}</td>"));
        }
        html.push_str("</tr>\n");
    }
    html.push_str("  </table>\n");
    html.push_str(&format!("  <figcaption>{caption}</figcaption>\n</figure>"));
    html
}
// Ornaments carrying no text at all.
const ORNAMENTS: &[&str] = &[
    "gap",
    "pbr",
    "nop",
    "Dhr",
    "smallrefs",
    "PSM rule",
    "reflist",
    "PD-old",
    r"PSMLayout\w+",
    "block center/s",
    "block center/e",
];
static ORNAMENT: LazyLock<Regex> = LazyLock::new(|| {
    let pattern = format!(
        r"(?i)\{{\{{(?:{})(?:\|[^{{}}]*)?\}}\}}",
        ORNAMENTS.join("|")
    );
    Regex::new(&pattern).unwrap()
});
static EMDASH: LazyLock<Regex> = LazyLock::new(|| Regex::new(r"\{\{--\}\}").unwrap());
// Ligature templates: {{ae}} → æ etc.
static LIGATURE: LazyLock<Regex> = LazyLock::new(|| Regex::new(r"\{\{(ae|AE|oe|OE)\}\}").unwrap());

fn unwrap_templates(text: &str) -> String {
    let mut t = text.to_string();
    // Innermost-first: {{smaller block|{{dent|0|2em|{{sc|b. c.}}}}}} needs
    // several passes to unwind.
    for _ in 0..8 {
        let before = t.clone();
        // Ornaments clear first: a multi-line wrapper cannot be unwrapped while
        // an inner {{gap}} still puts braces inside its content.
        t = ORNAMENT.replace_all(&t, "").into_owned();
        t = FRACTION.replace_all(&t, "$1/$2").into_owned();
        t = OVERLINE.replace_all(&t, "${1}\u{0305}").into_owned();
        t = SUBSUP
            .replace_all(&t, |c: &regex::Captures| super_subscript(&c[1], &c[2]))
            .into_owned();
        t = SUBSUP_TAG
            .replace_all(&t, |c: &regex::Captures| super_subscript(&c[1], &c[2]))
            .into_owned();
        t = DENT.replace_all(&t, "$1").into_owned();
        t = LEFT_MARGIN.replace_all(&t, "$1").into_owned();
        t = SIC.replace_all(&t, "$1").into_owned();
        t = WRAPPER.replace_all(&t, "$1").into_owned();
        if t == before {
            break;
        }
    }
    t = EMDASH.replace_all(&t, "\u{2014}").into_owned();
    t = LIGATURE
        .replace_all(&t, |c: &regex::Captures| {
            match &c[1] {
                "ae" => "æ",
                "AE" => "Æ",
                "oe" => "œ",
                _ => "Œ",
            }
            .to_string()
        })
        .into_owned();
    t = ELLIPSIS.replace_all(&t, "\u{2026}").into_owned();
    t = ORNAMENT.replace_all(&t, "").into_owned();
    CHROME.replace_all(&t, "").into_owned()
}

fn to_markdown(text: &str) -> String {
    let t = BOLD.replace_all(text, "**$1**").into_owned();
    let t = ITALIC.replace_all(&t, "_${1}_").into_owned();
    let t = PIPED_LINK.replace_all(&t, "$1").into_owned();
    let t = PLAIN_LINK.replace_all(&t, "$1").into_owned();
    TAGS.replace_all(&t, "").into_owned()
}

/// Lift `<ref>` bodies out into markdown footnote definitions.
fn extract_footnotes(text: &str) -> (String, Vec<(String, String)>) {
    let mut notes: Vec<(String, String)> = Vec::new();
    let out = REF.replace_all(text, |c: &regex::Captures| {
        let mark = mark(notes.len());
        let body = c
            .get(1)
            .map_or("", |m| m.as_str())
            .split_whitespace()
            .collect::<Vec<_>>()
            .join(" ");
        notes.push((mark.clone(), body));
        format!("[^{mark}]")
    });
    (out.into_owned(), notes)
}

/// A page break falling inside a sentence must not become a paragraph break.
/// Rust's regex has no lookbehind, so the preceding character is inspected here.
fn join_mid_sentence_breaks(text: &str) -> String {
    static RE: LazyLock<Regex> =
        LazyLock::new(|| Regex::new(r"(?s)(</figure>|.)\n\n(\{\{\{)").unwrap());
    RE.replace_all(text, |c: &regex::Captures| {
        let prev = c.get(1).unwrap().as_str();
        let marker = c.get(2).unwrap().as_str();
        // A figure is its own block; joining a marker onto it would swallow the
        // following paragraph into the figure.
        if prev == "</figure>" || matches!(prev, "." | "!" | "?" | ":" | "\"" | "'" | "\u{2019}") {
            // A real paragraph boundary that happens to fall on a page break.
            format!("{prev}\n\n{marker}")
        } else {
            format!("{prev} {marker}")
        }
    })
    .into_owned()
}

/// Printed page for a scan page.
///
/// An article occupies a contiguous run of scan pages, so the printed number
/// advances with the scan number from the paper's declared opening page. Many
/// transcriptions also record a running head; where one exists it is used as a
/// **check**, and a disagreement is an error rather than something to paper
/// over — it would mean the run is not contiguous after all (an inserted plate,
/// a mis-declared range) and every later page would be off by one.
fn resolve_page(paper: &Paper, scan: u32, raw: &str) -> Result<u32, String> {
    let derived = paper.first_page + (scan - paper.from);
    if let Some(head) = printed_page(raw)
        && let Ok(printed) = head.trim().parse::<u32>()
        && printed != derived
    {
        return Err(format!(
            "{}: scan page {scan} running head says {printed}, but the declared \
             range puts it at {derived}",
            paper.label
        ));
    }
    Ok(derived)
}

pub fn convert(paper: &Paper, raw_pages: &[(u32, String)]) -> String {
    let last = raw_pages.len().saturating_sub(1);
    let mut bodies: Vec<String> = Vec::new();
    let mut numbers: Vec<Option<u32>> = Vec::new();

    for (i, (scan, raw)) in raw_pages.iter().enumerate() {
        let page = resolve_page(paper, *scan, raw).unwrap_or_else(|e| panic!("{e}"));
        // The opening page's number lives in the front matter, not in a marker.
        numbers.push((i != 0).then_some(page));
        let mut body = raw.clone();
        if i == 0
            && let Some(s) = paper.from_section
        {
            body = take_section(&body, s);
        }
        if i == last
            && let Some(s) = paper.to_section
        {
            body = take_section(&body, s);
        }
        body = NOINCLUDE.replace_all(&body, "").into_owned();
        bodies.push(body);
    }

    rejoin_hyphenation(&mut bodies);

    // The journal's article masthead (series title, byline, affiliation, the
    // "FIRST PAPER.—…" line) is periodical apparatus like the running heads,
    // not Peirce's text; the paper's own title is the node label.
    if let Some(first) = bodies.first_mut() {
        *first = MASTHEAD.replace_all(first, "").into_owned();
    }

    let mut chunks: Vec<String> = Vec::new();
    for (body, number) in bodies.iter().zip(&numbers) {
        let body = unwrap_templates(body);
        let body = body.trim();
        if body.is_empty() {
            continue;
        }
        match number {
            Some(n) => chunks.push(format!(
                "{{{{{{ {} {}:{} }}}}}} {body}",
                paper.venue, paper.volume, n
            )),
            None => chunks.push(body.to_string()),
        }
    }

    let joined = chunks.join("\n\n");
    let joined = THEREFORE.replace_all(&joined, "\u{2234}").into_owned();
    let joined = Regex::new(r"(?s)<math>(.*?)</math>")
        .unwrap()
        .replace_all(&joined, |c: &regex::Captures| math_to_text(&c[1]))
        .into_owned();
    // A second unwrap pass: a wrapper around a formula ({{c|<math>…</math>}})
    // could not be unwrapped while the LaTeX braces were still inside it.
    let joined = unwrap_templates(&joined);
    // Verse quoted inside the prose: line breaks are the point, so they become
    // the `<br>` the verse corpora already use.
    let joined = POEM
        .replace_all(&joined, |c: &regex::Captures| {
            c[1].lines()
                .map(str::trim)
                .filter(|l| !l.is_empty())
                .collect::<Vec<_>>()
                .join("<br>")
        })
        .into_owned();
    let joined = FAMILYTREE
        .replace_all(&joined, |c: &regex::Captures| {
            format!("\n\n{}\n\n", familytree_to_figure(&c[1]))
        })
        .into_owned();
    // A plate: keep the printing's own caption, and record which image it was
    // in a comment so a later image pipeline can restore it. The caption is
    // source text; nothing is written in its place.
    let joined = FIS
        .replace_all(&joined, |c: &regex::Captures| {
            let mut file = String::new();
            let mut caption = String::new();
            for param in c[1].split('|') {
                if let Some((k, v)) = param.split_once('=') {
                    match k.trim() {
                        "file" => file = v.trim().to_string(),
                        "caption" => caption = v.trim().to_string(),
                        _ => {}
                    }
                }
            }
            format!(
                "\n\n<figure>\n  <!-- plate not reproduced: {file} -->\n  \
                 <figcaption>{caption}</figcaption>\n</figure>\n\n"
            )
        })
        .into_owned();
    // Each table becomes its own figure block, so it needs blank lines around it.
    // Any page marker caught inside the table is re-emitted ahead of the figure
    // rather than dropped — a page really does turn there.
    let joined = WIKITABLE
        .replace_all(&joined, |c: &regex::Captures| {
            let src = &c[0];
            let markers: Vec<&str> = MARKER.find_iter(src).map(|m| m.as_str()).collect();
            let lead = if markers.is_empty() {
                String::new()
            } else {
                format!("{}\n\n", markers.join(" "))
            };
            format!("\n\n{lead}{}\n\n", wikitable_to_figure(src))
        })
        .into_owned();

    let (joined, notes) = extract_footnotes(&joined);
    let joined = to_markdown(&joined);
    // Note bodies were lifted out before the markup pass, so their wiki
    // emphasis (''…'') must be converted separately or it reaches the reader
    // as literal quote marks.
    let notes: Vec<(String, String)> = notes
        .into_iter()
        .map(|(mark, text)| (mark, to_markdown(&text)))
        .collect();
    let joined = BLANKS.replace_all(&joined, "\n\n").into_owned();
    let joined = join_mid_sentence_breaks(&joined);

    let body: String = joined
        .split("\n\n")
        .map(str::trim)
        .filter(|p| !p.is_empty())
        .collect::<Vec<_>>()
        .join("\n\n");

    let mut out = format!(
        "---\nposition: {}\nlabel: \"{}\"\ndepth: 1\npage_pub: \"{} {}:{}\"\n---\n\n## {}\n\n\n{}\n",
        paper.position, paper.label, paper.venue, paper.volume, paper.first_page, paper.label, body,
    );
    // Each definition is its own block, so they are separated by blank lines —
    // run them together and the parser reads one block and keeps only the first
    // marker, collapsing every note in the file onto one number.
    for (mark, text) in notes {
        out.push_str(&format!("\n[^{mark}]: {text}\n"));
    }
    out
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn printed_page_reads_either_side_of_the_running_head() {
        assert_eq!(
            printed_page("{{rh|4|''THE POPULAR SCIENCE MONTHLY.''|}}").as_deref(),
            Some("4")
        );
        assert_eq!(
            printed_page("{{rh||''ILLUSTRATIONS.''|3}}").as_deref(),
            Some("3")
        );
        assert_eq!(printed_page("no running head here"), None);
    }

    #[test]
    fn hyphenation_rejoins_across_the_page_break() {
        let mut pages = vec![
            "will not {{hws|com|complain}}".to_string(),
            "{{hwe|plain|complain}} that there are blows".to_string(),
        ];
        rejoin_hyphenation(&mut pages);
        assert_eq!(pages[0], "will not complain");
        assert_eq!(pages[1], " that there are blows");
    }

    #[test]
    fn nested_formatting_templates_unwrap() {
        assert_eq!(
            unwrap_templates("{{c|{{fs90|{{sc|By C. S. PEIRCE,}}}}}}").trim(),
            "By C. S. PEIRCE,"
        );
    }

    #[test]
    fn a_shared_page_yields_only_its_own_section() {
        let raw = "earlier article text<section begin=E25 />ours continues<section end=E25 />next article";
        assert_eq!(take_section(raw, "E25"), "ours continues");
    }

    #[test]
    fn a_section_without_an_end_runs_to_the_foot_of_the_page() {
        let raw = "prior<section begin=B300 />ours to the end";
        assert_eq!(take_section(raw, "B300"), "ours to the end");
    }

    #[test]
    fn footnotes_become_marked_definitions() {
        let (text, notes) = extract_footnotes("a claim<ref>Not quite so.</ref> and more");
        assert_eq!(text, "a claim[^*] and more");
        assert_eq!(notes, vec![("*".to_string(), "Not quite so.".to_string())]);
    }

    #[test]
    fn a_page_break_inside_a_sentence_does_not_split_the_paragraph() {
        let joined = join_mid_sentence_breaks("about Nature\n\n{{{ PSM 12:2 }}} which the senses");
        assert_eq!(joined, "about Nature {{{ PSM 12:2 }}} which the senses");
    }

    #[test]
    fn exponents_become_unicode_not_sup_tags() {
        // A <sup> here would be read back as a footnote reference downstream,
        // silently attaching footnote 5 to Peirce's "2⁵".
        assert_eq!(super_subscript("sup", "5"), "⁵");
        assert_eq!(super_subscript("sup", "32"), "³²");
        assert_eq!(super_subscript("sub", "1"), "₁");
        assert_eq!(unwrap_templates("2{{sup|5}} or 32"), "2⁵ or 32");
        assert_eq!(unwrap_templates("2<sup>32</sup>"), "2³²");
        assert!(!unwrap_templates("{{sup|5}}").contains("<sup>"));
    }

    #[test]
    fn an_exponent_unicode_cannot_carry_stays_legible() {
        assert_eq!(super_subscript("sup", "p-1"), "^(p-1)");
    }

    #[test]
    fn formulae_become_readable_text() {
        assert_eq!(math_to_text(r"\sqrt{\tfrac{2p(1-p)}{s}}"), "√(2p(1-p)/s)");
        assert_eq!(math_to_text(r"(x+y)^{M+1}"), "(x+y)^(M+1)");
    }

    #[test]
    fn a_table_takes_its_printed_label_as_caption() {
        let src = "{|\n|-\n|Table II.\n|-\n|αβ\n|αβγ\n|}";
        let fig = wikitable_to_figure(src);
        assert!(fig.contains("<figcaption>Table II.</figcaption>"), "{fig}");
        assert!(fig.contains("<td>αβ</td>"), "{fig}");
    }

    #[test]
    fn an_unlabelled_table_still_carries_a_caption() {
        // The parser requires one: a figure's caption is its anchor sentence.
        let src = "{|\n|-\n|93\n|of\n|81\n|-\n|100\n|100\n|}";
        assert!(wikitable_to_figure(src).contains("<figcaption>Table</figcaption>"));
    }

    #[test]
    fn a_family_tree_becomes_the_nested_classification_it_draws() {
        let src = "{{familytree|border=0| | | |INF|INF=Inference.}}\n\
                   {{familytree|border=0| |,|-|-|^|-|-|-|.|}}\n\
                   {{familytree|border=0|DA| | | | |SINT|DA=Deductive.|SINT=Synthetic.}}\n\
                   {{familytree|border=0|||||IND||||HYP|IND=Induction.|HYP=Hypothesis.}}";
        let fig = familytree_to_figure(src);
        // Induction and Hypothesis hang under Synthetic, not under Deductive.
        let synth = fig.find("Synthetic.").expect("synthetic present");
        let ded = fig.find("Deductive.").expect("deductive present");
        let ind = fig.find("Induction.").expect("induction present");
        assert!(ded < synth && synth < ind, "{fig}");
        assert_eq!(fig.matches("<ul>").count(), 3, "{fig}");
    }

    #[test]
    fn a_page_break_at_a_real_paragraph_end_keeps_the_break() {
        let joined = join_mid_sentence_breaks("ends here.\n\n{{{ PSM 12:2 }}} A new paragraph");
        assert_eq!(joined, "ends here.\n\n{{{ PSM 12:2 }}} A new paragraph");
    }
}

#[cfg(test)]
mod emphasis_tests {
    use super::*;

    // Regression: the replacement "_$1_" parses as capture group `1_` in the
    // regex crate — nonexistent, so it expanded EMPTY and every italicised
    // word in the corpus was silently eaten. `${1}` is unambiguous.
    #[test]
    fn italics_keep_their_words() {
        let note = "''Logique''. The same is true.";
        assert_eq!(to_markdown(note), "_Logique_. The same is true.");
    }

    #[test]
    fn note_bodies_get_the_markup_pass_too() {
        let body = "a miracle.<ref>''Logique''. The same is true.</ref> I respect this.";
        let (out, notes) = extract_footnotes(body);
        assert_eq!(out, "a miracle.[^*] I respect this.");
        assert_eq!(to_markdown(&notes[0].1), "_Logique_. The same is true.");
    }
}
