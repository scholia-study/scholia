mod html;
mod import;
mod model;
mod parse;
mod roman;
mod structure;

use std::fs;
use std::io::Write;
use std::path::Path;

use clap::{Parser, Subcommand};

use common::kant1::filenames;
use common::kant1::toc;
use parse::{parse_blocks, parse_front_matter};
use structure::{build_output, ParsedFile};

#[derive(Parser)]
#[command(about = "Parse reviewed Kant KrV markdown into DB-ready JSON structures")]
struct Cli {
    /// Directory containing reviewed markdown files
    #[arg(long, default_value = "assets/kant1_md_reviewed")]
    input_dir: String,

    /// Output file (- for stdout)
    #[arg(long, default_value = "assets/kant1_md_to_struct/output.json")]
    output_file: String,

    #[command(subcommand)]
    command: Option<Command>,
}

#[derive(Subcommand)]
enum Command {
    /// Also import the JSON into PostgreSQL database
    Import {
        /// PostgreSQL connection URL (overrides DATABASE_URL env var)
        #[arg(long)]
        database_url: Option<String>,
    },
}

fn main() {
    let cli = Cli::parse();

    // Always extract
    let output_file = run_extract(&cli.input_dir, &cli.output_file);

    // Optionally import
    if let Some(Command::Import { database_url }) = cli.command {
        let rt = tokio::runtime::Runtime::new().expect("Failed to create tokio runtime");
        rt.block_on(async {
            if let Err(e) = import::run(&output_file, database_url).await {
                eprintln!("Import failed: {e}");
                std::process::exit(1);
            }
        });
    }
}

/// Run extraction, return the path to the written JSON file.
fn run_extract(input_dir_str: &str, output_file: &str) -> String {
    let input_dir = Path::new(input_dir_str);

    // 1. Get TOC and expected filenames
    let flat_entries = toc::flat_toc_entries();
    let expected_files = filenames::all_filenames();

    // 2. Scan input directory for existing files
    let dir_entries: Vec<String> = fs::read_dir(input_dir)
        .unwrap_or_else(|e| panic!("Cannot read {}: {e}", input_dir.display()))
        .filter_map(|e| e.ok())
        .map(|e| e.file_name().to_string_lossy().to_string())
        .filter(|name| name.ends_with(".md") && name != "000_toc.md")
        .collect();

    // 3. Match files to TOC entries
    let mut parsed_files: Vec<ParsedFile> = Vec::new();

    for &(flat_index, ref expected_name) in &expected_files {
        if !dir_entries.contains(expected_name) {
            eprintln!("info: skipping {} (file not found)", expected_name);
            continue;
        }

        let file_path = input_dir.join(expected_name);
        let content = fs::read_to_string(&file_path)
            .unwrap_or_else(|e| panic!("Cannot read {}: {e}", file_path.display()));

        // Parse front matter and validate
        let (fm, body) = parse_front_matter(&content)
            .unwrap_or_else(|| panic!("No front matter in {}", expected_name));

        let (_, aa_page, depth, label) = flat_entries[flat_index];

        if fm.position != flat_index + 1 {
            panic!(
                "{}: position mismatch: file has {}, expected {}",
                expected_name,
                fm.position,
                flat_index + 1
            );
        }
        if fm.label != label {
            panic!(
                "{}: label mismatch: file has {:?}, expected {:?}",
                expected_name, fm.label, label
            );
        }
        if fm.depth != depth {
            panic!(
                "{}: depth mismatch: file has {}, expected {}",
                expected_name, fm.depth, depth
            );
        }
        if fm.aa_page != aa_page {
            panic!(
                "{}: aa_page mismatch: file has {}, expected {}",
                expected_name, fm.aa_page, aa_page
            );
        }

        let blocks = parse_blocks(body);

        parsed_files.push(ParsedFile {
            flat_index,
            blocks,
        });
    }

    // Check for unexpected files
    let expected_names: Vec<&String> = expected_files.iter().map(|(_, n)| n).collect();
    for name in &dir_entries {
        if !expected_names.contains(&name) {
            panic!("Unexpected file in input directory: {}", name);
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
    let mut footnote_blocks = 0;
    let mut total_sentences = 0;
    let mut numbered_sentences = 0;
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
                "footnote" => footnote_blocks += 1,
                _ => {}
            }
            for sent in &block.sentences {
                total_sentences += 1;
                if sent.sentence_number.is_some() {
                    numbered_sentences += 1;
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
        "  content_blocks: {} ({} paragraphs, {} headings, {} footnotes)",
        total_blocks, para_blocks, heading_blocks, footnote_blocks
    );
    eprintln!(
        "  sentences:      {} ({} numbered)",
        total_sentences, numbered_sentences
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
        "-".to_string()
    } else {
        fs::write(output_file, &json)
            .unwrap_or_else(|e| panic!("Cannot write {}: {e}", output_file));
        eprintln!("Wrote {}", output_file);
        output_file.to_string()
    }
}
