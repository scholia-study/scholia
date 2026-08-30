//! Bootstrap the peirce1 curated layer from Wikisource's proofread
//! transcriptions.
//!
//! Two steps, because fetching and converting have different failure modes and
//! the network step should not be repeated while iterating on the parse:
//!
//!   scripts/peirce1_fetch.sh                 # → assets/peirce1/raw/ (gitignored)
//!   cargo run -p peirce1_wikisource_to_md    # raw/ → curated/md_reviewed/
//!
//! `--fetch-plan` prints the scan pages to retrieve, so the fetch script does
//! not carry a second copy of the source table.
//!
//! Like hobbes1's modernized layer, the curated markdown becomes hand-maintained
//! once reviewers touch it. Two papers are hand-curated from the outset and this
//! tool never writes them (see `papers::HAND_CURATED`); regenerating the rest
//! discards any hand edits, so re-run it only against unreviewed files.

mod convert;
mod papers;

use std::fs;
use std::path::PathBuf;

use clap::Parser;

const RAW_DIR: &str = "assets/peirce1/raw";
const CURATED_DIR: &str = "assets/peirce1/curated/md_reviewed";

#[derive(Parser)]
#[command(about = "Wikisource proofread transcriptions → peirce1 curated markdown")]
struct Cli {
    /// Print the scan pages to fetch, as `INDEX<TAB>PAGE<TAB>DEST`, and exit.
    #[arg(long)]
    fetch_plan: bool,
    /// Convert only this paper (its position number, e.g. 110).
    #[arg(long)]
    position: Option<usize>,
    /// Where the fetched wikitext lives.
    #[arg(long, default_value = RAW_DIR)]
    raw_dir: String,
    /// Where the curated markdown is written.
    #[arg(long, default_value = CURATED_DIR)]
    out_dir: String,
}

fn main() {
    let cli = Cli::parse();

    let selected: Vec<&papers::Paper> = match cli.position {
        Some(p) => vec![papers::by_position(p).unwrap_or_else(|| {
            panic!(
                "no paper at position {p}; hand-curated positions are {:?}",
                papers::HAND_CURATED
            )
        })],
        None => papers::PAPERS.iter().collect(),
    };

    if cli.fetch_plan {
        for paper in &selected {
            for page in paper.from..=paper.to {
                println!(
                    "{}\t{}\t{}/{:05}/p{}.txt",
                    paper.index, page, cli.raw_dir, paper.position, page
                );
            }
        }
        return;
    }

    fs::create_dir_all(&cli.out_dir).expect("cannot create output dir");

    let mut total_residue = 0usize;
    for paper in selected {
        let dir = PathBuf::from(&cli.raw_dir).join(format!("{:05}", paper.position));
        let mut raw_pages: Vec<(u32, String)> = Vec::new();
        for page in paper.from..=paper.to {
            let path = dir.join(format!("p{page}.txt"));
            let text = fs::read_to_string(&path).unwrap_or_else(|e| {
                panic!(
                    "{}: {e} — run scripts/peirce1_fetch.sh first",
                    path.display()
                )
            });
            raw_pages.push((page, text));
        }

        let md = convert::convert(paper, &raw_pages);
        debug_assert!(
            !md.contains("]:\n[^"),
            "footnote definitions must be separated"
        );
        let out = PathBuf::from(&cli.out_dir).join(paper.filename);
        fs::write(&out, &md).unwrap_or_else(|e| panic!("cannot write {}: {e}", out.display()));

        let markers = md.matches("{{{").count();
        let notes = md.matches("\n[^").count();
        // Any `{{…}}` left after conversion is an unhandled template. It must
        // not reach the parser, which would read it as a secondary page marker
        // — a corpus with no secondary system, so the junk lands in a nameless
        // reference system instead of failing.
        let stripped = md.replace("{{{", "");
        let residue = stripped.matches("{{").count()
            // Unconverted wiki emphasis, links, section headings, and
            // definition-list indents are residue too — each has surfaced in
            // the reader as literal markup once already.
            + stripped.matches("''").count()
            + stripped.matches("[[").count()
            + md.lines().filter(|l| l.starts_with("==") || l.starts_with("::")).count();
        total_residue += residue;
        eprintln!(
            "{:>3}  {:<52} {:>2} scan pages → {:>2} markers, {:>2} footnotes{}",
            paper.position,
            paper.label,
            raw_pages.len(),
            markers,
            notes,
            if residue > 0 {
                format!(", {residue} UNHANDLED TEMPLATES")
            } else {
                String::new()
            },
        );
    }

    if total_residue > 0 {
        eprintln!(
            "\n{total_residue} unhandled wikitext templates remain — these files are NOT \
             ready to import."
        );
        std::process::exit(1);
    }
}
