# Zero1 Best Practices

This guide provides best practices for writing effective Zero1 code that's efficient, maintainable, and optimized for LLM agent workflows.

## Designing with Capabilities

Capabilities are Zero1's security foundation. Use them wisely.

### Principle of Least Privilege

Grant only the capabilities you need:

```z1r
// ✗ AVOID - Too permissive
module app : 1.0
  caps = [net, fs.rw, time, crypto, env, unsafe]

// ✓ PREFER - Minimal necessary capabilities
module app : 1.0
  caps = [fs.ro]  // Only need to read files
```

### Separate Concerns by Capability

Split modules by capability requirements:

```z1r
// config.loader.z1r - Only reads config
module config.loader : 1.0
  caps = [fs.ro]

// config.writer.z1r - Only writes config
module config.writer : 1.0
  caps = [fs.rw]

// config.processor.z1r - Pure logic, no I/O
module config.processor : 1.0
  caps = []
```

### Read-Only vs Read-Write

Use `fs.ro` when possible:

```z1r
// ✓ GOOD - Read-only for inspection
module log.viewer : 1.0
  caps = [fs.ro]

fn viewLogs(path: Str) -> Str
  eff [fs]
{
  return Fs.readText(path);
}

// ✓ GOOD - Read-write only when needed
module log.rotator : 1.0
  caps = [fs.rw]

fn rotateLogs(oldPath: Str, newPath: Str) -> Unit
  eff [fs]
{
  Fs.writeText(newPath, Fs.readText(oldPath));
  Fs.remove(oldPath);
}
```

### Capability Composition

Combine capabilities when genuinely needed:

```z1r
// ✓ GOOD - Web scraper needs net + fs.rw
module scraper : 1.0
  caps = [net, fs.rw]

fn scrapeAndSave(url: Str, path: Str) -> Unit
  eff [net, fs]
{
  let content = Http.get(url);
  Fs.writeText(path, content);
}
```

## Managing Context Budgets

Context budgets keep code LLM-friendly. Follow these practices.

### Set Realistic Budgets

Base budgets on actual token usage:

```z1r
// ✗ TOO TIGHT - Will fail for non-trivial code
module app : 1.0
  ctx = 64

// ✗ TOO LOOSE - Defeats the purpose
module app : 1.0
  ctx = 10000

// ✓ APPROPRIATE - Allows useful code, enforces limits
module app : 1.0
  ctx = 512
```

### Estimate Before Declaring

Use the CLI to estimate token count:

```bash
# Write your module first
cargo run -p z1-cli -- ctx mymodule.z1r

# Output: Estimated: 387 tokens
# Then set budget: ctx = 512
```

### Split Large Modules

When approaching budget limits, split into smaller cells:

```z1r
// Before (single large module)
module data.processor : 1.0
  ctx = 800

// types (50 tokens)
type Record = { ... }
type Result = { ... }

// validation (200 tokens)
fn validate() { ... }
fn sanitize() { ... }

// transformation (350 tokens)
fn transform() { ... }
fn aggregate() { ... }

// After (split into focused modules)

// data.types.z1r (80 tokens)
module data.types : 1.0
  ctx = 128

type Record = { ... }
type Result = { ... }

// data.validator.z1r (256 tokens)
module data.validator : 1.0
  ctx = 320

use "data/types" as T

fn validate(r: T.Record) { ... }
fn sanitize(r: T.Record) { ... }

// data.transformer.z1r (400 tokens)
module data.transformer : 1.0
  ctx = 512

use "data/types" as T

fn transform(r: T.Record) { ... }
fn aggregate(rs: Vec<T.Record>) { ... }
```

### Optimize Token Usage

Reduce tokens without sacrificing clarity:

**Use compact syntax:**
```z1c
m app:1.0 ctx=256 caps=[net]
u "std/http/client" as H only [get]
f fetch(url:Str)->Str eff [net,async] { ret H.get(url); }
```

**Apply SymbolMaps:**
```z1c
#sym { fetchData: fD, processData: pD, User: U }

// Now "fetchData" compiles as "fD" in compact mode
```

**Import selectively:**
```z1r
// ✗ Imports everything
use "std/fs/core" as Fs

// ✓ Imports only what's needed
use "std/fs/core" as Fs only [readText, writeText]
```

## When to Use Compact vs Relaxed Syntax

Choose the right syntax for your context.

### Use Compact (`.z1c`) When:

1. **Feeding code to LLMs**
   - Minimize token count in prompts
   - Maximize context window efficiency

2. **Agent-to-agent communication**
   - Smaller payloads over network
   - Faster parsing by agents

3. **Storage optimization**
   - Archiving code efficiently
   - Version control with smaller diffs

4. **Context-constrained environments**
   - Mobile/embedded scenarios
   - Limited bandwidth situations

### Use Relaxed (`.z1r`) When:

1. **Human development**
   - Writing new code
   - Code review sessions
   - Pair programming

2. **Documentation**
   - Tutorial examples
   - API documentation
   - Learning materials

3. **Debugging**
   - Easier to read stack traces
   - Clearer error messages

4. **Team collaboration**
   - Pull request reviews
   - Design discussions

