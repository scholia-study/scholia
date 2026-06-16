mod import;

use clap::Parser;

#[derive(Parser)]
#[command(about = "Import the Shakespeare sonnets struct JSON into PostgreSQL")]
struct Cli {
    /// Input struct JSON (output of shakespeare1_md_to_struct).
    #[arg(long, default_value = "assets/shakespeare1/derived/output.json")]
    input_file: String,

    /// PostgreSQL connection URL (overrides POSTGRES_*/DATABASE_URL env).
    #[arg(long)]
    database_url: Option<String>,

    /// Delete an existing book with the same slug (cascading) before importing.
    #[arg(long)]
    replace: bool,

    /// Insert then roll back — validate without committing.
    #[arg(long)]
    dry_run: bool,
}

fn main() {
    let cli = Cli::parse();
    let rt = tokio::runtime::Runtime::new().expect("create tokio runtime");
    rt.block_on(async {
        if let Err(e) =
            import::run(&cli.input_file, cli.database_url, cli.replace, cli.dry_run).await
        {
            eprintln!("Import failed: {e}");
            std::process::exit(1);
        }
    });
}
