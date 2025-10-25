# z1-test

Test framework for Zero1 `.z1t` test files, providing spec tests and property-based testing.

## Features

- **Spec Tests**: Unit-style tests with assertions (`assert`, `assert_eq`, `assert_ne`)
- **Property Tests**: Property-based testing using `proptest` with type-driven value generation
- **Test Configuration**: File-level config for timeouts, tags, and seeds
- **Test Attributes**: Per-test attributes for skip, only, tags, and timeout overrides
- **Fixtures**: Reusable test data with optional type annotations
- **CLI Integration**: `z1test` command in the Zero1 CLI

## Test File Structure

### Basic Spec Test

```z1t
spec "test name" {
  assert 1 + 1 == 2;
}
```

### Property Test

```z1t
prop "addition is commutative"
for_all (a: U32, b: U32) runs 100 seed 42 {
  assert a + b == b + a;
}
```

### Configuration and Attributes

```z1t
config { timeout_ms: 3000 }

spec "tagged test" with { tags: ["unit"], timeout: 5000 } {
  // test body
}

spec "skipped test" with { skip: true } {
  // not executed
}
```

### Fixtures

```z1t
fixture base: U32 = { 42 };

spec "uses fixture" {
  let x = base + 10;
  assert_eq(x, 52);
}
```

## Usage

### From Command Line

```bash
# Run test file
cargo run -p z1-cli -- z1test tests/simple.z1t

# Run multiple files
cargo run -p z1-cli -- z1test tests/*.z1t

# Filter by tags
cargo run -p z1-cli -- z1test --tags unit,integration tests/mixed.z1t

# Verbose output
cargo run -p z1-cli -- z1test -v tests/simple.z1t
```

### From Rust

```rust
use z1_test::{parse_test_file, TestRunner, TestConfig};

let source = std::fs::read_to_string("tests/simple.z1t")?;
let file = parse_test_file(&source)?;

let config = TestConfig::default();
let mut runner = TestRunner::new(config);
let results = runner.run_file(&file);

println!("Passed: {}, Failed: {}", results.passed, results.failed);
```

## Supported Types for Property Tests

- `U32`, `u32` - Unsigned 32-bit integers
- `Str`, `String` - String values

More types will be supported in future releases.

## MVP Limitations

The current implementation is an MVP with the following limitations:

- **No full Z1 interpreter**: Test assertions are simplified pattern matching
- **No lifecycle hooks**: `before`, `after`, `before_each`, `after_each` are not yet supported
- **No mocks**: `mock` blocks for capability interception are deferred
- **No prompt-tests**: LLM-driven prompt-test blocks are marked as future work
- **Limited assertion support**: Only `assert`, `assert_eq`, `assert_ne` implemented
- **Basic type support**: Property tests only support primitive types

## Test Count

This crate contains 28 comprehensive tests:
- 22 unit tests (lexer, parser, runner)
- 6 integration tests (end-to-end with fixtures)

All tests verify actual functionality and will catch regressions.

## Grammar Reference

See `/Users/joeabbey/src/github.com/joeabbey/zero1/docs/dsl/test.md` for the complete test DSL specification.
