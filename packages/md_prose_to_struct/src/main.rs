//! Curated annotated-prose markdown → struct JSON. `--corpus kant1|kant3`
//! selects the config; `--translation` builds the single-layer English edition
//! instead of the two-layer German source. Output is consumed by
//! `struct_to_db`.

use std::fs;
use std::io::Write;
use std::path::Path;

use clap::Parser;

use common::sentences::{split_sentences, split_sentences_en};
use md_prose_to_struct::corpus::{self, Corpus, FlatEntry};
use md_prose_to_struct::html::{md_to_html, md_to_plain};
use md_prose_to_struct::parse::{self, ParsedBlockType, parse_blocks};
use md_prose_to_struct::structure::{ParsedFile, build_output};
use text_struct::parse::{FrontMatter, parse_front_matter, scan_md_files};

#[derive(Parser)]
#[command(about = "Parse curated annotated-prose markdown into DB-ready JSON")]
struct Cli {
    /// Which corpus to parse: kant1 | kant3
    #[arg(long)]
    corpus: String,
    /// Build the single-layer translation edition (md_modernized_translated)
    /// instead of the two-layer source.
    #[arg(long)]
    translation: bool,
    /// Override the modernized layer dir (defaults per corpus).
    #[arg(long)]
    modernized_dir: Option<String>,
    /// Override the reviewed layer dir (defaults per corpus).
    #[arg(long)]
    reviewed_dir: Option<String>,
    /// Override the translated layer dir (defaults per corpus).
    #[arg(long)]
    translated_dir: Option<String>,
    /// Override the output path (defaults per corpus; - for stdout).
    #[arg(long)]
    output_file: Option<String>,
}

fn main() {
    let cli = Cli::parse();
    let mut corpus = corpus::by_name(&cli.corpus, cli.translation)
        .unwrap_or_else(|| panic!("unknown corpus {:?} (expected kant1 | kant3)", cli.corpus));
    if let Some(d) = cli.modernized_dir {
        corpus.modernized_dir = d;
    }
    if let Some(d) = cli.reviewed_dir {
        corpus.reviewed_dir = d;
    }
    if let Some(d) = cli.translated_dir {
        corpus.translated_dir = d;
    }
    if let Some(f) = cli.output_file {
        corpus.output_file = f;
    }

    let parsed_files = if cli.translation {
        collect_translation_files(&corpus)
    } else {
        collect_source_files(&corpus)
    };

    let output = build_output(&corpus, cli.translation, &parsed_files);
    print_summary(&corpus, &output);
    write_output(&corpus.output_file, &output);
}

/// Scan a curated dir, panicking on IO errors (curated layers must exist).
fn scan_dir(dir: &Path) -> Vec<String> {
    scan_md_files(dir).unwrap_or_else(|e| panic!("Cannot read {}: {e}", dir.display()))
}

/// Validate a file's front matter against its TOC entry. `check_label` is off
/// for files whose language differs from the TOC's (German source files during
/// a translation build; translated files without an English TOC).
fn validate_front_matter(
    fm: &FrontMatter,
    file_label: &str,
    flat_index: usize,
    entry: &FlatEntry,
    position_number: fn(usize) -> usize,
    check_label: bool,
) {
    let (_, aa_page, depth, label, _) = *entry;
    if fm.position as usize != position_number(flat_index) {
        panic!(
            "{}: position mismatch: file has {}, expected {}",
            file_label,
            fm.position,
            position_number(flat_index)
        );
    }
    if check_label && fm.label != label {
        panic!(
            "{}: label mismatch: file has {:?}, expected {:?}",
            file_label, fm.label, label
        );
    }
    if fm.depth as u16 != depth {
        panic!(
            "{}: depth mismatch: file has {}, expected {}",
            file_label, fm.depth, depth
        );
    }
    let file_aa = fm
        .aa_page
        .unwrap_or_else(|| panic!("{file_label}: missing aa_page in front matter"));
    if file_aa != aa_page {
        panic!(
            "{}: aa_page mismatch: file has {}, expected {}",
            file_label, file_aa, aa_page
        );
    }
}

/// Parse one curated file, validating front matter against `entry`.
fn parse_file(
    dir: &Path,
    filename: &str,
    flat_index: usize,
    entry: &FlatEntry,
    position_number: fn(usize) -> usize,
    check_label: bool,
) -> (Vec<parse::ParsedBlock>, String) {
    let file_path = dir.join(filename);
    let content = fs::read_to_string(&file_path)
        .unwrap_or_else(|e| panic!("Cannot read {}: {e}", file_path.display()));
    let (fm, body) = parse_front_matter(&content)
        .unwrap_or_else(|| panic!("No front matter in {}", file_path.display()));
    validate_front_matter(
        &fm,
        &file_path.display().to_string(),
        flat_index,
        entry,
        position_number,
        check_label,
    );
    (parse_blocks(body), fm.label)
}

