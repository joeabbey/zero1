# Zero1 Benchmark Harness

Zero1’s value proposition lives in context efficiency, deterministic semantics, and safe agent tooling. The benchmark harness (`z1 bench`) packages those guarantees into a repeatable scorecard we can trend over time.

## What we measure

| Pillar | Metric | Source |
| --- | --- | --- |
| Context efficiency | Compact vs relaxed char/byte counts, compression ratio, context estimate + budget usage | `z1-fmt`, `z1-ctx` |
| Deterministic semantics | SemHash/FormHash pair, format-clean check for the target cell | `z1-hash`, `z1-fmt` |
| Toolchain throughput | Wall-clock durations + exit codes for `cargo fmt`, `cargo clippy -- -D warnings`, `cargo test` | Subprocess timers |

The default benchmark cell is `fixtures/cells/http_server.z1c`, but any cell can be supplied to probe other packs.

## Running the harness

```bash
# Fast local smoke run (fails on the first non-zero command)
cargo run -p z1-cli -- bench

# Capture a full artifact per commit, even if the repo currently fails clippy/tests
SHA=$(git rev-parse --short HEAD)
cargo run -p z1-cli -- bench \
  --cell fixtures/cells/http_server.z1c \
  --output benchmarks/$SHA.json \
  --continue-on-error
```

Flags:

- `--cell <path>` – benchmark a specific `.z1c`/`.z1r` cell (default: `fixtures/cells/http_server.z1c`)
- `--output <path>` – JSON destination (default: `benchmarks/latest.json`)
- `--continue-on-error` – keep running even if `fmt`/`clippy`/`test` fail; failures are still recorded in the report

The command assumes you run it from the workspace root so relative paths resolve correctly.

## Report format

`z1 bench` emits JSON with two top-level keys:

```jsonc
{
  "meta": {
    "git_head": "9f55d2c6df8f8a9b16e921da8fc2a55719f85986",
    "timestamp": "2024-04-05T19:13:24.918Z",
    "cell": "fixtures/cells/http_server.z1c",
    "cell_metrics": {
      "fmt_mode": "compact",
      "fmt_clean": true,
      "compact_bytes": 272,
      "relaxed_bytes": 412,
      "compression_ratio": 1.515,
      "...": "..."
    },
    "hashes": { "semhash": "...", "formhash": "..." },
    "context": {
      "total_tokens": 84,
      "budget": 128,
      "usage_percent": 65.62,
      "functions": [
        { "name": "handler", "tokens": 26 },
        { "name": "serve", "tokens": 24 }
      ]
    }
  },
  "commands": [
    {
      "label": "cargo fmt",
      "command": "cargo fmt --all",
      "duration_s": 0.54,
      "exit_code": 0,
      "success": true
    },
    {
      "label": "cargo clippy",
      "command": "cargo clippy --workspace --all-targets --all-features -- -D warnings",
      "duration_s": 18.12,
      "exit_code": 1,
      "success": false,
      "stderr": "...output truncated..."
    }
  ]
}
```

- `meta.cell_metrics` confirms compact↔relaxed drift, providing a quick sanity check on formatter regressions.
- `meta.hashes` anchors SemHash/FormHash so provenance tooling can see when a benchmark run introduces semantic churn.
- `meta.context` dumps the estimator results (total tokens, declared budget, per-function costs) to show whether edits push cells near their limits.
- `commands` captures timing/exit status plus truncated stdout/stderr for each toolchain command. Failures turn `success` to `false` but the harness only stops early when `--continue-on-error` is not set.

## Using the data

1. **Trend detection** – Commit each benchmark JSON under `benchmarks/<git-sha>.json` to build a historical series. Any spike in compression ratio or usage percent indicates a regression in context efficiency.
2. **Release gates** – CI can diff the last successful JSON against a PR’s output to ensure SemHash/FormHash, budget usage, and toolchain timings stay within agreed thresholds.
3. **Diagnostics** – Because command outputs are truncated but preserved, we can quickly see why `clippy`/`test` failed without rerunning the suite.

When adding new benchmark packs or metrics, document them here so every agent understands what the numbers mean and how to reproduce them.
