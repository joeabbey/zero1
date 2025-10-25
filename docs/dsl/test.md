# Part B — Z1 Test DSL (`.z1t`)

## B.1 Overview

* **Purpose:** Behavior tests, property tests, and **prompt‑tests** that validate agentic changes.
* **File extension:** `.z1t`.
* **Lexer:** Reuses Z1 lexical rules (identifiers, strings, comments).
* **Imports:** Same `use`/`as`/`only` form as Z1 cells.
* **Expressions & statements inside test blocks:** Reuse Z1 expression and statement grammar (Section 2.6+ in the language spec). Tests execute against compiled code of imported cells.

## B.2 Reserved test keywords

```
suite, spec, prop, prompt-test, fixture, mock,
before, after, before_each, after_each,
config, with, tags, timeout, retries, parallel, seed, runs,
using, model, request, inputs, expect, diff,
only, skip, snapshot
```

## B.3 High‑level grammar (EBNF)

> The grammar below references **Z1 types/exprs** by name (`TypeExpr`, `Expr`, `Block`, `Pattern`) from the language grammar you already have.

```
TestFile        ::= TestHeaderOpt { TestTopItem }
TestHeaderOpt   ::= [ "module" Path VersionOpt ]           # optional namespace for tests

TestTopItem     ::= Import
                  | FixtureDecl
                  | MockDecl
                  | LifecycleDecl
                  | SuiteDecl
                  | SpecDecl
                  | PropDecl
                  | PromptTestDecl
                  | ConfigDecl

Import          ::= KW_USE String AliasOpt OnlyOpt ";"

ConfigDecl      ::= "config" "{" TestConfigBody "}"
TestConfigBody  ::= { TestConfigKV ( "," | ";" ) }
TestConfigKV    ::= "timeout_ms" ":" Int
                  | "parallel"   ":" Int
                  | "seed"       ":" Int
                  | "tags.include" ":" "[" StringListOpt "]"
                  | "tags.exclude" ":" "[" StringListOpt "]"
                  | "snapshots.dir" ":" String
StringListOpt   ::= [ String { "," String } ]

SuiteDecl       ::= "suite" String AttrsOpt "{" { SuiteItem } "}"
AttrsOpt        ::= [ "with" "{" AttrKV { "," AttrKV } "}" ]
AttrKV          ::= "timeout" ":" Duration
                  | "retries" ":" Int
                  | "tags"    ":" "[" StringListOpt "]"
                  | "only"    ":" Bool
                  | "skip"    ":" Bool
                  | "parallel": Bool

SuiteItem       ::= LifecycleDecl | SpecDecl | PropDecl | PromptTestDecl | FixtureDecl | MockDecl

LifecycleDecl   ::= "before" Block
                  | "after" Block
                  | "before_each" Block
                  | "after_each" Block

FixtureDecl     ::= "fixture" Ident FixtureSigOpt "=" ( Expr | Block ) ";"
FixtureSigOpt   ::= [ ":" TypeExpr ]                         # optional type

MockDecl        ::= "mock" CapName "{" { MockRule } "}"
MockRule        ::= "when" Path "(" ArgPatternListOpt ")" "->" MockAction ";"
ArgPatternListOpt ::= [ Pattern { "," Pattern } ]
MockAction      ::= "returns" Expr
                  | "throws"  Expr
                  | "calls"   Path "(" ArgListOpt ")"

SpecDecl        ::= "spec" String AttrsOpt Block

PropDecl        ::= "prop" String AttrsOpt
                    "for_all" "(" GenBinding { "," GenBinding } ")" RunsSeedOpt Block
GenBinding      ::= Ident ":" TypeExpr GenWhereOpt GenGenOpt
GenWhereOpt     ::= [ "where" Expr ]                        # predicate over the variable
GenGenOpt       ::= [ "by" Path ]                           # custom generator function
RunsSeedOpt     ::= [ "runs" Int ] [ "seed" Int ]

PromptTestDecl  ::= "prompt-test" String AttrsOpt
                    "using" "model" "=" String ModelOptsOpt
                    "plan" "{" PromptPlanBody "}"
                    "expect" "{" ExpectBody "}"
ModelOptsOpt    ::= [ "," "sdict" "=" String ]              # optional path to SDict
PromptPlanBody  ::= PromptKV { "," PromptKV }
PromptKV        ::= "request" ":" ( String | "<<<" HereDoc ">>>")
                  | "inputs"  ":" "[" InputListOpt "]"
                  | "apply"   ":" Bool                      # default true
InputListOpt    ::= [ String { "," String } ]
HereDoc         ::= { any char except ">>>" }               # raw multi-line

ExpectBody      ::= { ExpectKV ( "," | ";" ) }
ExpectKV        ::= "compiles" ":" Bool
                  | "tests_pass" ":" Bool
                  | "semhash" ":" "unchanged" ":" Bool
                  | "formhash" ":" "changed" ":" Bool
                  | "diff" ":" "{" DiffKV { "," DiffKV } "}"
                  | "effects" ":" "{" EffKV { "," EffKV } "}"
                  | "caps" ":" "{" CapKV { "," CapKV } "}"
                  | "ctx" ":" "{" CtxKV { "," CtxKV } "}"
                  | "cells_changed" ":" "[" StringListOpt "]"
DiffKV          ::= "only" ":" "[" StringListOpt "]"
                  | "forbid" ":" "[" StringListOpt "]"
EffKV           ::= "not_added" ":" "[" StringListOpt "]"
                  | "not_removed" ":" "[" StringListOpt "]"
CapKV           ::= "not_added" ":" "[" StringListOpt "]"
                  | "not_removed" ":" "[" StringListOpt "]"
CtxKV           ::= "delta_max" ":" Int
                  | "fn_max"    ":" Int

# Reuse from Z1 core:
#   TypeExpr, Expr, Block, Pattern, ArgListOpt, Path, CapName, Int, String, Bool, Duration
```

