Awesome — let’s turn **Zero1 (Z1)** from spec to a working MVP. Below is a concrete **build plan + reference skeleton** you can drop into a repo and iterate on. I’ve optimized this for quick bring‑up: smallest viable grammar, canonical AST + hashing, compact↔relaxed formatter, effect/capability checks, provenance logging, and TS/WASM codegen stubs.

---

## 0) High‑level milestones

**M0: Bootstrapping core (week 0‑1)**

* Repo scaffold, crates/modules wired
* Lexer + minimal parser (modules, imports, types, fns, effects)
* Canonical AST + hash
* `z1fmt` compact↔relaxed formatter (lossless)

**M1: Semantics (week 1‑2)**

* Type checker (structural types, generics later)
* Effect/capability checker
* Context estimator (model‑agnostic; SDict‑aware later)

**M2: Provenance, policy, tests (week 2‑3)**

* Provenance store (append‑only chain, Merkle root)
* Ed25519 signature plumbing
* Policy gates (caps, ctx budgets)
* Spec/property tests; prompt‑test harness skeleton

**M3: Codegen & std (week 3‑4)**

* IR + TS codegen (WASM stub)
* Minimal `std` (http surface, time)
* CLI end‑to‑end (`z1c`, `z1fmt`, `z1prov`, `z1test`, `z1ctx`)

---

## 1) Repo scaffold

```
zero1/
 ├─ Cargo.toml                 # workspace
 ├─ crates/
 │   ├─ z1-ast/
 │   ├─ z1-lex/
 │   ├─ z1-parse/
 │   ├─ z1-fmt/
 │   ├─ z1-typeck/
 │   ├─ z1-effects/
 │   ├─ z1-hash/
 │   ├─ z1-prov/
 │   ├─ z1-codegen-ts/
 │   ├─ z1-codegen-wasm/
 │   └─ z1-cli/
 ├─ examples/http-example/
 │   ├─ manifest.z1m
 │   ├─ dicts/llm-x-2025-08.sdict
 │   ├─ cells/http.server.z1c
 │   ├─ tests/http.spec.z1r
 │   └─ prov/PROVCHAIN.z1p
 └─ README.md
```

**Workspace `Cargo.toml`**

```toml
[workspace]
members = [
  "crates/z1-ast","crates/z1-lex","crates/z1-parse","crates/z1-fmt",
  "crates/z1-typeck","crates/z1-effects","crates/z1-hash","crates/z1-prov",
  "crates/z1-codegen-ts","crates/z1-codegen-wasm","crates/z1-cli"
]
resolver = "2"
```

---

## 2) Canonical AST (semantics‑stable) + SymbolMap (formatting‑only)

> **Two hashes:**
> **SemHash** (semantics) excludes SymbolMap/comments; drives Merkle roots.
> **FormHash** (format) includes SymbolMap; lets you detect format‑only churn.
> Both are deterministic.

`crates/z1-ast/src/ast.rs` (excerpt)

