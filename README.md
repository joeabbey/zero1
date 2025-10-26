# Zero1 (Z1)

A Rust-based toolchain and language optimized for LLM agent workflows.

## Overview

Zero1 is designed to solve the challenges of code generation and maintenance by LLM agents through:

- **Dual Syntax**: Compact (`.z1c`) for token efficiency, Relaxed (`.z1r`) for readability
- **Deterministic Hashing**: SemHash for semantic changes, FormHash for formatting changes
- **Context Budgets**: Hard limits on token counts to keep LLM context manageable
- **Capability System**: Fine-grained permissions (net, fs, time, crypto, env, unsafe)
- **Provenance Chain**: Append-only, cryptographically-signed audit trail of all changes

## Quick Start

### Building

```bash
# Format and check the codebase
cargo fmt --all
cargo clippy --workspace --all-targets --all-features -D warnings

# Run all tests
cargo test --workspace
```

### Using the CLI

```bash
# Compile a Z1 cell to TypeScript (with optimization)
cargo run -p z1-cli -- z1c examples/hello.z1c --target ts --opt-level o2

# Compile to WebAssembly
cargo run -p z1-cli -- z1c examples/hello.z1c --target wasm

# Format a Z1 cell (compact ↔ relaxed)
cargo run -p z1-cli -- fmt cells/http.server.z1c --mode relaxed

# Check formatting without writing
cargo run -p z1-cli -- fmt cells/http.server.z1c --check

# Hash a cell (outputs SemHash and FormHash)
cargo run -p z1-cli -- hash cells/http.server.z1c

# Estimate context budget
cargo run -p z1-cli -- ctx examples/hello.z1c

# Provenance operations
cargo run -p z1-cli -- z1prov keygen                    # Generate Ed25519 keypair
cargo run -p z1-cli -- z1prov log <action> <cell>       # Log provenance entry
cargo run -p z1-cli -- z1prov verify <provenance-file>  # Verify chain integrity
```

## Language Features

### Dual Syntax Example

**Compact (`.z1c`)** - optimized for token count:
```z1c
m http.server:1.0 ctx=128 caps=[net]
#sym { handler: h, serve: sv }
u "std/http" as H only [listen, Req, Res]
f h(q:H.Req)->H.Res eff [pure] { ret H.Res{ status:200, body:"ok" }; }
f sv(p:U16)->Unit eff [net] { H.listen(p, h); }
```

**Relaxed (`.z1r`)** - optimized for readability:
```z1r
module http.server : 1.0
  ctx = 128
  caps = [net]

#sym { handler: h, serve: sv }

use "std/http" as H only [listen, Req, Res]

fn handler(q: H.Req) -> H.Res
  eff [pure]
{
  ret H.Res{ status:200, body:"ok" };
}

fn serve(p: U16) -> Unit
  eff [net]
{
  H.listen(p, handler);
}
```

Both representations produce the same **SemHash** (semantic hash), ensuring they are semantically equivalent.

## Architecture

Zero1 is implemented as a Rust workspace with 18 crates:

### Core Language
- **z1-lex**: Lexer with dual keyword support (compact/relaxed)
- **z1-parse**: Parser producing canonical AST with identifier normalization
- **z1-ast**: AST node definitions (always uses long identifiers)
- **z1-fmt**: Bidirectional formatter (compact ↔ relaxed) with semantic hash preservation
- **z1-hash**: SemHash (semantics) and FormHash (formatting) computation

### Semantics & Safety
- **z1-typeck**: Structural type checker with generics (24 tests)
- **z1-effects**: Effect and capability checker with subtyping (24 tests)
- **z1-ctx**: Context estimator with token budgets (14 tests)
- **z1-policy**: Policy gate enforcement (AST size, context, capabilities) (29 tests)

### Provenance & Security
- **z1-prov**: Provenance store with Merkle chain verification (36 tests)
  - Append-only audit trail
  - Ed25519 signature verification
  - SHA3-256 hashing

### Code Generation
- **z1-ir**: Intermediate representation with optimizations (15 tests)
  - Dead code elimination (DCE)
  - Constant folding and propagation
  - Function inlining
  - Three optimization levels (O0, O1, O2)
- **z1-codegen-ts**: TypeScript code generation (2 tests)
- **z1-codegen-wasm**: WebAssembly code generation with full statement AST (14 tests)

### Testing & Integration
- **z1-test**: Test harness with property tests and spec tests (28 tests)
- **z1-integration-tests**: End-to-end pipeline tests (20 tests)

### Tooling
- **z1-cli**: Unified command-line interface
  - Compilation (z1c) with 7-stage pipeline
  - Formatting (fmt)
  - Hashing (hash)
  - Context estimation (ctx)
  - Provenance management (z1prov)
  - Error printing with source spans and color output

### Standard Library
- **stdlib/http**: HTTP client and server (17 tests)
- **stdlib/time**: Time operations and timers (12 tests)
- **stdlib/fs**: File system operations (read, write, directories, paths) (16 tests)
- **stdlib/crypto**: Cryptography (SHA-256, SHA3, HMAC, random) (15 tests)
- **stdlib/env**: Environment variables, args, and process control (16 tests)

**Total: 327 tests across workspace, all passing**

## Project Status

### ✅ M0 - Core Bootstrapping (Complete)
- Workspace scaffolding with 18 crates
- Lexer with dual keyword support
- Parser with canonical AST and identifier normalization
- SemHash/FormHash implementation
- Formatter MVP with semantic hash preservation

