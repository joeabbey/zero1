Below is a concrete, end‑to‑end design for **Zero1 (Z1)** — a programming language optimized for LLM‑driven development, context/token efficiency, and auditable provenance. It includes the execution model, syntax, file layout, provenance/security, compact↔relaxed modes, and examples.

---

## 1) Goals & constraints

**Primary goals**

* **Context efficiency:** Minimize tokens for common constructs and enable model‑specific compaction.
* **Auditability:** Preserve prompts, model versions, and agent actions as first‑class provenance.
* **Composability by enforcement:** Keep code small, modular, and analyzable; make “tiny pieces loosely joined” a compile‑time rule, not a style.
* **Dual representation:** Every artifact exists in **compact** (LLM‑friendly) and **relaxed** (human‑friendly) modes, losslessly interconvertible.

**Secondary goals**

* **Deterministic AST:** Mode changes don’t change semantics or hashes of the canonical AST.
* **Capability & effect safety:** Strong effect annotations and capability scoping for agents.
* **Targetable:** Transpile to WASM/TypeScript/Rust backends without runtime surprises.

---

## 2) Source anatomy & packaging

A **Z1 pack** (folder or bundle) is content‑addressed and Merkle‑hashed. Canonical layout:

```
pkg/
 ├─ manifest.z1m           # package metadata, targets, budgets, caps, hashes
 ├─ cells/                 # the only place code lives; many tiny files
 │   ├─ <name>.z1c        # compact cells
 │   └─ <name>.z1r        # relaxed cells (optional; tool can generate)
 ├─ dicts/                 # model-specific compaction dictionaries (SDict)
 │   └─ <model>.sdict
 ├─ prov/                  # provenance chain (prompt lineage, signatures)
 │   ├─ PROVCHAIN.z1p
 │   └─ cells/<cell-id>.z1p
 ├─ tests/                 # spec/property tests (relaxed or compact)
 └─ assets/                # non-code artifacts; referenced by content hash
```

**Cells** are the unit of composition and enforcement:

* Hard limits (default): `<= 200 AST nodes`, `<= 800 compact tokens`, `<= 5 exports`.
* Fan‑in/out limits per cell (e.g., max 10 imports, max 10 dependents).
* Each cell declares its **context budget** and **capabilities**.

---

## 3) Compact vs relaxed modes

* **Compact (.z1c)**: Token‑lean syntax, short keywords, dictionary‑mapped symbols. Intended for LLMs.
* **Relaxed (.z1r)**: Expanded identifiers, full keywords, doc comments, human‑readable formatting.

**Guarantees**

* Both modes parse to the **same canonical AST**.
* A **SymbolMap** in each cell binds long names ⇄ short codes (stable within the cell).
* Optional **SDict** (per model) does model‑aware compaction at the I/O boundary to match that model’s tokenization.

**Example dictionary (dicts/gpt‑X.sdict)**

```toml
# Maps frequent domain symbols to short forms that this model merges well.
[types]
"Request" = "Q"
"Response" = "R"

[idents]
"serve"   = "sv"
"handler" = "h"
"UserService" = "US"
```

---

## 4) Core language (essentials)

### 4.1 Modules, imports, capabilities, budgets

**Relaxed**

```z1r
module http.server : 1.0
  ctx = 128
  caps = [net]

use "std/http" as http only [listen, Req, Res]
```

**Compact**

```z1c
m http.server:1.0 ctx=128 caps=[net]
u "std/http" as H only [listen, Req, Res]
```

### 4.2 Types & functions (with effects)

**Relaxed**

```z1r
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
```

**Compact (with SymbolMap at top)**

```z1c
#sym { handler: h, serve: sv }
t Health = { ok: Bool, msg: Str }

f h(q:H.Req)->H.Res eff [pure] { ret H.Res{ status:200, body:"ok" } }
f sv(p:U16)->Unit eff [net] { H.listen(p, h) }
```

### 4.3 Effects & capabilities

* **Effects** are static (`pure`, `fs`, `net`, `time`, `crypto`, `env`, `unsafe`).
* **Capabilities** are pack‑level grants; cells declare `caps=[...]`.
* Call‑sites must be effect‑compatible; compiler enforces **effect subtyping**.

### 4.4 Types

* **Structural typing** with optional nominal tags.
* **Refinements** (`where` clauses) for pre/post conditions.
* **Sum/union types** via `|`.
* **Generics** with `<>`.

```z1r
type Id<T> = { val: T } where T: Copy
type Result<T, E> = Ok{ T } | Err{ E }
```

### 4.5 Concurrency primitives

* **Tasks** (`task { ... }`) spawn fibers.
* **Channels** typed as `Chan<T>`.
* `await` for async results; determinism enforced via effect system (`async` effect).

---

## 5) Grammar sketch (subset)

Relaxed (selected):

