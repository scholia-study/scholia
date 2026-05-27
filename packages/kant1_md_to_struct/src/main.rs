mod structure;

use std::fs;
use std::io::Write;
use std::path::Path;

use clap::Parser;

use common::kant1::filenames;
use common::kant1::toc;
use common::kant1::toc_mod;
use kant1_md_to_struct::parse::{self, parse_blocks, parse_front_matter};
use structure::{ParsedFile, build_output};

#[derive(Parser)]
#[command(about = "Parse reviewed Kant KrV markdown into DB-ready JSON structures")]
struct Cli {
    /// Directory containing modernized markdown files (primary text/html)
    #[arg(long, default_value = "assets/kant1/curated/md_modernized")]
    modernized_dir: String,

    /// Directory containing reviewed markdown files (original_text/original_html)
    #[arg(long, default_value = "assets/kant1/curated/md_reviewed")]
    reviewed_dir: String,

    /// Output file (- for stdout)
    #[arg(long, default_value = "assets/kant1/derived/md_to_struct/output.json")]
    output_file: String,
}

fn main() {
    let cli = Cli::parse();
    run_extract(&cli.modernized_dir, &cli.reviewed_dir, &cli.output_file);
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

/// Parse and validate a single markdown file against its TOC entry.
fn parse_file(
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

    let (_, aa_page, depth, label, _) = flat_entries[flat_index];

    if fm.position != filenames::position_number(flat_index) {
        panic!(
            "{}: position mismatch: file has {}, expected {}",
            file_path.display(),
            fm.position,
            filenames::position_number(flat_index)
        );
    }
    if fm.label != label {
        panic!(
            "{}: label mismatch: file has {:?}, expected {:?}",
            file_path.display(),
            fm.label,
            label
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

fn run_extract(modernized_dir_str: &str, reviewed_dir_str: &str, output_file: &str) {
    let modernized_dir = Path::new(modernized_dir_str);
    let reviewed_dir = Path::new(reviewed_dir_str);

    // 1. Get TOC and expected filenames
    let flat_entries_reviewed = toc::flat_toc_entries();
    let flat_entries_modernized = toc_mod::flat_toc_entries();
    let expected_files = filenames::all_filenames();

    // 2. Scan both directories
    let modernized_entries = scan_dir(modernized_dir);
    let reviewed_entries = scan_dir(reviewed_dir);

    // 3. Match files to TOC entries — require both dirs to have the file
    let mut parsed_files: Vec<ParsedFile> = Vec::new();

    for &(flat_index, ref expected_name) in &expected_files {
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

        let blocks = parse_file(
            modernized_dir,
            expected_name,
            flat_index,
            &flat_entries_modernized,
        );
        let original_blocks = parse_file(
            reviewed_dir,
            expected_name,
            flat_index,
            &flat_entries_reviewed,
        );

        if blocks.len() != original_blocks.len() {
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
            original_blocks,
        });
    }

    // Check for unexpected files in both directories
    let expected_names: Vec<&String> = expected_files.iter().map(|(_, n)| n).collect();
    for name in &modernized_entries {
        if !expected_names.contains(&name) {
            panic!("Unexpected file in modernized directory: {}", name);
        }
    }
    for name in &reviewed_entries {
        if !expected_names.contains(&name) {
            panic!("Unexpected file in reviewed directory: {}", name);
        }
    }

    eprintln!(
        "Parsed {} files with {} total blocks",
        parsed_files.len(),
        parsed_files.iter().map(|f| f.blocks.len()).sum::<usize>()
    );

    // 4. Build output structures
    let output = build_output(&parsed_files);

    // Summary
    let mut total_blocks = 0;
    let mut para_blocks = 0;
    let mut heading_blocks = 0;
    let mut total_sentences = 0;
    let mut numbered_sentences = 0;
    let mut footnote_count = 0;
    let mut footnote_sentence_count = 0;
    let mut aa_markers = 0;
    let mut b_markers = 0;
    let mut aa_sort_values = Vec::new();
    let mut b_ref_values = Vec::new();

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
                    if pm.system == "aa_iii" {
                        aa_markers += 1;
                        aa_sort_values.push(pm.sort_order);
                    } else {
                        b_markers += 1;
                        b_ref_values.push(pm.ref_value.as_str());
                    }
                }
            }
        }
    }

    eprintln!();
    eprintln!("=== Output summary ===");
    eprintln!("  toc_nodes:      {}", output.toc_nodes.len());
    for node in &output.toc_nodes {
        eprintln!(
            "    {} {} (depth {}, {} blocks)",
            node.source_ref,
            node.label,
            node.depth,
            node.content_blocks.len()
        );
    }
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
        "  page_markers:   {} ({} AA, {} B-edition)",
        aa_markers + b_markers,
        aa_markers,
        b_markers
    );
    if !aa_sort_values.is_empty() {
        eprintln!(
            "    AA range:  {}–{}",
            aa_sort_values.iter().min().unwrap(),
            aa_sort_values.iter().max().unwrap()
        );
    }
    if !b_ref_values.is_empty() {
        eprintln!(
            "    B range:   {}–{}",
            b_ref_values.first().unwrap(),
            b_ref_values.last().unwrap()
        );
    }
    eprintln!("  ref_systems:    {}", output.reference_systems.len());

    // 5. Write JSON
    let json = serde_json::to_string_pretty(&output).expect("JSON serialization failed");

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
