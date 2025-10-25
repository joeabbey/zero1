# CLAUDE.md

This file provides guidance to Claude Code (claude.ai/code) when working with code in this repository.

## Project Overview

Zero1 (Z1) is a Rust-based toolchain and language designed for LLM agent workflows. It features dual compact/relaxed syntax, strict capability and context budgets, deterministic hashing for provenance, and code generation to TypeScript/WASM targets.

Key design principles:
- **Dual syntax**: `.z1c` (compact) and `.z1r` (relaxed) are views of the same AST
- **SemHash invariance**: Semantic hash excludes formatting, SymbolMap, and comments
- **FormHash tracking**: Detects format-only changes by including SymbolMap
- **Append-only provenance**: Ed25519-signed chain tracks all cell modifications
- **Context budgets**: Hard limits on token counts per function/cell to keep LLM context manageable
- **Capabilities**: Fine-grained permissions (net, fs.ro, fs.rw, time, crypto, env, unsafe)

## Repository Structure

```
zero1/
├── crates/              # Rust workspace with 11 crates
│   ├── z1-ast/          # Canonical AST structures (long names only)
│   ├── z1-lex/          # Lexer with dual keyword support
│   ├── z1-parse/        # Parser producing AST
│   ├── z1-fmt/          # Compact ↔ relaxed formatter
│   ├── z1-hash/         # SemHash (semantics) + FormHash (format)
│   ├── z1-typeck/       # Structural type checker
│   ├── z1-effects/      # Effect/capability checker
│   ├── z1-prov/         # Provenance store + Merkle roots
│   ├── z1-codegen-ts/   # TypeScript code generation
│   ├── z1-codegen-wasm/ # WASM code generation
│   └── z1-cli/          # CLI entry point
├── docs/                # Specifications and design docs
│   ├── design.md        # Build plan + skeleton
│   ├── grammar.md       # Complete EBNF grammar
│   ├── vision.md        # High-level goals
│   └── dsl/             # Manifest/test DSL specs
├── fixtures/            # Test fixtures
│   ├── cells/           # Sample Z1 cells
│   └── fmt/             # Formatter test cases
├── plan.md              # Implementation roadmap + milestones
└── AGENTS.md            # Repository guidelines for agents
```

## Build and Development Commands

### Core workflow
```bash
# Format all Rust code
cargo fmt --all

# Run linter (fail on warnings)
cargo clippy --workspace --all-targets --all-features -D warnings

# Run all tests
cargo test --workspace

# Run a single crate's tests
cargo test -p z1-fmt
```

### CLI commands
```bash
# Format a Z1 cell (compact → relaxed, or vice versa)
cargo run -p z1-cli -- fmt cells/http.server.z1c --mode relaxed

# Check formatting without writing
cargo run -p z1-cli -- fmt cells/http.server.z1c --check

# Format using stdin/stdout for editor integration
cargo run -p z1-cli -- fmt --stdin --mode compact < input.z1r > output.z1c

# Hash a cell (outputs both SemHash and FormHash)
cargo run -p z1-cli -- hash cells/http.server.z1c

# Run Z1 test suites (future)
cargo run -p z1-cli -- z1test tests/http.spec.z1t

# Provenance verification (future)
cargo run -p z1-cli -- z1prov verify prov/PROVCHAIN.z1p
```

## Key Architecture Concepts

### Canonical AST and Hashing
- **AST invariant**: The canonical AST uses only **long identifiers** (never short names)
- **SymbolMap**: Stores bidirectional long ↔ short name mappings for formatting only
- **SemHash**: Excludes SymbolMap, comments, shadow metadata → detects semantic changes
- **FormHash**: Includes SymbolMap → detects format-only changes
- Both hashes use SHA3-256 with deterministic serialization

### Dual Syntax
- **Compact (`.z1c`)**: Short keywords (`m`, `u`, `t`, `f`, `ret`), applies `SymbolMap.to_short`, minimal whitespace
- **Relaxed (`.z1r`)**: Long keywords (`module`, `use`, `type`, `fn`, `return`), long names, 2-space indent, readable formatting
- **Round-trip guarantee**: `parse(fmt_compact(AST))` and `parse(fmt_relaxed(AST))` produce identical SemHash

### Effects and Capabilities
- **Effects**: Tagged on functions (`eff [pure]`, `eff [net, async]`, etc.)
- **Capabilities**: Declared at module level (`caps=[net, time]`)
- **Enforcement**: Type checker ensures function effects are subsets of module capabilities
- **Effect types**: `Pure`, `Net`, `Fs`, `Time`, `Crypto`, `Env`, `Async`, `Unsafe`

### Provenance Chain
- Each cell edit creates a signed provenance entry with:
  - Previous entry hash (Merkle chain)
  - Actor (agent/developer)
  - Model used
  - Prompt hash + excerpt
  - Input cell hashes
  - Output SemHash + FormHash
  - Ed25519 signatures
- Stored as canonical JSON (sorted keys) in `.z1p` files
- Merkle root computed over all entries

## Code Style and Conventions

### Rust
- Rust 2021 edition, 4-space indents (via `cargo fmt`)
- Crates/modules/files: `snake_case`
- Prefix first-party crates: `z1-*`
- Keep workspace dependencies in root `Cargo.toml`

### Z1 Language
- Relaxed mode: 2 spaces, max width 100
- Cell names: dotted notation (`http.server`)
- Comments: `//` for line comments, `//@z1:` and `//:key:` for shadow metadata
- SymbolMap: `#sym { long_name: short, another: a }`

