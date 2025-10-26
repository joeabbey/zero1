# Zero1 Compilation Guide

This guide explains how Zero1 compiles your code from source to executable targets. Understanding the compilation pipeline helps you write better code and debug issues effectively.

## Overview

Zero1 compilation follows a 7-stage pipeline:

```
Source (.z1c/.z1r)
    ↓
[1] Lexing → Tokens
    ↓
[2] Parsing → AST
    ↓
[3] Type Checking
    ↓
[4] Effect Checking
    ↓
[5] Context Estimation
    ↓
[6] Policy Enforcement
    ↓
[7] Code Generation → TypeScript / WASM
```

Each stage validates different aspects of your program and provides actionable error messages.

## Stage 1: Lexical Analysis (Lexing)

The lexer converts source text into a stream of tokens.

### What It Does

- Recognizes keywords (both compact and relaxed forms)
- Identifies identifiers, numbers, strings, operators
- Skips whitespace and preserves comments
- Tracks source positions (spans) for error reporting

### Dual Keyword Support

The lexer recognizes both forms of keywords:

| Compact | Relaxed | Meaning |
|---------|---------|---------|
| `m` | `module` | Module declaration |
| `u` | `use` | Import statement |
| `t` | `type` | Type declaration |
| `f` | `fn` | Function declaration |
| `ret` | `return` | Return statement |
| `x` | `extern` | External declaration |

### Example

**Input:**
```z1c
m hello:1.0 ctx=64
f main()->Unit { ret (); }
```

**Token Stream:**
```
KW_MODULE "m" (1:1-1:2)
IDENT "hello" (1:3-1:8)
COLON ":" (1:8-1:9)
VERSION "1.0" (1:9-1:12)
KW_CTX "ctx" (1:13-1:16)
EQ "=" (1:16-1:17)
INT "64" (1:17-1:19)
NEWLINE
KW_FN "f" (2:1-2:2)
...
```

### Common Lexer Errors

```
Error: Unexpected character '@' at line 5, column 12
  │
5 │ let x = @value;
  │         ^
```

## Stage 2: Parsing

The parser builds an Abstract Syntax Tree (AST) from tokens.

### Canonical AST

Zero1 uses a **canonical AST** where:
- All identifiers are in **long form** (never short names)
- SymbolMap is stored separately (not in semantic nodes)
- Structure is independent of formatting

**Example:**

Both these parse to the same AST:

```z1c
#sym { greet: g }
f g(n:Str)->Str { ret "Hello, " + n; }
```

```z1r
#sym { greet: g }
fn greet(name: Str) -> Str {
  return "Hello, " + name;
}
```

**Canonical AST (long names only):**
```rust
FnDecl {
  name: "greet",           // Always long form
  params: [("name", Str)],
  ret: Str,
  effects: [Pure],
  body: ...
}
```

### AST Structure

**Module:**
```rust
Module {
  name: "hello",
  version: Some("1.0"),
  ctx_budget: Some(64),
  caps: [],
  imports: [],
  decls: [/* types and functions */],
  sym_map: SymbolMap { ... }
}
```

**Function:**
```rust
FnDecl {
  name: "add",
  params: [("x", U32), ("y", U32)],
  ret: U32,
  effects: [Pure],
  body: Block { stmts: [...] }
}
```

### Common Parse Errors

```
Error: Expected '->' after function parameters
  │
3 │ fn add(x: U32, y: U32) Unit
  │                       ^
  │ help: Add return type annotation: '-> Unit'
```

```
Error: Unmatched opening brace
  │
5 │ fn main() {
  │           ^
  │
8 │   // Missing closing brace
  │
  │ help: Add '}' to close block
```

## Stage 3: Type Checking

The type checker validates that all operations are type-safe.

### Structural Type Checking

Zero1 uses **structural typing** - types are compatible if their structure matches:

```z1r
type Point = { x: U32, y: U32 }
type Coord = { x: U32, y: U32 }

// These are COMPATIBLE - same structure
fn usePoint(p: Point) -> Unit { ... }
fn main() {
  let c: Coord = Coord{ x: 10, y: 20 };
  usePoint(c);  // ✓ OK - structures match
}
```

