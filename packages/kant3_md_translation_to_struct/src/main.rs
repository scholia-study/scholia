mod structure;

use std::fs;
use std::io::Write;
use std::path::Path;

use clap::Parser;

use common::kant3::{filenames, toc_mod};
use common::sentences::{split_sentences, split_sentences_en};
use kant1_md_to_struct::html::{md_to_html, md_to_plain};
use kant1_md_to_struct::parse::{self, ParsedBlockType, parse_blocks, parse_front_matter};
use structure::{ParsedFile, build_output};

#[derive(Parser)]
#[command(
    about = "Parse the English translation of Kant's Kritik der Urteilskraft into DB-ready JSON"
)]
struct Cli {
    /// Directory containing English translation markdown files.
    #[arg(long, default_value = "assets/kant3/curated/md_modernized_translated")]
    translation_dir: String,

    /// Directory containing the modernized German files (for sentence parity).
    #[arg(long, default_value = "assets/kant3/curated/md_modernized")]
    source_dir: String,

    /// Output file (- for stdout).
    #[arg(
        long,
        default_value = "assets/kant3/derived/md_translation_to_struct/output.json"
    )]
    output_file: String,
}

fn main() {
    let cli = Cli::parse();
    run_extract(&cli.translation_dir, &cli.source_dir, &cli.output_file);
}

/// Scan a directory for .md files (excluding 000_toc.md).
fn scan_dir(dir: &Path) -> Vec<String> {
    fs::read_dir(dir)
        .unwrap_or_else(|e| panic!("Cannot read {}: {e}", dir.display()))
        .filter_map(|e| e.ok())
        .map(|e| e.file_name().to_string_lossy().to_string())
        .filter(|name| name.ends_with(".md") && name != "000_toc.md")
        .collect()
}

/// Parse a translation file; validate its structural front matter (position,
/// depth, aa_page) against the German TOC and capture its English label. The
/// label is NOT validated against a table — kant3 has no English TOC; the
/// translated file's front matter IS the authority for the English label.
fn parse_translation_file(
    dir: &Path,
    filename: &str,
    flat_index: usize,
    flat_entries: &[(usize, u16, u16, &str, Option<&str>)],
) -> (Vec<parse::ParsedBlock>, String) {
    let file_path = dir.join(filename);
    let content = fs::read_to_string(&file_path)
        .unwrap_or_else(|e| panic!("Cannot read {}: {e}", file_path.display()));

    let (fm, body) = parse_front_matter(&content)
        .unwrap_or_else(|| panic!("No front matter in {}", file_path.display()));

    let (_, aa_page, depth, _de_label, _) = flat_entries[flat_index];

    if fm.position != filenames::position_number(flat_index) {
        panic!(
            "{}: position mismatch: file has {}, expected {}",
            file_path.display(),
            fm.position,
            filenames::position_number(flat_index)
        );
    }
    if fm.depth != depth {
        panic!(
            "{}: depth mismatch: file has {}, expected {}",
            file_path.display(),
            fm.depth,
            depth
        );
    }
    if fm.aa_page != aa_page {
        panic!(
            "{}: aa_page mismatch: file has {}, expected {}",
            file_path.display(),
            fm.aa_page,
            aa_page
        );
    }

    (parse_blocks(body), fm.label)
}

/// Parse a German source file; validate structural fields only (label differs).
fn parse_source_file(
    dir: &Path,
    filename: &str,
    flat_index: usize,
    flat_entries: &[(usize, u16, u16, &str, Option<&str>)],
) -> Vec<parse::ParsedBlock> {
    let file_path = dir.join(filename);
    let content = fs::read_to_string(&file_path)
        .unwrap_or_else(|e| panic!("Cannot read {}: {e}", file_path.display()));

    let (fm, body) = parse_front_matter(&content)
        .unwrap_or_else(|| panic!("No front matter in {}", file_path.display()));

    let (_, aa_page, depth, _label, _) = flat_entries[flat_index];

    if fm.position != filenames::position_number(flat_index) {
        panic!(
            "{}: position mismatch: file has {}, expected {}",
            file_path.display(),
            fm.position,
            filenames::position_number(flat_index)
        );
    }
    if fm.depth != depth {
        panic!(
            "{}: depth mismatch: file has {}, expected {}",
            file_path.display(),
            fm.depth,
            depth
        );
    }
    if fm.aa_page != aa_page {
        panic!(
            "{}: aa_page mismatch: file has {}, expected {}",
            file_path.display(),
            fm.aa_page,
            aa_page
        );
    }

    parse_blocks(body)
}

