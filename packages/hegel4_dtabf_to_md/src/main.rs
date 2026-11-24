//! CLI: GW20.xml (DTA-Basisformat TEI of the Digitale Hegel-Edition) → the
//! curated hegel4 layers. See `lib.rs` for the mapping.

use std::collections::BTreeMap;
use std::fs;
use std::path::{Path, PathBuf};

use clap::Parser;
use common::hegel4::{filenames, meta, toc};
use hegel4_dtabf_to_md::assemble::{assemble, build_blocks};
use hegel4_dtabf_to_md::hyphen::{Decision, Oracle, Rulings};
use hegel4_dtabf_to_md::indent_runs::IndentRuns;
use hegel4_dtabf_to_md::modernize::Modernizer;
use hegel4_dtabf_to_md::page_joins::PageJoins;
use hegel4_dtabf_to_md::render::Renderer;
use hegel4_dtabf_to_md::source;

#[derive(Parser)]
#[command(
    about = "Convert the DTA-Basisformat TEI of Hegel's Enzyklopädie (GW 20) to curated Markdown"
)]
struct Cli {
    /// Source TEI (GW20.xml).
    #[arg(long, default_value = "assets/hegel4/raw/GW20.xml")]
    input: PathBuf,
    /// The edition JSON, consulted only as the line-end hyphen oracle.
    #[arg(long, default_value = "assets/hegel4/raw/work-gw-20.js")]
    json: PathBuf,
    /// Hyphen rulings for the joins the JSON cannot settle (optional).
    #[arg(long, default_value = "assets/hegel4/curated/hyphen_rulings.tsv")]
    hyphen_rulings: PathBuf,
    /// Decisions for every page boundary that splits a paragraph.
    #[arg(long, default_value = "assets/hegel4/curated/page_joins.tsv")]
    page_joins: PathBuf,
    /// Paragraph ranges set as indented `+ ` runs.
    #[arg(long, default_value = "assets/hegel4/curated/indent_runs.tsv")]
    indent_runs: PathBuf,
    /// Output directory (defaults to the chosen layer's curated directory). It
    /// must hold no Markdown: the curated layers are hand-edited, so the
    /// converter never overwrites them.
    #[arg(long)]
    out_dir: Option<PathBuf>,
    /// `reviewed` extracts from the TEI; `modernized` re-spells the reviewed
    /// layer per the modernize rulings.
    #[arg(long, default_value = "reviewed")]
    layer: String,
    /// Reviewed layer read by `--layer modernized`.
    #[arg(long, default_value = meta::REVIEWED_DIR)]
    reviewed_dir: PathBuf,
    #[arg(long, default_value = "assets/hegel4/curated/modernize_rulings.tsv")]
    modernize_rulings: PathBuf,
}

fn main() {
    if let Err(e) = run(&Cli::parse()) {
        eprintln!("error: {e}");
        std::process::exit(1);
    }
}

fn run(cli: &Cli) -> Result<(), Box<dyn std::error::Error>> {
    match cli.layer.as_str() {
        "reviewed" => extract_reviewed(cli),
        "modernized" => modernize(cli),
        other => Err(format!("unknown layer {other:?} (expected reviewed | modernized)").into()),
    }
}

fn prepare_empty_dir(dir: &Path) -> Result<(), Box<dyn std::error::Error>> {
    fs::create_dir_all(dir)?;
    for entry in fs::read_dir(dir)? {
        if entry?.path().extension().is_some_and(|e| e == "md") {
            return Err(format!(
                "{} already holds Markdown; the curated layers are the editorial surface and are \
                 never regenerated — pass --out-dir to an empty scratch directory",
                dir.display()
            )
            .into());
        }
    }
    Ok(())
}

