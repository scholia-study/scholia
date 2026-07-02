//! Curated two-layer verse markdown → struct JSON, for any verse corpus.
//! `--corpus shakespeare1|milton` selects the config; the parser is shared.

use std::path::Path;

use clap::Parser;

use md_poetry_to_struct::{corpus, parse};

#[derive(Parser)]
#[command(about = "Parse curated two-layer verse markdown into DB-ready JSON")]
struct Cli {
    /// Which corpus to parse: shakespeare1 | milton
    #[arg(long)]
    corpus: String,
    /// Override the modernized layer dir (defaults per corpus).
    #[arg(long)]
    modernized_dir: Option<String>,
    /// Override the reviewed layer dir (defaults per corpus).
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
    let mut corpus = corpus::by_name(&cli.corpus).ok_or_else(|| {
        format!(
            "unknown corpus {:?} (expected shakespeare1 | milton)",
            cli.corpus
        )
    })?;
    if let Some(d) = &cli.modernized_dir {
        corpus.modernized_dir = d.clone();
    }
    if let Some(d) = &cli.reviewed_dir {
        corpus.reviewed_dir = d.clone();
    }
    let output_file = cli
        .output_file
        .clone()
        .unwrap_or_else(|| default_output(&cli.corpus));

    let output = parse::build(&corpus)?;
    let json = serde_json::to_string_pretty(&output)?;
    if let Some(parent) = Path::new(&output_file).parent() {
        std::fs::create_dir_all(parent)?;
    }
    std::fs::write(&output_file, json)?;

    let sentences: usize = output
        .toc_nodes
        .iter()
        .flat_map(|n| &n.content_blocks)
        .map(|b| b.sentences.len())
        .sum();
    eprintln!("=== output summary ({}) ===", cli.corpus);
    eprintln!("  toc_nodes:      {}", output.toc_nodes.len());
    eprintln!("  sentences:      {sentences}");
    eprintln!("  wrote {output_file}");
    Ok(())
}

fn default_output(corpus: &str) -> String {
    match corpus {
        "shakespeare1" | "shakespeare" => "assets/shakespeare1/derived/output.json".into(),
        "milton1" | "milton" => "assets/milton1/derived/output.json".into(),
        _ => "output.json".into(),
    }
}
