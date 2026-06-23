//! CLI: HIS TEI of Ibsen's *Kejser og Galilæer* → curated `md_reviewed` files.
//!
//! One book, two parts (KG1 *Cæsars Frafald*, KG2 *Kejser Julian*); each part is
//! a title-page node (depth 0, a content-bearing parent) over a cast list + five
//! acts (depth 1). 14 files, positions 1–14. See `lib.rs` for the convention.

use std::path::{Path, PathBuf};

use clap::Parser;
use ibsen1_xml_to_md::{Conv, compute_pb_pages, descendant, frontmatter, get_text, render};

#[derive(Parser)]
#[command(about = "Convert the HIS TEI of Ibsen's Kejser og Galilæer into curated drama Markdown")]
struct Cli {
    /// Source TEI XML.
    #[arg(default_value = "assets/ibsen1/raw/DRVIT_KG_KG73.xml")]
    input: PathBuf,
    /// Output directory for the curated md_reviewed files.
    #[arg(default_value = "assets/ibsen1/curated/md_reviewed")]
    out_dir: PathBuf,
}

/// (filename prefix, <text> xml:id, title-page label) per part.
const PARTS: [(&str, &str, &str); 2] = [
    ("cf", "KG1", "Cæsars Frafald"),
    ("kj", "KG2", "Kejser Julian"),
];
const ACT_LABELS: [&str; 5] = [
    "Første handling",
    "Anden handling",
    "Tredje handling",
    "Fjerde handling",
    "Femte handling",
];
const ACT_SLUGS: [&str; 5] = [
    "foerste_handling",
    "anden_handling",
    "tredje_handling",
    "fjerde_handling",
    "femte_handling",
];

fn main() {
    let cli = Cli::parse();
    if let Err(e) = run(&cli) {
        eprintln!("error: {e}");
        std::process::exit(1);
    }
}

fn run(cli: &Cli) -> Result<(), Box<dyn std::error::Error>> {
    let xml = std::fs::read_to_string(&cli.input)?;
    let doc = roxmltree::Document::parse(&xml)?;
    let root = doc.root_element();
    let conv = Conv::new(compute_pb_pages(root));
    std::fs::create_dir_all(&cli.out_dir)?;

    let mut seq = 0u32;
    let mut written = Vec::new();
    let mut emit = |prefix: &str,
                    slug: &str,
                    label: &str,
                    depth: u8,
                    blocks: &[Vec<String>]|
     -> std::io::Result<()> {
        seq += 1;
        let fname = format!("{seq:03}_{prefix}_{slug}.md");
        let md = format!("{}\n{}\n", frontmatter(seq, label, depth), render(blocks));
        std::fs::write(cli.out_dir.join(&fname), md)?;
        written.push(fname);
        Ok(())
    };

    for (prefix, xid, part_label) in PARTS {
        let part = get_text(root, xid).ok_or_else(|| format!("missing <text xml:id={xid}>"))?;
        emit(
            prefix,
            "titelblad",
            part_label,
            0,
            &conv.convert_titlepage(part),
        )?;

        let castlist =
            descendant(part, "hisCastList").ok_or_else(|| format!("{xid}: no hisCastList"))?;
        let setel = descendant(part, "set");
        emit(
            prefix,
            "de_optraedende",
            "De optrædende",
            1,
            &conv.convert_cast(castlist, setel),
        )?;

        let acts: Vec<_> = part
            .descendants()
            .filter(|d| {
                d.is_element() && d.tag_name().name() == "div" && d.attribute("type") == Some("act")
            })
            .collect();
        if acts.len() != ACT_LABELS.len() {
            return Err(format!(
                "{xid}: expected {} acts, found {}",
                ACT_LABELS.len(),
                acts.len()
            )
            .into());
        }
        for (i, act) in acts.into_iter().enumerate() {
            emit(
                prefix,
                ACT_SLUGS[i],
                ACT_LABELS[i],
                1,
                &conv.convert_act(act),
            )?;
        }
    }

    print_summary(&cli.out_dir, &written, &conv);
    Ok(())
}

fn print_summary(out_dir: &Path, written: &[String], conv: &Conv) {
    println!("wrote {} files to {}:", written.len(), out_dir.display());
    for f in written {
        println!("  {f}");
    }
    let unknown = conv.unknown.borrow();
    if unknown.is_empty() {
        println!("unknown elements: none");
    } else {
        println!("unknown elements: {unknown:?}");
    }
}