### Workflow Recommendation

```bash
# 1. Develop in relaxed mode
vim app.z1r

# 2. Test and iterate
cargo run -p z1-cli -- check app.z1r

# 3. Generate compact for production/LLMs
cargo run -p z1-cli -- fmt app.z1r --mode compact > app.z1c

# 4. Both versions committed to repo
git add app.z1r app.z1c
git commit -m "feat: add user authentication"
```

## Organizing Larger Projects

Structure projects for maintainability and clear boundaries.

### Directory Structure

```
myproject/
├── manifest.z1m              # Project metadata
├── cells/
│   ├── types/                # Shared type definitions
│   │   ├── user.z1c
│   │   └── config.z1c
│   ├── core/                 # Core logic (pure)
│   │   ├── validator.z1c
│   │   └── transformer.z1c
│   ├── io/                   # I/O operations
│   │   ├── file.z1c
│   │   └── http.z1c
│   └── app/                  # Application entry
│       └── main.z1c
├── tests/
│   └── integration.spec.z1r
└── prov/
    └── PROVCHAIN.z1p
```

### Module Naming Conventions

Use dotted names for hierarchical organization:

```z1r
// Type definitions
module myapp.types.user : 1.0
module myapp.types.config : 1.0

// Business logic
module myapp.core.validator : 1.0
module myapp.core.processor : 1.0

// I/O boundaries
module myapp.io.file : 1.0
module myapp.io.http : 1.0
```

### Dependency Management

Keep dependencies acyclic and shallow:

```
✓ GOOD - Clear hierarchy:
  myapp.app.main
    ↓
  myapp.core.processor
    ↓
  myapp.types.user

✗ BAD - Circular dependency:
  myapp.module.a → myapp.module.b
         ↑                ↓
         └────────────────┘
```

## Testing Strategies

Write testable code with clear specifications.

### Separate Pure and Effectful Code

```z1r
// ✓ GOOD - Pure transformation, easy to test
fn processData(data: Str) -> Str
  eff [pure]
{
  return transform(sanitize(data));
}

// ✓ GOOD - Thin I/O wrapper
fn loadAndProcess(path: Str) -> Str
  eff [fs]
{
  let data = Fs.readText(path);
  return processData(data);  // Calls pure function
}
```

### Write Spec Tests

Use Zero1's test DSL:

```z1r
// tests/processor.spec.z1r
spec "processData handles empty input" {
  let result = processData("");
  assert result == "";
}

spec "processData sanitizes HTML" {
  let result = processData("<script>alert(1)</script>");
  assert result == "";
}

spec "processData preserves valid content" {
  let result = processData("Hello, world!");
  assert result == "Hello, world!";
}
```

### Property-Based Testing

Test invariants:

```z1r
prop "round-trip formatting preserves SemHash" {
  forall cell: Cell {
    let compact = fmt_compact(cell);
    let relaxed = fmt_relaxed(cell);
    assert sem_hash(compact) == sem_hash(relaxed);
  }
}
```

## Performance Considerations

Zero1 prioritizes context efficiency over runtime performance, but you can still write efficient code.

### Minimize Allocations

```z1r
// ✗ AVOID - Repeated string concatenation
fn buildMessage(items: Vec<Str>) -> Str
  eff [pure]
{
  let mut result = "";
  let mut i = 0;
  while i < len(items) {
    result = result + items[i];  // Reallocates each time
    i = i + 1;
  }
  return result;
}

// ✓ PREFER - Single allocation (when available)
fn buildMessage(items: Vec<Str>) -> Str
  eff [pure]
{
  return join(items, "");
}
```

### Use Appropriate Types

```z1r
// ✗ AVOID - String for numeric data
fn sum(nums: Vec<Str>) -> Str { ... }

// ✓ PREFER - Proper numeric types
fn sum(nums: Vec<U32>) -> U32 { ... }
```

### Lazy Evaluation (Future)

```z1r
// Future feature: lazy sequences
fn processLarge(items: LazySeq<Item>) -> LazySeq<Result>
  eff [pure]
{
  return items.map(transform).filter(validate);
}
```

## Security Guidelines

Write secure code with Zero1's capability system.

### Validate External Input

```z1r
fn handleRequest(input: Str) -> Result
  eff [pure]
{
  // ✓ Validate before processing
  if !isValid(input) {
    return Err{ error: "Invalid input" };
  }
  return Ok{ value: process(input) };
}
```

### Sanitize User Data

```z1r
use "std/crypto/hash" as Hash

fn hashPassword(password: Str, salt: Str) -> Str
  eff [crypto]
{
  // ✓ Never log or store plaintext passwords
  return Hash.sha256(salt + password);
}
```

### Limit Resource Usage

```z1r
// ✓ Set reasonable limits
fn processFile(path: Str) -> Result
  eff [fs]
{
  let content = Fs.readText(path);
  if len(content) > 1000000 {  // 1MB limit
    return Err{ error: "File too large" };
  }
  return Ok{ value: process(content) };
}
```

### Avoid Unsafe Unless Necessary