// --- Source mode: two-layer pairing --------------------------------------

fn collect_source_files(corpus: &Corpus) -> Vec<ParsedFile> {
    let modernized_dir = Path::new(&corpus.modernized_dir);
    let reviewed_dir = Path::new(&corpus.reviewed_dir);

    let modernized_entries = scan_dir(modernized_dir);
    let reviewed_entries = scan_dir(reviewed_dir);

    let mut parsed_files: Vec<ParsedFile> = Vec::new();

    for &(flat_index, ref expected_name) in &corpus.filenames {
        let in_modernized = modernized_entries.contains(expected_name);
        let in_reviewed = reviewed_entries.contains(expected_name);

        if !in_modernized && !in_reviewed {
            eprintln!(
                "info: skipping {} (file not found in either dir)",
                expected_name
            );
            continue;
        }
        if !in_modernized {
            panic!(
                "{}: found in reviewed dir but missing from modernized dir",
                expected_name
            );
        }
        if !in_reviewed {
            panic!(
                "{}: found in modernized dir but missing from reviewed dir",
                expected_name
            );
        }

        let (blocks, _) = parse_file(
            modernized_dir,
            expected_name,
            flat_index,
            &corpus.toc_modernized[flat_index],
            corpus.position_number,
            true,
        );
        let (original_blocks, _) = parse_file(
            reviewed_dir,
            expected_name,
            flat_index,
            &corpus.toc_reviewed[flat_index],
            corpus.position_number,
            true,
        );

        if blocks.len() != original_blocks.len() {
            eprintln!("Error: block count mismatch in {}", expected_name);
            eprintln!("MODERNIZED blocks (len {}):", blocks.len());
            for (idx, b) in blocks.iter().enumerate() {
                eprintln!("  Block {}: {:?}", idx, b.block_type);
                eprintln!(
                    "    Content: {:?}",
                    b.text.chars().take(60).collect::<String>()
                );
            }
            eprintln!("REVIEWED blocks (len {}):", original_blocks.len());
            for (idx, b) in original_blocks.iter().enumerate() {
                eprintln!("  Block {}: {:?}", idx, b.block_type);
                eprintln!(
                    "    Content: {:?}",
                    b.text.chars().take(60).collect::<String>()
                );
            }
            panic!(
                "{}: block count mismatch: modernized has {}, reviewed has {}",
                expected_name,
                blocks.len(),
                original_blocks.len()
            );
        }

        parsed_files.push(ParsedFile {
            flat_index,
            blocks,
            original_blocks: Some(original_blocks),
            english_label: None,
        });
    }

    check_unexpected(&modernized_entries, &corpus.filenames, "modernized");
    check_unexpected(&reviewed_entries, &corpus.filenames, "reviewed");

    eprintln!(
        "Parsed {} files with {} total blocks",
        parsed_files.len(),
        parsed_files.iter().map(|f| f.blocks.len()).sum::<usize>()
    );

    parsed_files
}

// --- Translation mode: single layer, parity-validated against the source --

