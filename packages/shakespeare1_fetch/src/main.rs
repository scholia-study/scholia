//! Fetch Shakespeare's Sonnets into the curated MD layer (Kant-style):
//!   - curated/md_modernized/NNN_sonnet_N.md  ← PoetryDB (modern spelling)
//!   - curated/md_reviewed/NNN_sonnet_N.md    ← EEBO-TCP A12044 (1609 old spelling, CC0)

mod eebo;
mod poetrydb;

use std::fs;
use std::path::Path;

use clap::Parser;
use common::shakespeare1 as sonnets;

#[derive(Parser)]
#[command(about = "Fetch Shakespeare's Sonnets into the curated MD layer")]
struct Cli {
    #[arg(long, default_value = "assets/shakespeare1/curated/md_modernized")]
    modernized_dir: String,
    #[arg(long, default_value = "assets/shakespeare1/curated/md_reviewed")]
    reviewed_dir: String,

    #[arg(
        long,
        default_value = "https://poetrydb.org/author/William%20Shakespeare"
    )]
    poetrydb_url: String,
    #[arg(
        long,
        default_value = "https://raw.githubusercontent.com/textcreationpartnership/A12044/master/A12044.xml"
    )]
    eebo_url: String,

    /// Read PoetryDB JSON from a local file instead of fetching.
    #[arg(long)]
    poetrydb_file: Option<String>,
    /// Read EEBO-TCP XML from a local file instead of fetching.
    #[arg(long)]
    eebo_file: Option<String>,
    /// If set, also write the fetched raw sources here (reproducibility cache).
    #[arg(long)]
    raw_dir: Option<String>,

    /// Overwrite curated files that already exist (DESTROYS hand edits).
    #[arg(long)]
    force: bool,
}

fn main() {
    if let Err(e) = run() {
        eprintln!("error: {e}");
        std::process::exit(1);
    }
}

fn run() -> Result<(), Box<dyn std::error::Error>> {
    let cli = Cli::parse();

    let poetrydb_json = match &cli.poetrydb_file {
        Some(p) => fs::read_to_string(p)?,
        None => {
            eprintln!("fetching modern text: {}", cli.poetrydb_url);
            http_get(&cli.poetrydb_url)?
        }
    };
    let eebo_xml = match &cli.eebo_file {
        Some(p) => fs::read_to_string(p)?,
        None => {
            eprintln!("fetching old spelling: {}", cli.eebo_url);
            http_get(&cli.eebo_url)?
        }
    };
    if let Some(dir) = &cli.raw_dir {
        fs::create_dir_all(dir)?;
        fs::write(
            Path::new(dir).join("poetrydb_shakespeare.json"),
            &poetrydb_json,
        )?;
        fs::write(Path::new(dir).join("A12044.xml"), &eebo_xml)?;
    }

    let modern = poetrydb::parse(&poetrydb_json)?;
    let old = eebo::parse(&eebo_xml)?;
    eprintln!(
        "parsed: {} modern sonnets, {} old-spelling sonnets",
        modern.len(),
        old.len()
    );

    fs::create_dir_all(&cli.modernized_dir)?;
    fs::create_dir_all(&cli.reviewed_dir)?;

    let mut written = 0u32;
    let mut skipped = 0u32;
    let mut misprints: Vec<(u32, u32)> = Vec::new();
    let mut mismatches: Vec<(u32, usize, usize)> = Vec::new();
    let mut missing_old = Vec::new();

    for n in sonnets::sonnet_numbers() {
        let Some(modern_lines) = modern.get(&n) else {
            return Err(format!("missing modern text for sonnet {n}").into());
        };
        let old_sonnet = old.get((n - 1) as usize);
        match old_sonnet {
            None => missing_old.push(n),
            Some(o) => {
                if let Some(p) = o.printed_n
                    && p != n
                {
                    misprints.push((n, p));
                }
                if o.lines.len() != modern_lines.len() {
                    mismatches.push((n, modern_lines.len(), o.lines.len()));
                }
            }
        }
        let old_lines: &[String] = old_sonnet.map(|o| o.lines.as_slice()).unwrap_or(&[]);

        let filename = sonnets::filename(n);
        for (dir, lines) in [
            (&cli.modernized_dir, modern_lines.as_slice()),
            (&cli.reviewed_dir, old_lines),
        ] {
            if write_md(dir, &filename, n, lines, cli.force)? {
                written += 1;
            } else {
                skipped += 1;
            }
        }
    }

    eprintln!("=== fetch summary ===");
    eprintln!("  files written: {written}");
    if skipped > 0 {
        eprintln!("  files skipped (already exist; use --force to overwrite): {skipped}");
    }
    if !misprints.is_empty() {
        let list: Vec<String> = misprints
            .iter()
            .map(|(s, p)| format!("{s} prints {p}"))
            .collect();
        eprintln!(
            "  Quarto misprints (preserved by EEBO): {}",
            list.join(", ")
        );
    }
    if !missing_old.is_empty() {
        eprintln!("  WARNING old-spelling missing for: {missing_old:?}");
    }
    for (n, m, o) in &mismatches {
        eprintln!("  WARNING Sonnet {n}: modern {m} lines vs old {o} — reconcile before md→struct");
    }
    Ok(())
}

/// Write a curated MD file (front matter + verse lines). Returns `true` if
/// written, `false` if skipped because it already exists and `force` is false.
fn write_md(
    dir: &str,
    filename: &str,
    n: u32,
    lines: &[String],
    force: bool,
) -> Result<bool, Box<dyn std::error::Error>> {
    let path = Path::new(dir).join(filename);
    if path.exists() && !force {
        return Ok(false);
    }
    let mut out = String::new();
    out.push_str("---\n");
    out.push_str(&format!("position: {n}\n"));
    out.push_str(&format!("label: \"{}\"\n", sonnets::label(n)));
    out.push_str(&format!("depth: {}\n", sonnets::DEPTH));
    out.push_str("---\n");
    for line in lines {
        out.push_str(line);
        out.push('\n');
    }
    fs::write(&path, out)?;
    Ok(true)
}

fn http_get(url: &str) -> Result<String, Box<dyn std::error::Error>> {
    let client = reqwest::blocking::Client::builder()
        .user_agent("scholia-shakespeare1/0.1 (+https://scholia.app)")
        .timeout(std::time::Duration::from_secs(60))
        .build()?;
    Ok(client.get(url).send()?.error_for_status()?.text()?)
}
