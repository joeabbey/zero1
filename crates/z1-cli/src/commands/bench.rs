use std::fs;
use std::path::{Path, PathBuf};
use std::process::Command;
use std::time::Instant;

use anyhow::{bail, Context, Result};
use chrono::Utc;
use clap::Args;
use serde::Serialize;
use z1_ctx::{estimate_cell_with_config, EstimateConfig};
use z1_fmt::{FmtOptions, Mode};
use z1_hash::module_hashes;

const DEFAULT_CELL: &str = "fixtures/cells/http_server.z1c";
const DEFAULT_OUTPUT: &str = "benchmarks/latest.json";
const OUTPUT_TRIM_BYTES: usize = 2000;

#[derive(Debug, Args)]
pub struct BenchArgs {
    /// Path to the canonical benchmark cell.
    #[arg(long, default_value = DEFAULT_CELL)]
    pub cell: String,
    /// Destination JSON file for benchmark results.
    #[arg(long, default_value = DEFAULT_OUTPUT)]
    pub output: String,
    /// Continue running even if a command fails.
    #[arg(long)]
    pub continue_on_error: bool,
}

#[derive(Debug, Serialize)]
struct BenchReport {
    meta: MetaSection,
    commands: Vec<CommandReport>,
}

#[derive(Debug, Serialize)]
struct MetaSection {
    git_head: String,
    timestamp: String,
    cell: String,
    cell_metrics: CellMetrics,
    hashes: HashMetrics,
    context: ContextMetrics,
}

#[derive(Debug, Serialize)]
struct CellMetrics {
    fmt_mode: &'static str,
    fmt_clean: bool,
    compact_chars: usize,
    compact_bytes: usize,
    relaxed_chars: usize,
    relaxed_bytes: usize,
    compression_ratio: Option<f64>,
}

#[derive(Debug, Serialize)]
struct HashMetrics {
    semhash: String,
    formhash: String,
}

#[derive(Debug, Serialize)]
struct ContextMetrics {
    total_tokens: u32,
    budget: Option<u32>,
    usage_percent: Option<f64>,
    char_count: usize,
    functions: Vec<FnContextMetrics>,
}

#[derive(Debug, Serialize)]
struct FnContextMetrics {
    name: String,
    tokens: u32,
    chars: usize,
}

#[derive(Debug, Serialize)]
struct CommandReport {
    label: String,
    command: String,
    duration_s: f64,
    exit_code: Option<i32>,
    success: bool,
    stdout: Option<String>,
    stderr: Option<String>,
}

pub fn run(args: BenchArgs) -> Result<()> {
    let repo_root = std::env::current_dir().context("determine current directory")?;
    let cell_path = resolve_path(&args.cell, &repo_root);
    let output_path = resolve_path(&args.output, &repo_root);

    if !cell_path.exists() {
        bail!("benchmark cell not found at {}", cell_path.display());
    }

    let source = fs::read_to_string(&cell_path)
        .with_context(|| format!("failed to read {}", cell_path.display()))?;
    let module = z1_parse::parse_module(&source)?;

    let fmt_options = FmtOptions::default();
    let compact_text = z1_fmt::format_module(&module, Mode::Compact, &fmt_options)?;
    let relaxed_text = z1_fmt::format_module(&module, Mode::Relaxed, &fmt_options)?;

    let fmt_mode = detect_format_mode(&cell_path);
    let fmt_reference = match fmt_mode {
        Mode::Compact => &compact_text,
        Mode::Relaxed => &relaxed_text,
    };
    let fmt_clean = normalize_newlines(fmt_reference) == normalize_newlines(&source);

    let cell_metrics = CellMetrics::new(fmt_mode, fmt_clean, &compact_text, &relaxed_text);

    let hashes = module_hashes(&module);
    let hash_metrics = HashMetrics {
        semhash: hashes.semantic,
        formhash: hashes.format,
    };

    let ctx_config = EstimateConfig {
        enforce_budget: false,
        ..EstimateConfig::default()
    };
    let ctx_estimate = estimate_cell_with_config(&module, &ctx_config)?;
    let context_metrics = ContextMetrics::from_estimate(&ctx_estimate);

    let mut commands = Vec::new();
    let command_specs: &[(&str, &[&str])] = &[
        ("cargo fmt", &["cargo", "fmt", "--all"]),
        (
            "cargo clippy",
            &[
                "cargo",
                "clippy",
                "--workspace",
                "--all-targets",
                "--all-features",
                "--",
                "-D",
                "warnings",
            ],
        ),
        (
            "cargo test",
            &["cargo", "test", "--workspace", "--all-targets"],
        ),
    ];

    for (label, args_vec) in command_specs {
        let report = run_shell_command(label, args_vec, &repo_root)?;
        let success = report.success;
        commands.push(report);
        if !success && !args.continue_on_error {
            bail!(
                "command '{label}' failed (rerun with --continue-on-error to collect remaining metrics)"
            );
        }
    }

    let git_head = git_rev_parse(&repo_root)?;
    let timestamp = Utc::now().to_rfc3339();
    let cell_display = relative_display(&cell_path, &repo_root);

    let meta = MetaSection {
        git_head,
        timestamp,
        cell: cell_display,
        cell_metrics,
        hashes: hash_metrics,
        context: context_metrics,
    };

    let report = BenchReport { meta, commands };

    if let Some(parent) = output_path.parent() {
        fs::create_dir_all(parent)
            .with_context(|| format!("failed to create {}", parent.display()))?;
    }

    let file = fs::File::create(&output_path)
        .with_context(|| format!("failed to create {}", output_path.display()))?;
    serde_json::to_writer_pretty(file, &report)
        .with_context(|| format!("failed to write {}", output_path.display()))?;

    println!(
        "[bench] wrote {}",
        relative_display(&output_path, &repo_root)
    );

    Ok(())
}

