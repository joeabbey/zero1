# z1-ctx: Context Estimator for Zero1

Context budget estimation for Z1 modules (cells) and functions, enabling enforcement of token limits defined in module headers.

## Overview

The `z1-ctx` crate provides tools to estimate token usage for Zero1 source code, helping enforce the context budget constraints that are central to Z1's design philosophy of keeping code small, modular, and LLM-friendly.

## Features

- Naive token estimation using configurable character-to-token ratio (default: 3.8)
- Per-cell and per-function token counting
- Budget validation against module-declared limits (`ctx=N`)
- Actionable error messages with split suggestions
- CLI integration via `z1-cli ctx` command

## Token Cost Model

The MVP uses a simple heuristic:

```
tokens ≈ ceil(chars / 3.8)
```

This ratio is configurable and provides a reasonable baseline. Future versions will support model-specific dictionaries (SDict) for improved estimation accuracy.

## Usage

### As a Library

```rust
use z1_ctx::{estimate_cell, EstimateConfig};
use z1_parse::parse_module;

let source = r#"
m http.server:1.0 ctx=128 caps=[net]
f handler()->Unit eff [pure] { ret Unit }
"#;

let module = parse_module(source)?;
let estimate = estimate_cell(&module)?;

println!("Estimated tokens: {}", estimate.total_tokens);
if let Some(budget) = estimate.budget {
    println!("Budget: {} ({}% used)",
        budget,
        (estimate.total_tokens as f64 / budget as f64) * 100.0
    );
}
```

### Custom Configuration

```rust
let config = EstimateConfig {
    chars_per_token: 4.0,  // Custom ratio
    enforce_budget: false,  // Skip validation
};

let estimate = estimate_cell_with_config(&module, &config)?;
```

### Via CLI

```bash
# Basic estimation
z1-cli ctx cells/http.server.z1c

# Verbose output with per-function breakdown
z1-cli ctx -v cells/http.server.z1c

# Custom token ratio
z1-cli ctx --chars-per-token 4.0 cells/http.server.z1c

# Skip budget enforcement (estimate only)
z1-cli ctx --no-enforce cells/http.server.z1c
```

## Estimation Algorithm

1. Format AST to compact text using current SymbolMap (via `z1-fmt`)
2. Count characters in compact representation
3. Apply token estimation heuristic
4. Optionally validate against declared budget
5. Generate per-function estimates (approximate)
6. Provide actionable suggestions if budget is exceeded

## Budget Enforcement

When a module declares a context budget:

```z1c
m http.server:1.0 ctx=128 caps=[net]
```

The estimator will:
- Calculate total token count
- Compare against declared budget
- Return error if exceeded (when `enforce_budget: true`)
- Suggest split strategies in error messages

### Example Error Messages

**Single function exceeds budget:**
```
Error: cell exceeds context budget: 145/128 tokens.
Consider splitting function 'processRequest' (92 tokens) into smaller functions.
```

**Multiple functions, suggest extraction:**
```
Error: cell exceeds context budget: 156/128 tokens.
Consider moving function 'processHeaders' (67 tokens) to a separate cell.
```

## Future Work

### SDict Support (Model-Specific Dictionaries)

Currently stubbed. Future versions will:
- Apply model-specific token replacements for estimation
- Support multiple tokenization strategies
- Provide per-model estimation profiles
- Cache tokenization results

### Improved Per-Function Estimation

Current per-function estimates are approximate. Future improvements:
- Full statement-level AST formatting
- More accurate function body token counting
- Detection of shared vs. unique token usage
- Suggested refactoring to reduce token usage

## Testing

Run the test suite:

```bash
cargo test -p z1-ctx
```

Tests cover:
- Basic token estimation
- Budget validation
- Custom ratios
- Error message generation
- Real-world fixtures (http_server.z1c)

## API Reference

### Core Functions

- `estimate_cell(module: &Module) -> Result<CellEstimate, CtxError>`
- `estimate_cell_with_config(module: &Module, config: &EstimateConfig) -> Result<CellEstimate, CtxError>`

### Types

- `CellEstimate`: Overall cell estimation with per-function breakdown
- `FnEstimate`: Individual function token estimate
- `EstimateConfig`: Configuration for estimation behavior
- `CtxError`: Error types including budget violations

### Constants

- `DEFAULT_CHARS_PER_TOKEN`: 3.8 (configurable baseline)

## Integration Points

- Uses `z1-fmt` to format AST to compact mode
- Reads `ctx_budget: Option<u32>` from module AST
- Provides CLI command via `z1-cli ctx`
- Returns spans for error reporting integration

## Design Philosophy

The context estimator embodies Z1's core principle: **make small, composable code a compile-time invariant, not a style guide**. By providing fast, accurate token estimation with actionable feedback, we enable:

- LLM-friendly code size constraints
- Early detection of scope creep
- Automated suggestions for code organization
- Model-agnostic token budgeting
