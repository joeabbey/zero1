# Z1 Type Checker Progress

## Completed (M1 Milestone)

### Core Type Checking
- **Structural type system**: Records are compared structurally (field-order independent)
- **Type environment**: Tracks type definitions and imported types
- **Path type resolution**: Handles both local and imported types via aliases
- **Primitive types**: Bool, Str, Unit, U16, U32, U64
- **Record types**: Full structural equality checking
- **Function types**: Parameter and return type checking

### Effect System
- **Effect tracking**: Functions declare effects (pure, net, fs, time, crypto, env, async, unsafe)
- **Effect compatibility**: Call sites checked against available effects
- **Capability mapping**: Effects mapped to required capabilities
- **Capability checking**: Module capabilities verified against function requirements

### Context Management
- **Type environment**: Tracks type definitions and imports
- **Function context**: Manages function signatures and available effects
- **Variable scope**: Tracks variable types in current scope
- **Scope transitions**: Proper handling of function and block scopes

## Known Limitations (MVP)

### Statement and Expression Type Checking
The current AST has `Block.raw` as a String because statement parsing is not yet complete. This means:
- **Function bodies are not type checked** beyond signature validation
- **Expression type inference** is not implemented
- **Variable assignments** are not type checked
- **Return statement types** are not verified against function signature

This is an expected limitation for the MVP since we're focusing on type and function signature validation first.

### Import System
- **Imported types are treated as opaque**: We register them in the type environment but don't resolve their actual structure
- **No module loading**: The type checker doesn't load external modules to verify imports
- **Limited path resolution**: Only basic alias resolution is implemented

For a complete implementation, we would need:
- A module loader to read imported modules
- Full path resolution with multi-level imports
- Type information from external modules

### Generic Types
- **Basic generic support only**: Generic type syntax is parsed and stored
- **No type parameter checking**: Type parameters are not validated or substituted
- **No type inference**: Generic type arguments must be explicit

For full generic support, we would need:
- Type parameter constraint checking
- Type substitution during instantiation
- Type inference for generic function calls

### Sum Types / Variants
- **Syntax support only**: Sum type syntax is recognized but not fully implemented
- **No variant checking**: Variant constructors are not validated
- **No exhaustiveness checking**: Match statements are not checked for completeness

### Other Limitations
- **No recursive type checking**: Recursive types may cause issues
- **No higher-kinded types**: Only basic type constructors are supported
- **No trait/interface system**: No support for polymorphism beyond generics
- **Limited error messages**: Span information is basic, could be more detailed

## Testing

### Unit Tests (16 tests)
- Type equality checking (primitives, records, paths)
- Effect tracking and compatibility
- Capability checking
- Function call type checking (arity, parameter types)
- Context management (scopes, inheritance)

### Integration Tests (8 tests)
- Simple module with type declarations
- Functions with pure effects
- Capability requirement checking
- Import processing with aliases
- Structural record type equality
- Full http_server.z1c example

All tests pass with `cargo test -p z1-typeck`.

## Next Steps (Future Work)

1. **Complete statement AST**: Implement full statement parsing in z1-parse
2. **Expression type checking**: Add type inference and checking for expressions
3. **Control flow**: Type check if/else, while, match statements
4. **Module system**: Implement module loading and full import resolution
5. **Generic type checking**: Add type parameter validation and substitution
6. **Sum type checking**: Implement variant validation and exhaustiveness checking
7. **Better error messages**: Add more context and suggestions to type errors
8. **Type inference**: Implement Hindley-Milner style type inference where appropriate

## Architecture

### Module Structure
```
z1-typeck/
├── src/
│   ├── lib.rs        # Public API
│   ├── types.rs      # Type representation and equality
│   ├── env.rs        # Context and environment management
│   ├── checker.rs    # Main type checking logic
│   └── errors.rs     # Error types
└── tests/
    └── integration.rs # Integration tests
```

### Key Design Decisions

1. **Structural typing**: Records are compared by field names and types, not identity
2. **BTreeMap for records**: Ensures deterministic ordering for equality checks
3. **Opaque imports**: Treat imported types as path types for MVP
4. **Effect subtyping**: Function effects must be a subset of context effects
5. **Capability checking**: Enforced at function declaration time, not call time

## Performance Considerations

The current implementation is focused on correctness over performance:
- Type equality uses recursive structural comparison
- No caching of type resolution results
- Full AST traversal for each module check

For larger codebases, consider:
- Memoizing type equality checks
- Caching resolved types
- Incremental type checking
- Parallel module checking

## Compatibility

- Requires `z1-ast` for AST definitions
- Uses `thiserror` for error handling
- Uses `serde` for potential serialization (not yet used)
- Follows workspace-level Rust edition and version settings
