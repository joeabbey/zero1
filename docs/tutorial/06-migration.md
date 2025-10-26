# Migration Guide

This guide helps developers coming from other languages learn Zero1 quickly by relating concepts to familiar patterns.

## Coming from TypeScript

If you're familiar with TypeScript, you'll find Zero1's syntax approachable.

### Module System

**TypeScript:**
```typescript
// mymodule.ts
export function greet(name: string): string {
  return `Hello, ${name}!`;
}
```

**Zero1:**
```z1r
module mymodule : 1.0
  ctx = 128
  caps = []

fn greet(name: Str) -> Str
  eff [pure]
{
  return "Hello, " + name + "!";
}
```

**Key Differences:**
- Zero1 requires module header with context budget and capabilities
- Effect annotations (`eff [pure]`) are mandatory
- One module per file (cell)

### Type System

**TypeScript:**
```typescript
type User = {
  name: string;
  age: number;
  active: boolean;
};

type Result<T, E> =
  | { ok: true; value: T }
  | { ok: false; error: E };
```

**Zero1:**
```z1r
type User = {
  name: Str,
  age: U32,
  active: Bool
}

type Result<T, E> =
  Ok{ value: T } |
  Err{ error: E }
```

**Key Differences:**
- No semicolons in type definitions
- Sum types use `|` with named variants
- Integer types are explicit (`U16`, `U32`, `U64`)

### Functions

**TypeScript:**
```typescript
function add(x: number, y: number): number {
  return x + y;
}

async function fetchData(url: string): Promise<string> {
  const response = await fetch(url);
  return response.text();
}
```

**Zero1:**
```z1r
fn add(x: U32, y: U32) -> U32
  eff [pure]
{
  return x + y;
}

fn fetchData(url: Str) -> Str
  eff [net, async]
{
  let response = await Http.get(url);
  return response;
}
```

**Key Differences:**
- Return type uses `->` instead of `:`
- Effects must be declared (`eff [pure]`, `eff [net, async]`)
- `async` is an effect, not a function modifier

### Imports

**TypeScript:**
```typescript
import { get, post } from "http-client";
import * as Http from "http-client";
```

**Zero1:**
```z1r
use "std/http/client" as Http only [get, post]
use "std/http/client" as Http
```

**Key Differences:**
- Use `use` instead of `import`
- `only` keyword for selective imports
- Always requires an alias (`as`)

### Error Handling

**TypeScript:**
```typescript
try {
  const data = readFileSync("config.json");
  return JSON.parse(data);
} catch (error) {
  console.error("Failed:", error);
  return null;
}
```

**Zero1:**
```z1r
fn loadConfig() -> Result<Config, Str>
  eff [fs]
{
  let fileResult = Fs.readText("config.json");
  return match fileResult {
    Ok{ value: data } -> parseJson(data),
    Err{ error: msg } -> Err{ error: "Failed: " + msg }
  };
}
```

**Key Differences:**
- No exceptions - use `Result<T, E>` types
- Pattern matching with `match` instead of try-catch
- Errors are explicit in function signatures

### Common Patterns

| TypeScript | Zero1 |
|------------|-------|
| `let x: string = "hello"` | `let x: Str = "hello"` |
| `const arr: number[]` | `let arr: Vec<U32>` |
| `x === y` | `x == y` |
| `x !== y` | `x != y` |
| `!condition` | `!condition` |
| `a && b` | `a && b` |
| `a \|\| b` | `a \|\| b` |

### What Zero1 Doesn't Have (Yet)

- Classes and inheritance
- `null` or `undefined` (use `Option<T>`)
- Mutable by default (use `let mut`)
- Dynamic typing (all types are static)
- `any` type (strong typing enforced)

## Coming from Rust

Rust developers will find Zero1's semantics familiar but syntax lighter.

### Module and Visibility

**Rust:**
```rust
// lib.rs
pub mod user {
    pub struct User {
        pub name: String,
        pub age: u32,
    }
}

pub use user::User;
```

**Zero1:**
```z1r
// user.z1r
module myapp.user : 1.0
  ctx = 256
  caps = []

type User = {
  name: Str,
  age: U32
}
```

**Key Differences:**
- No visibility modifiers (all exports are public)
- One module per file automatically
- Context budgets required

### Type System

**Rust:**
```rust
enum Result<T, E> {
    Ok(T),
    Err(E),
}

struct Point {
    x: u32,
    y: u32,
}
```

**Zero1:**
```z1r
type Result<T, E> = Ok{ value: T } | Err{ error: E }

type Point = {
  x: U32,
  y: U32
}
```

