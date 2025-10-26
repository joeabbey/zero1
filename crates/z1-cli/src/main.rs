mod commands;

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
    /// Estimate context token usage for a cell.
    #[command(alias = "z1ctx")]
    Ctx(CtxArgs),
    /// Provenance chain management and verification.
    #[command(alias = "z1prov", subcommand)]
    Prov(commands::prov::ProvCommand),
    /// Run Z1 test files (.z1t).
    #[command(alias = "z1test")]
    Test(TestArgs),
    /// Run the benchmark harness.
    #[command(alias = "z1bench")]
    Bench(commands::bench::BenchArgs),
    /// Compile Z1 cell to target language.
    #[command(alias = "z1c")]
    Compile(CompileArgs),
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

#[derive(Debug, Args)]
struct CtxArgs {
    /// Path to the source cell to estimate.
    path: String,
    /// Custom characters-per-token ratio (default: 3.8).
    #[arg(long)]
    chars_per_token: Option<f64>,
    /// Skip budget enforcement (only show estimates).
    #[arg(long)]
    no_enforce: bool,
    /// Show detailed per-function breakdown.
    #[arg(long, short = 'v')]
    verbose: bool,
}

#[derive(Debug, Args)]
struct TestArgs {
    /// Paths to `.z1t` test files.
    paths: Vec<String>,
    /// Filter tests by tags (comma-separated).
    #[arg(long)]
    tags: Option<String>,
    /// Show verbose output.
    #[arg(long, short = 'v')]
    verbose: bool,
}

#[derive(Debug, Args)]
struct CompileArgs {
    /// Path to Z1 cell to compile
    path: String,
    /// Output file path (default: same name with target extension)
    #[arg(short, long)]
    output: Option<String>,
    /// Compilation target
    #[arg(short, long, value_enum, default_value_t = CompileTargetArg::TypeScript)]
    target: CompileTargetArg,
    /// Run all checks before compilation
    #[arg(long, default_value_t = true)]
    check: bool,
    /// Emit IR instead of target code
    #[arg(long)]
    emit_ir: bool,
    /// Verbose output
    #[arg(short, long)]
    verbose: bool,
}

#[derive(Debug, Clone, Copy, ValueEnum)]
enum CompileTargetArg {
    TypeScript,
    Wasm,
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
        Commands::Ctx(args) => handle_ctx(args),
        Commands::Prov(cmd) => handle_prov(cmd),
        Commands::Test(args) => handle_test(args),
        Commands::Bench(args) => commands::bench::run(args),
        Commands::Compile(args) => handle_compile(args),
    }
}

fn handle_compile(args: CompileArgs) -> Result<()> {
    let target = match args.target {
        CompileTargetArg::TypeScript => commands::compile::CompileTarget::TypeScript,
        CompileTargetArg::Wasm => commands::compile::CompileTarget::Wasm,
    };

    let opts = commands::compile::CompileOptions {
        input_path: args.path.into(),
        output_path: args.output.map(Into::into),
        target,
        check: args.check,
        emit_ir: args.emit_ir,
        verbose: args.verbose,
    };

    commands::compile::compile(opts)
}

fn handle_prov(cmd: commands::prov::ProvCommand) -> Result<()> {
    use commands::prov::ProvCommand;
    match cmd {
        ProvCommand::Log { file } => commands::prov::cmd_log(file),
        ProvCommand::Verify { file, keys } => commands::prov::cmd_verify(file, keys),
        ProvCommand::Keygen { output } => commands::prov::cmd_keygen(output),
    }
}

fn handle_test(args: TestArgs) -> Result<()> {
    if args.paths.is_empty() {
        anyhow::bail!("provide at least one .z1t test file");
    }

    // Parse tag filters if provided
    let tags_include = if let Some(tags) = &args.tags {
        tags.split(',').map(|s| s.trim().to_string()).collect()
    } else {
        vec![]
    };

    let config = z1_test::TestConfig {
        tags_include,
        ..Default::default()
    };

    let mut runner = z1_test::TestRunner::new(config);
    let mut total_passed = 0;
    let mut total_failed = 0;
    let mut total_skipped = 0;
    let mut all_failures = Vec::new();

    for path in &args.paths {
        println!("Running tests from: {path}");
        let source = fs::read_to_string(path)?;
        let file = z1_test::parse_test_file(&source)
            .map_err(|e| anyhow::anyhow!("Failed to parse {path}: {e}"))?;

        let results = runner.run_file(&file);

        total_passed += results.passed;
        total_failed += results.failed;
        total_skipped += results.skipped;

        if args.verbose {
            for failure in &results.failures {
                println!("  FAILED: {} - {}", failure.name, failure.error);
            }
        }

        all_failures.extend(results.failures);
    }

    println!("\nTest Results:");
    println!("  Passed:  {total_passed}");
    println!("  Failed:  {total_failed}");
    println!("  Skipped: {total_skipped}");

    if !all_failures.is_empty() {
        println!("\nFailures:");
        for failure in all_failures {
            println!("  - {}: {}", failure.name, failure.error);
        }
        std::process::exit(1);
    }

    Ok(())
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

fn handle_ctx(args: CtxArgs) -> Result<()> {
    let source = fs::read_to_string(&args.path)?;
    let module = z1_parse::parse_module(&source)?;

    let config = z1_ctx::EstimateConfig {
        chars_per_token: args
            .chars_per_token
            .unwrap_or(z1_ctx::DEFAULT_CHARS_PER_TOKEN),
        enforce_budget: !args.no_enforce,
    };

    match z1_ctx::estimate_cell_with_config(&module, &config) {
        Ok(estimate) => {
            if args.verbose {
                println!("{estimate}");
            } else {
                println!("Estimated tokens: {}", estimate.total_tokens);
                if let Some(budget) = estimate.budget {
                    let percentage = (estimate.total_tokens as f64 / budget as f64) * 100.0;
                    println!("Budget: {budget} ({percentage:.1}% used)");
                    if estimate.total_tokens <= budget {
                        println!("Status: OK (within budget)");
                    } else {
                        println!("Status: EXCEEDS BUDGET");
                    }
                }
            }
            Ok(())
        }
        Err(e) => {
            eprintln!("Context estimation failed: {e}");
            std::process::exit(1);
        }
    }
}