```rust
use std::collections::BTreeMap;

pub type Ident = String;
pub type Version = String;

#[derive(Debug, Clone)]
pub struct Module {
    pub name: Ident,          // "http.server"
    pub version: Option<Version>,
    pub ctx_budget: Option<u32>,
    pub caps: Vec<Capability>,
    pub imports: Vec<Import>,
    pub decls: Vec<Decl>,
    pub sym_map: SymbolMap,   // NOT part of Semantics AST (excluded from SemHash)
    pub span: Span,
}

#[derive(Debug, Clone, Default)]
pub struct SymbolMap { // long ↔ short
    pub to_short: BTreeMap<Ident, Ident>,
    pub to_long:  BTreeMap<Ident, Ident>,
}

#[derive(Debug, Clone)]
pub struct Import {
    pub path: String,             // "std/http"
    pub alias: Option<Ident>,     // "H"
    pub only: Vec<Ident>,         // ["listen","Req","Res"]
    pub span: Span,
}

#[derive(Debug, Clone)]
pub enum Decl {
    Type(TypeDecl),
    Fn(FnDecl),
}

#[derive(Debug, Clone)]
pub struct TypeDecl {
    pub name: Ident,
    pub ty: TypeExpr,
    pub span: Span,
}

#[derive(Debug, Clone)]
pub enum TypeExpr {
    Bool, Str, Unit,
    U16, U32, U64,
    Path(Vec<Ident>),                   // http.Res
    Record(Vec<(Ident, TypeExpr)>),     // { ok: Bool, msg: Str }
    Sum(Vec<(Ident, Option<TypeExpr>)>),// Ok{T} | Err{E}
    Generic { base: Box<TypeExpr>, args: Vec<TypeExpr> },
}

#[derive(Debug, Clone)]
pub struct FnDecl {
    pub name: Ident,
    pub params: Vec<(Ident, TypeExpr)>,
    pub ret: TypeExpr,
    pub effects: Vec<Effect>,
    pub body: Block,
    pub span: Span,
}

#[derive(Debug, Clone)]
pub enum Stmt {
    Let { name: Ident, expr: Expr },
    Expr(Expr),
    Return(Expr),
}

#[derive(Debug, Clone)]
pub enum Expr {
    LitStr(String),
    LitInt(u64),
    Path(Vec<Ident>),                    // http.listen
    Call { func: Box<Expr>, args: Vec<Expr> },
    Record { path: Vec<Ident>, fields: Vec<(Ident, Expr)> }, // H.Res{...}
    While { cond: Box<Expr>, body: Block },
    BinOp { op: BinOp, left: Box<Expr>, right: Box<Expr> },
    // … (await, task, chan etc in M2)
}

#[derive(Debug, Clone)] pub enum BinOp { Add, Sub, Eq }
#[derive(Debug, Clone)] pub struct Block { pub stmts: Vec<Stmt> }

#[derive(Debug, Clone)] pub enum Capability { Net, FsRo, FsRw, Time, Crypto, Env, Unsafe }
#[derive(Debug, Clone, Eq, PartialEq, Ord, PartialOrd)]
pub enum Effect { Pure, Net, Fs, Time, Crypto, Env, Async, Unsafe }

#[derive(Debug, Clone, Copy, Default)]
pub struct Span { pub start: u32, pub end: u32 }
```

---

## 3) Canonical hashing & Merkle root

**Normalization rules (SemHash):**

* Remove comments, shadow metadata lines, and SymbolMap.
* Expand all identifiers to their **long** form (consult `sym_map.to_long`).
* Serialize AST nodes with **sorted field keys** (CBOR/JSON with deterministic maps).
* Encode numeric types in canonical width; strings as UTF‑8.
* Children ordered structurally (as declared).

**Node hash:** `H(node) = SHA256(tag || enc(node_fields) || concat(H(children)))`
**Cell hash:** hash of Module node.
**Pack Merkle root:** Merkle over sorted list of `{path, cell_semhash}`, plus dicts/prov manifests.

`crates/z1-hash/src/lib.rs` (excerpt)

```rust
use sha2::{Digest, Sha256};
use z1_ast::*;

pub fn sem_hash_module(m: &Module) -> [u8; 32] {
    let normalized = normalize_module_semantics(m);
    sha256_node(&normalized)
}

// Pseudocode-ish: turn the normalized module into a canonical byte buffer
fn sha256_node(node: &NormNode) -> [u8; 32] {
    let mut hasher = Sha256::new();
    hasher.update(node.tag());
    hasher.update(node.canonical_bytes()); // sorted keys, long idents, no SymbolMap
    for child in node.children() {
        hasher.update(sha256_node(child));
    }
    hasher.finalize().into()
}
```

> **FormHash** runs the same pipeline but **includes** `SymbolMap` and comment blocks.

---

## 4) Lexer & grammar (compact + relaxed)

**Tokens (shared)**
`IDENT, INT, STR, KW(m|module|u|use|t|type|f|fn|ret|return|eff|ctx|caps|as|only|task|await|while|if|else|match)`
Punctuation: `{ } ( ) [ ] , : . -> = | #sym //@z1 //:prompt …`

**Comments**

* Line comments: `// ...`
* Shadow meta: `//@z1:` and `//:key:` preserved to provenance (not parsed as code)

**EBNF (subset, both modes)**