### Type Resolution

The type checker:
1. Resolves type names through imports
2. Validates record field types
3. Checks function parameter/return types
4. Verifies union variant compatibility
5. Ensures type consistency in expressions

### Common Type Errors

```
Error: Type mismatch
  │
7 │ let x: U32 = "hello";
  │              ^^^^^^^
  │              expected: U32
  │              found: Str
```

```
Error: Unknown field 'age' in record type
  │
5 │ let p = Point{ x: 10, y: 20, age: 30 };
  │                              ^^^
  │ help: Type 'Point' has fields: x, y
```

```
Error: Cannot use type before import
  │
3 │ fn handler(req: Http.Request) -> Http.Response { ... }
  │                 ^^^^
  │ help: Add 'use "std/http/server" as Http'
```

## Stage 4: Effect Checking

The effect checker ensures functions declare all their side effects and that module capabilities allow them.

### Effect Validation

Every function must declare its effects:

```z1r
fn readFile(path: Str) -> Str
  eff [fs]  // Declares file system effect
{
  // Can call fs functions here
}

fn compute(x: U32) -> U32
  eff [pure]  // Pure - no side effects
{
  // Cannot call readFile - effect mismatch!
}
```

### Capability Enforcement

Module capabilities must include all effects used:

```z1r
module app : 1.0
  caps = [net]  // Only network allowed

fn download() -> Str
  eff [net]  // ✓ OK - net in caps
{ ... }

fn readFile() -> Str
  eff [fs]  // ✗ ERROR - fs not in caps
{ ... }
```

### Effect Subtyping

Functions can only call functions with compatible effects:

```z1r
// Pure can call pure
fn add(x: U32, y: U32) -> U32
  eff [pure]
{
  return x + y;  // ✓ OK
}

// Net can call pure and net
fn fetchAndProcess() -> U32
  eff [net]
{
  let data = download();  // ✓ OK - net function
  let result = add(1, 2); // ✓ OK - pure function
}

// Pure CANNOT call net
fn broken() -> U32
  eff [pure]
{
  return download();  // ✗ ERROR - net effect not allowed
}
```

### Common Effect Errors

```
Error: Effect 'fs' not in module capabilities
  │
5 │ fn readConfig() -> Str
  │    ^^^^^^^^^^
  │    eff [fs]
  │
  │ help: Add 'fs.ro' to module caps:
  │       module app : 1.0
  │         caps = [fs.ro]
```

```
Error: Cannot call function with 'net' effect from pure context
  │
8 │ fn process() -> Str
  │    eff [pure]
  │ {
  │   return download();
  │          ^^^^^^^^
  │ help: Add 'net' to function effects or make this function pure
```

## Stage 5: Context Estimation

The context estimator calculates token usage and enforces budgets.

### Token Estimation

The estimator:
1. Formats AST to compact syntax
2. Counts tokens using a model (default: chars ÷ 3.8)
3. Compares to declared budget
4. Reports violations

### Budget Declaration

**Module-level:**
```z1r
module myapp : 1.0
  ctx = 512  // Maximum 512 tokens for entire module
```

**Function-level (future):**
```z1r
fn complexFunction() -> Unit
  ctx = 256  // This function limited to 256 tokens
  eff [pure]
{ ... }
```

### Budget Enforcement

```
Error: Module exceeds context budget
  │
  │ Declared budget: 512 tokens
  │ Actual usage:    687 tokens
  │ Overage:         175 tokens
  │
  │ Suggestion: Split module at function 'processData' (line 45)
  │             Functions before: 234 tokens
  │             Functions after:  278 tokens
```

### Optimization Tips

To reduce token count:
1. Use compact syntax (`.z1c`)
2. Apply SymbolMap for common identifiers
3. Keep functions small and focused
4. Extract reusable logic to separate modules

## Stage 6: Policy Enforcement

The policy engine enforces Zero1's composability rules.

### Enforced Policies

**Cell Size Limits (default):**
- Max AST nodes: 200
- Max exports: 5
- Max imports: 10