```
Module    ::= "module" Ident (":" Version)? ("ctx" "=" Int)? ("caps" "=" "[" CapList "]")?
Import    ::= "use" String ("as" Ident)? ("only" "[" Ident ("," Ident)* "]")?
TypeDecl  ::= "type" Ident "=" TypeExpr
FnDecl    ::= "fn" Ident "(" Params? ")" RetAnn EffAnn? Block
RetAnn    ::= "->" TypeExpr
EffAnn    ::= "eff" "[" EffList "]"
Block     ::= "{" Stmt* "}"
```

Compact deltas:

```
"module"→"m"  "use"→"u"  "type"→"t"  "fn"→"f"  "return"→"ret"
```

Whitespace is insignificant; indentation is advisory for relaxed formatting only.

---

## 6) Token & context optimization

1. **Short keywords** in compact mode (`m,u,t,f,ret`).
2. **SymbolMap** per cell (`#sym { LongName: LN, ... }`) — stable, hashed, and included in AST to avoid capture.
3. **SDict** (model‑specific) layer for I/O compaction; never affects canonical AST; used only when interacting with a particular LLM.
4. **Budgeting**:

   * `ctx=<N>` at module/cell/function level expresses **estimated input context** the compiler must meet.
   * The compiler rejects code that exceeds declared budgets unless overridden with `ctx+`.
5. **Constant pools & literals**:

   * Large strings move to `assets/` and are referenced by content hash: `"asset:sha256:..."`
6. **Import pruning**:

   * `use ... only [...]` trims name exposure & symbol tables → fewer tokens in prompts.

---

## 7) Provenance & prompt lineage (first‑class)

Z1 records “how this code came to be” in `prov/` as append‑only logs, plus **shadow comments** you can keep in‑file for auditing.

### 7.1 In‑file shadow comments (optional)

```z1r
//@z1: model="llm-x-2025-08", agent="z1-agent/1.2.3", ctx_in=2_340
//:prompt: "Refactor handler into pure function; no net effect."
//:inputs: ["cells/http.server.z1r@sha256:..."]
```

These lines are ignored by the parser but preserved by the formatter and included in hash calculations **only if** `prov.inline=true` in `manifest.z1m`.

### 7.2 Provenance entries (prov/cells/http.server.z1p)

```yaml
entry_id: cell:http.server@v3
prev: sha256:3f7...ab2
actor: agent:z1-agent/1.2.3
model: llm-x-2025-08
prompt_sha256: 5fd...c91
prompt_excerpt: "Refactor handler into pure function..."
tools: ["rename-symbols", "effect-infer", "z1fmt --compact"]
diff_sha256: a12...ee0
timestamp: 2025-10-25T16:03:10Z
signatures:
  - by: dev:alice@keys/ed25519
    sig: ed25519:ab8...2f1
  - by: agent:z1-agent/1.2.3
    sig: ed25519:9c1...77d
```

**PROVCHAIN.z1p** is a Merkle chain over all entries; the pack’s `manifest.z1m` pins the current root.

---

## 8) Security & integrity

* **Content addressing** for cells/assets; pack hash is Merkle root.
* **Sig‑chain**: human and agent signatures over each prov entry and the pack manifest.
* **Capability seals**: issuing authority signs the list of allowed caps in `manifest.z1m`; the compiler refuses to compile if new caps appear without a new seal.
* **Reproducible builds**: codegen emits deterministic WASM/TS; build hash recorded in provenance.
* **Policy hooks**: orgs can mandate “no unsigned diffs”, “no `unsafe` effects”, “max ctx per cell”, etc.

---

## 9) Enforcing small, composable code

Compiler checks (configurable; defaults shown):

* `cell.max_ast_nodes = 200`
* `cell.max_exports = 5`
* `deps.max_fanin = 10`, `deps.max_fanout = 10`
* `fn.max_params = 6`, `fn.max_locals = 32`
* `ctx.max_per_fn = 256`
* `lint.enforce_single_responsibility = true` (data‑flow heuristic)
* Violations produce fix‑it hints (e.g., “Split cell; suggested cut at function `parse_headers`”).

---

## 10) File formats

### 10.1 `manifest.z1m` (TOML‑like)

```toml
[package]
name = "http-example"
version = "0.1.0"
target = ["wasm32", "ts"]
license = "Apache-2.0"

[budgets]
ctx.fn_default = 256
ctx.cell_default = 512

[capabilities]
allow = ["net"]

[provenance]
inline = true
required_signers = ["dev:*", "agent:z1-agent/*"]

[security]
merkle_root = "sha256:8fe...c3a"
```

### 10.2 Symbol map header (inside cells)

```z1c
#sym { serve: sv, handler: h, Request: Q, Response: R }
```

* Present in both modes; ignored by relaxed formatter when rendering names long‑form.

---

## 11) Interop & codegen

* **WASM**: default target; memory/host calls generated from effects/caps (e.g., `net` → host shim).
* **TypeScript**: ergonomic dev target with generated type defs and effect wrappers.
* **Rust**: for high‑performance backends; Z1 maps structural types to Rust structs/enums.
* **FFI**: `extern` bindings describe foreign functions plus required effects.

```z1r
extern fn now_ms() -> U64 eff [time]
```

