//! Perseus EpiDoc TEI (`tlg0059.tlg023.perseus-grc2.xml`, Plato's *Gorgias* in
//! Burnet's text) → plato2's curated `md_modernized`.
//!
//! Run-once bootstrap converter: after this runs, the curated markdown is the
//! editorial surface and this binary is not run again. See
//! `packages/common/src/plato2/` for the TOC and filenames it targets, and
//! `packages/common/src/greek.rs` for the normalization it applies.
//!
//! Unlike the *Republic*, the *Gorgias* is staged: its block structure comes
//! from the 1,107 speech turns, not from reconstructed paragraphing. A
//! division never opens mid-speech, so every division boundary lands exactly
//! on a genuine (non-`rend="merge"`) turn's `Piece::TurnStart` — no sentence
//! snapping is needed the way plato1 needs it for its prose divisions.

mod convert;
mod xml;

use std::collections::BTreeMap;
use std::fs;
use std::path::{Path, PathBuf};

use clap::Parser;

use common::greek::normalize_greek;
use common::plato2::{filenames, toc};
use convert::{Ctx, ReportRow};

#[derive(Parser)]
#[command(
    about = "Convert the Perseus EpiDoc TEI for Plato's Gorgias into plato2 curated markdown"
)]
struct Cli {
    /// Path to tlg0059.tlg023.perseus-grc2.xml
    #[arg(
        long,
        default_value = "assets/plato2/raw/tlg0059.tlg023.perseus-grc2.xml"
    )]
    tei_file: PathBuf,
    #[arg(long, default_value = "assets/plato2/curated/md_modernized")]
    out_dir: PathBuf,
    #[arg(long, default_value = "assets/plato2/derived/quotations_report.tsv")]
    report: PathBuf,
    /// Report the source's shape and exit without writing markdown. The element
    /// set is verified either way — `xml::parse_body` refuses anything outside
    /// it.
    #[arg(long)]
    survey: bool,
}

fn main() {
    let cli = Cli::parse();
    if cli.survey {
        if let Err(e) = survey(&cli.tei_file) {
            eprintln!("error: {e}");
            std::process::exit(1);
        }
        return;
    }

    let result = convert_tei_file(&cli.tei_file).unwrap_or_else(|e| {
        eprintln!("error: {e}");
        std::process::exit(1);
    });

    write_files(&cli.out_dir, &result.files);
    write_report(&cli.report, &result.report);
    print_self_check(&result);
}

struct ConversionResult {
    /// (filename, file content), in TOC order.
    files: Vec<(String, String)>,
    report: Vec<ReportRow>,
    /// `{{{ ... }}}` values, in document order.
    markers: Vec<String>,
    /// (speaker, said_index, section) for every genuine turn, in document
    /// order.
    turns: Vec<(String, usize, Option<String>)>,
    /// The `said_index` each division resolved to, in TOC order — the split
    /// the run actually made, for the caller to check.
    division_opening_said_indices: Vec<usize>,
}

fn yaml_escape(s: &str) -> String {
    s.replace('\\', "\\\\").replace('"', "\\\"")
}

fn convert_tei_file(tei_path: &Path) -> Result<ConversionResult, String> {
    let xml = fs::read_to_string(tei_path)
        .map_err(|e| format!("failed to read TEI file {}: {e}", tei_path.display()))?;
    convert_tei_str(&xml)
}

