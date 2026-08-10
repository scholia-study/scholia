//! GW 9 page-marker injection.
//!
//! The checked-in concordance `assets/hegel1/curated/gw9_markers.tsv` anchors
//! every Gesammelte-Werke Bd. 9 (Bonsiepen/Heede 1980) page start to a
//! sentence of the corpus: file position, body-paragraph ordinal, and the
//! sentence's first words in both orthographies. This pass inserts a
//! `{{ N }}` marker (the inline second-system syntax, cf. kant1's B-edition)
//! in front of the anchored words of the emitted markdown. Anchors must
//! match exactly once inside their paragraph; anything else refuses the
//! build, same contract as the ops table.

pub struct GwRow {
    pub gw: u16,
    pub file_position: u16,
    pub paragraph: usize,
    pub anchor_reviewed: String,
    pub anchor_modernized: String,
}

pub fn load(tsv: &str) -> Result<Vec<GwRow>, String> {
    let mut rows = Vec::new();
    for (i, line) in tsv.lines().enumerate().skip(1) {
        if line.trim().is_empty() {
            continue;
        }
        let f: Vec<&str> = line.split('\t').collect();
        if f.len() < 7 {
            return Err(format!(
                "gw9_markers.tsv line {}: expected 8 columns",
                i + 1
            ));
        }
        rows.push(GwRow {
            gw: f[0]
                .parse()
                .map_err(|_| format!("line {}: bad gw page", i + 1))?,
            file_position: f[1]
                .parse()
                .map_err(|_| format!("line {}: bad position", i + 1))?,
            paragraph: f[2]
                .parse()
                .map_err(|_| format!("line {}: bad paragraph", i + 1))?,
            anchor_reviewed: f[4].to_string(),
            anchor_modernized: f[5].to_string(),
        });
    }
    Ok(rows)
}

/// Insert every row's marker into its file. `reviewed` selects the anchor
/// orthography. Returns the number of markers placed.
pub fn apply(
    outputs: &mut [(String, String)],
    rows: &[GwRow],
    reviewed: bool,
) -> Result<usize, String> {
    let mut placed = 0;
    for (file, md) in outputs.iter_mut() {
        let pos: u16 = file
            .get(..3)
            .and_then(|s| s.parse().ok())
            .ok_or_else(|| format!("{file}: no NNN position prefix"))?;
        let mut file_rows: Vec<&GwRow> = rows.iter().filter(|r| r.file_position == pos).collect();
        if file_rows.is_empty() {
            continue;
        }
        // Apply back-to-front so earlier paragraph indices stay valid.
        file_rows.sort_by_key(|r| std::cmp::Reverse((r.paragraph, r.gw)));

        let mut blocks: Vec<String> = md.split("\n\n").map(str::to_string).collect();
        // Body paragraphs: every block after the `## ` heading block.
        let heading_idx = blocks
            .iter()
            .position(|b| b.starts_with("## "))
            .ok_or_else(|| format!("{file}: no heading block"))?;
        for row in file_rows {
            let bi = heading_idx + row.paragraph;
            let block = blocks.get_mut(bi).ok_or_else(|| {
                format!(
                    "{file}: gw {} targets missing paragraph {}",
                    row.gw, row.paragraph
                )
            })?;
            let anchor = if reviewed {
                &row.anchor_reviewed
            } else {
                &row.anchor_modernized
            };
            *block = insert_marker(block, anchor, row.gw)
                .map_err(|e| format!("{file} ¶{} gw {}: {e}", row.paragraph, row.gw))?;
            placed += 1;
        }
        *md = blocks.join("\n\n");
    }
    if placed != rows.len() {
        return Err(format!("placed {placed} of {} gw markers", rows.len()));
    }
    Ok(placed)
}

/// Plain-text projection of a markdown paragraph: page markers (both `{{{ }}}`
/// and `{{ }}`, plus one following space), emphasis syntax (`_`, `*`) and
/// `<i>`/`</i>` tags are dropped; newlines count as spaces. Returns the
/// projection and a map from projection byte index to source byte index.
fn projection(md: &str) -> (String, Vec<usize>) {
    let bytes = md.as_bytes();
    let mut plain = String::new();
    let mut map = Vec::new();
    let mut i = 0;
    while i < bytes.len() {
        let rest = &md[i..];
        if rest.starts_with("{{")
            && let Some(end) = rest.find("}}")
        {
            let mut skip = end + 2;
            while md[i + skip..].starts_with('}') {
                skip += 1;
            }
            if md[i + skip..].starts_with(' ') {
                skip += 1;
            }
            i += skip;
            continue;
        }
        if rest.starts_with("<i>") {
            i += 3;
            continue;
        }
        if rest.starts_with("</i>") {
            i += 4;
            continue;
        }
        let c = rest.chars().next().unwrap();
        if c == '_' || c == '*' {
            i += c.len_utf8();
            continue;
        }
        if c == '\n' {
            plain.push(' ');
            map.push(i);
            i += 1;
            continue;
        }
        map.push(i);
        for _ in 0..c.len_utf8().saturating_sub(1) {
            map.push(i);
        }
        plain.push(c);
        i += c.len_utf8();
    }
    (plain, map)
}

fn insert_marker(block: &str, anchor: &str, gw: u16) -> Result<String, String> {
    let (plain, map) = projection(block);
    let needle = anchor.split_whitespace().collect::<Vec<_>>().join(" ");
    let hits: Vec<usize> = plain.match_indices(&needle).map(|(i, _)| i).collect();
    match hits.as_slice() {
        [at] => {
            let mut src = map[*at];
            // Step back over emphasis-open syntax so the marker sits outside
            // the span (`{{ N }} _Wort_`, never `_{{ N }} Wort_`).
            loop {
                if src > 0 && matches!(block.as_bytes()[src - 1], b'_' | b'*') {
                    src -= 1;
                } else if block[..src].ends_with("<i>") {
                    src -= 3;
                } else {
                    break;
                }
            }
            Ok(format!("{}{{{{ {gw} }}}} {}", &block[..src], &block[src..]))
        }
        [] => Err(format!("anchor not found: {needle:?}")),
        _ => Err(format!(
            "anchor ambiguous ({} hits): {needle:?}",
            hits.len()
        )),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn inserts_before_anchor_skipping_markup() {
        let block = "Die {{{ 12 }}} _Erfahrung_, welche das Bewusstsein macht.";
        let out = insert_marker(block, "Erfahrung, welche", 63).unwrap();
        assert_eq!(
            out,
            "Die {{{ 12 }}} {{ 63 }} _Erfahrung_, welche das Bewusstsein macht."
        );
    }

    #[test]
    fn anchor_at_block_start() {
        let out = insert_marker("Das Wissen, welches zuerst.", "Das Wissen,", 9).unwrap();
        assert!(out.starts_with("{{ 9 }} Das Wissen,"));
    }

    #[test]
    fn intraword_antiqua_is_transparent() {
        let block = "so ist das <i>An</i>sichsein gesetzt.";
        let out = insert_marker(block, "das Ansichsein gesetzt.", 100).unwrap();
        assert!(out.contains("{{ 100 }} das <i>An</i>sichsein"));
    }
}
