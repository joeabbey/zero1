# z1-ir

Zero1 Intermediate Representation (IR) crate.

## Overview

This crate provides a simplified, lower-level representation of Zero1 code optimized for code generation. The IR eliminates syntactic sugar and normalizes the AST into a form that's easier to compile to target languages.

## Features

- Converts Z1 AST to a simplified IR representation
- Preserves all semantic information (types, effects, imports)
- Provides error handling for unsupported constructs
- Foundation for code generation to TypeScript, WASM, and other targets

## Usage

```rust
use z1_ir::*;
use z1_ast as ast;

// Parse and lower a Z1 module to IR
let module: ast::Module = /* ... */;
let ir = lower_to_ir(&module)?;

// IR can now be used for code generation
println!("Module {} has {} functions", ir.name, ir.functions.len());
```

## IR Structure

The IR consists of:

- `IrModule`: Top-level module with imports, types, functions, and exports
- `IrType`: Simplified type representation (Bool, Str, U16, U32, U64, Unit, Named, Record, Union, Generic)
- `IrFunction`: Function with parameters, return type, effects, and body
- `IrStmt`: Statements (Let, Assign, If, While, Return, Expr)
- `IrExpr`: Expressions (Var, Literal, BinOp, UnaryOp, Call, Field, Record, Path)

## Testing

The crate includes comprehensive tests covering:

- Module lowering
- Type definitions (records, primitives)
- Functions with parameters and effects
- Statements (let, if, return)
- Expressions (binary operations, calls, field access, record literals)
- Import and export preservation
- Complex nested expressions

Run tests with:

```bash
cargo test -p z1-ir
```