fn convert_tei_str(xml: &str) -> Result<ConversionResult, String> {
    let body_span = xml::extract_body(xml)?;
    let nodes = xml::parse_body(body_span)?;

    let mut ctx = Ctx::default();
    let pieces = convert::render_block(&nodes, &mut ctx);
    let markers = convert::find_markers(&pieces);
    let turns = convert::find_turns(&pieces);

    let depth1_boundaries: Vec<&str> = toc::TOC
        .iter()
        .filter(|e| e.depth == 1)
        .map(|e| e.stephanus)
        .collect();

    // A division opens on the FIRST genuine turn whose section-in-force is its
    // anchor. 14 of the 19 anchors have several turns in the same section, so
    // this is a real rule and not an accident of the table: the anchors were
    // chosen so that the first such turn is the intended seam in every case.
    // A division that opened later would need its own anchor, since nothing in
    // the source distinguishes the second turn of a section from the first.
    let mut boundary_positions = Vec::with_capacity(depth1_boundaries.len());
    let mut division_opening_said_indices = Vec::with_capacity(depth1_boundaries.len());
    let mut searched_from = 0usize;
    for stephanus in &depth1_boundaries {
        let offset = turns[searched_from..]
            .iter()
            .position(|&(_, _, _, section)| section == Some(*stephanus))
            .ok_or_else(|| format!("no speech turn opens division {stephanus}"))?;
        let (piece_idx, _, said_index, _) = turns[searched_from + offset];
        boundary_positions.push(piece_idx);
        division_opening_said_indices.push(said_index);
        searched_from += offset + 1;
    }

    let all_filenames = filenames::all_filenames();
    let mut files = Vec::with_capacity(all_filenames.len());
    let mut division_idx = 0usize;

    for (flat_index, filename) in &all_filenames {
        let entry = &toc::TOC[*flat_index];
        let position = filenames::position_number(*flat_index);
        let front_matter = format!(
            "---\nposition: {}\nlabel: \"{}\"\ndepth: {}\npage_stephanus: \"{}\"\n---\n\n## {}\n",
            position,
            yaml_escape(entry.label),
            entry.depth,
            entry.stephanus,
            entry.label,
        );

        let content = if entry.depth == 0 {
            // Depth-0 conversation files carry only the front matter and
            // heading — no body, no marker (that marker belongs to the
            // conversation's first division instead, so every marker
            // appears exactly once).
            front_matter
        } else {
            let start = boundary_positions[division_idx];
            let end = boundary_positions
                .get(division_idx + 1)
                .copied()
                .unwrap_or(pieces.len());
            division_idx += 1;

            let body = normalize_greek(&convert::render_speeches(&pieces[start..end]));
            format!("{front_matter}\n{body}\n")
        };

        files.push((filename.clone(), content));
    }

    if division_idx != depth1_boundaries.len() {
        return Err(format!(
            "internal error: consumed {division_idx} of {} division boundaries",
            depth1_boundaries.len()
        ));
    }

    let turns = turns
        .into_iter()
        .map(|(_, speaker, said_index, section)| {
            (speaker.to_string(), said_index, section.map(str::to_string))
        })
        .collect();

    Ok(ConversionResult {
        files,
        report: ctx.report,
        markers,
        turns,
        division_opening_said_indices,
    })
}

fn write_files(out_dir: &Path, files: &[(String, String)]) {
    fs::create_dir_all(out_dir)
        .unwrap_or_else(|e| panic!("failed to create {}: {e}", out_dir.display()));
    for (name, content) in files {
        let path = out_dir.join(name);
        fs::write(&path, content)
            .unwrap_or_else(|e| panic!("failed to write {}: {e}", path.display()));
    }
}

fn write_report(path: &Path, report: &[ReportRow]) {
    if let Some(parent) = path.parent() {
        fs::create_dir_all(parent)
            .unwrap_or_else(|e| panic!("failed to create {}: {e}", parent.display()));
    }
    let mut out = String::from("stephanus\ttype\tperseus_bibl\tgreek_text\n");
    for row in report {
        out.push_str(&format!(
            "{}\t{}\t{}\t{}\n",
            row.stephanus.replace('\t', " "),
            row.kind.as_str(),
            row.perseus_bibl.replace('\t', " "),
            row.greek_text.replace('\t', " "),
        ));
    }
    fs::write(path, out).unwrap_or_else(|e| panic!("failed to write {}: {e}", path.display()));
}

