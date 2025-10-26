# Zero1 Language Tour

This guide provides a comprehensive tour of Zero1's language features, from basic syntax to advanced capabilities.

## Module System

Every Zero1 source file is a **cell** containing exactly one module.

### Module Declaration

**Compact:**
```z1c
m myapp.core:1.2.0 ctx=512 caps=[net, fs.ro]
```

**Relaxed:**
```z1r
module myapp.core : 1.2.0
  ctx = 512
  caps = [net, fs.ro]
```

**Components:**
- **Name**: `myapp.core` - dotted identifier for hierarchical organization
- **Version**: `1.2.0` - semantic versioning (optional)
- **Context budget**: `ctx=512` - maximum token count (optional, enforced by compiler)
- **Capabilities**: `caps=[net, fs.ro]` - permissions this module requires

### Imports

Import other modules using `use` (or `u` in compact):

**Compact:**
```z1c
u "std/http/server" as H only [listen, Req, Res]
u "std/time/core" as T
u "myapp/types"
```

**Relaxed:**
```z1r
use "std/http/server" as H only [listen, Req, Res]
use "std/time/core" as T
use "myapp/types"
```

**Import Options:**
- **Alias**: `as H` - Use `H.listen()` instead of full path
- **Selective**: `only [...]` - Import specific items, reduce token count
- **Full**: Without `only`, all exports are available

## Type System

Zero1 uses **structural typing** - types are compatible if their structure matches, regardless of name.

### Primitive Types

```z1r
Bool    // true or false
Str     // UTF-8 string
Unit    // Empty type, like void
U16     // Unsigned 16-bit integer
U32     // Unsigned 32-bit integer
U64     // Unsigned 64-bit integer
```

### Type Declarations

Define custom types with `type` (or `t` in compact):

**Compact:**
```z1c
t User = { name: Str, age: U32, active: Bool }
t Status = { code: U32, msg: Str }
```

**Relaxed:**
```z1r
type User = {
  name: Str,
  age: U32,
  active: Bool
}

type Status = {
  code: U32,
  msg: Str
}
```

### Record Types

Records are structural - order doesn't matter:

```z1r
type Point2D = { x: U32, y: U32 }
type Point3D = { x: U32, y: U32, z: U32 }

// These are equivalent:
type A = { name: Str, id: U32 }
type B = { id: U32, name: Str }  // Same structure!
```

### Union Types

Express alternatives with `|`:

**Compact:**
```z1c
t Result = Ok{ value: Str } | Err{ error: Str }
t Option = Some{ val: U32 } | None{}
```

**Relaxed:**
```z1r
type Result = Ok{ value: Str } | Err{ error: Str }
type Option = Some{ val: U32 } | None{}
```

### Generic Types

Generics allow type parameters:

```z1r
type Box<T> = { value: T }
type Result<T, E> = Ok{ val: T } | Err{ err: E }

// Usage:
type IntBox = Box<U32>
type StrResult = Result<Str, Str>
```

### Path Types

Reference types from imports:

```z1r
use "std/http/server" as H

fn handler(req: H.Req) -> H.Res {
  // req has type from imported module
}
```

## Functions

Functions are the core of Zero1 programs.

### Function Declaration

**Compact:**
```z1c
f add(x:U32, y:U32)->U32 eff [pure] {
  ret x + y;
}
```

**Relaxed:**
```z1r
fn add(x: U32, y: U32) -> U32
  eff [pure]
{
  return x + y;
}
```

**Components:**
- **Name**: `add`
- **Parameters**: `(x: U32, y: U32)` - name and type
- **Return type**: `-> U32`
- **Effect annotation**: `eff [pure]` - declares side effects
- **Body**: Block with statements

### Effect System

Every function declares its effects:

```z1r
// Pure function - no side effects
fn calculate(x: U32) -> U32
  eff [pure]
{
  return x * 2;
}

// Network access
fn fetch(url: Str) -> Str
  eff [net, async]
{
  // Can make HTTP requests
}

// File system read
fn readConfig() -> Str
  eff [fs]
{
  // Can read files
}

// Multiple effects
fn processFile(path: Str) -> Unit
  eff [fs, time]
{
  // Can read files and access time
}
```

**Available Effects:**
- `pure` - No side effects, deterministic
- `net` - Network access
- `fs` - File system access
- `time` - Time/clock access
- `crypto` - Cryptographic operations
- `env` - Environment variables, process control
- `async` - Asynchronous operations
- `unsafe` - Unchecked operations

### External Functions

Declare functions implemented externally:

**Compact:**
```z1c
x f now_ms()->U64 eff [time];
```

**Relaxed:**
```z1r
extern fn now_ms() -> U64
  eff [time];
```

