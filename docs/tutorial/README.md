# Zero1 Tutorial

Welcome to the Zero1 tutorial series! This collection of guides will help you master Zero1, from basic concepts to advanced patterns.

## Tutorial Contents

### 1. [Getting Started](01-getting-started.md)

**What you'll learn:**
- Installing Zero1 and building from source
- Writing your first Zero1 program
- Understanding dual syntax (compact vs relaxed)
- Using CLI tools: `z1fmt`, `z1hash`, `z1c`, `z1ctx`
- Basic project structure

**Who it's for:** Complete beginners to Zero1

**Time:** 30-45 minutes

**Start here if:** You've never written Zero1 code before

---

### 2. [Language Tour](02-language-tour.md)

**What you'll learn:**
- Module system and imports
- Type system: primitives, records, unions, generics
- Functions and effect annotations
- Capability-based security model
- Statements and control flow
- Operators and expressions
- Context budgets and Symbol maps

**Who it's for:** Developers learning the language features

**Time:** 60-90 minutes

**Start here if:** You want a comprehensive overview of the language

---

### 3. [Standard Library Reference](03-stdlib-reference.md)

**What you'll learn:**
- Overview of all stdlib modules
- HTTP client and server (`std/http`)
- Time and timers (`std/time`)
- File system operations (`std/fs`)
- Cryptographic primitives (`std/crypto`)
- Environment and process control (`std/env`)
- Complete API reference with examples

**Who it's for:** Developers building real applications

**Time:** Reference material (browse as needed)

**Start here if:** You need to look up stdlib functionality

---

### 4. [Compilation Guide](04-compilation.md)

**What you'll learn:**
- The 7-stage compilation pipeline
- Lexical analysis and parsing
- Type checking and effect checking
- Context estimation and policy enforcement
- Code generation (TypeScript and WASM)
- Debugging compilation errors
- Understanding hashes (SemHash vs FormHash)

**Who it's for:** Developers who want to understand the toolchain

**Time:** 45-60 minutes

**Start here if:** You want to understand how Zero1 works under the hood

---

### 5. [Best Practices](05-best-practices.md)

**What you'll learn:**
- Designing with capabilities (principle of least privilege)
- Managing context budgets effectively
- When to use compact vs relaxed syntax
- Organizing larger projects
- Testing strategies
- Performance considerations
- Security guidelines
- Error handling patterns

**Who it's for:** Developers writing production code

**Time:** 45-60 minutes

**Start here if:** You want to write idiomatic, maintainable Zero1 code

---

### 6. [Migration Guide](06-migration.md)

**What you'll learn:**
- Coming from TypeScript
- Coming from Rust
- Coming from Python
- Translation cheat sheets
- Common gotchas
- Learning paths by background

**Who it's for:** Experienced developers from other languages

**Time:** 30-45 minutes

**Start here if:** You're already proficient in TypeScript, Rust, or Python

---

## Learning Paths

Different backgrounds suggest different learning paths:

### Complete Beginner Path

1. [Getting Started](01-getting-started.md) - Build and run your first program
2. [Language Tour](02-language-tour.md) - Learn all language features
3. Work through examples in `examples/` directory
4. [Standard Library Reference](03-stdlib-reference.md) - Learn stdlib APIs
5. [Best Practices](05-best-practices.md) - Write production-quality code

**Estimated time:** 6-8 hours

### TypeScript Developer Path

1. [Migration Guide](06-migration.md) - Map TypeScript concepts to Zero1
2. [Getting Started](01-getting-started.md) - Set up toolchain
3. [Language Tour](02-language-tour.md) (focus on: effects, capabilities, types)
4. [Standard Library Reference](03-stdlib-reference.md) - Browse available APIs
5. Try the HTTP example: `examples/http-hello/`
6. [Best Practices](05-best-practices.md) - Learn Zero1 idioms

**Estimated time:** 3-4 hours

### Rust Developer Path

1. [Migration Guide](06-migration.md) - Understand differences from Rust
2. [Getting Started](01-getting-started.md) - Install and basics
3. [Language Tour](02-language-tour.md) (focus on: effect system, capabilities)
4. Try the file-copy example: `examples/file-copy/`
5. [Compilation Guide](04-compilation.md) - Understand the pipeline
6. [Best Practices](05-best-practices.md) - Zero1 patterns

**Estimated time:** 2-3 hours

### Python Developer Path

