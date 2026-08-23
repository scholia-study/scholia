//! CLI: the DTA TEI of Hegel's *Phänomenologie des Geistes* (1807) → the curated
//! `md_reviewed` files plus the five sidecar reports that record what the
//! diplomatic layer deliberately leaves behind. See `lib.rs` for the mapping.

use std::collections::HashMap;
use std::path::{Path, PathBuf};

use clap::Parser;
use hegel1_tei_to_md::{
    Conv, Layer, collect_reports, frontmatter, gw, heading, modernized, rejoining_words, render,
    toc_nodes,
};

#[derive(Parser)]
#[command(about = "Convert the DTA TEI of Hegel's Phänomenologie into curated Markdown")]
struct Cli {
    /// Source TEI P5 XML (the att.linguistic edition — its <w> tokens span <lb/>).
    #[arg(
        default_value = "assets/hegel1/raw/hegel_phaenomenologie_1807_TEI_P5_inkl_att_linguistic.xml"
    )]
    input: PathBuf,
    /// Output directory; defaults to the layer's curated directory.
    out_dir: Option<PathBuf>,
    /// Which layer to emit: "reviewed" (diplomatic) or "modernized".
    #[arg(long, default_value = "reviewed")]
    layer: String,
    /// The pass-2 decision table (modernized layer only).
    #[arg(long, default_value = "assets/hegel1/curated/modernize_rulings.tsv")]
    rulings: PathBuf,
    /// The pass-3 ops table — errata and print-split repairs (modernized
    /// layer only; omit to skip the pass while the table is being curated).
    #[arg(long)]
    ops: Option<PathBuf>,
    /// Output directory for the sidecar TSV reports (reviewed layer only).
    #[arg(long, default_value = "assets/hegel1/derived")]
    reports: PathBuf,
    /// The GW 9 page concordance; every row becomes a `{{ N }}` marker.
    #[arg(long, default_value = "assets/hegel1/curated/gw9_markers.tsv")]
    gw_markers: PathBuf,
    /// Front-matter page key (the corpus's page system): "page_1807" for
    /// hegel1, "page_1812" for hegel3 — the same DTA TEI shape serves both.
    #[arg(long, default_value = "page_1807")]
    page_key: String,
    /// Lift a div (matched by head text, slug-folded) one level up, subtree
    /// and all — where the print's own contents disagree with the TEI
    /// nesting. Repeatable.
    #[arg(long)]
    promote_head: Vec<String>,
}

fn main() {
    let cli = Cli::parse();
    if let Err(e) = run(&cli) {
        eprintln!("error: {e}");
        std::process::exit(1);
    }
}

