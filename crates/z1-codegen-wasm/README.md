# z1-codegen-wasm

WebAssembly code generator for Zero1, supporting both text (WAT) and binary (WASM) output formats.

## Features

- **WAT Generation**: Generates WebAssembly Text format (.wat) from Zero1 IR
- **Binary Generation**: Generates binary WebAssembly (.wasm) using the `wat` crate
- **Validation**: Validates generated binaries using `wasmparser`
- **Optimization**: Supports multiple optimization levels (O0, O1, O2)
- **Complete Statement Support**: Full implementation of Zero1 statements, expressions, and control flow

## Usage

### Text Format (WAT)

Generate WebAssembly Text format:

```rust
use z1_codegen_wasm::generate_wasm;
use z1_ir::IrModule;

let module: IrModule = /* ... */;
let wat_code = generate_wasm(&module);
println!("{}", wat_code);
```

With optimization:

```rust
use z1_codegen_wasm::generate_wasm_optimized;
use z1_ir::optimize::OptLevel;

let wat_code = generate_wasm_optimized(&module, OptLevel::O2);
```

### Binary Format (WASM)

Generate binary WebAssembly:

```rust
use z1_codegen_wasm::generate_wasm_binary;

let binary = generate_wasm_binary(&module)
    .expect("Binary generation failed");

// Write to file
std::fs::write("output.wasm", binary)?;
```

With optimization:

```rust
use z1_codegen_wasm::generate_wasm_binary_optimized;
use z1_ir::optimize::OptLevel;

let binary = generate_wasm_binary_optimized(&module, OptLevel::O2)
    .expect("Binary generation failed");
```

### Validation

Validate generated binaries:

```rust
use z1_codegen_wasm::validate_wasm_binary;

let binary = generate_wasm_binary(&module)?;
validate_wasm_binary(&binary)
    .expect("Binary validation failed");
```

## CLI Usage

### Generate WAT (Text Format)

```bash
# Default: generates .wat file
z1c compile input.z1c --target wasm

# Explicit output path
z1c compile input.z1c --target wasm --output output.wat
```

### Generate WASM (Binary Format)

```bash
# Binary output with --binary flag
z1c compile input.z1c --target wasm --binary

# With optimization
z1c compile input.z1c --target wasm --binary -O O2

# Custom output path
z1c compile input.z1c --target wasm --binary --output output.wasm
```

## Supported Zero1 Features

### Types
- Primitives: `Bool`, `U16`, `U32`, `U64`, `Str`, `Unit`
- Records: `{ x: U32, y: U32 }`
- Named types with generics

### Statements
- Variable declarations: `let x = 42;`
- Assignments: `x = y + 1;`
- Conditionals: `if cond { } else { }`
- Loops: `while cond { }`
- Returns: `ret value;`
- Expression statements

### Expressions
- Literals: integers, booleans, strings
- Variables
- Binary operations: `+`, `-`, `*`, `/`, `%`, `==`, `!=`, `<`, `>`, `<=`, `>=`, `&&`, `||`
- Unary operations: `-`, `!`
- Function calls: `foo(arg1, arg2)`
- Field access: `record.field`
- Record construction: `{ x: 10, y: 20 }`

## WebAssembly Output

### Module Structure

Generated WASM modules include:

1. **Memory Section**: Linear memory (1 page = 64KB initially)
2. **Import Section**: External function imports
3. **Function Section**: Module functions
4. **Export Section**: Exported functions and memory
5. **Data Section**: String literals and static data

### Example Output

For a simple Zero1 function:

```zero1
fn add(x: U32, y: U32) -> U32
  eff [pure]
{
  ret x + y;
}
```

Generates WAT:

```wat
(module
  (memory $mem 1)
  (export "memory" (memory $mem))

  (func $add (param $x i32) (param $y i32) (result i32)
    local.get $x
    local.get $y
    i32.add
    return
  )
  (export "add" (func $add))
)
```

And binary WASM with magic number `0x00 0x61 0x73 0x6D` (WASM version 1).

## Memory Management

- **Heap allocation**: Starts at offset 1024 (first 1KB reserved)
- **Records**: Allocated on heap with 4-byte alignment
- **Strings**: Stored in data section with pointers
- **Stack variables**: Mapped to WASM locals

## Limitations

- **Field access**: Record field offsets are currently simplified (4 bytes per field)
- **Indirect calls**: Not yet fully implemented
- **Multi-value returns**: Limited support
- **Threading**: No support for WASM threads yet

## Testing

Run tests:

```bash
cargo test -p z1-codegen-wasm
```

Tests include:
- WAT generation for all statement/expression types
- Binary generation and validation
- Round-trip WAT → binary → validation
- Module structure verification
- Optimization level testing
- String literal handling

## Dependencies

- `z1-ir`: Zero1 intermediate representation
- `wat`: WAT-to-binary assembler (from Bytecode Alliance)
- `wasmparser`: Binary validation (from Bytecode Alliance)
- `anyhow`: Error handling
- `thiserror`: Custom errors

## References

- [WebAssembly Specification](https://webassembly.github.io/spec/)
- [WAT Format](https://webassembly.github.io/spec/core/text/index.html)
- [Bytecode Alliance Tools](https://github.com/bytecodealliance/wasm-tools)