/// Rough word count over a division file's dialogue text only: front matter,
/// the `## Label` heading, `@ Speaker` lines, `{{{ ... }}}` markers and `| `
/// verse prefixes stripped first, then a plain whitespace split. Exact enough
/// for a self-check figure, not a linguistic tokenizer.
fn count_words(content: &str) -> usize {
    let Some((_, after_front_matter)) = content.split_once("---\n\n") else {
        return 0;
    };
    let body = after_front_matter
        .split_once('\n')
        .map(|(_, rest)| rest.trim_start_matches('\n'))
        .unwrap_or("");
    body.lines()
        .filter(|l| !l.starts_with("@ "))
        .map(|l| l.strip_prefix("| ").unwrap_or(l))
        .map(|l| convert::marker_free(l).split_whitespace().count())
        .sum()
}

fn print_self_check(result: &ConversionResult) {
    let all_filenames = filenames::all_filenames();
    println!("{} files written", result.files.len());
    let names_match = result
        .files
        .iter()
        .map(|(n, _)| n.as_str())
        .eq(all_filenames.iter().map(|(_, n)| n.as_str()));
    println!("filenames match all_filenames(): {names_match}");

    println!("{} Stephanus section markers total", result.markers.len());
    let mut seen = std::collections::HashSet::new();
    let unique = result.markers.iter().all(|m| seen.insert(m.clone()));
    println!("all markers unique: {unique}");
    let mut ascending = true;
    for w in result.markers.windows(2) {
        if toc::stephanus_key(&w[0]) >= toc::stephanus_key(&w[1]) {
            ascending = false;
        }
    }
    println!("markers strictly ascending: {ascending}");

    let depth0_marker_free = filenames::all_filenames().iter().all(|(idx, name)| {
        toc::TOC[*idx].depth != 0
            || !result
                .files
                .iter()
                .find(|(n, _)| n == name)
                .map(|(_, c)| c.contains("{{{"))
                .unwrap_or(false)
    });
    println!("depth-0 files carry no marker: {depth0_marker_free}");

    println!("{} '@ ' speaker lines", result.turns.len());
    let mut by_speaker: BTreeMap<&str, usize> = BTreeMap::new();
    for (speaker, _, _) in &result.turns {
        *by_speaker.entry(speaker.as_str()).or_default() += 1;
    }
    for (speaker, n) in &by_speaker {
        println!("  {speaker}: {n}");
    }

    let has_u02bc = result.files.iter().any(|(_, c)| c.contains('\u{02BC}'));
    println!("U+02BC present: {has_u02bc}");

    let total_words: usize = result.files.iter().map(|(_, c)| count_words(c)).sum();
    println!("total Greek word count (approx): {total_words}");

    let braces_balanced = result
        .files
        .iter()
        .all(|(_, c)| c.matches('{').count() == c.matches('}').count());
    println!("braces balanced in every file: {braces_balanced}");

    println!("{} quotations report rows written", result.report.len());
    println!(
        "divisions opened on turns: {:?}",
        result.division_opening_said_indices
    );
}

fn survey(tei_path: &Path) -> Result<(), String> {
    let raw = fs::read_to_string(tei_path)
        .map_err(|e| format!("failed to read TEI file {}: {e}", tei_path.display()))?;
    let nodes = xml::parse_body(xml::extract_body(&raw)?)?;

    let mut turns = 0usize;
    let mut speakers: BTreeMap<String, usize> = BTreeMap::new();
    let mut sections: Vec<String> = Vec::new();
    let mut pages = 0usize;
    walk(&nodes, &mut turns, &mut speakers, &mut sections, &mut pages);

    println!("speech turns:        {turns}");
    for (who, n) in &speakers {
        println!("  {who}: {n}");
    }
    println!("section milestones:  {}", sections.len());
    println!("page milestones:     {pages} (dropped as redundant)");
    println!(
        "Stephanus span:      {} … {}",
        sections.first().map(String::as_str).unwrap_or("-"),
        sections.last().map(String::as_str).unwrap_or("-")
    );

    let mut seen = std::collections::HashSet::new();
    let dupes: Vec<&String> = sections.iter().filter(|s| !seen.insert(*s)).collect();
    if !dupes.is_empty() {
        return Err(format!("duplicate section milestones: {dupes:?}"));
    }
    Ok(())
}

