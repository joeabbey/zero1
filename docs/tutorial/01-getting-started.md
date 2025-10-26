# Getting Started with Zero1

Welcome to Zero1 (Z1), a programming language optimized for LLM agent workflows. This guide will help you install Zero1, write your first program, and understand the basics of the toolchain.

## What is Zero1?

Zero1 is designed to solve unique challenges in LLM-driven development:

- **Token Efficiency**: Minimize token usage for LLM context with compact syntax
- **Auditability**: Track every code change with cryptographic provenance
- **Safety**: Fine-grained capabilities and effect system prevent unauthorized operations
- **Dual Syntax**: Write in compact mode for LLMs, relaxed mode for humans

## Installation

### Prerequisites

Zero1 is written in Rust and requires:

- **Rust 1.75+** - Install from [rustup.rs](https://rustup.rs)
- **Git** - For cloning the repository

### Building from Source

```bash
# Clone the repository
git clone https://github.com/yourusername/zero1.git
cd zero1

# Build the toolchain
cargo build --release

# The CLI will be available at:
# target/release/z1-cli
```

### Verify Installation

```bash
# Check that everything works
cargo run -p z1-cli -- --help

# You should see the Zero1 CLI help message
```

## Your First Zero1 Program

Let's write a classic "Hello, World!" program in both Zero1 syntaxes.

### Compact Syntax (`.z1c`)

Create a file called `hello.z1c`:

```z1c
m hello:1.0 ctx=64 caps=[]

#sym { main: mn, greet: gr }

f gr(name:Str)->Str eff [pure] {
  ret "Hello, " + name + "!";
}

f mn()->Unit eff [pure] {
  let msg: Str = gr("World");
  ret ();
}
```

### Relaxed Syntax (`.z1r`)

The same program in relaxed syntax (`hello.z1r`):

```z1r
module hello : 1.0
  ctx = 64
  caps = []

#sym { main: mn, greet: gr }

fn greet(name: Str) -> Str
  eff [pure]
{
  return "Hello, " + name + "!";
}

fn main() -> Unit
  eff [pure]
{
  let msg: Str = greet("World");
  return ();
}
```

### Understanding the Program

Let's break down what each part does:

**Module Declaration**
```z1c
m hello:1.0 ctx=64 caps=[]
```
- `m` (or `module`) - Declares a module named `hello`, version `1.0`
- `ctx=64` - Context budget: this module uses at most 64 tokens
- `caps=[]` - No special capabilities needed (no network, file system, etc.)

**Symbol Map**
```z1c
#sym { main: mn, greet: gr }
```
- Maps long names to short names for compact syntax
- `main` becomes `mn`, `greet` becomes `gr`
- Only affects formatting, not semantics

**Function Declaration**
```z1c
f gr(name:Str)->Str eff [pure] { ... }
```
- `f` (or `fn`) - Declares a function
- `gr` - Function name (short form of `greet`)
- `name:Str` - Parameter: `name` of type `Str`
- `->Str` - Returns a `Str`
- `eff [pure]` - Pure effect (no side effects)

## Using the CLI Tools

Zero1 provides several command-line tools:

### Formatting (`z1fmt`)

Convert between compact and relaxed syntax:

```bash
# Convert compact to relaxed
cargo run -p z1-cli -- fmt hello.z1c --mode relaxed > hello.z1r

# Convert relaxed to compact
cargo run -p z1-cli -- fmt hello.z1r --mode compact > hello.z1c

# Check if file is properly formatted
cargo run -p z1-cli -- fmt hello.z1c --check
```

### Hashing (`z1hash`)

Compute semantic and formatting hashes:

```bash
# Hash a cell
cargo run -p z1-cli -- hash hello.z1c

# Output shows:
# SemHash:  sha3-256:abc123...  (semantic content)
# FormHash: sha3-256:def456...  (includes formatting)
```

**Important**: Both `.z1c` and `.z1r` versions produce the **same SemHash** because they're semantically identical.

### Type Checking

Verify types and effects:

```bash
# Parse and type-check a cell
cargo run -p z1-cli -- z1c hello.z1c

# This will report any type errors, effect violations, or capability issues
```

### Context Estimation

Check token usage against budget:

```bash
# Estimate context usage
cargo run -p z1-cli -- ctx hello.z1c

# Shows token count and compares to declared budget
```

## Project Structure

A typical Zero1 project looks like this:

```
my-project/
├── manifest.z1m           # Project metadata and configuration
├── cells/                 # Source code (one module per file)
│   ├── main.z1c          # Compact syntax
│   └── main.z1r          # Relaxed syntax (optional)
├── tests/                 # Test specifications
│   └── main.spec.z1r
├── prov/                  # Provenance chain (audit trail)
│   └── PROVCHAIN.z1p
└── dicts/                 # Model-specific dictionaries (optional)
    └── gpt-4.sdict
```

## Understanding Dual Syntax

Zero1's unique feature is dual syntax - every program can be written in two ways:

### Compact Syntax (`.z1c`)
- **Purpose**: Minimize tokens for LLM context
- **Keywords**: Short (`m`, `u`, `t`, `f`, `ret`)
- **Names**: Use SymbolMap short forms
- **Whitespace**: Minimal
- **Use Case**: Feeding code to LLMs, context-constrained scenarios

### Relaxed Syntax (`.z1r`)
- **Purpose**: Human readability
- **Keywords**: Long (`module`, `use`, `type`, `fn`, `return`)
- **Names**: Full descriptive names
- **Whitespace**: 2-space indentation, vertical spacing
- **Use Case**: Development, code review, documentation

### Key Guarantee

Both syntaxes parse to the **same canonical AST** and produce the **same SemHash**. You can switch between them freely:

```bash
# Compact → Relaxed → Compact produces identical SemHash
cargo run -p z1-cli -- hash hello.z1c
cargo run -p z1-cli -- fmt hello.z1c --mode relaxed | cargo run -p z1-cli -- hash --stdin
# Hashes match!
```

## Common Workflows

### Development Workflow

1. **Write code** in relaxed syntax (`.z1r`) for readability
2. **Test** using the CLI tools
3. **Format to compact** (`.z1c`) when feeding to LLMs
4. **Hash** to verify semantic consistency

### LLM Agent Workflow

1. **Receive compact code** (`.z1c`) from previous agent
2. **Verify hash** to ensure integrity
3. **Expand to relaxed** for analysis (optional)
4. **Make changes** in compact syntax
5. **Record provenance** with prompt and model info
6. **Hash output** for next agent

## Next Steps

Now that you have Zero1 installed and understand the basics:

1. **[Language Tour](02-language-tour.md)** - Learn all language features in depth
2. **[Standard Library Reference](03-stdlib-reference.md)** - Explore available modules
3. **[Compilation Guide](04-compilation.md)** - Understand the compilation pipeline
4. **[Best Practices](05-best-practices.md)** - Write idiomatic Zero1 code

## Try It Yourself

Create a simple calculator program:

```z1c
m calc:1.0 ctx=128 caps=[]

#sym { add: a, multiply: m, calculate: c }

f a(x:U32,y:U32)->U32 eff [pure] { ret x + y; }
f m(x:U32,y:U32)->U32 eff [pure] { ret x * y; }

f c()->U32 eff [pure] {
  let sum: U32 = a(5, 3);
  let product: U32 = m(4, 2);
  ret a(sum, product);
}
```

**Exercises**:
1. Convert this to relaxed syntax
2. Verify both versions produce the same SemHash
3. Add a `subtract` function
4. Check that context stays within the 128-token budget

## Getting Help

- **Documentation**: `docs/` directory in the repository
- **Grammar**: `docs/grammar.md` for complete syntax reference
- **Examples**: `examples/` directory for real-world code
- **Issues**: GitHub Issues for bugs and feature requests

Welcome to Zero1! Happy coding!
