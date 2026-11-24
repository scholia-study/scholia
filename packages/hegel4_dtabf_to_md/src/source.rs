//! TEI → item stream. Only the element vocabulary inventoried in GW20.xml is
//! accepted; anything else is an error rather than a guess.

use std::collections::HashMap;

use roxmltree::{Document, Node};

use crate::Result;

const XML_NS: &str = "http://www.w3.org/XML/1998/namespace";

#[derive(Debug, Clone, PartialEq)]
pub enum Piece {
    Text(String),
    Lb,
    Pb(u32),
    Open,
    Close,
    Note(String),
}

#[derive(Debug, Clone)]
pub enum Item {
    Pb(u32),
    Head(Vec<Piece>),
    Para { remark: bool, pieces: Vec<Piece> },
}

#[derive(Debug)]
pub struct NotePart {
    pub pieces: Vec<Piece>,
    pub next: Option<String>,
    pub prev: Option<String>,
    pub mark_lacks_paren: bool,
}

#[derive(Debug, Default)]
pub struct SourceStats {
    pub milestones: usize,
    pub vertical_spaces: usize,
    pub foreign: usize,
}

#[derive(Debug)]
pub struct Source {
    pub items: Vec<Item>,
    pub notes: HashMap<String, NotePart>,
    pub stats: SourceStats,
}

pub fn parse(xml: &str) -> Result<Source> {
    let doc = Document::parse(xml).map_err(|e| format!("XML: {e}"))?;
    let body = doc
        .descendants()
        .find(|n| n.has_tag_name("body"))
        .ok_or("no <body>")?;
    let mut source = Source {
        items: Vec::new(),
        notes: HashMap::new(),
        stats: SourceStats::default(),
    };
    walk_blocks(body, &mut source)?;
    Ok(source)
}

fn walk_blocks(parent: Node, source: &mut Source) -> Result<()> {
    for child in parent.children() {
        if child.is_text() {
            let t = child.text().unwrap_or("");
            if !t.trim().is_empty() {
                return Err(format!("stray block-level text {t:?}"));
            }
            continue;
        }
        if !child.is_element() {
            continue;
        }
        match child.tag_name().name() {
            "div" => walk_blocks(child, source)?,
            "head" => {
                let mut pieces = Vec::new();
                inline(child, &mut pieces, source, false)?;
                if pieces
                    .iter()
                    .any(|p| matches!(p, Piece::Pb(_) | Piece::Note(_)))
                {
                    return Err("pb or note inside <head>".into());
                }
                source.items.push(Item::Head(pieces));
            }
            "p" => {
                let remark = match child.attribute("rendition") {
                    None => false,
                    Some("#et #smaller") => true,
                    Some(other) => return Err(format!("unknown <p rendition={other:?}>")),
                };
                let mut pieces = Vec::new();
                inline(child, &mut pieces, source, false)?;
                source.items.push(Item::Para { remark, pieces });
            }
            "pb" => source.items.push(Item::Pb(page_number(child)?)),
            "fw" => check_running_head(child)?,
            "lb" => {}
            "milestone" => count_milestone(child, source)?,
            "space" => {
                if child.attribute("dim") != Some("vertical") {
                    return Err("<space> that is not dim=vertical".into());
                }
                source.stats.vertical_spaces += 1;
            }
            other => return Err(format!("unexpected block-level <{other}>")),
        }
    }
    Ok(())
}

