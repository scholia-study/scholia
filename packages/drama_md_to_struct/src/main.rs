//! Curated two-layer drama markdown → struct JSON. `--corpus ibsen1` selects the
//! config; the parser is shared. Output is consumed by `struct_to_db`.

use std::path::Path;

use clap::Parser;

use drama_md_to_struct::{corpus, parse};

#[derive(Parser)]
#[command(about = "Parse curated two-layer drama markdown into DB-ready JSON")]
struct Cli {
    /// Which corpus to parse: ibsen1
    #[arg(long)]
    corpus: String,
    /// Parse the single-layer translation edition (md_modernized_translated)
    /// instead of the two-layer source.
    #[arg(long)]
    translation: bool,
    /// Override the modernized/primary layer dir (defaults per corpus).
    #[arg(long)]
    modernized_dir: Option<String>,
    /// Override the reviewed layer dir (defaults per corpus; ignored for a
    /// translation edition).
    #[arg(long)]
    reviewed_dir: Option<String>,
    /// Override the output path (defaults per corpus).
    #[arg(long)]
    output_file: Option<String>,
}

fn main() {
    let cli = Cli::parse();
    if let Err(e) = run(&cli) {
        eprintln!("error: {e}");
        std::process::exit(1);
    }
}

fn run(cli: &Cli) -> Result<(), Box<dyn std::error::Error>> {
    let mut corpus = corpus::by_name(&cli.corpus, cli.translation)
        .ok_or_else(|| format!("unknown corpus {:?}", cli.corpus))?;
    if let Some(d) = &cli.modernized_dir {
        corpus.modernized_dir = d.clone();
    }
    if let Some(d) = &cli.reviewed_dir {
        corpus.reviewed_dir = Some(d.clone());
    }
    let output_file = cli
        .output_file
        .clone()
        .unwrap_or_else(|| default_output(&cli.corpus, cli.translation));

    let output = parse::build(&corpus)?;
    let json = serde_json::to_string_pretty(&output)?;
    if let Some(parent) = Path::new(&output_file).parent() {
        std::fs::create_dir_all(parent)?;
    }
    std::fs::write(&output_file, json)?;

    let (blocks, sentences, markers) = output.toc_nodes.iter().fold((0, 0, 0), |acc, n| {
        let b = n.content_blocks.len();
        let s: usize = n.content_blocks.iter().map(|b| b.sentences.len()).sum();
        let m: usize = n
            .content_blocks
            .iter()
            .flat_map(|b| &b.sentences)
            .map(|s| s.page_markers.len())
            .sum();
        (acc.0 + b, acc.1 + s, acc.2 + m)
    });
    eprintln!("=== output summary ({}) ===", cli.corpus);
    eprintln!("  toc_nodes:      {}", output.toc_nodes.len());
    eprintln!("  content_blocks: {blocks}");
    eprintln!("  sentences:      {sentences}");
    eprintln!("  page_markers:   {markers}");
    eprintln!("  wrote {output_file}");
    Ok(())
}

fn default_output(corpus: &str, translation: bool) -> String {
    match (corpus, translation) {
        ("ibsen1" | "ibsen", false) => "assets/ibsen1/derived/output.json".into(),
        ("ibsen1" | "ibsen", true) => "assets/ibsen1/derived/translation_output.json".into(),
        _ => "output.json".into(),
    }
}
