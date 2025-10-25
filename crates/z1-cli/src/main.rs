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
    /// Paths to `.z1c` / `.z1r` cells. Omit when using --stdin/--files-from.
    #[arg(value_name = "PATH", num_args = 0..)]
    paths: Vec<String>,
    /// Read additional newline-delimited paths from file.
    #[arg(long = "files-from")]
    files_from: Option<String>,
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
    let mut targets = args.paths.clone();
    if let Some(list_path) = &args.files_from {
        targets.extend(read_file_list(list_path)?);
    }

    if args.stdin {
        if !targets.is_empty() {
            anyhow::bail!("--stdin cannot be combined with positional paths or --files-from");
        }
        if !args.stdout && !args.check {
            anyhow::bail!("--stdin requires --stdout or --check");
        }
        format_stream(&args)?;
        return Ok(());
    }

    if targets.is_empty() {
        anyhow::bail!("provide at least one path, --files-from file, or --stdin");
    }

    if args.stdout && (args.check || targets.len() > 1) {
        anyhow::bail!("--stdout only supported for single file without --check");
    }

    let mut changes_needed = false;
    for path in targets {
        let changed = format_file(&path, &args)?;
        changes_needed |= changed;
    }

    if args.check && changes_needed {
        anyhow::bail!("formatting changes needed");
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

fn read_file_list(path: &str) -> Result<Vec<String>> {
    let contents = fs::read_to_string(path)?;
    Ok(contents
        .lines()
        .map(|line| line.trim())
        .filter(|line| !line.is_empty() && !line.starts_with('#'))
        .map(|line| line.to_string())
        .collect())
}

fn format_stream(args: &FmtArgs) -> Result<()> {
    let mut source = String::new();
    io::stdin().read_to_string(&mut source)?;
    let mode = args.mode.map(Into::into).unwrap_or(z1_fmt::Mode::Relaxed);
    let options = z1_fmt::FmtOptions {
        symmap_style: args.symmap.into(),
    };
    let module = z1_parse::parse_module(&source)?;
    let formatted = z1_fmt::format_module(&module, mode, &options)?;
    if args.check {
        if normalize_newlines(&formatted) != normalize_newlines(&source) {
            anyhow::bail!("formatting changes needed");
        }
        return Ok(());
    }
    print!("{formatted}");
    Ok(())
}

fn format_file(path: &str, args: &FmtArgs) -> Result<bool> {
    let source = fs::read_to_string(path)?;
    let mode = args
        .mode
        .map(Into::into)
        .unwrap_or_else(|| infer_mode(Some(path)));
    let options = z1_fmt::FmtOptions {
        symmap_style: args.symmap.into(),
    };
    let module = z1_parse::parse_module(&source)?;
    let formatted = z1_fmt::format_module(&module, mode, &options)?;
    let changed = normalize_newlines(&formatted) != normalize_newlines(&source);
    if args.check {
        return Ok(changed);
    }
    if args.stdout {
        print!("{formatted}");
        return Ok(changed);
    }
    if changed {
        fs::write(path, formatted)?;
    }
    Ok(changed)
}