fn inline(parent: Node, out: &mut Vec<Piece>, source: &mut Source, in_note: bool) -> Result<()> {
    let mut stray_paren = false;
    for child in parent.children() {
        if child.is_text() {
            let mut text = child.text().unwrap_or("");
            if std::mem::take(&mut stray_paren) {
                text = text.strip_prefix(')').unwrap_or(text);
            }
            match out.last_mut() {
                Some(Piece::Text(prev)) => prev.push_str(text),
                _ => out.push(Piece::Text(text.to_string())),
            }
            continue;
        }
        if !child.is_element() {
            continue;
        }
        stray_paren = false;
        match child.tag_name().name() {
            "lb" => out.push(Piece::Lb),
            "pb" => {
                if in_note {
                    return Err("pb inside <note>".into());
                }
                out.push(Piece::Pb(page_number(child)?));
            }
            "fw" => check_running_head(child)?,
            "hi" => {
                if child.attribute("rendition") != Some("#g") {
                    return Err(format!(
                        "unknown <hi rendition={:?}>",
                        child.attribute("rendition")
                    ));
                }
                out.push(Piece::Open);
                inline(child, out, source, in_note)?;
                out.push(Piece::Close);
            }
            "milestone" => count_milestone(child, source)?,
            "foreign" => {
                source.stats.foreign += 1;
                inline(child, out, source, in_note)?;
            }
            "note" => {
                if in_note {
                    return Err("nested <note>".into());
                }
                if child.attribute("place") != Some("foot") {
                    return Err("<note> that is not place=foot".into());
                }
                let id = child
                    .attribute((XML_NS, "id"))
                    .map_or_else(|| format!("auto-{}", source.notes.len()), str::to_string);
                let mut pieces = Vec::new();
                inline(child, &mut pieces, source, true)?;
                if child.attribute("n").is_some_and(|mark| !mark.contains(')'))
                    && let Some(Piece::Text(first)) = pieces.first_mut()
                    && let Some(rest) = first.strip_prefix(')')
                {
                    *first = rest.to_string();
                }
                let part = NotePart {
                    pieces,
                    next: child.attribute("next").map(link_target),
                    prev: child.attribute("prev").map(link_target),
                    mark_lacks_paren: child.attribute("n").is_some_and(|m| !m.contains(')')),
                };
                let mark_lacks_paren = part.mark_lacks_paren;
                if source.notes.insert(id.clone(), part).is_some() {
                    return Err(format!("duplicate note id {id}"));
                }
                stray_paren = mark_lacks_paren;
                out.push(Piece::Note(id));
            }
            other => return Err(format!("unexpected inline <{other}>")),
        }
    }
    Ok(())
}

fn link_target(value: &str) -> String {
    value.trim_start_matches('#').to_string()
}

fn page_number(pb: Node) -> Result<u32> {
    pb.attribute("n")
        .and_then(|n| n.parse().ok())
        .ok_or_else(|| "<pb> without a numeric n".to_string())
}

fn check_running_head(fw: Node) -> Result<()> {
    if fw.attribute("type") == Some("header") {
        Ok(())
    } else {
        Err(format!("unknown <fw type={:?}>", fw.attribute("type")))
    }
}

fn count_milestone(m: Node, source: &mut Source) -> Result<()> {
    if m.attribute("rendition") != Some("#vr") {
        return Err(format!(
            "unknown <milestone rendition={:?}>",
            m.attribute("rendition")
        ));
    }
    source.stats.milestones += 1;
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    fn wrap(inner: &str) -> String {
        format!(
            "<TEI xmlns=\"http://www.tei-c.org/ns/1.0\"><text><body>{inner}</body></text></TEI>"
        )
    }

    #[test]
    fn remark_paragraphs_are_told_apart_by_rendition() {
        let xml =
            wrap("<div><head>§. 1.</head><p>Haupt</p><p rendition=\"#et #smaller\">Anm</p></div>");
        let s = parse(&xml).unwrap();
        assert!(matches!(&s.items[1], Item::Para { remark: false, .. }));
        assert!(matches!(&s.items[2], Item::Para { remark: true, .. }));
    }

    #[test]
    fn unknown_elements_are_refused() {
        assert!(parse(&wrap("<div><p><figure/></p></div>")).is_err());
        assert!(parse(&wrap("<div><p rendition=\"#x\">a</p></div>")).is_err());
    }

    #[test]
    fn notes_keep_their_links_and_milestones_are_counted() {
        let xml = wrap(
            "<div><p>a<note place=\"foot\" xml:id=\"n1\" next=\"#n2\">x<milestone unit=\"section\" rendition=\"#vr\"/></note></p>\
             <p>b<note place=\"foot\" xml:id=\"n2\" prev=\"#n1\">y</note></p></div>",
        );
        let s = parse(&xml).unwrap();
        assert_eq!(s.notes["n1"].next.as_deref(), Some("n2"));
        assert_eq!(s.notes["n2"].prev.as_deref(), Some("n1"));
        assert_eq!(s.stats.milestones, 1);
    }
}