**Function Complexity (default):**
- Max parameters: 6
- Max local variables: 32

**Dependency Limits:**
- Max fan-in: 10 (modules that import this one)
- Max fan-out: 10 (modules this one imports)

### Policy Violations

```
Error: Module exceeds maximum AST node count
  │
  │ Limit: 200 nodes
  │ Actual: 247 nodes
  │
  │ Suggestion: Split module into smaller cells
  │   Group 1 (types): 45 nodes
  │   Group 2 (utilities): 89 nodes
  │   Group 3 (main logic): 113 nodes
```

```
Error: Function has too many parameters
  │
5 │ fn process(a, b, c, d, e, f, g) -> Unit { ... }
  │            ^^^^^^^^^^^^^^^^^^^^
  │            7 parameters (limit: 6)
  │
  │ help: Use a record type to group related parameters
```

### Policy Configuration

Policies are configurable in `manifest.z1m`:

```toml
[policy]
cell.max_ast_nodes = 250  # Increase limit
cell.max_exports = 10
fn.max_params = 8
```

## Stage 7: Code Generation

The final stage generates executable code for target platforms.

### Target Languages

Zero1 generates:
1. **TypeScript** - For Node.js/browser runtimes
2. **WebAssembly** - For sandboxed, deterministic execution

### Code Generation Strategy

Zero1 uses **direct codegen** (not LLVM):
- Fast compilation (milliseconds, not minutes)
- Small binary size (CLI < 10MB)
- Deterministic output (same input → same code)
- Full control over generated code

See [ADR 001](../adr/001-codegen-strategy.md) for the design rationale.

### TypeScript Output

**Input:**
```z1r
fn add(x: U32, y: U32) -> U32
  eff [pure]
{
  return x + y;
}
```

**Generated TypeScript:**
```typescript
/**
 * @pure
 */
export function add(x: number, y: number): number {
  return x + y;
}
```

**Features:**
- Type annotations preserved
- Effect annotations in JSDoc
- Idiomatic TypeScript code
- ES6 module exports

### WebAssembly Output

**Input:**
```z1r
fn factorial(n: U32) -> U32
  eff [pure]
{
  return if n <= 1 { 1 } else { n * factorial(n - 1) };
}
```

**Generated WAT (WebAssembly Text):**
```wasm
(module
  (func $factorial (param $n i32) (result i32)
    (if (result i32)
      (i32.le_u (local.get $n) (i32.const 1))
      (then (i32.const 1))
      (else
        (i32.mul
          (local.get $n)
          (call $factorial
            (i32.sub (local.get $n) (i32.const 1)))))))
  (export "factorial" (func $factorial)))
```

**Features:**
- Deterministic binary output
- Capability-based imports (host functions)
- Optimized for small code size
- Stackful execution model

### Effect Metadata

Effects are preserved in generated code:

**TypeScript:**
```typescript
/** @effects [net, async] */
export async function download(url: string): Promise<string> {
  // Implementation uses network
}
```

**WASM:**
```wasm
;; Effects: [fs]
(import "host" "readFile" (func $readFile (param i32) (result i32)))
```

## Compilation Workflow

### Command Line

```bash
# Full compilation (all stages)
cargo run -p z1-cli -- z1c myapp.z1r --target ts

# Type check only
cargo run -p z1-cli -- check myapp.z1r

# Context estimation only
cargo run -p z1-cli -- ctx myapp.z1r

# Generate both targets
cargo run -p z1-cli -- z1c myapp.z1r --target ts --target wasm
```

### Compilation Output

```
Compiling myapp.z1r...
[1/7] Lexing...            ✓
[2/7] Parsing...           ✓
[3/7] Type checking...     ✓
[4/7] Effect checking...   ✓
[5/7] Context estimation... ✓ (234/512 tokens)
[6/7] Policy enforcement... ✓
[7/7] Code generation...   ✓

Output:
  myapp.ts      (TypeScript)
  myapp.wasm    (WebAssembly)
```

## Debugging Compilation Errors

### Read Error Messages Carefully

Zero1 provides detailed error messages with source context:

