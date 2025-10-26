# Zero1 Implementation Plan

## 1. Vision Snapshot
Zero1 (Z1) is a Rust-based toolchain plus language optimized for LLM agent workflows: dual compact/relaxed syntax (`docs/grammar.md`), strict capability & context budgets (`docs/vision.md`), TOML manifest/test DSLs (`docs/dsl`). This plan decomposes that vision into actionable, trackable steps so agents (and subagents) can pick up focused contexts quickly.

## 2. Workstreams & Ownership Handoffs
| Workstream | Scope | Key Context Files | Handoff Notes |
|------------|-------|-------------------|---------------|
| Core Language | Lexer, parser, AST, hashing, fmt | `docs/grammar.md`, `docs/design.md` | Share crate-specific TODOs per `crates/z1-*` README; attach failing fixtures. |
| Semantics & Checks | Type/effect checker, context estimator | `docs/design.md` (M1), `docs/vision.md` | Record sample cells triggering each rule for regression contexts. |
| Tooling & CLI | `z1-cli`, `z1fmt`, `z1test`, `z1prov` | CLI specs from `docs/design.md` | Document command contracts + snapshots under `tests/cli/`. |
| DSLs & Runtimes | Manifest/test parsers, prompt-test harness | `docs/dsl/manifest.md`, `docs/dsl/test.md` | Capture schema changes in `docs/changelog.md` (add if missing). |
| Provenance & Security | Merkle chains, signatures, policy gates | `docs/vision.md` §7 | Maintain fixture packs in `examples/` for verification. |

## 3. Milestones with Actionable Tasks
### M0 – Core Bootstrapping (Week 0‑1)
- [x] Scaffold Cargo workspace per `docs/design.md` (create `crates/z1-*`, shared `Cargo.toml`).
- [x] Implement lexer/token enums with dual keyword support; add snapshot tests covering compact vs relaxed samples.
- [x] Build parser + AST (include `SymbolMap`); block semantic fields per design. _(Symbol maps, type/fn placeholders, and block capture now parsed; stmt-level AST still TODO.)_
- [x] Implement canonical SemHash/FormHash functions and golden tests. _(Hash crate + CLI `hash` subcommand live; expand coverage as AST grows.)_
- [x] Ship `z1fmt` MVP supporting compact↔relaxed round-trip on sample cells. _(Formatter library + CLI wired; covers module header, symbol maps, imports, types, and fn signatures per `docs/fmt-plan.md`.)_

### M1 – Semantics & Context (Week 1‑2) ✅
- [x] Implement structural type checker with basic generics; add fixtures under `tests/typeck/`. _(Complete: `z1-typeck` crate with 24 passing tests; structural equality, path resolution, import handling)_
- [x] Add effect/capability checker that enforces module budgets/caps. _(Complete: `z1-effects` crate with 24 passing tests; effect subtyping, capability enforcement)_
- [x] Build context estimator + SDict hooks (stubbed) so we can reject over-budget cells. _(Complete: `z1-ctx` crate + CLI integration with 14 passing tests)_
- [ ] **[Future]** Extend `z1fmt` to expand identifiers in function bodies (requires statement AST or text scanner).

### M2 – Provenance, Policy, Testing (Week 2‑3)
- [x] Implement append-only provenance store + Merkle root calculation. _(Complete: `z1-prov` crate with 36 passing tests; SHA3-256 hashing, Merkle chain verification, Ed25519 signatures, file I/O)_
- [x] Wire Ed25519 signature verification + CLI commands (`z1prov log`, `z1prov verify`). _(Complete: CLI commands implemented with 7 integration tests; keygen, log, and verify working with signature validation)_
- [x] Enforce policy gates (caps, ctx budgets, AST size) during compilation. _(Complete: `z1-policy` crate with 29 passing tests; enforces all limits from vision.md §9)_
- [x] Stand up Rust property tests + `.z1t` prompt-test harness with sample packs in `examples/`. _(Complete: `z1-test` crate with 28 passing tests; spec tests, property tests with proptest, CLI integration)_

### M3 – Codegen & CLI UX (Week 3‑4) ✅
- [x] Define IR plus TS/WASM codegen stubs; ensure CLI emits diagnostics referencing cells/effects. _(Complete: `z1-ir`, `z1-codegen-ts`, `z1-codegen-wasm` crates with MVPs; `z1c` command integrates full pipeline)_
- [x] Build minimal stdlib (http/time) to unblock examples. _(Complete: `stdlib/http/server.z1c` and `stdlib/http/client.z1c` with 17 passing tests; example app in `examples/http-hello/`; `stdlib/time/core.z1c` and `stdlib/time/timer.z1c` with 12 passing tests; example app in `examples/time-demo/`)_
- [ ] Finish CLI surface: `z1c` (compile), `z1fmt`, `z1prov`, `z1test`, `z1ctx`.
- [x] Add end-to-end integration test suite: manifest → compile → format → hash → checks. _(Complete: `z1-integration-tests` crate with 20 passing tests covering full pipeline, validation, error handling, and toolchain integration)_

### M4 – Ecosystem & Production Readiness (Week 4‑7)

