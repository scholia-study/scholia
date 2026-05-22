mod import;

use clap::Parser;

#[derive(Parser)]
#[command(about = "Import structured Kant KrV JSON into PostgreSQL")]
struct Cli {
    /// Input JSON file (output of kant1_md_to_struct)
    #[arg(long, default_value = "assets/kant1/derived/md_to_struct/output.json")]
    input_file: String,

    /// PostgreSQL connection URL (overrides DATABASE_URL env var)
    #[arg(long)]
    database_url: Option<String>,

    /// Source book slug (for translation imports — links to existing book)
    #[arg(long)]
    source_book_slug: Option<String>,
}

fn main() {
    let cli = Cli::parse();

    let rt = tokio::runtime::Runtime::new().expect("Failed to create tokio runtime");
    rt.block_on(async {
        if let Err(e) = import::run(&cli.input_file, cli.database_url, cli.source_book_slug).await {
            eprintln!("Import failed: {e}");
            std::process::exit(1);
        }
    });
}