impl CellMetrics {
    fn new(fmt_mode: Mode, fmt_clean: bool, compact: &str, relaxed: &str) -> Self {
        let compact_chars = compact.chars().count();
        let relaxed_chars = relaxed.chars().count();
        let compact_bytes = compact.len();
        let relaxed_bytes = relaxed.len();
        let compression_ratio = if compact_bytes > 0 {
            Some((relaxed_bytes as f64 / compact_bytes as f64 * 10_000.0).round() / 10_000.0)
        } else {
            None
        };
        Self {
            fmt_mode: mode_label(fmt_mode),
            fmt_clean,
            compact_chars,
            compact_bytes,
            relaxed_chars,
            relaxed_bytes,
            compression_ratio,
        }
    }
}

impl ContextMetrics {
    fn from_estimate(estimate: &z1_ctx::CellEstimate) -> Self {
        let usage_percent = estimate.budget.map(|budget| {
            (estimate.total_tokens as f64 / budget as f64 * 10_000.0).round() / 100.0
        });
        let functions = estimate
            .functions
            .iter()
            .map(|f| FnContextMetrics {
                name: f.name.clone(),
                tokens: f.tokens,
                chars: f.chars,
            })
            .collect();
        Self {
            total_tokens: estimate.total_tokens,
            budget: estimate.budget,
            usage_percent,
            char_count: estimate.char_count,
            functions,
        }
    }
}

fn run_shell_command(label: &str, args: &[&str], root: &Path) -> Result<CommandReport> {
    let (cmd, tail) = args
        .split_first()
        .context("must provide at least one arg per command")?;
    let mut command = Command::new(cmd);
    command.current_dir(root);
    command.args(tail);
    let display_cmd = format_command_string(args);
    let started = Instant::now();
    let output = command
        .output()
        .with_context(|| format!("failed to run {label}"))?;
    let duration = started.elapsed().as_secs_f64();
    let stdout = capture_output(&output.stdout);
    let stderr = capture_output(&output.stderr);
    Ok(CommandReport {
        label: label.to_string(),
        command: display_cmd,
        duration_s: duration,
        exit_code: output.status.code(),
        success: output.status.success(),
        stdout,
        stderr,
    })
}

fn git_rev_parse(root: &Path) -> Result<String> {
    let output = Command::new("git")
        .arg("rev-parse")
        .arg("HEAD")
        .current_dir(root)
        .output()
        .context("failed to execute git rev-parse")?;
    if !output.status.success() {
        bail!("git rev-parse failed");
    }
    let text = String::from_utf8(output.stdout)?;
    Ok(text.trim().to_string())
}

fn capture_output(bytes: &[u8]) -> Option<String> {
    if bytes.is_empty() {
        return None;
    }
    let text = String::from_utf8_lossy(bytes);
    Some(truncate_output(&text))
}

fn truncate_output(text: &str) -> String {
    if text.len() <= OUTPUT_TRIM_BYTES {
        return text.to_string();
    }
    let mut end = OUTPUT_TRIM_BYTES;
    while end > 0 && !text.is_char_boundary(end) {
        end -= 1;
    }
    let omitted = text.len().saturating_sub(end);
    format!("{}...[truncated {} bytes]", &text[..end], omitted)
}

fn format_command_string(args: &[&str]) -> String {
    args.iter()
        .map(|arg| {
            if arg.chars().any(char::is_whitespace) {
                format!("{arg:?}")
            } else {
                (*arg).to_string()
            }
        })
        .collect::<Vec<_>>()
        .join(" ")
}

fn normalize_newlines(input: &str) -> String {
    input.replace("\r\n", "\n")
}

fn detect_format_mode(path: &Path) -> Mode {
    match path.extension().and_then(|ext| ext.to_str()) {
        Some("z1r") => Mode::Relaxed,
        _ => Mode::Compact,
    }
}

fn mode_label(mode: Mode) -> &'static str {
    match mode {
        Mode::Compact => "compact",
        Mode::Relaxed => "relaxed",
    }
}

fn resolve_path(input: &str, root: &Path) -> PathBuf {
    let path = PathBuf::from(input);
    if path.is_absolute() {
        path
    } else {
        root.join(path)
    }
}

fn relative_display(path: &Path, root: &Path) -> String {
    path.strip_prefix(root)
        .map(|p| p.to_string_lossy().to_string())
        .unwrap_or_else(|_| path.to_string_lossy().to_string())
}