#### Phase 1 - Foundation (Week 4) ✅
- [x] **stdlib/fs** - File system operations (read, write, directories, paths). Requires `fs.ro`/`fs.rw` capabilities. _(Complete: 3 modules - core.z1c, dir.z1c, path.z1c with 16 passing tests; file-copy example; full README documentation)_
- [x] **stdlib/crypto** - Cryptographic operations (SHA-256, SHA3, HMAC, random). Requires `crypto` capability. _(Complete: 3 modules with 15 passing tests; hash.z1c, hmac.z1c, random.z1c; password-hash example)_
- [x] **stdlib/env** - Environment & process (env vars, args, exit). Requires `env` capability. 10+ tests. _(Complete: `stdlib/env/vars.z1c`, `stdlib/env/args.z1c`, `stdlib/env/process.z1c` with 16 passing tests; example in `examples/config-loader/`)_
- [x] **Enhanced error messages** - Add source spans to all errors, pretty-print with snippets, color-coded output. _(Complete: error_printer module with 13 passing integration tests; pretty formatting with source context, line/column numbers, color support; integrated into CLI commands for parse, type, and effect errors)_

#### Phase 2 - Examples & Codegen (Week 5-6) ✅
- [x] **Tutorial documentation** - Getting Started, Language Tour, Stdlib Reference, Best Practices. _(Complete: 7 comprehensive guides in `docs/tutorial/` covering all aspects of Zero1 development - 01-getting-started.md, 02-language-tour.md, 03-stdlib-reference.md, 04-compilation.md, 05-best-practices.md, 06-migration.md, plus comprehensive README.md with learning paths for different backgrounds)_
- [x] **Real-world examples** - HTTP API server and CLI tool with comprehensive READMEs and tests. _(Complete: 2 production-quality examples; api-server demonstrates REST API with routing, JSON handling, and file serving (15 functions, 5 types, 8 HTTP endpoints); cli-tool demonstrates argument parsing, file I/O, and env vars (11 functions, 3 types, stdin/stdout support); both with compact/relaxed versions, comprehensive READMEs, and 18 passing parsing tests)_
- [x] **Full WASM statement AST** - Complete statement generation, string handling, records, function calls. 14 tests. _(Complete: Full WASM codegen with statement generation (let, if, while, return, assign), expression handling (literals, binops, unary ops, calls, fields), type mapping, string literals with data section, record construction and field access, memory management)_
- [ ] **IR optimizations** - Dead code elimination, constant folding/propagation, inlining. 12+ tests.

#### Phase 3 - Polish & Automation (Week 7)
- [x] **CI/CD pipeline** - GitHub Actions with tests, clippy, format checks, documentation builds. _(Complete: Comprehensive CI workflow with 6 jobs (test, lint, format, docs, examples, security), release workflow with multi-platform binaries, dependabot automation, and full documentation in docs/ci.md; CI badge added to README)_
- [x] **WASM binary output** - Integrate `wat2wasm` for direct `.wasm` generation. 5+ tests. _(Complete: Binary generation using `wat` crate with WAT-to-binary conversion; validation function using `wasmparser`; CLI `--binary` flag for `.wasm` output; 9 unit tests in z1-codegen-wasm + 7 integration tests in z1-cli; comprehensive README documentation; tutorial updated with binary WASM section)_
- [x] **Improved diagnostics** - Warnings, suggestions, multi-error reporting. 8+ tests. _(Complete: `z1-cli/src/diagnostics.rs` module with Diagnostic types, DiagnosticCollector, DiagnosticConfig; warning detection in `z1-typeck/src/warnings.rs` and `z1-effects/src/warnings.rs`; fuzzy name matching with Levenshtein distance; CLI flags (--warn-level, --max-errors, --json, --no-color); pretty-printed output with colors, symbols, and suggestions; 19 passing integration tests; 9 passing unit tests in warning modules)_

## 4. Cross-Cutting Tasks
- [x] Author CONTRIBUTING + `AGENTS.md` (guide published) plus crate-level READMEs to give subagents quick starts.
- [x] Add CI pipeline (fmt, clippy, cargo test, `z1test`) with artifact caching. _(Complete: GitHub Actions CI/CD with comprehensive coverage, release automation, and dependency monitoring)_
- [ ] Create template packs (`examples/http-example/`) for regression; keep snapshots updated. _(Scheduled for M4 Phase 2)_
- [ ] Track security items: capability audits, SDict handling, provenance replays.

## 5. Progress Tracking Guidance
1. Update this file when tasks move to ✅ / 🚧 with brief links to PRs.
2. For subagent handoffs, include: target crate, failing test command, and relevant spec lines (quote from docs).
3. Record open questions in a `docs/rfcs/` note to keep future contexts lightweight.

## 6. Immediate Next Steps
1. ✅ Confirm AGENTS guide (done).
2. ✅ Initialize Cargo workspace + minimal crates.
3. ✅ Draft lexer/parser scaffolding with fixture cells to unblock downstream agents.
4. ✅ Extend AST + parser with SymbolMap, type/fn decl placeholders, and richer tests.
5. ✅ Sketch SemHash/FormHash crate API (`z1-hash`) and wire into CLI for smoke tests.
6. ✅ Lay down `z1-fmt` plan (CLI flags, formatting strategy, test fixtures) to prep for round-trip support (`docs/fmt-plan.md`).
7. ✅ Implement `z1-fmt` MVP per plan (format module header + imports + symbol map) and add formatter tests.
8. [x] Extend formatter coverage (fn bodies/statements, CLI streaming flags) and document usage in `AGENTS.md` per fmt plan checklist. _(MVP complete: basic block formatting with indentation working; known limitations documented in `crates/z1-fmt/PROGRESS.md`; identifier expansion in function bodies remains future work.)_