```
Module     := ModHeader Import* Decl*
ModHeader  := ("m" | "module") Ident (":" Version)? ("ctx" "=" Int)? ("caps" "=" "[" CapList "]")? SymHdr?
SymHdr     := "#sym" "{" SymPair ("," SymPair)* "}"
SymPair    := Ident ":" Ident

Import     := ("u" | "use") String ("as" Ident)? ("only" "[" Ident ("," Ident)* "]")?

Decl       := TypeDecl | FnDecl

TypeDecl   := ("t" | "type") Ident "=" TypeExpr
TypeExpr   := "Bool" | "Str" | "Unit" | "U16" | "U32" | "U64"
            | Path
            | "{" FieldList? "}"
            | TypeExpr "|" TypeExpr
            | TypeExpr "<" TypeExpr ("," TypeExpr)* ">"
Path       := Ident ("." Ident)*     // allows alias.name

FnDecl     := ("f" | "fn") Ident "(" Params? ")" "->" TypeExpr EffAnn? Block
EffAnn     := "eff" "[" EffList "]"
Params     := Param ("," Param)*
Param      := Ident ":" TypeExpr

Block      := "{" Stmt* "}"
Stmt       := "ret" Expr
            | ("return")? Expr
            | "let" Ident "=" Expr
            | While
While      := "while" Expr Block

Expr       := Lit | Path RecordInit? CallSuffix*
RecordInit := "{" FieldList? "}"
FieldList  := Ident ":" Expr ("," Ident ":" Expr)*
CallSuffix := "(" ArgList? ")"
ArgList    := Expr ("," Expr)*

Lit        := STR | INT
```

`crates/z1-parse` can use a combinator parser (e.g., `chumsky`/`winnow`) or hand‑rolled; keep spans for diagnostics.

---

## 5) Compact ↔ Relaxed formatter (`z1fmt`)

Rules:

* **Compact print**: short keywords, apply `SymbolMap.to_short`, no optional whitespace, inline blocks when safe.
* **Relaxed print**: long keywords, restore long names, multiline blocks, trailing commas for records.

`crates/z1-fmt/src/lib.rs` (snippet)

```rust
pub enum Mode { Compact, Relaxed }

pub fn format_module(m: &Module, mode: Mode) -> String {
    match mode {
        Mode::Compact => fmt_compact(m),
        Mode::Relaxed => fmt_relaxed(m),
    }
}

// Example: function header
fn fmt_fn_header(f: &FnDecl, m: &Module, compact: bool) -> String {
    let kw = if compact { "f" } else { "fn" };
    let name = map_ident(&f.name, &m.sym_map, compact);
    let ret  = fmt_type(&f.ret, m, compact);
    let params = f.params.iter()
        .map(|(n,t)| format!("{}: {}", map_ident(n,&m.sym_map,compact), fmt_type(t,m,compact)))
        .collect::<Vec<_>>().join(", ");
    let eff = if f.effects.is_empty() {
        "".into()
    } else {
        let e = f.effects.iter().map(|e| fmt!("{e:?}").to_lowercase()).collect::<Vec<_>>().join(", ");
        format!(" eff [{}]", e)
    };
    format!("{kw} {name}({params})->{ret}{eff}")
}
```

**Round‑trip guarantee:** `parse(fmt_compact(AST))` and `parse(fmt_relaxed(AST))` both reproduce the same **SemHash**.

---

## 6) Type & effect checking (M1)

### Typing (structural)

* Records are equal if fields match (order‑insensitive).
* Sum types unify by variant labels.
* Path types resolve via imports (alias or full path).
* Simple generics later; MVP has monomorphic types.

### Effects

* Each `FnDecl` declares `effects: Set<Effect>`.
* Call site must be within a function whose effect set is a **superset** (effect subtyping).
* Intrinsics (e.g., `sleep_ms`) carry effects (time), enforced at check time.

Typing judgment sketches:

```
Γ ⊢ expr : T     Γ ⊢ f : (T1,..,Tn) -> Tr !E
---------------------------------------------
Γ ⊢ f(e1,..,en) : Tr    requires E ⊆ E_context
```

**Capabilities vs Effects**

