//! Piece stream → one markdown paragraph: whitespace collapsed, line-end
//! hyphens settled, page markers placed, Sperrsatz as `_…_`, footnote call
//! sites as `[^@xml-id]` placeholders (renumbered per node file later).

use std::collections::{HashMap, HashSet};

use crate::Result;
use crate::hyphen::{Decision, Oracle, Rulings};
use crate::source::{NotePart, Piece};

pub const GW_VOLUME: u32 = 20;

pub fn gw_marker(page: u32) -> String {
    format!("{{{{{{ {GW_VOLUME}.{page} }}}}}} ")
}

#[derive(Debug, Default)]
pub struct PageState {
    pub page: u32,
    pub pending: Option<u32>,
}

#[derive(Debug)]
pub struct HyphenLogEntry {
    pub page: u32,
    pub left: String,
    pub right: String,
    pub decision: Decision,
    pub via: &'static str,
}

pub struct Renderer<'a> {
    pub oracle: &'a Oracle,
    pub rulings: &'a Rulings,
    pub notes: &'a HashMap<String, NotePart>,
    pub consumed: HashSet<String>,
    pub definitions: HashMap<String, String>,
    pub hyphens: Vec<HyphenLogEntry>,
    pub skipped_pages: Vec<u32>,
    pub unresolved: Vec<String>,
    pub joined_paragraphs: Vec<String>,
}

type Chars = Vec<(char, bool)>;

struct PendingHyphen {
    token_start: usize,
    page: u32,
    markers: Vec<u32>,
}

impl<'a> Renderer<'a> {
    pub fn new(
        oracle: &'a Oracle,
        rulings: &'a Rulings,
        notes: &'a HashMap<String, NotePart>,
    ) -> Self {
        Renderer {
            oracle,
            rulings,
            notes,
            consumed: HashSet::new(),
            definitions: HashMap::new(),
            hyphens: Vec::new(),
            skipped_pages: Vec::new(),
            unresolved: Vec::new(),
            joined_paragraphs: Vec::new(),
        }
    }

    pub fn render(&mut self, pieces: &[Piece], state: &mut PageState) -> Result<String> {
        let mut buf: Chars = Vec::new();
        let mut emph = false;
        let mut pending: Option<PendingHyphen> = None;
        for piece in pieces {
            match piece {
                Piece::Open => emph = true,
                Piece::Close => emph = false,
                Piece::Lb => {
                    if pending.is_none() {
                        if let Some(token_start) = hyphen_token_start(&buf) {
                            pending = Some(PendingHyphen {
                                token_start,
                                page: state.page,
                                markers: Vec::new(),
                            });
                        } else {
                            ensure_space(&mut buf);
                        }
                    }
                }
                Piece::Pb(page) => {
                    state.page = *page;
                    match pending.as_mut() {
                        Some(p) => p.markers.push(*page),
                        None => {
                            if let Some(lost) = state.pending.replace(*page) {
                                self.skipped_pages.push(lost);
                            }
                        }
                    }
                }
                Piece::Text(raw) => {
                    let text = raw.replace(['\n', '\r', '\t'], " ");
                    if let Some(p) = pending.take() {
                        let body = text.trim_start();
                        if body.is_empty() {
                            pending = Some(p);
                        } else {
                            self.resolve(p, body, emph, &mut buf)?;
                        }
                        continue;
                    }
                    let body = text.trim_start();
                    if body.is_empty() {
                        ensure_space(&mut buf);
                        continue;
                    }
                    if text.starts_with(' ') {
                        ensure_space(&mut buf);
                    }
                    if let Some(page) = state.pending.take() {
                        ensure_space(&mut buf);
                        push(&mut buf, &gw_marker(page), false);
                    }
                    push(&mut buf, body, emph);
                }
                Piece::Note(id) => {
                    let continuation = self.notes.get(id).is_some_and(|n| n.prev.is_some());
                    if pending.is_some() && !continuation {
                        return Err(format!("note {id} directly after an unresolved hyphen"));
                    }
                    self.note(id, state, &mut buf)?;
                }
            }
        }
        if pending.is_some() {
            let tail: String = buf
                .iter()
                .rev()
                .take(30)
                .map(|(c, _)| *c)
                .collect::<Vec<_>>()
                .into_iter()
                .rev()
                .collect();
            return Err(format!(
                "paragraph ends on a line-end hyphen on GW 20.{}: …{tail}",
                state.page
            ));
        }
        Ok(finish(buf))
    }