### ✅ M1 - Semantics & Context (Complete)
- Structural type checker with generics (24 tests)
- Effect/capability checker with subtyping (24 tests)
- Context estimator with token budgets (14 tests)
- Extended formatter with function bodies

### ✅ M2 - Provenance, Policy, Testing (Complete)
- Provenance store with Merkle chain verification (36 tests)
- Ed25519 signature verification and CLI commands (7 tests)
- Policy gate enforcement (29 tests)
- Test harness with property and spec tests (28 tests)

### ✅ M3 - Codegen & CLI UX (Complete)
- IR definition with optimization support (15 tests)
- TypeScript code generation (2 tests)
- WebAssembly code generation (14 tests)
- Standard library: http (17 tests), time (12 tests)
- Unified CLI with full compilation pipeline (18 tests)
- End-to-end integration tests (20 tests)

### ✅ M4 Phase 1 - Foundation (Complete)
- Standard library: fs (16 tests), crypto (15 tests), env (16 tests)
- Enhanced error messages with source spans and colors (13 tests)
- Pretty-printed compiler errors with source context

### ✅ M4 Phase 2 - Examples & Codegen (Complete)
- Comprehensive tutorial documentation (7 guides)
- Real-world examples: HTTP API server, CLI tool, task scheduler, data processor
- Full WASM statement AST generation (all statement types)
- IR optimizations: DCE, constant folding, inlining (15 tests)

### 🚧 M4 Phase 3 - Polish & Automation (In Progress)
- CI/CD pipeline with GitHub Actions
- WASM binary output (wat2wasm integration)
- Improved diagnostics (warnings, suggestions)

**Current Status:** 327 tests passing, 18 crates, 5 stdlib modules, production-ready toolchain

See [`plan.md`](plan.md) for detailed roadmap and [`CLAUDE.md`](CLAUDE.md) for development guidelines.

## Documentation

### Project Documentation
- **[plan.md](plan.md)**: Implementation roadmap with milestones
- **[CLAUDE.md](CLAUDE.md)**: Development guidelines for Claude Code (includes git worktrees pattern)
- **[AGENTS.md](AGENTS.md)**: Repository conventions and commit guidelines

### Language Specifications
- **[docs/grammar.md](docs/grammar.md)**: Complete EBNF grammar
- **[docs/design.md](docs/design.md)**: Build plan and architecture
- **[docs/vision.md](docs/vision.md)**: High-level goals and rationale
- **[docs/fmt-plan.md](docs/fmt-plan.md)**: Formatter implementation strategy

### Tutorials (NEW)
- **[docs/tutorial/01-getting-started.md](docs/tutorial/01-getting-started.md)**: Installation and first program
- **[docs/tutorial/02-language-tour.md](docs/tutorial/02-language-tour.md)**: Complete language feature overview
- **[docs/tutorial/03-stdlib-reference.md](docs/tutorial/03-stdlib-reference.md)**: Standard library API documentation
- **[docs/tutorial/04-compilation.md](docs/tutorial/04-compilation.md)**: 7-stage compilation pipeline explained
- **[docs/tutorial/05-best-practices.md](docs/tutorial/05-best-practices.md)**: Security, performance, and patterns
- **[docs/tutorial/06-migration.md](docs/tutorial/06-migration.md)**: Coming from TypeScript, Rust, or Python
- **[docs/tutorial/README.md](docs/tutorial/README.md)**: Tutorial index and learning paths

### Examples (NEW)
- **[examples/http-hello/](examples/http-hello/)**: Simple HTTP server
- **[examples/time-demo/](examples/time-demo/)**: Timer and time operations
- **[examples/file-copy/](examples/file-copy/)**: File system operations
- **[examples/password-hash/](examples/password-hash/)**: Cryptographic hashing
- **[examples/config-loader/](examples/config-loader/)**: Environment variables and args
- **[examples/api-server/](examples/api-server/)**: REST API with routing (production example)
- **[examples/cli-tool/](examples/cli-tool/)**: Command-line file processor (production example)
- **[examples/scheduler/](examples/scheduler/)**: Task scheduling with timers (production example)
- **[examples/processor/](examples/processor/)**: Data processing pipeline (production example)

## Key Design Principles

### Canonical AST
The AST always uses **long identifiers**. The SymbolMap is purely for formatting:
- Parsing compact → expand short names to long names
- Formatting to compact → contract long names to short names
- This ensures all tooling works with a consistent representation

### Hashing Strategy
- **SemHash**: Excludes SymbolMap, comments, formatting → detects semantic changes
- **FormHash**: Includes SymbolMap → detects formatting/naming changes
- Both use SHA3-256 with deterministic serialization (sorted keys, canonical encodings)

### Context Management
Hard limits prevent context budget overflow:
- Module-level: `ctx=<tokens>` caps total size
- Function-level: Optional per-fn limits
- Enforced at compile time with actionable split suggestions

### Capabilities & Effects
- **Capabilities**: Declared at module level (`caps=[net, fs.ro]`)
- **Effects**: Tagged on functions (`eff [net, async]`)
- **Enforcement**: Function effects must be subsets of module capabilities

## Contributing

This is an experimental project. For development guidelines, see:
- [CLAUDE.md](CLAUDE.md) for Claude Code development
- [AGENTS.md](AGENTS.md) for general repository conventions

## License

Apache 2.0