**Key Differences:**
- Sum types use `|` syntax, not `enum`
- Struct fields in records don't need `pub`
- Named fields in variants (not tuple variants)

### Functions and Effects

**Rust:**
```rust
fn add(x: u32, y: u32) -> u32 {
    x + y
}

async fn fetch(url: &str) -> Result<String, Error> {
    // Network I/O
}
```

**Zero1:**
```z1r
fn add(x: U32, y: U32) -> U32
  eff [pure]
{
  return x + y;
}

fn fetch(url: Str) -> Result<Str, Str>
  eff [net, async]
{
  // Network I/O
}
```

**Key Differences:**
- Effect annotations required (`eff [pure]`, `eff [net]`)
- `async` is an effect, not a keyword
- No implicit return (must use `return`)

### Ownership and Borrowing

**Rust:**
```rust
fn process(data: &str) -> String {
    data.to_uppercase()
}

let s = String::from("hello");
let result = process(&s);
// s still valid here
```

**Zero1:**
```z1r
fn process(data: Str) -> Str
  eff [pure]
{
  return toUpperCase(data);
}

let s = "hello";
let result = process(s);
// Zero1 has no ownership system (runtime manages memory)
```

**Key Differences:**
- No ownership/borrowing (runtime-managed)
- No lifetimes
- Simpler mental model (at cost of runtime overhead)

### Pattern Matching

**Rust:**
```rust
match result {
    Ok(value) => println!("Got: {}", value),
    Err(e) => eprintln!("Error: {}", e),
}
```

**Zero1:**
```z1r
match result {
  Ok{ value } -> handleSuccess(value),
  Err{ error } -> handleError(error)
}
```

**Key Differences:**
- Named fields in patterns (`Ok{ value }` not `Ok(value)`)
- Use `->` not `=>`
- No `println!` macro (use functions)

### What Zero1 Doesn't Have (Yet)

- Ownership/borrowing system
- Lifetimes
- Macros
- Traits (use structural typing)
- `impl` blocks (functions are top-level)
- Crates system (uses module paths)

## Coming from Python

Python developers will appreciate Zero1's explicit types and safety.

### Basic Syntax

**Python:**
```python
def greet(name: str) -> str:
    return f"Hello, {name}!"

def add(x: int, y: int) -> int:
    return x + y
```

**Zero1:**
```z1r
fn greet(name: Str) -> Str
  eff [pure]
{
  return "Hello, " + name + "!";
}

fn add(x: U32, y: U32) -> U32
  eff [pure]
{
  return x + y;
}
```

**Key Differences:**
- Type annotations are required, not optional
- Effect annotations (`eff [pure]`) mandatory
- Braces instead of indentation
- Semicolons after statements

### Data Structures

**Python:**
```python
from typing import Dict, List, Optional

User = Dict[str, any]  # Or use dataclass

def process(users: List[User]) -> Optional[User]:
    if len(users) > 0:
        return users[0]
    return None
```

**Zero1:**
```z1r
type User = {
  name: Str,
  age: U32
}

type UserOption = Some{ user: User } | None{}

fn process(users: Vec<User>) -> UserOption
  eff [pure]
{
  if len(users) > 0 {
    return Some{ user: users[0] };
  } else {
    return None{};
  }
}
```

**Key Differences:**
- Structural types, not duck typing
- No `None` built-in (define `Option<T>`)
- Explicit variant construction

### Imports

**Python:**
```python
from http.client import get, post
import time
```

**Zero1:**
```z1r
use "std/http/client" as Http only [get, post]
use "std/time/core" as Time
```

**Key Differences:**
- Must use alias (`as`)
- `only` for selective imports
- String paths, not dotted names

### Error Handling

**Python:**
```python
try:
    with open("config.json") as f:
        data = f.read()
    return json.loads(data)
except FileNotFoundError:
    return None
except json.JSONDecodeError as e:
    print(f"Parse error: {e}")
    return None
```

**Zero1:**
```z1r
fn loadConfig() -> Result<Config, Str>
  eff [fs]
{
  let fileResult = Fs.readText("config.json");
  return match fileResult {
    Ok{ value: data } -> parseConfig(data),
    Err{ error } -> Err{ error: "Failed to load: " + error }
  };
}
```

**Key Differences:**
- No exceptions (use `Result<T, E>`)
- Pattern matching instead of try-except
- Errors in return type

### Effects vs Side Effects

