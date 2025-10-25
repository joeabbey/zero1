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
- [ ] Wire Ed25519 signature verification + CLI commands (`z1prov log`, `z1prov verify`).
- [ ] Enforce policy gates (caps, ctx budgets, AST size) during compilation.
- [ ] Stand up Rust property tests + `.z1t` prompt-test harness with sample packs in `examples/`.

### M3 – Codegen & CLI UX (Week 3‑4)
- [ ] Define IR plus TS/WASM codegen stubs; ensure CLI emits diagnostics referencing cells/effects.
- [ ] Build minimal stdlib (http/time) to unblock examples.
- [ ] Finish CLI surface: `z1c` (compile), `z1fmt`, `z1prov`, `z1test`, `z1ctx`.
- [ ] Add end-to-end integration test: manifest → build → provenance verify.

## 4. Cross-Cutting Tasks
- [x] Author CONTRIBUTING + `AGENTS.md` (guide published) plus crate-level READMEs to give subagents quick starts.
- [ ] Add CI pipeline (fmt, clippy, cargo test, `z1test`) with artifact caching.
- [ ] Create template packs (`examples/http-example/`) for regression; keep snapshots updated.
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