fn collect_translation_files(corpus: &Corpus) -> Vec<ParsedFile> {
    let translation_dir = Path::new(&corpus.translated_dir);
    let source_dir = Path::new(&corpus.modernized_dir);

    // With an English TOC (kant1): English filenames + labels validated against
    // it, German counterparts found by flat index. Without one (kant3): German
    // filenames are shared and the file's front-matter label IS the authority.
    let (expected_files, en_entries): (&[(usize, String)], Option<&[FlatEntry]>) =
        match &corpus.toc_en {
            Some(en) => (&en.filenames, Some(&en.entries)),
            None => (&corpus.filenames, None),
        };

    let translation_entries = scan_dir(translation_dir);
    let mut parsed_files: Vec<ParsedFile> = Vec::new();
    let mut mismatches: Vec<String> = Vec::new();

    for &(flat_index, ref filename) in expected_files {
        if !translation_entries.contains(filename) {
            continue;
        }

        let de_filename = &corpus.filenames[flat_index].1;
        let de_file_path = source_dir.join(de_filename);
        if !de_file_path.exists() {
            panic!("German source file {de_filename} not found for translation {filename}");
        }

        // Translated file: structural validation always; label validation only
        // against an English TOC.
        let (en_blocks, english_label) = match en_entries {
            Some(entries) => parse_file(
                translation_dir,
                filename,
                flat_index,
                &entries[flat_index],
                corpus.position_number,
                true,
            ),
            None => parse_file(
                translation_dir,
                filename,
                flat_index,
                &corpus.toc_modernized[flat_index],
                corpus.position_number,
                false,
            ),
        };
        // German source file: structural fields only (label differs).
        let (de_blocks, _) = parse_file(
            source_dir,
            de_filename,
            flat_index,
            &corpus.toc_reviewed[flat_index],
            corpus.position_number,
            false,
        );

        // Block-count parity (excluding footnotes).
        let en_content_count = content_block_count(&en_blocks);
        let de_content_count = content_block_count(&de_blocks);
        if en_content_count != de_content_count {
            mismatches.push(format!(
                "{filename}: BLOCK count mismatch — EN {en_content_count} vs DE {de_content_count} (file skipped)"
            ));
            continue;
        }

        // Per-block sentence-count parity.
        let en_content_blocks: Vec<&parse::ParsedBlock> = en_blocks
            .iter()
            .filter(|b| !matches!(&b.block_type, ParsedBlockType::Footnote { .. }))
            .collect();
        let de_content_blocks: Vec<&parse::ParsedBlock> = de_blocks
            .iter()
            .filter(|b| !matches!(&b.block_type, ParsedBlockType::Footnote { .. }))
            .collect();

        for (block_pos, (en_block, de_block)) in en_content_blocks
            .iter()
            .zip(de_content_blocks.iter())
            .enumerate()
        {
            // Figures are verbatim HTML with a single anchor sentence, not
            // markdown prose — sentence-splitting them is meaningless. Require
            // that figures align with figures and move on.
            let en_is_figure = matches!(en_block.block_type, ParsedBlockType::Figure);
            let de_is_figure = matches!(de_block.block_type, ParsedBlockType::Figure);
            if en_is_figure || de_is_figure {
                if en_is_figure != de_is_figure {
                    mismatches.push(format!(
                        "{filename}: figure/non-figure misalignment at block {block_pos}"
                    ));
                }
                continue;
            }

            // Separators are contentless thematic breaks; require alignment.
            let en_is_sep = matches!(en_block.block_type, ParsedBlockType::Separator { .. });
            let de_is_sep = matches!(de_block.block_type, ParsedBlockType::Separator { .. });
            if en_is_sep || de_is_sep {
                if en_is_sep != de_is_sep {
                    mismatches.push(format!(
                        "{filename}: separator misalignment at block {block_pos}"
                    ));
                }
                continue;
            }

            // Page markers ride in block.text; strip them off the rendered
            // text before counting so a marker at a sentence boundary can't
            // suppress the split (it would leave `. {{ N }} Capital`).
            let (en_plain, _) = parse::strip_markers(&md_to_plain(&en_block.text));
            let (en_html, _) = parse::strip_markers(&md_to_html(&en_block.text));
            let en_sentences = split_sentences_en(&en_plain, &en_html);

            let (de_plain, _) = parse::strip_markers(&md_to_plain(&de_block.text));
            let (de_html, _) = parse::strip_markers(&md_to_html(&de_block.text));
            let de_sentences = split_sentences(&de_plain, &de_html);

            if en_sentences.len() != de_sentences.len() {
                let de_list = de_sentences
                    .iter()
                    .enumerate()
                    .map(|(i, (t, _))| format!("      D{}. {t}", i + 1))
                    .collect::<Vec<_>>()
                    .join("\n");
                let en_list = en_sentences
                    .iter()
                    .enumerate()
                    .map(|(i, (t, _))| format!("      E{}. {t}", i + 1))
                    .collect::<Vec<_>>()
                    .join("\n");
                mismatches.push(format!(
                    "{filename}  block {block_pos}: DE {} vs EN {}\n{de_list}\n      ---\n{en_list}",
                    de_sentences.len(),
                    en_sentences.len(),
                ));
            }
        }

        parsed_files.push(ParsedFile {
            flat_index,
            blocks: en_blocks,
            original_blocks: None,
            english_label: en_entries.is_none().then_some(english_label),
        });
    }

    check_unexpected(&translation_entries, expected_files, "translation");

    if !mismatches.is_empty() {
        eprintln!(
            "=== {} parity mismatch(es) — 1:1 violations to fix ===",
            mismatches.len()
        );
        for m in &mismatches {
            eprintln!("  {m}");
        }
        eprintln!(
            "\nFix each by splitting/merging the English so its sentence count matches the German, then re-run."
        );
        std::process::exit(1);
    }

    eprintln!(
        "Verified + parsed {} translation files ({} total blocks); 1:1 parity holds",
        parsed_files.len(),
        parsed_files.iter().map(|f| f.blocks.len()).sum::<usize>()
    );

    parsed_files
}

