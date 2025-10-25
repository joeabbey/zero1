# z1-policy

Policy enforcement for Zero1 compilation.

This crate implements compile-time policy gates that enforce limits on cell-level, function-level, and module-level constraints as specified in the Zero1 vision (section 9).

## Features

### Cell-level Constraints
- **AST node limit**: Maximum number of AST nodes per cell (default: 200)
- **Export limit**: Maximum public functions and types per cell (default: 5)
- **Import limit (fanin)**: Maximum dependencies per cell (default: 10)

### Function-level Constraints
- **Parameter limit**: Maximum parameters per function (default: 6)
- **Local variable limit**: Maximum local variables per function (default: 32)
- **Context budget**: Maximum tokens per function (default: 256)

### Module-level Constraints
- **Effect/capability checking**: Function effects must be subset of module capabilities
- **Cell context budget**: Total tokens must not exceed declared budget

## Usage

```rust
use z1_policy::{PolicyChecker, PolicyLimits};
use z1_parse::parse_module;

// Parse a module
let source = r#"
m http.server:1.0 ctx=128 caps=[net]
f handler()->Unit eff [net] { ret Unit }
"#;
let module = parse_module(source).unwrap();

// Check with default policy limits
let checker = PolicyChecker::with_defaults();
match checker.check_module(&module) {
    Ok(()) => println!("Module passes all policy checks"),
    Err(violations) => {
        for violation in violations {
            eprintln!("Policy violation: {}", violation);
        }
    }
}

// Check with custom limits
let custom_limits = PolicyLimits {
    cell_max_exports: 3,  // Stricter than default
    ..Default::default()
};
let strict_checker = PolicyChecker::new(custom_limits);
strict_checker.check_module(&module).expect("Module must pass strict policy");
```

## Policy Violation Types

All violations are represented by the `PolicyViolation` enum with descriptive error messages:

- `AstNodeLimitExceeded` - Cell has too many AST nodes
- `ExportLimitExceeded` - Cell exports too many functions/types
- `FaninLimitExceeded` - Cell imports from too many dependencies
- `ParamLimitExceeded` - Function has too many parameters
- `LocalsLimitExceeded` - Function has too many local variables
- `ContextBudgetExceeded` - Function exceeds context token budget
- `CellContextBudgetExceeded` - Cell exceeds declared context budget
- `EffectNotInCapabilities` - Function effect not in module capabilities

## Integration

This crate integrates with:
- `z1-ast` - AST traversal and node counting
- `z1-effects` - Effect/capability validation
- `z1-ctx` - Context budget estimation

## Design Philosophy

These limits are designed to keep code small, modular, and tractable for LLM agents. They enforce the "many tiny pieces loosely joined" principle at compile time rather than as a style guideline.

## Testing

The crate includes 29 comprehensive tests covering all policy gates:
- Cell-level tests (AST nodes, exports, imports)
- Function-level tests (parameters, locals, context)
- Effect/capability tests
- Integration tests
- Edge cases

Run tests:
```bash
cargo test -p z1-policy
```

## Test Fixtures

Example modules demonstrating policy violations are in `fixtures/policy/`:
- `valid_small.z1c` - Passes all checks
- `too_many_params.z1c` - Parameter limit violation
- `missing_capability.z1c` - Effect/capability mismatch
- `too_many_exports.z1c` - Export limit violation
