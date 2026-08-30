mod canonical;
mod import;
mod reconcile_input;

use clap::Parser;

#[derive(Parser)]
#[command(about = "Import a structured-text struct JSON into PostgreSQL")]
struct Cli {
    /// Input struct JSON (output of a *_md_to_struct parser).
    #[arg(long)]
    input_file: String,

    /// PostgreSQL connection URL (overrides POSTGRES_*/DATABASE_URL env).
    #[arg(long)]
    database_url: Option<String>,

    /// Import as a translation edition locked 1:1 to this existing source book
    /// (slug). Each sentence links to its source counterpart by natural key for
    /// quotation projection + side-by-side alignment. Omit for a standalone book.
    #[arg(long)]
    source_book_slug: Option<String>,

    /// Delete an existing book with the same slug (cascading) and re-insert
    /// fresh, instead of reconciling it in place.
    #[arg(long)]
    replace: bool,

    /// Insert/reconcile then roll back — validate without committing.
    #[arg(long)]
    dry_run: bool,

    /// Permit deleting a sentence that still has quotations/resources anchored
    /// to it (otherwise such a delete aborts the run). Reconcile path only.
    #[arg(long)]
    force: bool,

    /// Permit inserting nodes mid-book (a growing selection like peirce1).
    /// Existing book-global numbering (paragraph/figure numbers, footnote and
    /// margin-note numbers) is renumbered to make room; body-sentence identity
    /// and anchored quotations are untouched. Off by default: for fixed corpora
    /// a mid-book structural addition is always a mistake, and the strict abort
    /// is the corruption detector. Reconcile path only.
    #[arg(long)]
    allow_insertion: bool,

    /// Bypass content-hash checks: treat every node as changed, force-rewrite
    /// every sentence (bumping updated_at), always renumber, and rewrite all
    /// stored hashes. The escape hatch when hashes may be stale or after a
    /// hash-format change. Reconcile path only.
    #[arg(long)]
    full_rewrite: bool,
}

fn main() {
    let cli = Cli::parse();
    let rt = tokio::runtime::Runtime::new().expect("create tokio runtime");
    rt.block_on(async {
        if let Err(e) = import::run(
            &cli.input_file,
            cli.database_url,
            cli.source_book_slug,
            cli.replace,
            cli.dry_run,
            cli.force,
            cli.full_rewrite,
            cli.allow_insertion,
        )
        .await
        {
            eprintln!("Import failed: {e}");
            std::process::exit(1);
        }
    });
}