    fn resolve(&mut self, p: PendingHyphen, body: &str, emph: bool, buf: &mut Chars) -> Result<()> {
        let left: String = buf[p.token_start..].iter().map(|(c, _)| c).collect();
        let left = left
            .trim_start_matches(|c: char| !c.is_alphabetic())
            .to_string();
        let token = body.split_whitespace().next().unwrap_or("");
        let chained = token.ends_with('-');
        let right: String = token
            .chars()
            .take_while(|c| c.is_alphabetic() || *c == '-')
            .collect::<String>()
            .trim_end_matches('-')
            .to_string();
        if right.is_empty() {
            return Err(format!(
                "hyphen join on GW 20.{}: {left} | {body:.20} — right fragment is not a word",
                p.page
            ));
        }
        let (decision, via) = self
            .oracle
            .decide(self.rulings, p.page, &left, &right, chained)
            .unwrap_or_else(|e| {
                self.unresolved.push(e);
                (Decision::Join, "unresolved")
            });
        self.hyphens.push(HyphenLogEntry {
            page: p.page,
            left: left.clone(),
            right: right.clone(),
            decision,
            via,
        });
        let marker_chars = |markers: &[u32]| -> Chars {
            markers
                .iter()
                .flat_map(|m| gw_marker(*m).chars().map(|c| (c, false)).collect::<Chars>())
                .collect()
        };
        match decision {
            Decision::Join | Decision::Keep | Decision::Ck => {
                let tail = match decision {
                    Decision::Join => {
                        buf.pop();
                        body
                    }
                    Decision::Ck => {
                        buf.pop();
                        buf.pop();
                        push(buf, "ck", emph_of_last(buf, emph));
                        &body[body.char_indices().nth(1).map_or(body.len(), |(i, _)| i)..]
                    }
                    _ => body,
                };
                buf.splice(p.token_start..p.token_start, marker_chars(&p.markers));
                push(buf, tail, emph);
            }
            Decision::Space => {
                ensure_space(buf);
                buf.extend(marker_chars(&p.markers));
                push(buf, body, emph);
            }
        }
        Ok(())
    }

    fn note(&mut self, id: &str, state: &PageState, buf: &mut Chars) -> Result<()> {
        let notes = self.notes;
        let first = notes.get(id).ok_or_else(|| format!("unknown note {id}"))?;
        if first.prev.is_some() {
            if !self.consumed.contains(id) {
                return Err(format!("continuation note {id} precedes its first part"));
            }
            return Ok(());
        }
        let mut pieces: Vec<Piece> = first.pieces.clone();
        self.consumed.insert(id.to_string());
        let mut current = id.to_string();
        let mut next = first.next.clone();
        while let Some(next_id) = next {
            let part = notes
                .get(&next_id)
                .ok_or_else(|| format!("note {current} links to missing {next_id}"))?;
            if part.prev.as_deref() != Some(current.as_str()) {
                return Err(format!("note {next_id} does not link back to {current}"));
            }
            pieces.push(Piece::Lb);
            pieces.extend(part.pieces.iter().cloned());
            self.consumed.insert(next_id.clone());
            next = part.next.clone();
            current = next_id;
        }
        let mut note_state = PageState {
            page: state.page,
            pending: None,
        };
        let text = self.render(&pieces, &mut note_state)?;
        self.definitions.insert(id.to_string(), text);
        let had_space = buf.last().is_some_and(|(c, _)| *c == ' ');
        if had_space {
            buf.pop();
        }
        push(buf, &format!("[^@{id}]"), false);
        if had_space {
            buf.push((' ', false));
        }
        Ok(())
    }
}