fn modernize(cli: &Cli) -> Result<(), Box<dyn std::error::Error>> {
    let mut modernizer = Modernizer::parse(&fs::read_to_string(&cli.modernize_rulings)?)?;
    let out_dir = cli
        .out_dir
        .clone()
        .unwrap_or_else(|| PathBuf::from(meta::MODERNIZED_DIR));
    prepare_empty_dir(&out_dir)?;
    let mut files = 0;
    for (_, name) in filenames::all_filenames() {
        let reviewed = fs::read_to_string(cli.reviewed_dir.join(&name))?;
        fs::write(out_dir.join(&name), modernizer.file(&reviewed))?;
        files += 1;
    }
    if !modernizer.untabled.is_empty() {
        modernizer.untabled.sort();
        modernizer.untabled.dedup();
        return Err(format!("untabled pre-reform surfaces: {:?}", modernizer.untabled).into());
    }
    eprintln!("modernized {files} files");
    Ok(())
}

fn extract_reviewed(cli: &Cli) -> Result<(), Box<dyn std::error::Error>> {
    let xml = fs::read_to_string(&cli.input)?;
    let oracle = Oracle::from_json(&fs::read_to_string(&cli.json)?)?;
    let rulings = match fs::read_to_string(&cli.hyphen_rulings) {
        Ok(tsv) => Rulings::parse(&tsv)?,
        Err(_) => Rulings::default(),
    };
    let joins = PageJoins::parse(&fs::read_to_string(&cli.page_joins)?)?;
    let indents = IndentRuns::parse(&fs::read_to_string(&cli.indent_runs)?)?;
    let src = source::parse(&xml)?;
    let mut renderer = Renderer::new(&oracle, &rulings, &src.notes);
    let blocks = build_blocks(&src, &mut renderer, &joins)?;
    let nodes = assemble(
        blocks,
        toc::entries(),
        &renderer.definitions,
        &indents,
        |e| format!("{:03}_{}.md", e.position, e.slug),
    )?;
    debug_assert_eq!(nodes.len(), filenames::all_filenames().len());

    let out_dir = cli
        .out_dir
        .clone()
        .unwrap_or_else(|| PathBuf::from(meta::REVIEWED_DIR));
    prepare_empty_dir(&out_dir)?;
    for node in &nodes {
        fs::write(out_dir.join(&node.file_name), &node.content)?;
    }

    let sections: usize = nodes.iter().map(|n| n.sections.len()).sum();
    let fences: usize = nodes.iter().map(|n| n.remark_fences).sum();
    let footnotes: usize = nodes.iter().map(|n| n.footnotes).sum();
    let mut by_decision: BTreeMap<String, usize> = BTreeMap::new();
    for h in &renderer.hyphens {
        let kind = match h.decision {
            Decision::Join => "join",
            Decision::Keep => "keep",
            Decision::Space => "space",
            Decision::Ck => "ck",
        };
        *by_decision.entry(format!("{kind}/{}", h.via)).or_default() += 1;
    }
    eprintln!("nodes: {}", nodes.len());
    eprintln!(
        "§ headings moved to the node they head: {}",
        nodes.iter().map(|n| n.moved_headings).sum::<usize>()
    );
    eprintln!("§ headings: {sections}");
    eprintln!("remark fences: {fences}");
    eprintln!("footnotes: {footnotes}");
    eprintln!("hyphen joins: {by_decision:?}");
    eprintln!(
        "dropped from the source: {} milestones, {} vertical spaces; {} <foreign> kept",
        src.stats.milestones, src.stats.vertical_spaces, src.stats.foreign
    );
    eprintln!("pages without a marker: {:?}", renderer.skipped_pages);
    for joined in &renderer.joined_paragraphs {
        eprintln!("joined split paragraph: {joined}");
    }
    let mut queued: Vec<(u32, &str)> = nodes
        .iter()
        .flat_map(|node| node.queued_sections.iter().copied())
        .collect();
    queued.sort_unstable();
    queued.dedup();
    for (n, why) in queued {
        eprintln!("queue: § {n}: {why}");
    }
    Ok(())
}