### Duration literal

```
Duration         ::= Int ( "ms" | "s" | "min" )
```

### Attributes (inheritance)

* Attributes (`with { ... }`) may appear on `suite`, `spec`, `prop`, and `prompt-test`.
* Precedence: file‑level `config` → suite attributes → test attributes.
* Inheritable keys: `timeout`, `retries`, `tags`, `parallel`. `only` and `skip` do **not** inherit.

## B.4 Built‑in test APIs (available inside blocks)

**Assertions (statements):**

* `assert <Expr>;` — truthy
* `assert_eq(<Expr>, <Expr>);`
* `assert_ne(<Expr>, <Expr>);`
* `assert_approx(<Expr>, <Expr>, <Expr epsilon>);`
* `assert_matches(<Expr>, <Pattern>);`
* `assert_throws(<Expr>);`
* `fail(<String>);` — immediate failure

**Snapshots:**

* `snapshot(name: Str, value: any);` — writes or compares JSON snapshot at `snapshots.dir/name.snap.json` (controlled by `Z1_UPDATE_SNAPSHOTS=1` or `--update-snapshots`).

**Time & concurrency helpers:**

* `sleep_ms(n: U32);`
* `with_timeout(ms: U32, fn () -> Unit)`

**Spies (lightweight):**

* `spy(fnref)` returns handle with `calls()` and `last_args()`.

> These are **host‑provided** by the test runtime; they don’t require declaring Z1 effects.

## B.5 Fixtures & lifecycle

* `fixture x = Expr;` evaluates once per **file**.
* `fixture x = { ... }` block can contain Z1 statements; last expression is the value.
* You can scope a fixture to a suite by placing it inside the suite block.
* `before/after` run once per **suite** (file‑level defines the outer suite).
* `before_each/after_each` run around each test in the containing suite.

## B.6 Mocks

`mock <capability> { when path(patterns...) -> returns expr; }`

* Only mocks capabilities granted by `[capabilities].allow` in the manifest.
* Patterns use Z1 `Pattern`.
* Actions: `returns`, `throws`, or `calls` (delegate).

Example:

```z1t
mock net {
  when H.listen(port, _) -> returns 0U16;
}
```

## B.7 Property tests

* `for_all (x: T where predicate by gen, y: U, ...)`
* Default generator exists for primitives and record types; custom `by <Path>` must return a generator object understood by the test runtime.
* `runs N` defaults to 100; `seed` inherits from file config or manifest.
* Shrinking: the runtime attempts simple structural shrinking on failure (numbers toward 0, shorter strings, first variants).

## B.8 Prompt‑tests

* Purpose: exercise agentic “plan/apply” over code with an NL request and verify **diff constraints** and **invariants**.
* `using model="<id>", sdict="<path>"?` — SDict optional (for estimation & formatting).
* `plan { request: "...", inputs: ["cells/http.server.z1c"], apply: true }`
* `expect { compiles: true, semhash: unchanged: true, diff: { only: ["rename","export-update"] } }`

**Semantics**

1. Collect input cells; prepare a **context slice** respecting budgets.
2. Provide `request` and SDict to the configured planner.
3. If `apply: true` (default), apply the patch to a temp workspace; otherwise validate the plan only.
4. Build & run tests if `tests_pass` is asserted.
5. Compute **semhash/formhash** deltas and check `effects`, `caps`, `ctx`, and `diff` constraints.

**Recognized diff labels** (non‑exhaustive; tool‑controlled):

* `"rename"`, `"move-cell"`, `"split"`, `"merge"`, `"export-update"`, `"type-change"`, `"behavior-change"`, `"format-only"`

> If both `semhash.unchanged: true` and any semantic diff label are observed, the test **fails**.

