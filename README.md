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
# Format a Z1 cell
cargo run -p z1-cli -- fmt cells/http.server.z1c --mode relaxed

# Check formatting without writing
cargo run -p z1-cli -- fmt cells/http.server.z1c --check

# Hash a cell (outputs SemHash and FormHash)
cargo run -p z1-cli -- hash cells/http.server.z1c
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

Zero1 is implemented as a Rust workspace with multiple crates:

### Core Language
- **z1-lex**: Lexer with dual keyword support (compact/relaxed)
- **z1-parse**: Parser producing canonical AST
- **z1-ast**: AST node definitions (always uses long identifiers)
- **z1-fmt**: Bidirectional formatter (compact ↔ relaxed)
- **z1-hash**: SemHash (semantics) and FormHash (formatting) computation

### Semantics & Safety
- **z1-typeck**: Structural type checker
- **z1-effects**: Effect and capability checker
- **z1-prov**: Provenance store with Merkle roots and Ed25519 signatures

### Code Generation
- **z1-codegen-ts**: TypeScript code generation
- **z1-codegen-wasm**: WebAssembly code generation

### Tooling
- **z1-cli**: Command-line interface for all tools

## Project Status

### Completed (M0)
- ✅ Workspace scaffolding
- ✅ Lexer with dual keywords
- ✅ Parser with AST (SymbolMap, types, functions)
- ✅ SemHash/FormHash implementation
- ✅ Formatter MVP (module headers, imports, types, functions, basic blocks)

### In Progress (M1)
- 🚧 Structural type checker
- 🚧 Effect/capability checker
- 🚧 Context estimator

### Planned (M2+)
- Provenance store and signature verification
- Policy gates and enforcement
- Full statement AST and advanced formatting
- TS/WASM code generation
- Standard library (http, time)

See [`plan.md`](plan.md) for detailed roadmap and [`CLAUDE.md`](CLAUDE.md) for development guidelines.

## Documentation

- **[plan.md](plan.md)**: Implementation roadmap with milestones
- **[CLAUDE.md](CLAUDE.md)**: Development guidelines for Claude Code
- **[AGENTS.md](AGENTS.md)**: Repository conventions and commit guidelines
- **[docs/](docs/)**: Language specifications
  - [grammar.md](docs/grammar.md): Complete EBNF grammar
  - [design.md](docs/design.md): Build plan and architecture
  - [vision.md](docs/vision.md): High-level goals and rationale
  - [fmt-plan.md](docs/fmt-plan.md): Formatter implementation strategy

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
