# M1: Semantics & Context - Status

## Milestone Overview
M1 focuses on semantic analysis for Zero1, including type checking, effect/capability checking, and context estimation.

## Tasks

### 1. Type Checker (structural types, generics later)
Status: PENDING
- Not yet implemented
- Will go in `crates/z1-typeck/`

### 2. Effect/Capability Checker
Status: COMPLETE
- Implemented in `crates/z1-effects/src/lib.rs`
- Key features:
  - Effect types: Pure, Net, Fs, Time, Crypto, Env, Async, Unsafe
  - Module-level capability declarations enforced
  - Function effects must be subset of module capabilities
  - Pure functions allowed in all contexts
  - Fine-grained capabilities (e.g., fs.ro, fs.rw) supported
  - Case-insensitive effect matching
  - Clear error messages with source spans
- Tests:
  - 12 unit tests in lib.rs
  - 12 integration tests covering realistic scenarios
  - All tests passing
  - Clippy clean with `-D warnings`

### 3. Context Estimator (model-agnostic; SDict-aware later)
Status: PENDING
- Not yet implemented
- Will go in `crates/z1-ctx/` or as part of z1-cli

## Next Steps
1. Implement type checker in z1-typeck
2. Implement context estimator
3. Integrate effect checker into CLI/compiler pipeline
