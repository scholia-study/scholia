//! Perseus EpiDoc TEI (`tlg0059.tlg030.perseus-grc2.xml`, Plato's *Republic*
//! in Burnet's text) → plato1's curated `md_modernized`.
//!
//! Run-once bootstrap converter (DEC-16): after this runs, the curated
//! markdown is the editorial surface and this binary is not run again. See
//! `packages/common/src/plato1/` for the TOC and filenames it targets, and
//! `packages/common/src/greek.rs` for the normalization it applies.

mod convert;
mod xml;

use std::collections::HashMap;
use std::fs;
use std::path::{Path, PathBuf};

use clap::Parser;

use common::greek::normalize_greek;
use common::plato1::{filenames, toc};
use convert::{Ctx, Piece, ReportRow};

#[derive(Parser)]
#[command(
    about = "Convert the Perseus EpiDoc TEI for Plato's Republic into plato1 curated markdown"
)]
struct Cli {
    /// Path to tlg0059.tlg030.perseus-grc2.xml
    #[arg(long)]
    tei_file: PathBuf,
    #[arg(long, default_value = "assets/plato1/curated/md_modernized")]
    out_dir: PathBuf,
    #[arg(long, default_value = "assets/plato1/derived/quotations_report.tsv")]
    report: PathBuf,
}

fn main() {
    let cli = Cli::parse();
    let result = convert_tei_file(&cli.tei_file).unwrap_or_else(|e| {
        eprintln!("error: {e}");
        std::process::exit(1);
    });

    write_files(&cli.out_dir, &result.files);
    write_report(&cli.report, &result.report);

    println!(
        "wrote {} files ({} bytes of markdown) to {}",
        result.files.len(),
        result.files.iter().map(|(_, c)| c.len()).sum::<usize>(),
        cli.out_dir.display()
    );
    println!("{} Stephanus section markers emitted", result.markers.len());
    println!(
        "{} quotations report rows written to {}",
        result.report.len(),
        cli.report.display()
    );
}

struct ConversionResult {
    /// (filename, file content), in TOC order.
    files: Vec<(String, String)>,
    report: Vec<ReportRow>,
    /// `{{{ ... }}}` values, in document order.
    markers: Vec<String>,
}

fn yaml_escape(s: &str) -> String {
    s.replace('\\', "\\\\").replace('"', "\\\"")
}

/// True if the text immediately before piece index `at` ends a sentence.
/// Markers and empty prose are transparent; a paragraph break or a verse line
/// is itself a boundary.
fn is_sentence_boundary(pieces: &[Piece], at: usize) -> bool {
    for i in (0..at).rev() {
        match &pieces[i] {
            Piece::ParaBreak | Piece::Line(_) => return true,
            Piece::Prose(s) => {
                if convert::marker_value(s).is_some() || s.trim().is_empty() {
                    continue;
                }
                let trimmed = s.trim();
                return trimmed.ends_with('.') || trimmed.ends_with(';');
            }
        }
    }
    true
}