## Capability-Based Security

Capabilities control what a module can do.

### Module Capabilities

Declare required capabilities in module header:

```z1r
module secure.app : 1.0
  caps = [net, fs.ro, time]
```

**Available Capabilities:**
- `net` - Network operations
- `fs.ro` - Read-only file system
- `fs.rw` - Read-write file system
- `time` - Access to time/clock
- `crypto` - Cryptographic functions
- `env` - Environment variables
- `unsafe` - Unsafe operations

### Capability Enforcement

Function effects must be subsets of module capabilities:

```z1r
module app : 1.0
  caps = [net]  // Only network allowed

// ✓ OK - net effect allowed
fn download(url: Str) -> Str
  eff [net]
{ ... }

// ✗ ERROR - fs not in module caps
fn readFile(path: Str) -> Str
  eff [fs]
{ ... }
```

## Statements and Expressions

### Variable Binding

**Compact:**
```z1c
let x: U32 = 42;
let msg: Str = "hello";
let flag: Bool = true;
```

**Relaxed:**
```z1r
let x: U32 = 42;
let msg: Str = "hello";
let flag: Bool = true;
```

Type annotations are optional if the compiler can infer:

```z1r
let x = 42;        // Inferred as U32
let msg = "hello"; // Inferred as Str
```

### Mutable Variables

Use `mut` for mutable bindings:

```z1r
let mut counter: U32 = 0;
counter = counter + 1;
counter = 10;
```

### Return Statement

**Compact:**
```z1c
f max(a:U32, b:U32)->U32 eff [pure] {
  ret if a > b { a } else { b };
}
```

**Relaxed:**
```z1r
fn max(a: U32, b: U32) -> U32
  eff [pure]
{
  return if a > b { a } else { b };
}
```

### Control Flow

#### If Expressions

```z1r
fn absolute(x: I32) -> I32
  eff [pure]
{
  let result = if x < 0 {
    -x
  } else {
    x
  };
  return result;
}

// If-else chains
fn classify(score: U32) -> Str
  eff [pure]
{
  return if score >= 90 {
    "A"
  } else if score >= 80 {
    "B"
  } else if score >= 70 {
    "C"
  } else {
    "F"
  };
}
```

#### While Loops

```z1r
fn countDown(n: U32) -> Unit
  eff [pure]
{
  let mut i = n;
  while i > 0 {
    i = i - 1;
  }
  return ();
}
```

#### Match Expressions

Pattern matching on union types:

```z1r
type Result = Ok{ value: U32 } | Err{ msg: Str }

fn unwrap(r: Result) -> U32
  eff [pure]
{
  return match r {
    Ok{ value } -> value,
    Err{ msg } -> 0
  };
}
```

### Operators

**Arithmetic:**
```z1r
x + y    // Addition
x - y    // Subtraction
x * y    // Multiplication
x / y    // Division
x % y    // Modulo
```

**Comparison:**
```z1r
x == y   // Equality
x != y   // Inequality
x < y    // Less than
x <= y   // Less than or equal
x > y    // Greater than
x >= y   // Greater than or equal
```

**Logical:**
```z1r
x && y   // Logical AND
x || y   // Logical OR
!x       // Logical NOT
```

**Precedence** (high to low):
1. Unary: `!`, `-`, `await`
2. Multiplicative: `*`, `/`, `%`
3. Additive: `+`, `-`
4. Comparison: `<`, `<=`, `>`, `>=`
5. Equality: `==`, `!=`
6. Logical AND: `&&`
7. Logical OR: `||`

### Function Calls

```z1r
// Simple call
let sum = add(5, 3);

// Qualified call (through import alias)
let server = Http.createServer(8080);

// Nested calls
let result = process(transform(data));
```

### Record Creation

```z1r
type User = { name: Str, age: U32 }

fn makeUser() -> User
  eff [pure]
{
  return User{ name: "Alice", age: 30 };
}
```

### Field Access

```z1r
let user = User{ name: "Bob", age: 25 };
let userName = user.name;  // "Bob"
let userAge = user.age;    // 25
```

## Context Budgets

Context budgets enforce token limits to keep code manageable for LLMs.

### Module-Level Budget

```z1r
module large.module : 1.0
  ctx = 800  // Maximum 800 tokens for entire module
```

The compiler estimates token count and rejects modules exceeding their budget.

### Function-Level Budgets (Future)

```z1r
// Future feature
fn complexFunction() -> Unit
  ctx = 256  // This function limited to 256 tokens
  eff [pure]
{ ... }
```

### Budget Violations

If your module exceeds its budget, the compiler suggests splitting:

```
Error: Module exceeds context budget
  Budget: 512 tokens
  Actual: 687 tokens
  Suggestion: Split at function 'processData' (line 45)
```

## Symbol Maps

Symbol maps provide stable short names for compact syntax.

### Declaring Symbols

**In Module Header:**
```z1c
m app:1.0 ctx=128 caps=[]
#sym { handler: h, process: p, User: U }
```

**Standalone:**
```z1r
#sym {
  handleRequest: hReq,
  processData: pData,
  UserData: UD
}
```

### How Symbol Maps Work

1. **Parsing compact** → expand short names to long names
2. **AST uses long names** → canonical representation
3. **Formatting compact** → contract long names to short names
4. **Hashing** → SymbolMap excluded from SemHash, included in FormHash

### Best Practices

- Map long, descriptive names to 1-3 character short forms
- Be consistent across related modules
- Don't worry about conflicts - maps are per-module

## Asynchronous Operations

Use `async` effect and `await` for asynchronous code:

```z1r
use "std/http/client" as Http

fn fetchData(url: Str) -> Str
  eff [net, async]
{
  let response = await Http.get(url);
  return response;
}
```

### Task Spawning

Create concurrent tasks:

```z1r
fn parallel() -> Unit
  eff [async]
{
  let t1 = task {
    // Runs concurrently
    doWork1()
  };

  let t2 = task {
    // Also runs concurrently
    doWork2()
  };

  await t1;
  await t2;
  return ();
}
```

## Comments

```z1r
// Line comment

// Multi-line comments via multiple line comments
// Second line
// Third line
```

### Shadow Metadata

Special comments for provenance (ignored by parser):

```z1r
//@z1: model="claude-3", agent="z1-agent/1.0", ctx_in=2340
//:prompt: "Refactor handler into pure function"
//:inputs: ["cells/http.server.z1r@sha3-256:..."]
```

These are preserved by the formatter and used for audit trails.

## Complete Example

Here's a complete Zero1 program demonstrating multiple features:

**Compact (`example.z1c`):**
```z1c
m todo.app:1.0 ctx=512 caps=[fs.rw]

u "std/fs/core" as F only [readFile, writeFile]

#sym { TodoItem: T, createTodo: cT, saveTodos: sT, main: mn }

t T = { id: U32, text: Str, done: Bool }

f cT(id:U32, text:Str)->T eff [pure] {
  ret T{ id: id, text: text, done: false };
}

f sT(todos:Vec<T>, path:Str)->Unit eff [fs] {
  F.writeFile(path, serialize(todos));
  ret ();
}

f mn()->Unit eff [fs] {
  let todo1: T = cT(1, "Learn Zero1");
  let todo2: T = cT(2, "Build app");
  sT([todo1, todo2], "todos.json");
  ret ();
}
```

**Relaxed (`example.z1r`):**
```z1r
module todo.app : 1.0
  ctx = 512
  caps = [fs.rw]

use "std/fs/core" as F only [readFile, writeFile]

#sym { TodoItem: T, createTodo: cT, saveTodos: sT, main: mn }

type TodoItem = {
  id: U32,
  text: Str,
  done: Bool
}

fn createTodo(id: U32, text: Str) -> TodoItem
  eff [pure]
{
  return TodoItem{
    id: id,
    text: text,
    done: false
  };
}

fn saveTodos(todos: Vec<TodoItem>, path: Str) -> Unit
  eff [fs]
{
  F.writeFile(path, serialize(todos));
  return ();
}

fn main() -> Unit
  eff [fs]
{
  let todo1: TodoItem = createTodo(1, "Learn Zero1");
  let todo2: TodoItem = createTodo(2, "Build app");
  saveTodos([todo1, todo2], "todos.json");
  return ();
}
```

## Try It Yourself

**Exercise 1**: HTTP Server
```z1r
// Create an HTTP server that responds to requests
// Requirements:
// - Use std/http/server
// - Module needs [net] capability
// - Handler should be pure
// - Server function needs [net, async] effects
```

**Exercise 2**: File Processor
```z1r
// Read a file, transform its contents, write back
// Requirements:
// - Use std/fs/core
// - Module needs [fs.rw] capability
// - Separate pure transformation function
// - I/O functions need [fs] effect
```

**Exercise 3**: Type Safety
```z1r
// Define Result<T, E> type
// Implement unwrapOr(result, default) function
// Use pattern matching
```

## Next Steps

- **[Standard Library Reference](03-stdlib-reference.md)** - Explore available modules
- **[Compilation Guide](04-compilation.md)** - Understand the compilation pipeline
- **[Best Practices](05-best-practices.md)** - Write idiomatic Zero1 code
- **[Grammar Reference](../grammar.md)** - Complete syntax specification