/// Count non-footnote blocks.
fn content_block_count(blocks: &[parse::ParsedBlock]) -> usize {
    blocks
        .iter()
        .filter(|b| !matches!(&b.block_type, ParsedBlockType::Footnote { .. }))
        .count()
}

fn check_unexpected(found: &[String], expected: &[(usize, String)], dir_label: &str) {
    let expected_names: Vec<&String> = expected.iter().map(|(_, n)| n).collect();
    for name in found {
        if !expected_names.contains(&name) {
            panic!("Unexpected file in {dir_label} directory: {name}");
        }
    }
}

// --- Summary + output ------------------------------------------------------

fn print_summary(corpus: &Corpus, output: &text_struct::model::Output) {
    let mut total_blocks = 0;
    let mut para_blocks = 0;
    let mut heading_blocks = 0;
    let mut total_sentences = 0;
    let mut numbered_sentences = 0;
    let mut footnote_count = 0;
    let mut footnote_sentence_count = 0;
    let mut aa_markers = 0;
    let mut ed_markers = 0;
    let mut aa_sort_values = Vec::new();
    let mut ed_ref_values = Vec::new();

    for node in &output.toc_nodes {
        for block in &node.content_blocks {
            total_blocks += 1;
            match block.block_type.as_str() {
                "paragraph" => para_blocks += 1,
                "heading" => heading_blocks += 1,
                _ => {}
            }
            for sent in &block.sentences {
                total_sentences += 1;
                if sent.sentence_number.is_some() {
                    numbered_sentences += 1;
                }
                for footnote in &sent.footnotes {
                    footnote_count += 1;
                    footnote_sentence_count += footnote.sentences.len();
                }
                for pm in &sent.page_markers {
                    if pm.system == corpus.aa_system_slug {
                        aa_markers += 1;
                        aa_sort_values.push(pm.sort_order);
                    } else {
                        ed_markers += 1;
                        ed_ref_values.push(pm.ref_value.as_str());
                    }
                }
            }
        }
    }

    let (aa_label, ed_label) = corpus.marker_labels;
    eprintln!();
    eprintln!("=== Output summary ({}) ===", corpus.name);
    eprintln!(
        "  book:           {} ({})",
        output.book.title, output.book.language
    );
    eprintln!("  toc_nodes:      {}", output.toc_nodes.len());
    eprintln!(
        "  content_blocks: {} ({} paragraphs, {} headings)",
        total_blocks, para_blocks, heading_blocks
    );
    eprintln!(
        "  sentences:      {} ({} numbered)",
        total_sentences, numbered_sentences
    );
    eprintln!(
        "  footnotes:      {} ({} footnote sentences)",
        footnote_count, footnote_sentence_count
    );
    eprintln!(
        "  page_markers:   {} ({} {}, {} {})",
        aa_markers + ed_markers,
        aa_markers,
        aa_label,
        ed_markers,
        ed_label
    );
    if !aa_sort_values.is_empty() {
        eprintln!(
            "    {} range: {}–{}",
            aa_label,
            aa_sort_values.iter().min().unwrap(),
            aa_sort_values.iter().max().unwrap()
        );
    }
    if !ed_ref_values.is_empty() {
        eprintln!(
            "    {} range: {}–{}",
            ed_label,
            ed_ref_values.first().unwrap(),
            ed_ref_values.last().unwrap()
        );
    }
    eprintln!("  ref_systems:    {}", output.reference_systems.len());
}

fn write_output(output_file: &str, output: &text_struct::model::Output) {
    let json = serde_json::to_string_pretty(output).expect("JSON serialization failed");
    if output_file == "-" {
        std::io::stdout()
            .write_all(json.as_bytes())
            .expect("Failed to write to stdout");
        println!();
    } else {
        if let Some(parent) = Path::new(output_file).parent() {
            fs::create_dir_all(parent)
                .unwrap_or_else(|e| panic!("Cannot create directory {}: {e}", parent.display()));
        }
        fs::write(output_file, &json)
            .unwrap_or_else(|e| panic!("Cannot write {}: {e}", output_file));
        eprintln!("Wrote {}", output_file);
    }
}
