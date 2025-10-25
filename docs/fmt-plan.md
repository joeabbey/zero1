# z1-fmt Implementation Plan

## 1. Goals & Scope
- Lossless round-trip between `.z1c` (compact) and `.z1r` (relaxed) files, preserving SemHash invariants from `docs/design.md`.
- Deterministic whitespace and ordering so repeated runs are idempotent.
- Tooling target: Linux/macOS CLI invoked via `z1 fmt …`, plus reusable library entry points for other crates/tests.

## 2. CLI Surface (z1-cli `fmt` subcommand)
| Flag | Description |
| ---- | ----------- |
| `--mode <compact|relaxed>` | Selects formatter mode (default: `relaxed` for `.z1c`, `compact` for `.z1r`). |
| `--check` | Parse + format + diff in memory; exit non-zero on change. |
| `--write` | Overwrite the provided files in place (default). |
| `--stdin` / `--stdout` | Stream input/output for editor integrations (mutually exclusive with `--write`). |
| `--symmap <respect|reflow>` | `respect` preserves existing ordering; `reflow` sorts pairs alphabetically (default: `respect`). |
| `--files-from <path>` | Optional newline-delimited list when formatting batches. |

Rules:
1. Detect mode automatically from extension unless `--mode` is set.
2. Always emit diagnostics referencing source spans for parse failures.
3. Wire `--check` and `--write` into CI guidance in `AGENTS.md`.

## 3. Formatter Architecture (`crates/z1-fmt`)
1. **AST Canonicalization**: accept `z1_ast::Module`, expand SymbolMap (long names) for relaxed mode, collapse for compact mode.
2. **Layout Engine**:
   - Shared `Formatter` struct with buffer + indentation stack.
   - Helpers per node type: `fmt_module`, `fmt_import`, `fmt_symbol_map`, `fmt_type_decl`, `fmt_fn_decl`.
   - Relaxed mode: keywords spelled out, 2-space indents, trailing commas for records, blank line groups (`module` header, sym map, imports, decls).
   - Compact mode: single spaces around `=`, no trailing commas, `SymbolMap.to_short` substitutions, inline `eff` sections.
3. **Symbol Handling**: read/write `SymbolMap` but never mutate semantics; expose hooks to keep order or reflow.
4. **Error Surfacing**: return `FormatterError` (enum) so CLI can display context.

## 4. Test Strategy
| Layer | Approach |
| ----- | -------- |
| Unit | Snapshot-style tests in `crates/z1-fmt/tests/` comparing `fmt_relaxed`/`fmt_compact` output for `fixtures/cells/http_server.z1c`. |
| Property | Round-trip check: `source -> parse -> fmt_compact -> parse -> fmt_relaxed -> parse`; assert SemHash matches using `z1-hash`. |
| CLI | Add `tests/cli/z1fmt.rs` using `assert_cmd` to cover `--check`, `--mode`, `--stdin/--stdout`. |

Add new fixtures under `fixtures/fmt/`:
- `http_server.compact.z1c` (matches current sample).
- `http_server.relaxed.z1r` (expected relaxed output).
- `symbol_map_override.z1c` (exercises `#sym` ordering and reflow).
- `statements.compact.z1c` / `.relaxed.z1r` covering `return`, `let`, `if/else`, nested blocks.

## 5. Rollout Checklist
1. Implement `crates/z1-fmt` with the API `pub fn format(module: &Module, mode: Mode, options: &FmtOptions) -> Result<String, Error>`.
2. Update `z1-cli` to load files, parse once, call formatter, and honor CLI flags.
3. Extend `plan.md` (M0 + Immediate Next Steps) to track formatter MVP progress + testing requirements.
4. Document usage in `AGENTS.md` + `docs/design.md` once behavior stabilizes.
