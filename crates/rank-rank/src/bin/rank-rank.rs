use clap::{Parser, Subcommand};
use rank_rank::prelude::*;

#[derive(Parser)]
#[command(author, version, about, long_about = None)]
struct Cli {
    #[command(subcommand)]
    command: Commands,
}

#[derive(Subcommand)]
enum Commands {
    /// Run a search query against an index (placeholder)
    Search {
        query: String,
    },
    /// Evaluate a pipeline configuration
    Eval,
}

fn main() {
    let cli = Cli::parse();

    match &cli.command {
        Commands::Search { query } => {
            println!("Searching for: {}", query);
            println!("(Pipeline implementation coming soon)");
        }
        Commands::Eval => {
            println!("Running evaluation...");
        }
    }
}
