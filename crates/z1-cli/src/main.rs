use anyhow::Result;
use clap::{Args, Parser, Subcommand, ValueEnum};
use std::fs;
use std::io::{self, Read};
use std::path::Path;
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
    /// Format Z1 cells in compact or relaxed mode.
    #[command(alias = "z1fmt")]
    Fmt(FmtArgs),
    /// Display toolchain and provenance information.
    Info,
    /// Compute semantic + format hashes for a `.z1c`/`.z1r` cell.
    Hash {
        /// Path to the source cell.
        path: String,
    },
}

#[derive(Debug, Args)]
struct FmtArgs {
    /// Path to a `.z1c` or `.z1r` cell (omit when using --stdin).
    path: Option<String>,
    /// Run in check-only mode without writing files.
    #[arg(long)]
    check: bool,
    /// Read source contents from stdin.
    #[arg(long)]
    stdin: bool,
    /// Emit formatted output to stdout.
    #[arg(long)]
    stdout: bool,
    /// Override formatter mode.
    #[arg(long, value_enum)]
    mode: Option<FmtModeArg>,
    /// Symbol map ordering behaviour.
    #[arg(long, value_enum, default_value_t = FmtSymmapArg::Respect)]
    symmap: FmtSymmapArg,
}

#[derive(Debug, Clone, Copy, ValueEnum)]
enum FmtModeArg {
    Compact,
    Relaxed,
}

impl From<FmtModeArg> for z1_fmt::Mode {
    fn from(value: FmtModeArg) -> Self {
        match value {
            FmtModeArg::Compact => z1_fmt::Mode::Compact,
            FmtModeArg::Relaxed => z1_fmt::Mode::Relaxed,
        }
    }
}

#[derive(Debug, Clone, Copy, ValueEnum)]
enum FmtSymmapArg {
    Respect,
    Reflow,
}

impl From<FmtSymmapArg> for z1_fmt::SymMapStyle {
    fn from(value: FmtSymmapArg) -> Self {
        match value {
            FmtSymmapArg::Respect => z1_fmt::SymMapStyle::Respect,
            FmtSymmapArg::Reflow => z1_fmt::SymMapStyle::Reflow,
        }
    }
}

fn main() -> Result<()> {
    tracing_subscriber::fmt::init();
    let cli = Cli::parse();
    match cli.command {
        Commands::Fmt(args) => handle_fmt(args),
        Commands::Info => {
            info!("Zero1 CLI scaffolding is ready for agent contributions.");
            Ok(())
        }
        Commands::Hash { path } => handle_hash(path),
    }
}

fn handle_fmt(args: FmtArgs) -> Result<()> {
    let mut source = String::new();
    if args.stdin {
        io::stdin().read_to_string(&mut source)?;
    } else {
        let path = args
            .path
            .as_deref()
            .ok_or_else(|| anyhow::anyhow!("path is required unless --stdin is set"))?;
        source = fs::read_to_string(path)?;
    }

    let module = z1_parse::parse_module(&source)?;
    let mode = args
        .mode
        .map(Into::into)
        .unwrap_or_else(|| infer_mode(args.path.as_deref()));
    let options = z1_fmt::FmtOptions {
        symmap_style: args.symmap.into(),
    };
    let formatted = z1_fmt::format_module(&module, mode, &options)?;

    if args.check {
        if normalize_newlines(&formatted) != normalize_newlines(&source) {
            anyhow::bail!("formatting changes needed");
        }
        return Ok(());
    }

    let emit_stdout = args.stdout || (args.stdin && args.path.is_none());
    if emit_stdout {
        print!("{}", formatted);
    }

    if !emit_stdout {
        let path = args
            .path
            .as_deref()
            .ok_or_else(|| anyhow::anyhow!("path must be provided when not using --stdout"))?;
        fs::write(path, formatted)?;
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

fn infer_mode(path: Option<&str>) -> z1_fmt::Mode {
    if let Some(path) = path {
        if let Some(ext) = Path::new(path).extension().and_then(|ext| ext.to_str()) {
            return match ext {
                "z1r" => z1_fmt::Mode::Compact,
                _ => z1_fmt::Mode::Relaxed,
            };
        }
    }
    z1_fmt::Mode::Relaxed
}

fn normalize_newlines(input: &str) -> String {
    input.replace("\r\n", "\n")
}
