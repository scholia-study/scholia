mod structure;

use std::fs;
use std::io::Write;
use std::path::Path;

use clap::Parser;

use common::kant1::filenames;
use common::kant1::filenames_en;
use common::kant1::toc_en;
use common::sentences::{split_sentences, split_sentences_en};
use kant1_md_to_struct::html::{md_to_html, md_to_plain};
use kant1_md_to_struct::parse::{self, parse_blocks, parse_front_matter, ParsedBlockType};
use structure::{build_output, ParsedFile};

#[derive(Parser)]
#[command(about = "Parse English translation of Kant KrV into DB-ready JSON structures")]
struct Cli {
    /// Directory containing English translation markdown files
    #[arg(long, default_value = "assets/kant1_md_modernized_translated")]
    translation_dir: String,

    /// Directory containing German modernized markdown files (for sentence parity validation)
    #[arg(long, default_value = "assets/kant1_md_modernized")]
    source_dir: String,

    /// Output file (- for stdout)
    #[arg(long, default_value = "assets/kant1_md_translation_to_struct/output.json")]
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

/// Parse and validate a single translation file against its English TOC entry.
fn parse_translation_file(
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

    if fm.position != flat_index + 1 {
        panic!(
            "{}: position mismatch: file has {}, expected {}",
            file_path.display(),
            fm.position,
            flat_index + 1
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

/// Parse a German source file (same parser, just different TOC for validation).
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

    if fm.position != flat_index + 1 {
        panic!(
            "{}: position mismatch: file has {}, expected {}",
            file_path.display(),
            fm.position,
            flat_index + 1
        );
    }
    // Validate structural fields (aa_page, depth) but not label (German vs English)
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

    // 1. Get TOCs and expected filenames
    let en_flat_entries = toc_en::flat_toc_entries_en();
    let de_flat_entries = common::kant1::toc::flat_toc_entries();
    let en_expected_files = filenames_en::all_filenames_en();
    let de_expected_files = filenames::all_filenames();

    // 2. Scan translation directory
    let translation_entries = scan_dir(translation_dir);

    // 3. Match files to TOC entries and validate against German source
    let mut parsed_files: Vec<ParsedFile> = Vec::new();

    for &(flat_index, ref en_filename) in &en_expected_files {
        if !translation_entries.contains(en_filename) {
            continue;
        }

        // Find corresponding German filename
        let de_filename = &de_expected_files[flat_index].1;
        let de_file_path = source_dir.join(de_filename);
        if !de_file_path.exists() {
            panic!(
                "German source file {} not found for translation {}",
                de_filename, en_filename
            );
        }

        // Parse both files
        let en_blocks =
            parse_translation_file(translation_dir, en_filename, flat_index, &en_flat_entries);
        let de_blocks =
            parse_source_file(source_dir, de_filename, flat_index, &de_flat_entries);

        // Validate content block counts match (excluding footnotes)
        let en_content_count = content_block_count(&en_blocks);
        let de_content_count = content_block_count(&de_blocks);
        if en_content_count != de_content_count {
            panic!(
                "{}: content block count mismatch: translation has {}, German source has {}",
                en_filename, en_content_count, de_content_count
            );
        }

        // Validate sentence counts per block
        let en_content_blocks: Vec<&parse::ParsedBlock> = en_blocks
            .iter()
            .filter(|b| !matches!(&b.block_type, ParsedBlockType::Footnote { .. }))
            .collect();
        let de_content_blocks: Vec<&parse::ParsedBlock> = de_blocks
            .iter()
            .filter(|b| !matches!(&b.block_type, ParsedBlockType::Footnote { .. }))
            .collect();

        for (block_pos, (en_block, de_block)) in
            en_content_blocks.iter().zip(de_content_blocks.iter()).enumerate()
        {
            let en_plain = md_to_plain(&en_block.text);
            let en_html = md_to_html(&en_block.text);
            let en_sentences = split_sentences_en(&en_plain, &en_html);

            let de_plain = md_to_plain(&de_block.text);
            let de_html = md_to_html(&de_block.text);
            let de_sentences = split_sentences(&de_plain, &de_html);

            if en_sentences.len() != de_sentences.len() {
                let block_type = match &en_block.block_type {
                    ParsedBlockType::Heading => "heading",
                    ParsedBlockType::Paragraph => "paragraph",
                    ParsedBlockType::Footnote { .. } => "footnote",
                };
                panic!(
                    "{}: sentence count mismatch in block {} ({}): \
                     English has {} sentences, German has {}\n  \
                     EN first: {:?}\n  \
                     DE first: {:?}",
                    en_filename,
                    block_pos,
                    block_type,
                    en_sentences.len(),
                    de_sentences.len(),
                    en_sentences.first().map(|(t, _)| t.as_str()).unwrap_or(""),
                    de_sentences.first().map(|(t, _)| t.as_str()).unwrap_or(""),
                );
            }
        }

        eprintln!(
            "  ok: {} ({} blocks, validated against {})",
            en_filename, en_content_count, de_filename
        );

        parsed_files.push(ParsedFile {
            flat_index,
            blocks: en_blocks,
        });
    }

    // Check for unexpected files
    let expected_names: Vec<&String> = en_expected_files.iter().map(|(_, n)| n).collect();
    for name in &translation_entries {
        if !expected_names.contains(&name) {
            panic!("Unexpected file in translation directory: {}", name);
        }
    }

    eprintln!(
        "Parsed {} translation files with {} total blocks",
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
                    } else {
                        b_markers += 1;
                    }
                }
            }
        }
    }

    eprintln!();
    eprintln!("=== Output summary ===");
    eprintln!("  book:           {} ({})", output.book.title, output.book.language);
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
