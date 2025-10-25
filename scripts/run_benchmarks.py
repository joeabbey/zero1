#!/usr/bin/env python3
"""
Zero1 benchmarking harness.

This utility captures repeatable metrics that reflect Zero1's goals:
token efficiency, deterministic hashing, and toolchain throughput.
It produces a JSON report that downstream dashboards/agents can diff.
"""

from __future__ import annotations

import argparse
import json
import re
import shlex
import subprocess
import sys
import time
from dataclasses import asdict, dataclass
from datetime import datetime, timezone
from pathlib import Path
from typing import Dict, List, Optional

REPO_ROOT = Path(__file__).resolve().parents[1]
DEFAULT_CELL = Path("fixtures/cells/http_server.z1c")
TRUNCATE_LIMIT = 2000


@dataclass
class CommandResult:
    label: str
    command: str
    duration_s: float
    returncode: int
    stdout: Optional[str] = None
    stderr: Optional[str] = None


def run_command(
    label: str,
    args: List[str],
    capture_output: bool = False,
) -> CommandResult:
    """Run a subprocess relative to the repo root and measure duration."""
    start = time.perf_counter()
    result = subprocess.run(
        args,
        cwd=REPO_ROOT,
        capture_output=capture_output,
        text=True,
        check=False,
    )
    duration = time.perf_counter() - start
    cmd_str = " ".join(shlex.quote(part) for part in args)
    entry = CommandResult(
        label=label,
        command=cmd_str,
        duration_s=duration,
        returncode=result.returncode,
        stdout=result.stdout if capture_output else None,
        stderr=result.stderr if capture_output else None,
    )
    if result.returncode != 0:
        if capture_output:
            sys.stderr.write(result.stdout or "")
            sys.stderr.write(result.stderr or "")
        raise subprocess.CalledProcessError(result.returncode, args)
    return entry


def truncate(text: Optional[str], limit: int = TRUNCATE_LIMIT) -> Optional[str]:
    if text is None:
        return None
    if len(text) <= limit:
        return text
    trimmed = text[:limit]
    omitted = len(text) - limit
    return f"{trimmed}\n...[truncated {omitted} chars]"


def git_head() -> str:
    result = subprocess.run(
        ["git", "rev-parse", "HEAD"],
        cwd=REPO_ROOT,
        capture_output=True,
        text=True,
        check=True,
    )
    return result.stdout.strip()


def parse_hash_output(output: str) -> Dict[str, str]:
    hashes: Dict[str, str] = {}
    for line in output.splitlines():
        parts = line.split(":", 1)
        if len(parts) != 2:
            continue
        key = parts[0].strip()
        value = parts[1].strip()
        if key in {"semhash", "formhash"}:
            hashes[key] = value
    return hashes


def parse_ctx_output(output: str) -> Dict[str, object]:
    metrics: Dict[str, object] = {"raw": output}
    functions: List[Dict[str, object]] = []
    fn_pattern = re.compile(
        r"-\s*(?P<name>[^:]+):\s*(?P<tokens>\d+)\s+tokens\s+\((?P<chars>\d+)\s+chars\)"
    )
    for line in output.splitlines():
        stripped = line.strip()
        if stripped.startswith("Total tokens:"):
            value = stripped.split(":", 1)[1].strip()
            if value:
                metrics["total_tokens"] = int(value)
        elif stripped.startswith("Budget:"):
            value = stripped.split(":", 1)[1].strip()
            if value:
                metrics["budget"] = int(value)
        elif stripped.startswith("Usage:"):
            usage = stripped.split(":", 1)[1].strip().rstrip("%")
            try:
                metrics["usage_percent"] = float(usage)
            except ValueError:
                pass
        elif stripped.startswith("Characters:"):
            metrics["char_count"] = int(stripped.split(":", 1)[1].strip())
        else:
            fn_match = fn_pattern.match(stripped)
            if fn_match:
                functions.append(
                    {
                        "name": fn_match.group("name").strip(),
                        "tokens": int(fn_match.group("tokens")),
                        "chars": int(fn_match.group("chars")),
                    }
                )
    if functions:
        metrics["functions"] = functions
    return metrics