fn run(cli: &Cli) -> Result<(), Box<dyn std::error::Error>> {
    let layer = match cli.layer.as_str() {
        "reviewed" => Layer::Reviewed,
        "modernized" => Layer::Modernized,
        other => return Err(format!("unknown layer `{other}`").into()),
    };
    let xml = std::fs::read_to_string(&cli.input)?;
    let doc = roxmltree::Document::parse(&xml)?;
    let root = doc.root_element();

    let (conv, lexicon) = match layer {
        Layer::Reviewed => (Conv::new(rejoining_words(root)), HashMap::new()),
        Layer::Modernized => {
            let rulings = modernized::load_rulings(&std::fs::read_to_string(&cli.rulings)?)?;
            match modernized::replacements(root, &rulings) {
                Ok((map, lexicon)) => (Conv::modernized(rejoining_words(root), map), lexicon),
                Err(missing) => {
                    eprintln!(
                        "refusing to emit: {} surfaces lack a decision-table row",
                        missing.len()
                    );
                    eprintln!("surface\tcab_norm\tcount\tclass");
                    for m in &missing {
                        eprintln!("{}\t{}\t{}\t{}", m.surface, m.cab_norm, m.count, m.class);
                    }
                    return Err("decision table incomplete".into());
                }
            }
        }
    };
    let out_dir = cli.out_dir.clone().unwrap_or_else(|| {
        PathBuf::from(match layer {
            Layer::Reviewed => "assets/hegel1/curated/md_reviewed",
            Layer::Modernized => "assets/hegel1/curated/md_modernized",
        })
    });
    // TOC structure — labels, slugs, filenames — always derives from the
    // reviewed rendering, so both layers emit the same 50 file names.
    let structure = Conv::new(rejoining_words(root));
    let nodes = toc_nodes(root, &structure, &cli.promote_head);
    std::fs::create_dir_all(&out_dir)?;

    let mut owners = HashMap::new();
    let mut outputs: Vec<(String, String)> = Vec::new();
    for node in &nodes {
        let file = node.file_name();
        let blocks = conv.blocks(node.div);
        let label = node.display_label(&conv);
        let mut md = format!(
            "{}\n{}\n",
            frontmatter(
                &cli.page_key,
                node.position,
                &label,
                node.depth,
                node.page.as_deref(),
            ),
            heading(&label, node.heading_page.as_deref()),
        );
        if !blocks.is_empty() {
            md.push_str(&format!("\n{}\n", render(&blocks)));
        }
        owners.insert(node.div.id(), file.clone());
        outputs.push((file, md));
    }
    if let (Layer::Modernized, Some(ops_path)) = (layer, &cli.ops) {
        let ops = modernized::load_ops(&std::fs::read_to_string(ops_path)?)?;
        let applied = modernized::apply_ops(&mut outputs, &ops, &lexicon)?;
        println!("ops applied: {applied}");
    }
    if cli.gw_markers.exists() {
        let rows = gw::load(&std::fs::read_to_string(&cli.gw_markers)?)?;
        let placed = gw::apply(&mut outputs, &rows, layer == Layer::Reviewed)?;
        println!("gw markers placed: {placed}");
    }
    for (file, md) in &outputs {
        std::fs::write(out_dir.join(file), md)?;
    }

    if layer == Layer::Modernized {
        println!("wrote {} files to {}", nodes.len(), out_dir.display());
        let unknown = conv.unknown.borrow();
        if unknown.is_empty() {
            println!("unknown elements: none");
        } else {
            println!("warning: unhandled elements dropped: {unknown:?}");
        }
        return Ok(());
    }

    let reports = collect_reports(root, &owners);
    let separators: Vec<[String; 3]> = conv
        .dropped_separators
        .borrow()
        .iter()
        .map(|s| {
            [
                owners.get(&s.div).cloned().unwrap_or_default(),
                s.index.to_string(),
                format!("{} → {}", s.before, s.after),
            ]
        })
        .collect();

    std::fs::create_dir_all(&cli.reports)?;
    write_tsv(
        &cli.reports.join("sic_corr_report.tsv"),
        &["file", "page", "sic", "corr"],
        reports
            .sic_corr
            .iter()
            .map(|r| vec![&r.file, &r.page, &r.sic, &r.corr]),
    )?;
    write_tsv(
        &cli.reports.join("dropped_markup.tsv"),
        &["file", "page", "element", "rendition", "text"],
        reports
            .dropped
            .iter()
            .map(|r| vec![&r.file, &r.page, &r.element, &r.rendition, &r.text]),
    )?;
    write_tsv(
        &cli.reports.join("page_gaps.tsv"),
        &["facs", "preceding_page", "context"],
        reports
            .page_gaps
            .iter()
            .map(|r| vec![&r.facs, &r.preceding_page, &r.context]),
    )?;
    write_tsv(
        &cli.reports.join("dropped_separators.tsv"),
        &["file", "position", "neighbours"],
        separators.iter().map(|s| s.iter().collect()),
    )?;
    write_tsv(
        &cli.reports.join("errata_1807.tsv"),
        &["file", "page", "item"],
        reports
            .errata
            .iter()
            .map(|r| vec![&r.file, &r.page, &r.item]),
    )?;

    println!("wrote {} files to {}", nodes.len(), out_dir.display());
    println!(
        "reports in {}: sic_corr {} · dropped_markup {} · page_gaps {} · dropped_separators {} · errata {}",
        cli.reports.display(),
        reports.sic_corr.len(),
        reports.dropped.len(),
        reports.page_gaps.len(),
        separators.len(),
        reports.errata.len(),
    );
    let unknown = conv.unknown.borrow();
    if unknown.is_empty() {
        println!("unknown elements: none");
    } else {
        println!("warning: unhandled elements dropped: {unknown:?}");
    }
    Ok(())
}

fn write_tsv<'a>(
    path: &Path,
    header: &[&str],
    rows: impl Iterator<Item = Vec<&'a String>>,
) -> std::io::Result<()> {
    let mut out = header.join("\t");
    out.push('\n');
    for row in rows {
        out.push_str(
            &row.iter()
                .map(|c| c.replace(['\t', '\n', '\r'], " "))
                .collect::<Vec<_>>()
                .join("\t"),
        );
        out.push('\n');
    }
    std::fs::write(path, out)
}
