# z1-codegen-ts

TypeScript code generator for Zero1.

## Overview

This crate generates clean, idiomatic TypeScript code from Zero1 IR. The generated code can be used in Node.js or browser environments.

## Features

- Generates TypeScript interfaces from Z1 record types
- Generates TypeScript union types from Z1 union types
- Handles async functions (detects `async` effect)
- Preserves type safety with TypeScript type annotations
- Generates ES6 module imports/exports
- Produces readable, properly indented code

## Usage

```rust
use z1_codegen_ts::*;
use z1_ir::*;

// Create or obtain an IR module
let ir_module: IrModule = /* ... */;

// Generate TypeScript code
let typescript_code = generate_typescript(&ir_module);

// Write to file
std::fs::write("output.ts", typescript_code)?;
```

## Generated Code Examples

### Record Types

Z1:
```z1
type Point = { x: U32, y: U32 }
```

TypeScript:
```typescript
export interface Point {
  x: number;
  y: number;
}
```

### Functions

Z1:
```z1
fn add(a: U32, b: U32) -> U32 eff [pure] {
  ret a + b;
}
```

TypeScript:
```typescript
export function add(a: number, b: number): number {
  return a + b;
}
```

### Async Functions

Z1:
```z1
fn fetchData() -> Str eff [async, net] {
  // ...
}
```

TypeScript:
```typescript
export async function fetchData(): string {
  // ...
}
```

## Testing

Run tests with:

```bash
cargo test -p z1-codegen-ts
```

Tests include:
- Simple function generation
- Type interface generation
- Import generation
- Async function detection
- Control flow (if/else, while)
- Record literals
- Binary operations
- Field access
- End-to-end module compilation