def ensure_cell(cell: Path) -> Path:
    absolute = (REPO_ROOT / cell).resolve()
    if not absolute.exists():
        raise FileNotFoundError(f"Cell {cell} not found (expected at {absolute})")
    return absolute


def main() -> None:
    parser = argparse.ArgumentParser(description="Run Zero1 benchmark suite.")
    parser.add_argument(
        "--cell",
        default=str(DEFAULT_CELL),
        help="Path to the canonical benchmark cell (default: fixtures/cells/http_server.z1c)",
    )
    parser.add_argument(
        "--output",
        default="benchmarks/latest.json",
        help="Destination JSON report (default: benchmarks/latest.json)",
    )
    args = parser.parse_args()

    cell_path = ensure_cell(Path(args.cell))
    output_path = (REPO_ROOT / args.output).resolve()
    output_path.parent.mkdir(parents=True, exist_ok=True)

    command_results: List[CommandResult] = []

    # Toolchain throughput
    command_results.append(run_command("cargo fmt", ["cargo", "fmt", "--all"]))
    command_results.append(
        run_command(
            "cargo clippy",
            [
                "cargo",
                "clippy",
                "--workspace",
                "--all-targets",
                "--all-features",
                "--",
                "-D",
                "warnings",
            ],
        )
    )
    command_results.append(
        run_command(
            "cargo test",
            [
                "cargo",
                "test",
                "--workspace",
                "--all-targets",
            ],
        )
    )

    # Formatting invariants and relaxed projection
    command_results.append(
        run_command(
            "z1 fmt --check",
            ["cargo", "run", "-p", "z1-cli", "--", "fmt", str(cell_path), "--check"],
        )
    )
    relaxed_result = run_command(
        "z1 fmt relaxed",
        [
            "cargo",
            "run",
            "-p",
            "z1-cli",
            "--",
            "fmt",
            str(cell_path),
            "--mode",
            "relaxed",
            "--stdout",
        ],
        capture_output=True,
    )

    # Hashes and context estimates
    hash_result = run_command(
        "z1 hash",
        ["cargo", "run", "-p", "z1-cli", "--", "hash", str(cell_path)],
        capture_output=True,
    )
    ctx_result = run_command(
        "z1 ctx",
        [
            "cargo",
            "run",
            "-p",
            "z1-cli",
            "--",
            "ctx",
            str(cell_path),
            "--verbose",
            "--no-enforce",
        ],
        capture_output=True,
    )

    compact_text = cell_path.read_text()
    relaxed_text = relaxed_result.stdout or ""

    compact_bytes = len(compact_text.encode("utf-8"))
    relaxed_bytes = len(relaxed_text.encode("utf-8"))

    metrics = {
        "git_head": git_head(),
        "timestamp": datetime.now(timezone.utc).isoformat(),
        "cell": str(cell_path.relative_to(REPO_ROOT)),
        "cell_metrics": {
            "compact_chars": len(compact_text),
            "compact_bytes": compact_bytes,
            "relaxed_chars": len(relaxed_text),
            "relaxed_bytes": relaxed_bytes,
            "compression_ratio": (
                round(relaxed_bytes / compact_bytes, 4) if compact_bytes else None
            ),
        },
        "hashes": parse_hash_output(hash_result.stdout or ""),
        "context": parse_ctx_output(ctx_result.stdout or ""),
    }

    report = {
        "meta": metrics,
        "commands": [],
    }

    for result in command_results + [relaxed_result, hash_result, ctx_result]:
        payload = asdict(result)
        payload["stdout"] = truncate(payload.get("stdout"))
        payload["stderr"] = truncate(payload.get("stderr"))
        report["commands"].append(payload)

    with output_path.open("w", encoding="utf-8") as fh:
        json.dump(report, fh, indent=2)
        fh.write("\n")

    try:
        rel_path = output_path.relative_to(REPO_ROOT)
    except ValueError:
        rel_path = output_path
    print(f"[bench] wrote {rel_path}")


if __name__ == "__main__":
    main()
