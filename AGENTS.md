# Repository Guidelines

## Project Structure & Module Organization
Specs live in `docs/` (`design.md`, `vision.md`, `grammar.md`), while DSL references sit in `docs/dsl/` (`manifest.md`, `test.md`). When you scaffold the workspace described in `docs/design.md`, mirror its layout: crates under `crates/z1-*`, CLI glue in `crates/z1-cli`, runnable demos in `examples/`, and Z1 artifacts in `cells/`, `tests/`, `dicts/`, and `prov/` so hashing/provenance tools stay deterministic.

## Build, Test, and Development Commands
- `cargo fmt --all`: keep Rust sources consistent.
- `cargo clippy --workspace --all-targets --all-features -D warnings`: fail fast on capability/effect lint warnings.
- `cargo test --workspace`: run every crate’s unit and integration suites.
- `cargo run -p z1-cli -- z1fmt cells/http.server.z1c --check`: ensure compact↔relaxed parity without writing files.
- `cargo run -p z1-cli -- z1test tests/http.spec.z1t`: execute `.z1t` suites, property tests, and prompt-tests.

## Coding Style & Naming Conventions
Stick to Rust 2021, 4-space indent, and `cargo fmt` defaults; `.z1r` relaxed blocks prefer 2 spaces and max width 100 (per `docs/grammar.md`). Crates/modules/files are `snake_case`, exported cell names stay dotted (`http.server`), and compact aliases live in `#sym` maps. Prefix first-party crates with `z1-`, keep TOML keys lowercase (`docs/dsl/manifest.md`), and generate relaxed companions with `z1fmt --relax` before review.

## Testing Guidelines
Pair each crate with unit/integration coverage plus property tests that assert SemHash stability, context budgets, and capability caps from `docs/grammar.md`. `.z1t` suites follow `docs/dsl/test.md`; name them after the target cell (`suite "http.server"`), co-locate fixtures, and store snapshots in `tests/snapshots/`, only updating with `Z1_UPDATE_SNAPSHOTS=1`. Prompt-tests must specify `expect.diff` and `expect.effects` clauses so CI can reject unintended mutations.

## Commit & Pull Request Guidelines
History so far uses short sentence-case subjects, so stick with `<area>: <imperative summary>` under 72 chars (e.g., `z1-parse: enforce ctx limit`). Call out SemHash-impacting changes, mention updated docs or manifests, and capture provenance/logging adjustments in the body. PRs should link to the motivating issue or prompt, include `fmt/clippy/test/z1test` results, and add screenshots or diffs when CLI UX changes.

## Security & Configuration Tips
After editing `manifest.z1m` or `prov/`, run `cargo run -p z1-cli -- z1prov verify prov/PROVCHAIN.z1p` to confirm hashes. Keep capability grants minimal (match `[capabilities]` allow/deny lists) and describe any temporary escalation in the PR. SDict files with model-sensitive tokens should be referenced via `asset:` or `sha256:` handles (`docs/vision.md`) rather than stored in-repo.