**Python:**
```python
# Implicit effects
def fetch_data(url: str) -> str:
    response = requests.get(url)  # Network I/O
    return response.text

def read_config() -> dict:
    with open("config.json") as f:  # File I/O
        return json.load(f)
```

**Zero1:**
```z1r
// Explicit effects
fn fetchData(url: Str) -> Str
  eff [net, async]  // Declares network access
{
  return Http.get(url);
}

fn readConfig() -> Config
  eff [fs]  // Declares file system access
{
  return Fs.readText("config.json");
}
```

**Key Differences:**
- All effects must be declared
- Compiler enforces capability requirements
- No silent side effects

### What Zero1 Doesn't Have (Yet)

- Dynamic typing
- Decorators
- List comprehensions
- Generators
- Multiple inheritance
- `None` (use `Option<T>`)

## Common Gotchas

### All Languages

**1. Must Declare Effects**
```z1r
// ✗ ERROR - Missing effect annotation
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

**2. Must Declare Capabilities**
```z1r
// ✗ ERROR - Missing capability
module app : 1.0
  caps = []

fn download() -> Str
  eff [net]  // Error: net not in caps
{ ... }

// ✓ CORRECT
module app : 1.0
  caps = [net]
```

**3. Explicit Return Required**
```z1r
// ✗ ERROR - Missing return
fn add(x: U32, y: U32) -> U32
  eff [pure]
{
  x + y  // Error: not a statement
}

// ✓ CORRECT
fn add(x: U32, y: U32) -> U32
  eff [pure]
{
  return x + y;
}
```

**4. Context Budget Required**
```z1r
// ✗ WARNING - No budget
module app : 1.0
  caps = []
// Compiler will suggest adding ctx=N

// ✓ CORRECT
module app : 1.0
  ctx = 512
  caps = []
```

## Translation Cheat Sheet

### TypeScript → Zero1

| TypeScript | Zero1 |
|------------|-------|
| `string` | `Str` |
| `number` | `U32` (or `U16`, `U64`) |
| `boolean` | `Bool` |
| `void` | `Unit` |
| `Array<T>` | `Vec<T>` |
| `T \| null` | `Option<T>` (define it) |
| `Promise<T>` | Effect `[async]` |
| `export function` | `fn` (auto-exported) |
| `import { x } from "m"` | `use "m" as M only [x]` |
| `try/catch` | `match Result<T, E>` |

### Rust → Zero1

| Rust | Zero1 |
|------|-------|
| `String` / `&str` | `Str` |
| `u32` | `U32` |
| `bool` | `Bool` |
| `()` | `Unit` |
| `Vec<T>` | `Vec<T>` |
| `Option<T>` | Define as sum type |
| `Result<T, E>` | `Ok{} \| Err{}` |
| `async fn` | `eff [async]` |
| `pub fn` | `fn` (all public) |
| `use module::item` | `use "module" as M only [item]` |
| `match` | `match` (same concept) |

### Python → Zero1

| Python | Zero1 |
|--------|-------|
| `str` | `Str` |
| `int` | `U32` |
| `bool` | `Bool` |
| `None` | `None{}` variant |
| `List[T]` | `Vec<T>` |
| `Optional[T]` | `Some{} \| None{}` |
| `Dict[K, V]` | Record type |
| `def` | `fn` |
| `async def` | `eff [async]` |
| `from m import x` | `use "m" as M only [x]` |
| `try/except` | `match Result<T, E>` |

## Learning Path by Background

### For TypeScript Developers

1. Start with [Getting Started](01-getting-started.md)
2. Focus on effect system - similar to `async`, but broader
3. Learn capability model - new concept for web developers
4. Study error handling with `Result<T, E>` instead of exceptions
5. Practice with [examples/http-hello](../../examples/http-hello/)

### For Rust Developers

1. Skim [Getting Started](01-getting-started.md)
2. Note differences: no ownership, simpler mental model
3. Learn effect system - maps to side effect tracking
4. Understand capability enforcement - similar to type system
5. Practice with [examples/file-copy](../../examples/file-copy/)

### For Python Developers

1. Read [Getting Started](01-getting-started.md) carefully
2. Get comfortable with static typing - no runtime type changes
3. Learn effect annotations - explicit side effects
4. Study pattern matching - replaces many if-elif chains
5. Practice with [examples/config-loader](../../examples/config-loader/)

## Next Steps

- **[Language Tour](02-language-tour.md)** - Comprehensive language overview
- **[Best Practices](05-best-practices.md)** - Write idiomatic Zero1
- **[Standard Library](03-stdlib-reference.md)** - Explore available modules
- **[Examples](../../examples/)** - Real-world code samples
