//! Old-spelling layer: the 1609 Quarto from EEBO-TCP text A12044 (CC0).
//!
//! `<div type="sonnet">` holds one `<l>` per line, with `<hi>` for the
//! Quarto's italicised words. We walk events (rather than serde-deserialize)
//! because `<l>` is mixed content. Output is **markdown** (italics as `*...*`)
//! — the curated layer the md→struct stage later parses.
//!
//! Sonnets are returned in **document order**, NOT keyed by the `n` attribute:
//! the Quarto misprints its own numerals (119 appears twice, 116 is skipped)
//! and EEBO preserves those faithfully. Document order is the canonical modern
//! sequence; the printed numeral is kept only to report the misprints.

use quick_xml::events::Event;
use quick_xml::reader::Reader;

pub struct Sonnet {
    /// The numeral printed in the Quarto (faithful, so possibly wrong/duplicate).
    pub printed_n: Option<u32>,
    /// One markdown line per verse line (italics as `*...*`).
    pub lines: Vec<String>,
}

/// Parse the TEI into sonnets in document order (old spelling, ſ already
/// normalised to round s by EEBO-TCP).
pub fn parse(xml: &str) -> Result<Vec<Sonnet>, Box<dyn std::error::Error>> {
    let mut reader = Reader::from_str(xml);
    let mut buf = Vec::new();

    let mut out: Vec<Sonnet> = Vec::new();
    let mut in_sonnet = false;
    let mut printed_n: Option<u32> = None;
    let mut cur_lines: Vec<String> = Vec::new();
    let mut cur_line: Option<String> = None;
    let mut note_depth: i32 = 0;

    loop {
        match reader.read_event_into(&mut buf)? {
            Event::Start(e) => match e.name().as_ref() {
                b"div" => {
                    let (typ, n) = div_attrs(&e);
                    if typ.as_deref() == Some("sonnet") {
                        in_sonnet = true;
                        printed_n = n;
                        cur_lines = Vec::new();
                    }
                }
                b"l" if in_sonnet && note_depth == 0 => cur_line = Some(String::new()),
                b"hi" => {
                    if let Some(l) = cur_line.as_mut() {
                        l.push('*');
                    }
                }
                b"note" => note_depth += 1,
                _ => {}
            },
            Event::End(e) => match e.name().as_ref() {
                b"div" if in_sonnet => {
                    in_sonnet = false;
                    let mut lines = std::mem::take(&mut cur_lines);
                    if let Some(first) = lines.first_mut() {
                        *first = normalize_dropcap(first);
                    }
                    out.push(Sonnet {
                        printed_n: printed_n.take(),
                        lines,
                    });
                }
                b"l" => {
                    if let Some(l) = cur_line.take() {
                        // Collapse intra-line whitespace: some <l> wrap across
                        // source lines around <pb>/<gap> (e.g. Sonnet 41's
                        // illegible 〈…〉), which must stay one verse line.
                        cur_lines.push(l.split_whitespace().collect::<Vec<_>>().join(" "));
                    }
                }
                b"hi" => {
                    if let Some(l) = cur_line.as_mut() {
                        l.push('*');
                    }
                }
                b"note" => note_depth = (note_depth - 1).max(0),
                _ => {}
            },
            Event::Text(e) if note_depth == 0 => {
                if let Some(l) = cur_line.as_mut() {
                    l.push_str(&e.unescape()?);
                }
            }
            Event::Eof => break,
            _ => {}
        }
        buf.clear();
    }
    Ok(out)
}

/// Read the `type` and numeric `n` attributes off a `<div>` start tag.
fn div_attrs(e: &quick_xml::events::BytesStart) -> (Option<String>, Option<u32>) {
    let mut typ = None;
    let mut n = None;
    for attr in e.attributes().flatten() {
        match attr.key.as_ref() {
            b"type" => typ = Some(String::from_utf8_lossy(&attr.value).into_owned()),
            b"n" => n = String::from_utf8_lossy(&attr.value).parse::<u32>().ok(),
            _ => {}
        }
    }
    (typ, n)
}

/// The Quarto opens each sonnet with a drop-cap + small-caps run that EEBO
/// renders as leading capitals (`FRom`, `WHen`, `MIne`). Lower-case the run
/// after its first letter so it reads naturally (`From`, `When`, `Mine`).
fn normalize_dropcap(s: &str) -> String {
    let chars: Vec<char> = s.chars().collect();
    let run = chars.iter().take_while(|c| c.is_ascii_uppercase()).count();
    if run < 2 {
        return s.to_string();
    }
    let mut out = String::new();
    out.push(chars[0]);
    for &c in &chars[1..run] {
        out.extend(c.to_lowercase());
    }
    out.extend(&chars[run..]);
    out
}