1. [Migration Guide](06-migration.md) - TypeScript and static typing primer
2. [Getting Started](01-getting-started.md) - Installation and first program
3. [Language Tour](02-language-tour.md) (focus on: types, effects, pattern matching)
4. Try the config-loader example: `examples/config-loader/`
5. [Standard Library Reference](03-stdlib-reference.md) - Explore APIs
6. [Best Practices](05-best-practices.md) - Best practices

**Estimated time:** 4-5 hours

### Quick Reference Path

For experienced developers who want quick answers:

1. [Migration Guide](06-migration.md) - Translation cheat sheets
2. [Standard Library Reference](03-stdlib-reference.md) - API quick reference
3. [Grammar Reference](../grammar.md) - Complete syntax
4. Browse `examples/` directory for patterns

**Estimated time:** 1-2 hours

## Additional Resources

### Documentation

- **[Grammar Reference](../grammar.md)** - Complete EBNF grammar
- **[Design Document](../design.md)** - Build plan and architecture
- **[Vision Document](../vision.md)** - High-level goals and philosophy
- **[ADR 001](../adr/001-codegen-strategy.md)** - Codegen strategy decisions

### Code Examples

Located in `examples/` directory:

- **http-hello/** - Simple HTTP server
- **file-copy/** - File I/O and error handling
- **password-hash/** - Cryptographic operations
- **time-demo/** - Timestamps and timers
- **config-loader/** - Environment variables and args

Each example includes:
- README with explanation
- Both compact (`.z1c`) and relaxed (`.z1r`) versions
- Comments explaining key concepts

### Standard Library Source

Browse the stdlib source code in `stdlib/`:

- **http/** - HTTP client and server
- **time/** - Time and timer modules
- **fs/** - File system operations
- **crypto/** - Cryptographic primitives
- **env/** - Environment and process control

Each module includes a README with API documentation.

## Getting Help

### Common Questions

**Q: Which syntax should I use?**
A: Use relaxed (`.z1r`) for development and compact (`.z1c`) for LLM contexts. See [Best Practices - Dual Syntax](05-best-practices.md#when-to-use-compact-vs-relaxed-syntax).

**Q: What are effects and why are they required?**
A: Effects declare side effects (I/O, network, etc.). They enable the compiler to enforce capability restrictions. See [Language Tour - Effect System](02-language-tour.md#effect-system).

**Q: How do I handle errors without exceptions?**
A: Use `Result<T, E>` types and pattern matching. See [Language Tour - Error Handling](02-language-tour.md#error-handling-patterns) and [Best Practices - Error Patterns](05-best-practices.md#error-handling-patterns).

**Q: What's the difference between SemHash and FormHash?**
A: SemHash excludes formatting (detects real changes), FormHash includes formatting (detects style changes). See [Compilation Guide - Hashing](04-compilation.md#provenance-and-hashing).

**Q: How do I stay within my context budget?**
A: Use compact syntax, Symbol maps, selective imports, and split large modules. See [Best Practices - Context Budgets](05-best-practices.md#managing-context-budgets).

### Troubleshooting

**Parse errors:**
- Check for missing closing braces `}`
- Ensure effect annotations are present
- Verify semicolons after statements

**Type errors:**
- Check import statements include required types
- Ensure record field names match exactly
- Verify type compatibility (structural matching)

**Effect errors:**
- Add missing effect to function signature
- Add required capability to module header
- Check effect subtyping (pure can't call net)

**Context budget errors:**
- Estimate actual usage with `z1ctx` command
- Split large modules into smaller cells
- Use compact syntax and Symbol maps

See [Compilation Guide - Debugging](04-compilation.md#debugging-compilation-errors) for detailed troubleshooting.

### Community and Support

- **GitHub Issues** - Bug reports and feature requests
- **Examples** - Real-world code samples
- **CLAUDE.md** - Development guidelines for the project

## Tutorial Feedback

These tutorials are actively maintained. If you find:

- Errors or outdated information
- Confusing explanations
- Missing topics
- Broken examples

Please open an issue on GitHub with details.

## Next Steps

Ready to start? Choose your path:

- **New to Zero1?** → [Getting Started](01-getting-started.md)
- **Coming from TypeScript/Rust/Python?** → [Migration Guide](06-migration.md)
- **Want a complete overview?** → [Language Tour](02-language-tour.md)
- **Building an application?** → [Standard Library Reference](03-stdlib-reference.md)
- **Need quick syntax reference?** → [Grammar Reference](../grammar.md)

Welcome to Zero1!