```z1r
// ✗ AVOID
module app : 1.0
  caps = [unsafe]

// ✓ PREFER - Use safe capabilities
module app : 1.0
  caps = [fs.ro, net]
```

## Code Formatting Conventions

Maintain consistent formatting for readability.

### Relaxed Mode Style

**Indentation:** 2 spaces
```z1r
fn example() -> Unit
  eff [pure]
{
  if condition {
    doSomething();
  } else {
    doOtherThing();
  }
}
```

**Line Length:** Max 100 characters
```z1r
// ✓ GOOD
fn short(x: U32) -> U32 { return x + 1; }

// ✓ GOOD - Multi-line when needed
fn longFunctionName(
  firstParameter: LongTypeName,
  secondParameter: AnotherLongTypeName
) -> ReturnType
  eff [pure]
{
  return process(firstParameter, secondParameter);
}
```

**Vertical Spacing:**
```z1r
type User = { name: Str, age: U32 }

fn createUser(name: Str, age: U32) -> User
  eff [pure]
{
  return User{ name: name, age: age };
}

fn validateUser(user: User) -> Bool
  eff [pure]
{
  return user.age >= 18;
}
```

### Compact Mode

Let the formatter handle compact mode:
```bash
cargo run -p z1-cli -- fmt code.z1r --mode compact
```

## Error Handling Patterns

Handle errors explicitly and gracefully.

### Use Result Types

```z1r
type Result<T> = Ok{ value: T } | Err{ error: Str }

fn safeOperation(input: Str) -> Result<Output>
  eff [fs]
{
  let fileResult = Fs.readText(input);
  return match fileResult {
    Ok{ value: content } -> {
      Ok{ value: process(content) }
    },
    Err{ error: msg } -> {
      Err{ error: "Failed to read: " + msg }
    }
  };
}
```

### Early Returns for Errors

```z1r
fn validate(data: Data) -> Result<Data>
  eff [pure]
{
  if !hasRequiredFields(data) {
    return Err{ error: "Missing required fields" };
  }

  if !isValidFormat(data) {
    return Err{ error: "Invalid format" };
  }

  return Ok{ value: data };
}
```

### Provide Context in Errors

```z1r
// ✗ AVOID - Vague error
return Err{ error: "Failed" };

// ✓ PREFER - Descriptive error
return Err{
  error: "Failed to parse config: missing 'port' field at line 12"
};
```

## Provenance and Documentation

Maintain clear audit trails.

### Shadow Metadata

Use shadow comments for agent-generated code:

```z1r
//@z1: model="claude-3.5", agent="z1-agent/1.0.0", ctx_in=2340
//:prompt: "Implement user authentication with JWT tokens"
//:inputs: ["cells/user.types.z1r@sha3-256:abc123..."]

module auth.jwt : 1.0
  ctx = 512
  caps = [crypto, time]

// Implementation follows...
```

### Regular Comments

Document complex logic:

```z1r
// Calculate checksum using custom algorithm
// See: RFC-1234 for specification
fn calculateChecksum(data: Str) -> U32
  eff [pure]
{
  // Implementation...
}
```

### Type Documentation

Document custom types:

```z1r
// User record with authentication metadata
// - id: Unique user identifier
// - email: User's email address (must be validated)
// - hashedPassword: Bcrypt-hashed password (never plaintext)
type User = {
  id: U32,
  email: Str,
  hashedPassword: Str
}
```

## Migration and Refactoring

Safely evolve your codebase.

### Preserve SemHash When Refactoring

```bash
# Before refactoring
cargo run -p z1-cli -- hash module.z1r
# SemHash: sha3-256:abc123...

# After refactoring
cargo run -p z1-cli -- hash module.z1r
# SemHash: sha3-256:abc123...  (same!)
```

### Version Bumping

Update versions when making changes:

```z1r
// Before
module user.auth : 1.0

// After breaking change
module user.auth : 2.0

// After compatible change
module user.auth : 1.1
```

### Deprecation Pattern

```z1r
// Old function (deprecated)
// DEPRECATED: Use authenticateV2 instead
fn authenticate(user: User) -> Bool
  eff [crypto]
{
  return authenticateV2(user);
}

// New function
fn authenticateV2(user: User) -> Result<Session>
  eff [crypto, time]
{
  // New implementation
}
```

## Summary Checklist

- [ ] Use minimal capabilities (least privilege)
- [ ] Set realistic context budgets (estimate first)
- [ ] Split large modules (keep under 200 AST nodes)
- [ ] Separate pure and effectful code
- [ ] Write spec tests for business logic
- [ ] Validate all external input
- [ ] Use Result types for error handling
- [ ] Document complex logic with comments
- [ ] Use compact syntax for LLM contexts
- [ ] Use relaxed syntax for human development
- [ ] Maintain provenance with shadow metadata
- [ ] Version modules appropriately
- [ ] Run formatter before committing

## Next Steps

- **[Migration Guide](06-migration.md)** - Coming from other languages
- **[Language Tour](02-language-tour.md)** - Review language features
- **[Compilation Guide](04-compilation.md)** - Understand the pipeline
- **[Examples](../../examples/)** - See real-world code
