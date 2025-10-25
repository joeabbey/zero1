use anyhow::Result;
use clap::{Parser, Subcommand};
use std::fs;
use tracing::info;

/// Zero1 CLI entry point. Commands are stubs until the corresponding crates land.
#[derive(Parser, Debug)]
#[command(name = "z1", author = "Zero1 Contributors", version)]
struct Cli {
    #[command(subcommand)]
    command: Commands,
}

#[derive(Subcommand, Debug)]
enum Commands {
    /// Placeholder for the future formatter pipeline.
    Fmt {
        /// Path to a `.z1c` or `.z1r` cell.
        path: String,
        /// Run in check-only mode without writing files.
        #[arg(long)]
        check: bool,
    },
    /// Display toolchain and provenance information.
    Info,
    /// Compute semantic + format hashes for a `.z1c`/`.z1r` cell.
    Hash {
        /// Path to the source cell.
        path: String,
    },
}

fn main() -> Result<()> {
    tracing_subscriber::fmt::init();
    let cli = Cli::parse();
    match cli.command {
        Commands::Fmt { path, check } => handle_fmt(path, check),
        Commands::Info => {
            info!("Zero1 CLI scaffolding is ready for agent contributions.");
            Ok(())
        }
        Commands::Hash { path } => handle_hash(path),
    }
}

fn handle_fmt(path: String, check: bool) -> Result<()> {
    info!(
        action = "fmt",
        %path,
        check,
        "formatter stub invoked; implement in crates/z1-fmt"
    );
    if !check {
        // In a future milestone this will dispatch into the formatter crate.
        info!("no formatting performed (stub)");
    }
    Ok(())
}

fn handle_hash(path: String) -> Result<()> {
    let source = fs::read_to_string(&path)?;
    let module = z1_parse::parse_module(&source)?;
    let hashes = z1_hash::module_hashes(&module);
    println!("semhash: {}", hashes.semantic);
    println!("formhash: {}", hashes.format);
    Ok(())
}