fn emph_of_last(buf: &Chars, fallback: bool) -> bool {
    buf.last().map_or(fallback, |(_, e)| *e)
}

fn push(buf: &mut Chars, text: &str, emph: bool) {
    buf.extend(text.chars().map(|c| (c, emph)));
}

fn ensure_space(buf: &mut Chars) {
    if buf.last().is_some_and(|(c, _)| *c != ' ') {
        buf.push((' ', false));
    }
}

/// Start of the word the buffer ends in when it ends on a letter-hyphen.
fn hyphen_token_start(buf: &Chars) -> Option<usize> {
    let n = buf.len();
    if n < 2 || buf[n - 1].0 != '-' || !buf[n - 2].0.is_alphabetic() {
        return None;
    }
    Some(
        buf.iter()
            .rposition(|(c, _)| *c == ' ')
            .map_or(0, |i| i + 1),
    )
}

/// Collapse whitespace, trim, and write emphasis runs as `_…_` (or `<i>…</i>`
/// where the run abuts a letter and markdown underscores cannot express it).
fn finish(buf: Chars) -> String {
    let mut collapsed: Chars = Vec::with_capacity(buf.len());
    for (c, e) in buf {
        if c.is_whitespace() {
            match collapsed.last_mut() {
                Some((' ', le)) => *le = *le && e,
                _ => collapsed.push((' ', e)),
            }
        } else {
            collapsed.push((c, e));
        }
    }
    while collapsed.last().is_some_and(|(c, _)| *c == ' ') {
        collapsed.pop();
    }
    let lead = collapsed.iter().take_while(|(c, _)| *c == ' ').count();
    collapsed.drain(..lead);

    let mut out = String::new();
    let mut i = 0;
    while i < collapsed.len() {
        let emph = collapsed[i].1;
        let mut j = i;
        while j < collapsed.len() && collapsed[j].1 == emph {
            j += 1;
        }
        let run: String = collapsed[i..j].iter().map(|(c, _)| c).collect();
        if !emph || run.trim().is_empty() {
            out.push_str(&run);
        } else {
            let core = run.trim();
            let lead = &run[..run.len() - run.trim_start().len()];
            let trail = &run[run.trim_end().len()..];
            let before = if lead.is_empty() {
                collapsed[..i].last().map(|(c, _)| *c)
            } else {
                None
            };
            let after = if trail.is_empty() {
                collapsed.get(j).map(|(c, _)| *c)
            } else {
                None
            };
            let intraword = before.is_some_and(char::is_alphanumeric)
                || after.is_some_and(char::is_alphanumeric);
            out.push_str(lead);
            if intraword {
                out.push_str(&format!("<i>{core}</i>"));
            } else {
                out.push_str(&format!("_{core}_"));
            }
            out.push_str(trail);
        }
        i = j;
    }
    out
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::source::{self, Item};

    fn render_xml(inner: &str, json_pages: &[(u32, &str)], rulings: &str) -> Result<String> {
        let xml = format!(
            "<TEI xmlns=\"http://www.tei-c.org/ns/1.0\"><text><body><div>{inner}</div></body></text></TEI>"
        );
        let src = source::parse(&xml)?;
        let oracle = Oracle::from_page_texts(json_pages);
        let rulings = Rulings::parse(rulings)?;
        let mut r = Renderer::new(&oracle, &rulings, &src.notes);
        let mut state = PageState::default();
        let mut out = Vec::new();
        for item in &src.items {
            match item {
                Item::Pb(p) => {
                    state.page = *p;
                    state.pending = Some(*p);
                }
                Item::Para { pieces, .. } => out.push(r.render(pieces, &mut state)?),
                Item::Head(_) => {}
            }
        }
        if !r.unresolved.is_empty() {
            return Err(r.unresolved.join("\n"));
        }
        Ok(out.join("\n"))
    }

    #[test]
    fn hyphen_joins_follow_the_json_and_emphasis_survives() {
        let out = render_xml(
            "<pb n=\"5\"/><p>Das <hi rendition=\"#g\">Bewußt-</hi><lb/><hi rendition=\"#g\">seyn</hi> und ein Experimental-<lb/>Physik.<lb/></p>",
            &[(5, "Das Bewußtseyn und ein Experimental-Physik.")],
            "",
        )
        .unwrap();
        assert_eq!(
            out,
            "{{{ 20.5 }}} Das _Bewußtseyn_ und ein Experimental-Physik."
        );
    }

    #[test]
    fn a_page_break_inside_a_hyphenated_word_moves_the_marker_before_the_word() {
        let out = render_xml(
            "<pb n=\"62\"/><p>Nun ohne Be-<lb/><pb n=\"63\"/><fw type=\"header\">X</fw><lb/>weis Statt.</p>",
            &[(62, "Nun ohne Beweis Statt.")],
            "",
        )
        .unwrap();
        assert_eq!(out, "{{{ 20.62 }}} Nun ohne {{{ 20.63 }}} Beweis Statt.");
    }

    #[test]
    fn a_suspended_hyphen_keeps_its_space_and_the_marker_follows_it() {
        let out = render_xml(
            "<pb n=\"6\"/><p>ein an-<lb/><pb n=\"7\"/>und aus.</p>",
            &[(7, "ein an- und aus.")],
            "",
        )
        .unwrap();
        assert_eq!(out, "{{{ 20.6 }}} ein an- {{{ 20.7 }}} und aus.");
    }

    #[test]
    fn old_ck_line_breaks_become_ck() {
        let out = render_xml("<p>zu rük-<lb/>ken.</p>", &[(0, "zu rücken.")], "").unwrap();
        assert_eq!(out, "zu rücken.");
    }

    #[test]
    fn a_ruling_settles_what_the_json_cannot() {
        let json = [(1, "WortStück Wort-Stück")];
        let xml = "<pb n=\"1\"/><p>Wort-<lb/>Stück</p>";
        assert!(render_xml(xml, &json, "").is_err());
        let out = render_xml(xml, &json, "1\tWort-\tStück\tkeep\n").unwrap();
        assert_eq!(out, "{{{ 20.1 }}} Wort-Stück");
    }

    #[test]
    fn footnote_parts_are_rejoined_into_one_definition() {
        let xml = "<p>Satz.<note place=\"foot\" xml:id=\"a\" n=\"∗)\" next=\"#b\">Erster Teil mit ge-<lb/></note></p>\
                   <pb n=\"2\"/><p>Weiter.<note place=\"foot\" xml:id=\"b\" prev=\"#a\">wissem Rest.</note></p>";
        let src = source::parse(&format!(
            "<TEI xmlns=\"http://www.tei-c.org/ns/1.0\"><text><body><div>{xml}</div></body></text></TEI>"
        ))
        .unwrap();
        let oracle = Oracle::from_page_texts(&[(1, "Erster Teil mit gewissem Rest.")]);
        let rulings = Rulings::default();
        let mut r = Renderer::new(&oracle, &rulings, &src.notes);
        let mut state = PageState {
            page: 1,
            pending: None,
        };
        let mut paras = Vec::new();
        for item in &src.items {
            if let Item::Para { pieces, .. } = item {
                paras.push(r.render(pieces, &mut state).unwrap());
            }
        }
        assert_eq!(paras[0], "Satz.[^@a]");
        assert_eq!(paras[1], "Weiter.");
        assert_eq!(r.definitions["a"], "Erster Teil mit gewissem Rest.");
        assert_eq!(r.consumed.len(), 2);
    }

    #[test]
    fn intraword_emphasis_falls_back_to_html() {
        let out = render_xml("<p>un<hi rendition=\"#g\">ter</hi>scheiden</p>", &[], "").unwrap();
        assert_eq!(out, "un<i>ter</i>scheiden");
    }
}