/// Count non-footnote blocks.
fn content_block_count(blocks: &[parse::ParsedBlock]) -> usize {
    blocks
        .iter()
        .filter(|b| !matches!(&b.block_type, ParsedBlockType::Footnote { .. }))
        .count()
}

fn run_extract(translation_dir_str: &str, source_dir_str: &str, output_file: &str) {
    let translation_dir = Path::new(translation_dir_str);
    let source_dir = Path::new(source_dir_str);

    // The German TOC supplies structure (depth/aa_page/position); filenames are
    // shared between the German and translated layers (German slugs).
    let de_flat_entries = toc_mod::flat_toc_entries();
    let expected_files = filenames::all_filenames();
    let translation_entries = scan_dir(translation_dir);

    let mut parsed_files: Vec<ParsedFile> = Vec::new();
    let mut mismatches: Vec<String> = Vec::new();

    for &(flat_index, ref filename) in &expected_files {
        if !translation_entries.contains(filename) {
            continue;
        }
        let de_file_path = source_dir.join(filename);
        if !de_file_path.exists() {
            panic!("German source file {filename} not found for translation");
        }

        let (en_blocks, english_label) =
            parse_translation_file(translation_dir, filename, flat_index, &de_flat_entries);
        let de_blocks = parse_source_file(source_dir, filename, flat_index, &de_flat_entries);

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
            english_label,
        });
    }

    // Unexpected files guard.
    let expected_names: Vec<&String> = expected_files.iter().map(|(_, n)| n).collect();
    for name in &translation_entries {
        if !expected_names.contains(&name) {
            panic!("Unexpected file in translation directory: {name}");
        }
    }

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

    let output = build_output(&parsed_files);

    let mut total_blocks = 0;
    let mut total_sentences = 0;
    let mut footnote_count = 0;
    let (mut aa_markers, mut ed_markers) = (0u32, 0u32);
    for node in &output.toc_nodes {
        for block in &node.content_blocks {
            total_blocks += 1;
            for sent in &block.sentences {
                total_sentences += 1;
                footnote_count += sent.footnotes.len();
                for pm in &sent.page_markers {
                    if pm.system == "aa_v" {
                        aa_markers += 1;
                    } else {
                        ed_markers += 1;
                    }
                }
            }
        }
    }

    eprintln!();
    eprintln!("=== Output summary ===");
    eprintln!(
        "  book:           {} ({})",
        output.book.title, output.book.language
    );
    eprintln!("  toc_nodes:      {}", output.toc_nodes.len());
    eprintln!("  content_blocks: {total_blocks}");
    eprintln!("  sentences:      {total_sentences}");
    eprintln!("  footnotes:      {footnote_count}");
    eprintln!(
        "  page_markers:   {} ({aa_markers} AA Bd. V, {ed_markers} 1790)",
        aa_markers + ed_markers
    );
    eprintln!("  ref_systems:    {}", output.reference_systems.len());

    let json = serde_json::to_string_pretty(&output).expect("JSON serialization failed");
    if output_file == "-" {
        std::io::stdout()
            .write_all(json.as_bytes())
            .expect("write stdout");
        println!();
    } else {
        if let Some(parent) = Path::new(output_file).parent() {
            fs::create_dir_all(parent)
                .unwrap_or_else(|e| panic!("Cannot create directory {}: {e}", parent.display()));
        }
        fs::write(output_file, &json).unwrap_or_else(|e| panic!("Cannot write {output_file}: {e}"));
        eprintln!("Wrote {output_file}");
    }
}