---

## 12) Testing: specs, properties, prompt‑tests

* `tests/*.z1r` contain `spec` and `prop` blocks.
* **Prompt‑tests** verify a change request + seed + model version results in a constrained diff (useful for agent governance).

```z1r
spec "handler returns ok" {
  let r = handler(http.Req{...});
  assert r.status == 200;
}

prompt-test "rename handler to h" using model="llm-x-2025-08" {
  request: "Rename handler to h; update exports; no behavior change."
  expect: diff.only = ["rename", "export-update"]
}
```

---

## 13) Example: compact vs relaxed (side‑by‑side)

**cells/http.server.z1c (compact)**

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

**cells/http.server.z1r (relaxed)**

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

> The relaxed file can be auto‑generated from the compact file (and vice‑versa) via `z1fmt --expand` / `--compact`. The AST and Merkle hashes (for code parts) remain stable.

---

## 14) Concurrency example (channels + effects)

**cells/ping.z1c**

```z1c
m ex.ping:1 caps=[time]
t Tick = { at: U64 }

f start(n:U32)->Chan<Tick> eff [time, async] {
  let c = chan<Tick>(1)
  task {
    let i=0
    while i<n {
      send(c, Tick{ at: now_ms() })
      sleep_ms(100)   // time effect
      i = i+1
    }
    close(c)
  }
  ret c
}
```

---

## 15) Agent workflows (designed in)

* **Context slices**: `z1 ctx-slice` emits the minimal transitive closure of cells needed for a task, respecting budgets, with SDict applied — ideal for feeding an LLM.
* **Plan/Apply**: `z1 plan` generates a patch from a natural‑language request; `z1 apply` applies it only if it validates (effects, budgets, tests). Both write to `prov/`.
* **Role hints**: shadow comments can tag desired agent roles: `//:role: "rename-only"`.

---

## 16) Tooling

* `z1c`: compiler (typecheck, effects, budgets) → WASM/TS/Rust.
* `z1fmt`: compact↔relaxed formatter; maintains SymbolMap.
* `z1prov`: provenance viewer and signature verifier.
* `z1lint`: composability & policy enforcement.
* `z1ctx`: context estimator (per function/cell/pack).
* `z1test`: spec/prop/prompt‑tests.

---

## 17) Design choices explained (high level)

* **Why SymbolMap + SDict?**
  BPE merges differ by model; SymbolMap gives stable short names within a cell, while SDict lets you optionally align with a specific model’s merges without touching semantics or hashes.

* **Why two modes?**
  Agents thrive on minimal tokens; humans want clarity. Keeping a single canonical AST ensures you can switch views freely without “format churn.”

* **Why enforced smallness?**
  LLM agents plan better with many small units than with a few monoliths. Limits are tuned for tractable reasoning steps and precise provenance.

* **Why capabilities/effects?**
  They provide guardrails for agent‑authored code, ease review, and enable minimal host shims for WASM.

---

## 18) Quick cheat sheet

**Compact keywords:** `m,u,t,f,ret,mut,if,else,while,match,task,await`

**Effects:** `[pure, fs, net, time, crypto, env, async, unsafe]`

**Caps examples:** `caps=[fs.ro, net, time]`

**Headers:** `m name:ver ctx=<N> caps=[...]`
**Imports:** `u "path" as A only [x,y]`
**Types:** `t Name = { field:Type, ... } | Sum{ ... }`
**Fns:** `f name(args)->Type eff [..] { ... }`
**Return:** `ret expr`

---

## 19) What a minimal starter project looks like

```
http-example/
 ├─ manifest.z1m
 ├─ dicts/
 │   └─ llm-x-2025-08.sdict
 ├─ cells/
 │   ├─ http.server.z1c
 │   └─ http.types.z1r
 ├─ tests/
 │   └─ http.spec.z1r
 └─ prov/
     ├─ PROVCHAIN.z1p
     └─ cells/http.server.z1p
```

Run:

```
z1fmt --expand cells/http.server.z1c > cells/http.server.z1r
z1c --target wasm32
z1test
z1prov verify
```

---

## 20) Roadmap (optional extensions)

* **Semantic chunking hints** for editors to generate perfect LLM context slices.
* **In‑cell doc embeddings** referencing external design docs by hash (no large text in code).
* **Capability escrow** for CI/CD (temporary caps during build/test, dropped for production).
* **Typed resources** (e.g., `File<ReadOnly>`), unified with effects.

---

### Summary

Zero1 gives you:

* A **compact** syntax engineered for token efficiency and LLM interaction.
* A **relaxed** view for humans, both mapping to a **single canonical AST**.
* Enforced **small, composable cells** with explicit **context budgets**.
* Built‑in **provenance**, prompt lineage, signatures, and **capability/effect safety**.
* A pragmatic toolchain to format, verify, test, and compile to real targets.

If you’d like, I can generate a tiny reference implementation plan (parser + AST + formatter + type/effect checker outline) and a few sample `z1fmt` transformations to make this immediately buildable.