## B.9 Runner semantics

* Discovery: all files matching manifest `[test].include` minus `[test].exclude`.
* Selection:

  * `--tags a,b` further filters by union of tags.
  * Any `only: true` marks all others skipped (unless `--no-only-error`).
* Execution:

  * Concurrency controlled by `parallel` (tests within a file may still run sequentially if they share fixtures).
  * Timeouts: per test from attributes → suite → file config → manifest.
  * Retries: only on failure; `retries` decremented per attempt.
* Reporting: TAP‑like and JSON output (`--report json > report.json`).

---

## B.10 Complete examples

### 1) Unit + snapshot + mock

```z1t
use "std/http" as H only [listen, Req, Res];

config { timeout_ms: 3000, snapshots.dir: "tests/__snapshots__" }

fixture base_req: H.Req = H.Req{ method: "GET", path: "/health" };

mock net {
  when H.listen(port, _) -> returns 0U16;
}

spec "handler returns ok" with { tags: ["unit","http"] } {
  let r = handler(base_req);
  assert_eq(r.status, 200U16);
  snapshot("health-body", r.body);
}
```

### 2) Property test

```z1t
use "acme.http" as S only [route];

prop "routing is idempotent" with { retries: 1 }
for_all (p: Str where p != "", q: Str where q != p) runs 200 seed 42 {
  let r1 = route(p);
  let r2 = route(p);
  assert_eq(r1, r2);
  assert_ne(route(p), route(q));
}
```

### 3) Prompt‑test with constraints

```z1t
prompt-test "rename handler to h"
using model="llm-x-2025-08", sdict="dicts/llm-x-2025-08.sdict"
plan {
  request: "Rename function `handler` to `h`; update exports; no behavior change.",
  inputs: ["cells/http.server.z1c"],
  apply: true
}
expect {
  compiles: true,
  tests_pass: true,
  semhash: unchanged: true,
  formhash: changed: true,
  diff: { only: ["rename","export-update"] },
  effects: { not_added: ["net","fs","time"] },
  caps:    { not_added: ["net","fs","time"] },
  ctx:     { delta_max: 32, fn_max: 256 },
  cells_changed: ["cells/http.server.z1c"]
}
```

### 4) Suite with lifecycle & fixtures

```z1t
suite "http suite" with { timeout: 5s, tags: ["integration"] } {
  before { /* start test server */ }
  after  { /* stop test server */ }

  fixture port: U16 = 8080U16;

  spec "server starts" {
    let rc = serve(port);
    assert_eq(rc, 0U16);
  }

  spec "health endpoint" {
    let r = handler(H.Req{ method:"GET", path:"/health" });
    assert_eq(r.status, 200U16);
    assert_matches(r, H.Res{ status: 200U16, body: _ });
  }
}
```

---

## B.11 Validation & errors

* `only: true` in multiple tests → allowed; runner executes only those with `only`. If none present, run all non‑`skip`.
* Unknown attributes inside `with { ... }` → warning.
* Fixtures must not be cyclic (detect via dependency graph of fixture names referenced inside fixture expressions).
* Mock rules must match **some** call by the end of the test **if** they include a `_verify` clause (advanced option); otherwise no assertion is implied.

---

## B.12 Mapping to an AST (for implementers)

Minimal nodes to add on top of the Z1 AST:

```text
TestFile
  - header: Option<ModuleHeader>         # for namespacing
  - config: TestConfig
  - items: Vec<TestItem>

TestItem = Import | Fixture | Mock | Suite | Spec | Prop | PromptTest | Lifecycle

Attrs { timeout: Option<Duration>, retries: u32, tags: Vec<String>, only: bool, skip: bool, parallel: Option<bool> }

Spec { name: String, attrs: Attrs, body: Block, span }
Prop { name, attrs, bindings: Vec<GenBinding>, runs: u32, seed: u64, body: Block }
PromptTest { name, attrs, model: String, sdict: Option<Path>, plan: PromptPlan, expect: Expect }
Fixture { name: Ident, ty: Option<TypeExpr>, init: Either<Expr, Block>, scope: File|Suite }
Mock { cap: CapName, rules: Vec<MockRule> }
Lifecycle { kind: Before|After|BeforeEach|AfterEach, body: Block }
```

This AST reuses `Expr`, `Block`, `Pattern`, `Path`, `TypeExpr` from Z1 core.

---

### That’s the complete spec

* **Manifest DSL**: fully specified keys, validation, precedence, and canonical JSON schema.
* **Test DSL**: full grammar, semantics, and examples for unit/spec, property, mocks, snapshots, and prompt‑tests.

If you want, I can generate **ANTLR4 grammars** (for `.z1t`) and a **Rust parser skeleton** (`winnow`/`chumsky`) that implements these rules, plus test fixtures for the examples above.