fn walk(
    nodes: &[xml::Node],
    turns: &mut usize,
    speakers: &mut BTreeMap<String, usize>,
    sections: &mut Vec<String>,
    pages: &mut usize,
) {
    for node in nodes {
        let xml::Node::Element {
            name,
            attrs,
            children,
        } = node
        else {
            continue;
        };
        match name.as_str() {
            "said" => {
                *turns += 1;
                let who = attrs
                    .get("who")
                    .map(|w| w.trim_start_matches('#').to_string())
                    .unwrap_or_else(|| "?".into());
                *speakers.entry(who).or_default() += 1;
            }
            "milestone" => match attrs.get("unit").map(String::as_str) {
                Some("section") => sections.push(attrs.get("n").cloned().unwrap_or_default()),
                Some("page") => *pages += 1,
                _ => {}
            },
            _ => {}
        }
        walk(children, turns, speakers, sections, pages);
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    const REAL_TEI: &str = concat!(
        env!("CARGO_MANIFEST_DIR"),
        "/../../assets/plato2/raw/tlg0059.tlg023.perseus-grc2.xml"
    );

    /// Runs the real converter once and shares the result across every test
    /// in this module — cheap enough that each test re-running it
    /// independently would just be wasted wall time.
    fn convert_real() -> Option<ConversionResult> {
        if !Path::new(REAL_TEI).exists() {
            eprintln!(
                "skipping plato2_tei_to_md integration tests: {REAL_TEI} not present on this machine"
            );
            return None;
        }
        Some(convert_tei_file(Path::new(REAL_TEI)).expect("conversion should succeed"))
    }

    #[test]
    fn exactly_22_files_named_by_all_filenames() {
        let Some(result) = convert_real() else {
            return;
        };
        let expected: Vec<String> = filenames::all_filenames()
            .into_iter()
            .map(|(_, n)| n)
            .collect();
        let actual: Vec<String> = result.files.iter().map(|(n, _)| n.clone()).collect();
        assert_eq!(actual, expected);
        assert_eq!(actual.len(), 22);
    }

    #[test]
    fn markers_total_404_unique_and_ascending() {
        let Some(result) = convert_real() else {
            return;
        };
        assert_eq!(result.markers.len(), 404);
        let mut seen = std::collections::HashSet::new();
        for m in &result.markers {
            assert!(seen.insert(m.clone()), "marker {m} appeared more than once");
        }
        for w in result.markers.windows(2) {
            assert!(
                toc::stephanus_key(&w[0]) < toc::stephanus_key(&w[1]),
                "markers not strictly increasing: {} then {}",
                w[0],
                w[1]
            );
        }
    }

    #[test]
    fn every_marker_appears_exactly_once_across_the_written_files() {
        let Some(result) = convert_real() else {
            return;
        };
        let mut counts: std::collections::HashMap<String, u32> = std::collections::HashMap::new();
        for (_, content) in &result.files {
            let mut rest = content.as_str();
            while let Some(open) = rest.find("{{{ ") {
                let after = &rest[open + 4..];
                let Some(close) = after.find(" }}}") else {
                    break;
                };
                *counts.entry(after[..close].to_string()).or_default() += 1;
                rest = &after[close + 4..];
            }
        }
        assert_eq!(counts.len(), 404);
        for (value, n) in &counts {
            assert_eq!(*n, 1, "marker {value} appeared {n} times across files");
        }
    }

    #[test]
    fn depth_zero_files_carry_no_marker_and_no_body() {
        let Some(result) = convert_real() else {
            return;
        };
        for (flat_index, filename) in filenames::all_filenames() {
            let entry = &toc::TOC[flat_index];
            if entry.depth != 0 {
                continue;
            }
            let (_, content) = result
                .files
                .iter()
                .find(|(n, _)| n == &filename)
                .expect("conversation file present");
            assert!(
                !content.contains("{{{"),
                "{filename} (depth 0) contains a marker"
            );
            let body_after_heading = content
                .split_once(&format!("## {}\n", entry.label))
                .map(|x| x.1)
                .unwrap_or("");
            assert!(
                body_after_heading.trim().is_empty(),
                "{filename} (depth 0) has body content: {body_after_heading:?}"
            );
        }
    }

    #[test]
    fn speaker_lines_total_1081_with_the_expected_distribution() {
        let Some(result) = convert_real() else {
            return;
        };
        assert_eq!(result.turns.len(), 1081);
        let mut by_speaker: BTreeMap<&str, usize> = BTreeMap::new();
        for (speaker, _, _) in &result.turns {
            *by_speaker.entry(speaker.as_str()).or_default() += 1;
        }
        let expected: BTreeMap<&str, usize> = [
            ("Σωκράτης", 527),
            ("Καλλίκλης", 236),
            ("Πῶλος", 207),
            ("Γοργίας", 97),
            ("Χαιρεφῶν", 14),
        ]
        .into_iter()
        .collect();
        assert_eq!(by_speaker, expected);
    }

    #[test]
    fn no_modifier_letter_apostrophe_survives() {
        let Some(result) = convert_real() else {
            return;
        };
        for (name, content) in &result.files {
            assert!(
                !content.contains('\u{02BC}'),
                "{name} still contains U+02BC (elision not unified)"
            );
        }
    }

    #[test]
    fn editorial_bracket_shapes_match_source_counts() {
        let Some(result) = convert_real() else {
            return;
        };
        let mut angle = 0usize;
        let mut square = 0usize;
        for (_, content) in &result.files {
            angle += content.matches('⟨').count();
            assert_eq!(content.matches('⟨').count(), content.matches('⟩').count());
            square += content.matches('[').count();
            assert_eq!(content.matches('[').count(), content.matches(']').count());
        }
        assert_eq!(angle, 4, "<add> count");
        assert_eq!(square, 17, "<del> count");
    }

    #[test]
    fn verse_lines_total_10() {
        let Some(result) = convert_real() else {
            return;
        };
        let count: usize = result
            .files
            .iter()
            .map(|(_, c)| c.lines().filter(|l| l.starts_with("| ")).count())
            .sum();
        assert_eq!(count, 10);
    }

    #[test]
    fn quotations_report_has_one_row_per_quote_or_q() {
        let Some(result) = convert_real() else {
            return;
        };
        // 15 <quote> + 33 <q> = 48.
        assert_eq!(result.report.len(), 48);
    }

    /// The rule is "first genuine turn whose section-in-force is the anchor",
    /// and it has to be a rule rather than a lookup table because 14 of the 19
    /// anchors sit in a section carrying several turns. This pins the turns it
    /// resolves to, so a change in the source or the TOC that moves a division
    /// boundary fails here rather than silently re-splitting the corpus.
    #[test]
    fn divisions_open_on_the_first_turn_of_their_anchor_section() {
        let Some(result) = convert_real() else {
            return;
        };
        assert_eq!(
            result.division_opening_said_indices,
            vec![
                0, 46, 99, 153, 211, 273, 359, 401, 481, 624, 634, 690, 793, 892, 969, 984, 1029,
                1069, 1102,
            ]
        );
    }

    #[test]
    fn division_boundaries_strictly_increase() {
        let Some(result) = convert_real() else {
            return;
        };
        let idx = &result.division_opening_said_indices;
        assert!(idx.windows(2).all(|w| w[0] < w[1]));
        assert_eq!(idx.len(), 19);
    }

    #[test]
    fn no_file_contains_a_bare_double_brace() {
        let Some(result) = convert_real() else {
            return;
        };
        for (name, content) in &result.files {
            let bytes = content.as_bytes();
            let mut i = 0;
            while let Some(rel) = content[i..].find("{{") {
                let idx = i + rel;
                assert!(
                    bytes[idx..].starts_with(b"{{{"),
                    "{name} contains a bare {{{{ not part of a {{{{{{ marker at byte {idx}"
                );
                i = idx + 2;
            }
        }
    }
}