* Pack manifest grants caps (`[net, time, ...]`).
* If a function has `eff [net]`, the cell’s `caps` must include `net`. Compiler rejects otherwise.

---

## 7) Context estimator (`z1ctx`)

* **Token cost model:** naive default `tokens ≈ ceil(chars / 3.8)` (configurable), improved by SDict (M2).
* **Granularity:** per‑fn, per‑cell.
* Enforced budgets: `m ... ctx=128` on module; optional `//@z1: ctx_fn=...` inline overrides.

Algorithm:

1. Format AST to **compact** text with current `SymbolMap`.
2. Apply SDict replacements (model‑specific) **for estimation only**.
3. Count tokens via configured model (or heuristic).
4. Compare to budgets; emit diagnostics & “split suggestions”.

---

## 8) Provenance & signatures (`z1-prov`)

**Entry format (`.z1p`, canonical JSON with sorted keys):**

```json
{
  "entry_id":"cell:http.server@3",
  "prev":"sha256:3f7...ab2",
  "actor":"agent:z1-agent/1.2.3",
  "model":"llm-x-2025-08",
  "prompt_sha256":"5fd...c91",
  "prompt_excerpt":"Refactor handler into pure function...",
  "inputs":["cells/http.server.z1c@sha256:..."],
  "cell_semhash":"sha256:a12...ee0",
  "cell_formhash":"sha256:bb1...992",
  "timestamp":"2025-10-25T16:03:10Z",
  "signatures":[
    {"by":"dev:alice@keys/ed25519","sig":"ed25519:ab8...2f1"},
    {"by":"agent:z1-agent/1.2.3","sig":"ed25519:9c1...77d"}
  ]
}
```

**Merkle root:** `PROVCHAIN.z1p` stores a vector of entry hashes and a root; `manifest.z1m` pins the current root.

Rust verification skeleton:

```rust
pub fn verify_entry(e: &ProvEntry, key: &PublicKey) -> bool {
    let canon = canonical_json_without_sigs(e); // sorted keys, no signatures
    ed25519_verify(key, &sha256(&canon), &e.signatures[..])
}
```

**Shadow metadata in code** (optional, preserved by formatter; excluded from SemHash):

```
//@z1: model="llm-x-2025-08", agent="z1-agent/1.2.3", ctx_in=2340
//:prompt: "Create trivial http server with pure handler."
```

---

## 9) CLI (`z1-cli`)

Commands:

```
z1fmt [--compact|--relaxed] <cell>
z1c   build --target ts|wasm --pack <dir>
z1ctx estimate <cell|pack> --model llm-x-2025-08 --sdict dicts/...
z1prov log add --cell cells/http.server.z1c --prompt prompt.txt --sign dev:alice
z1prov verify --pack .
z1test run
```

Use `clap` for parsing; delegate to crates.

---

## 10) Codegen (M3)

**IR (tiny SSA‑ish, expression‑oriented)**
Nodes: `Const`, `Param`, `Call`, `RecordNew`, `Return`, typed edges, effect annotations (metadata only).

**TS codegen skeleton** (`crates/z1-codegen-ts/src/lib.rs`)

```rust
pub fn emit_ts(m: &Module) -> String {
    let mut out = String::new();
    out.push_str("// Generated by z1\n");
    for d in &m.decls {
        match d {
            Decl::Type(td) => out.push_str(&emit_ts_type(td)),
            Decl::Fn(fd)   => out.push_str(&emit_ts_fn(fd, m)),
        }
    }
    out
}

fn emit_ts_fn(f: &FnDecl, m: &Module) -> String {
    let name = &f.name; // long name; formatter maps to short for compact
    let params = f.params.iter()
      .map(|(n,t)| format!("{}: {}", n, ts_type(t)))
      .collect::<Vec<_>>().join(", ");
    let ret = ts_type(&f.ret);
    // MVP: only returns literals/records/calls
    format!("export function {name}({params}): {ret} {{\n  {body}\n}}\n",
        body = emit_ts_block(&f.body, m))
}
```

**WASM backend** (M3)

* Lower IR to a minimal stack‑machine subset; host shims for `net`/`time` based on caps; deterministic build.

---

## 11) Policies (enforced by `z1-typeck`/`z1-effects`)