```
Error: Type mismatch in function call
  ┌─ cells/app.z1r:12:15
  │
12│   let result = process("hello");
  │                ^^^^^^^^^^^^^^^^
  │                expected: U32
  │                found: Str
  │
  = note: Function 'process' expects U32, got Str
  = help: Convert string to number: parseU32("hello")
```

### Use Incremental Checking

Test each stage independently:

```bash
# 1. Check parsing
cargo run -p z1-cli -- parse myapp.z1r

# 2. Check types
cargo run -p z1-cli -- check myapp.z1r

# 3. Check effects
cargo run -p z1-cli -- effects myapp.z1r

# 4. Check context
cargo run -p z1-cli -- ctx myapp.z1r
```

### Common Pitfalls

**1. Forgetting to declare effects**
```z1r
// ✗ WRONG - fs effect not declared
fn readFile(path: Str) -> Str {
  return Fs.readText(path);
}

// ✓ CORRECT
fn readFile(path: Str) -> Str
  eff [fs]
{
  return Fs.readText(path);
}
```

**2. Missing capability in module**
```z1r
// ✗ WRONG - no net capability
module app : 1.0
  caps = []

fn download() -> Str
  eff [net]  // Error: net not in caps
{ ... }

// ✓ CORRECT
module app : 1.0
  caps = [net]
```

**3. Context budget too small**
```z1r
// ✗ WRONG - budget too tight
module bigapp : 1.0
  ctx = 128  // Too small for complex app

// ✓ CORRECT - allocate realistic budget
module bigapp : 1.0
  ctx = 512
```

## Intermediate Representation (IR)

For complex optimizations, Zero1 uses a lightweight intermediate representation:

### IR Structure

```rust
enum IrExpr {
  Const(Value),
  Param(usize),
  Call { func: String, args: Vec<IrExpr> },
  BinOp { op: BinOp, left: Box<IrExpr>, right: Box<IrExpr> },
  RecordNew { fields: Vec<(String, IrExpr)> },
  Return(Box<IrExpr>)
}
```

### Optimization Passes

**Dead Code Elimination:**
```rust
// Before
let x = 42;
let y = compute();
return y;

// After (x eliminated)
let y = compute();
return y;
```

**Constant Folding:**
```rust
// Before
let x = 5 + 3 * 2;

// After
let x = 11;
```

**Constant Propagation:**
```rust
// Before
let x = 42;
let y = x + 10;
return y;

// After
return 52;
```

## Build Artifacts

A successful compilation produces:

```
output/
├── myapp.ts           # TypeScript output
├── myapp.d.ts         # TypeScript declarations
├── myapp.wasm         # WebAssembly binary
├── myapp.wat          # WebAssembly text (debug)
└── myapp.hash         # SemHash and FormHash
```

## Performance

Compilation times for typical cells:

| Stage | Time (typical) |
|-------|---------------|
| Lexing | < 1ms |
| Parsing | < 5ms |
| Type checking | < 10ms |
| Effect checking | < 5ms |
| Context estimation | < 10ms |
| Policy enforcement | < 5ms |
| Code generation (TS) | < 10ms |
| Code generation (WASM) | < 20ms |
| **Total** | **< 100ms** |

Zero1's direct codegen approach ensures fast iteration cycles for agent workflows.

## Provenance and Hashing

After compilation, Zero1 computes hashes:

**SemHash** (semantic hash):
- Excludes formatting, SymbolMap, comments
- Detects actual code changes
- Used in provenance chain

**FormHash** (formatting hash):
- Includes SymbolMap and formatting
- Detects style-only changes
- Used for format tracking

```bash
cargo run -p z1-cli -- hash myapp.z1c

Output:
  SemHash:  sha3-256:abc123...
  FormHash: sha3-256:def456...
```

## Next Steps

- **[Best Practices](05-best-practices.md)** - Write efficient, idiomatic Zero1 code
- **[Standard Library Reference](03-stdlib-reference.md)** - Use the standard library
- **[Grammar Reference](../grammar.md)** - Complete syntax specification
- **[ADR 001](../adr/001-codegen-strategy.md)** - Deep dive into codegen strategy