### Testing
- Unit tests in each crate under `tests/`
- Fixtures in `fixtures/` directory
- Property tests for SemHash stability
- Snapshot tests for formatter output
- Test names reflect what they verify

## Current Implementation Status (from plan.md)

### M0 – Core Bootstrapping (Week 0-1) ✅
- ✅ Cargo workspace scaffolded
- ✅ Lexer with dual keyword support
- ✅ Parser + AST (SymbolMap, type/fn placeholders)
- ✅ SemHash/FormHash implementation + CLI
- ✅ `z1fmt` MVP (module header, imports, symbol maps, types, fn signatures)

### M1 – Semantics & Context (Week 1-2) 🚧
- Structural type checker with generics
- Effect/capability checker
- Context estimator + SDict hooks
- Extended formatter (fn bodies, statements)

### M2 – Provenance, Policy, Testing (Week 2-3)
- Provenance store + Merkle roots
- Ed25519 signature verification
- Policy gates enforcement
- `.z1t` prompt-test harness

### M3 – Codegen & CLI UX (Week 3-4)
- IR definition
- TS/WASM codegen
- Minimal stdlib (http, time)
- Full CLI surface

## Important Files for Context

When working on specific areas, consult:

| Area | Key Files |
|------|-----------|
| Language grammar | `docs/grammar.md` (complete EBNF) |
| Design decisions | `docs/design.md`, `docs/vision.md` |
| Formatter behavior | `docs/fmt-plan.md` |
| DSL specs | `docs/dsl/manifest.md`, `docs/dsl/test.md` |
| Implementation plan | `plan.md` (milestones + task tracking) |
| Agent guidelines | `AGENTS.md` (coding style + commit conventions) |
| Sample code | `fixtures/cells/http_server.z1c` |

## Common Development Patterns

### Adding a new language feature
1. Update `docs/grammar.md` with grammar changes
2. Add AST node to `crates/z1-ast/src/ast.rs`
3. Update lexer in `crates/z1-lex` if new keywords needed
4. Extend parser in `crates/z1-parse`
5. Update hashing in `crates/z1-hash` (exclude formatting-only fields from SemHash)
6. Add formatter support in `crates/z1-fmt`
7. Add test fixtures in `fixtures/`
8. Update type checker in `crates/z1-typeck` if semantically relevant

### Testing formatter changes
```bash
# Run formatter tests
cargo test -p z1-fmt

# Add new fixture pairs in fixtures/fmt/
# - feature_name.compact.z1c
# - feature_name.relaxed.z1r

# Verify round-trip preserves SemHash
cargo run -p z1-cli -- hash fixtures/fmt/feature_name.compact.z1c
cargo run -p z1-cli -- fmt fixtures/fmt/feature_name.compact.z1c --mode relaxed | \
  cargo run -p z1-cli -- hash --stdin
# (hashes should match)
```

### Debugging parse/format issues
- Parser uses `logos` for lexing and produces spans for all nodes
- Enable tracing with `RUST_LOG=debug cargo test`
- Check `Span` information in AST nodes for source location mapping
- Use `--check` flag to see formatting diffs without modifying files

## Security and Provenance

### When modifying manifest or provenance
```bash
# Verify provenance chain integrity
cargo run -p z1-cli -- z1prov verify prov/PROVCHAIN.z1p
```

### Capability guidelines
- Keep capability grants minimal
- Match `[capabilities]` allow/deny lists in manifests
- Document any temporary capability escalations in commit messages
- Never commit SDict files with model-sensitive tokens directly (use `asset:` or `sha256:` handles)

## Commit Message Format

Follow existing convention from git history:
```
<area>: <imperative summary under 72 chars>

Optional body explaining:
- SemHash-impacting changes
- Updated docs or manifests
- Provenance/logging adjustments
```

Examples:
- `z1-parse: enforce ctx limit`
- `z1-fmt: add statement formatting`
- `docs: record formatter usage`

## Integration with CI (Future)

Planned CI checks:
- `cargo fmt --all --check`
- `cargo clippy --workspace --all-targets --all-features -D warnings`
- `cargo test --workspace`
- `cargo run -p z1-cli -- fmt fixtures/**/*.z1c --check`
- `cargo run -p z1-cli -- z1test tests/**/*.z1t`

## Working with Subagents

When handing off to subagents:
1. Specify target crate
2. Include failing test command
3. Quote relevant spec lines from `docs/`
4. Reference fixture cells that demonstrate the issue
5. Update `plan.md` when tasks complete

## Troubleshooting

### Build fails
- Ensure Rust 1.75+ installed: `rustc --version`
- Clean build: `cargo clean && cargo build`
- Check workspace member dependencies are correctly specified

### Tests fail
- Run single test: `cargo test -p <crate> <test_name>`
- Enable logging: `RUST_LOG=debug cargo test`
- Check fixtures haven't been modified unintentionally

### Formatter produces unexpected output
- Verify input parses: `cargo run -p z1-cli -- hash <file>`
- Compare AST debug output before/after formatting
- Check SymbolMap is correctly applied in the expected direction
- Ensure relaxed/compact mode matches file extension

## Additional Resources

All specifications live in `docs/`:
- Complete grammar rules
- Design rationale
- Vision and roadmap
- DSL specifications for manifests and tests

The `AGENTS.md` file contains additional style guidelines and Git workflow details.