Defaults (configurable in `manifest.z1m`):

```
cell.max_ast_nodes = 200
cell.max_exports   = 5
deps.max_fanin     = 10
deps.max_fanout    = 10
ctx.max_per_fn     = 256
```

Violations return actionable diagnostics (with suggested split points).

---

## 12) Example package (ready to parse/format)

**`examples/http-example/cells/http.server.z1c`**

```z1c
m http.server:1.0 ctx=128 caps=[net]
u "std/http" as H only [listen, Req, Res]
#sym { serve: sv, handler: h, Health: Hl }

t Hl = { ok: Bool, msg: Str }
f h(q:H.Req)->H.Res eff [pure] { ret H.Res{ status:200, body:"ok" } }
f sv(p:U16)->Unit eff [net] { H.listen(p, h) }

//@z1: model="llm-x-2025-08", agent="z1-agent/1.2.3"
//:prompt: "Create trivial http server with pure handler."
```

**Relaxed view (output of `z1fmt --relaxed`)**

```z1r
module http.server : 1.0
  ctx = 128
  caps = [net]

use "std/http" as http only [listen, Req, Res]

// SymbolMap: { serve ↔ sv, handler ↔ h, Health ↔ Hl }

type Health = { ok: Bool, msg: Str }

fn handler(q: http.Req) -> http.Res
  eff [pure]
{
  return http.Res{ status: 200, body: "ok" };
}

fn serve(port: U16) -> Unit
  eff [net]
{
  http.listen(port, handler);
}

//@z1: model="llm-x-2025-08", agent="z1-agent/1.2.3"
//:prompt: "Create trivial http server with pure handler."
```

---

## 13) Tests (spec + prompt‑test skeleton)

**`examples/http-example/tests/http.spec.z1r`**

```z1r
spec "handler returns ok" {
  let r = handler(http.Req{ /*...*/ });
  assert r.status == 200;
}

prompt-test "rename handler to h" using model="llm-x-2025-08" {
  request: "Rename handler to h; update exports; no behavior change."
  expect: diff.only = ["rename","export-update"]
}
```

The `prompt-test` runner stubs a model adapter interface; in CI you can simulate with scripted refactors first.

---

## 14) SDict (model‑aware compaction)

`examples/http-example/dicts/llm-x-2025-08.sdict`

```toml
[types]
"Request" = "Q"
"Response" = "R"

[idents]
"serve" = "sv"
"handler" = "h"
```

**Rules**

* SDict is applied only at **format time** (and for ctx estimation).
* Never affects SemHash; safe to swap per target model.

---

## 15) Initial “z1 split” heuristic (to enforce small cells)

* If `ast_nodes(cell) > limit`, compute **dominator tree** over call graph.
* Choose split cut at fn with maximal `(fanout * body_size)` that keeps both halves under limits.
* Emit plan:

  * new cell with moved decls
  * rewrite imports/exports
  * update provenance with `kind: "split"`

---

## 16) Immediate next steps you can run with

1. **Initialize repo** with the scaffold above.
2. Implement lexer (ident/strings/keywords + comment capture) in `z1-lex`.
3. Implement the subset parser in `z1-parse` to cover the example cell.
4. Implement `z1-fmt` round‑trip for the example; verify **SemHash** stays stable across modes.
5. Wire `z1-cli z1fmt` to call parse→format.
6. Add `z1-hash` SemHash + FormHash; print both in `z1-cli`.
7. Add `z1-prov log add` to append a provenance entry and compute a Merkle root over the chain.

> From here, you can progressively add type/effect checks and codegen.

---

## 17) Design invariants (to keep us honest)

* **Canonical AST only uses long identifiers.** SymbolMap is formatting‑only.
* **SemHash excludes** comments, shadow metadata, and SymbolMap → alpha‑rename invariant.
* **FormHash includes** SymbolMap → detects name‑shortening churn.
* **Compact/relaxed** are **views**, not different programs.
* **Provenance is append‑only**; signatures verify over canonical JSON.

---

If you want, I can generate **concrete code files** for the lexer, a minimal parser covering the example grammar, and a working `z1fmt` in Rust so you can paste them straight into `crates/`.