/// Move a boundary to the nearest sentence boundary, forward by default.
fn snap_boundary(pieces: &[Piece], at: usize, backward: bool) -> usize {
    if is_sentence_boundary(pieces, at) {
        return at;
    }
    if backward {
        for i in (0..at).rev() {
            if is_sentence_boundary(pieces, i) {
                return i;
            }
        }
    } else {
        for i in (at + 1)..=pieces.len() {
            if is_sentence_boundary(pieces, i) {
                return i;
            }
        }
    }
    at
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

    let marker_positions: HashMap<&str, usize> = pieces
        .iter()
        .enumerate()
        .filter_map(|(i, p)| match p {
            Piece::Prose(s) => convert::marker_value(s).map(|v| (v, i)),
            _ => None,
        })
        .collect();

    let depth1_boundaries: Vec<&str> = toc::TOC
        .iter()
        .filter(|e| e.depth == 1)
        .map(|e| e.stephanus)
        .collect();

    let mut boundary_positions = Vec::with_capacity(depth1_boundaries.len());
    for s in &depth1_boundaries {
        let pos = *marker_positions.get(s).ok_or_else(|| {
            format!("division boundary {s} not found among the source's Stephanus markers")
        })?;
        boundary_positions.push(pos);
    }

    for (i, pos) in boundary_positions.iter_mut().enumerate() {
        if i == 0 {
            // Book I's opening division is never snapped — nothing precedes it.
            continue;
        }
        let backward = toc::BACKWARD_SNAP.contains(&depth1_boundaries[i]);
        *pos = snap_boundary(&pieces, *pos, backward);
    }

    for i in 1..boundary_positions.len() {
        if boundary_positions[i - 1] >= boundary_positions[i] {
            return Err(format!(
                "division boundaries collided after snapping: {} ({}) and {} ({})",
                depth1_boundaries[i - 1],
                boundary_positions[i - 1],
                depth1_boundaries[i],
                boundary_positions[i],
            ));
        }
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
            // Depth-0 book files carry only the front matter and heading —
            // no body, no marker (that marker belongs to the book's first
            // division instead, so every marker appears exactly once).
            front_matter
        } else {
            let start = boundary_positions[division_idx];
            let end = boundary_positions
                .get(division_idx + 1)
                .copied()
                .unwrap_or(pieces.len());
            division_idx += 1;

            let paragraphs = convert::render_paragraphs(&pieces[start..end]);
            let body = normalize_greek(&paragraphs.join("\n\n"));
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

    Ok(ConversionResult {
        files,
        report: ctx.report,
        markers,
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

#[cfg(test)]
mod tests {
    use super::*;

    const REAL_TEI: &str = "/home/filip/dev/play/canonical-greekLit/data/tlg0059/tlg030/tlg0059.tlg030.perseus-grc2.xml";

    /// Runs the real converter once and shares the result across every test
    /// in this module — cheap enough (single-digit seconds) that each test
    /// re-running it independently would just be wasted wall time, and every
    /// assertion below is read-only over the result.
    fn convert_real() -> Option<ConversionResult> {
        if !Path::new(REAL_TEI).exists() {
            eprintln!(
                "skipping plato1_tei_to_md integration tests: {REAL_TEI} not present on this machine"
            );
            return None;
        }
        Some(convert_tei_file(Path::new(REAL_TEI)).expect("conversion should succeed"))
    }

    fn parse_stephanus(s: &str) -> (u32, char) {
        let letter = s.chars().last().expect("empty stephanus value");
        let digits: String = s.chars().take_while(char::is_ascii_digit).collect();
        (
            digits
                .parse()
                .unwrap_or_else(|_| panic!("bad stephanus value {s}")),
            letter,
        )
    }

    /// The count is hardcoded on purpose rather than read from `toc::TOC`:
    /// the converter builds one file per TOC slot, so deriving the expectation
    /// would make this tautological. An independent number catches an
    /// unintended change to the division set — 10 books + 106 divisions.
    #[test]
    fn exactly_116_files_are_produced() {
        let Some(result) = convert_real() else {
            return;
        };
        assert_eq!(result.files.len(), 116);
    }

    #[test]
    fn markers_appear_exactly_once_and_strictly_increase() {
        let Some(result) = convert_real() else {
            return;
        };
        assert_eq!(result.markers.len(), 1355, "total marker count");

        let mut seen = std::collections::HashSet::new();
        for m in &result.markers {
            assert!(seen.insert(m.clone()), "marker {m} appeared more than once");
        }

        let mut prev: Option<(u32, char)> = None;
        for m in &result.markers {
            let cur = parse_stephanus(m);
            if let Some(p) = prev {
                assert!(
                    cur > p,
                    "markers not strictly increasing: {p:?} then {cur:?} ({m})"
                );
            }
            prev = Some(cur);
        }
    }

    #[test]
    fn every_marker_also_appears_exactly_once_across_the_written_files() {
        let Some(result) = convert_real() else {
            return;
        };
        let mut counts: HashMap<&str, u32> = HashMap::new();
        for (_, content) in &result.files {
            for line in content.lines() {
                let mut rest = line;
                while let Some(open) = rest.find("{{{ ") {
                    let after = &rest[open + 4..];
                    let Some(close) = after.find(" }}}") else {
                        break;
                    };
                    *counts.entry(&after[..close]).or_default() += 1;
                    rest = &after[close + 4..];
                }
            }
        }
        assert_eq!(counts.len(), 1355);
        for (value, n) in &counts {
            assert_eq!(*n, 1, "marker {value} appeared {n} times across files");
        }
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
                .expect("book file present");
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
        assert!(angle >= 1);
        assert!(square >= 1);
        assert_eq!(angle, 17, "<add> count");
        assert_eq!(square, 31, "<del> count");
    }

    #[test]
    fn verse_lines_total_92() {
        let Some(result) = convert_real() else {
            return;
        };
        let count: usize = result
            .files
            .iter()
            .map(|(_, c)| c.lines().filter(|l| l.starts_with("+ ")).count())
            .sum();
        assert_eq!(count, 92);
    }

    #[test]
    fn quotations_report_has_one_row_per_quote_or_q() {
        let Some(result) = convert_real() else {
            return;
        };
        // 87 <quote> + 43 <q> = 130.
        assert_eq!(result.report.len(), 130);
    }
}
