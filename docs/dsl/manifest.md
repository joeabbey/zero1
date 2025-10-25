# Part A — Z1 Manifest DSL (`manifest.z1m`)

## A.1 Format

* **File format:** **TOML 1.0** (normative). Use standard TOML grammar for tables, arrays, numbers, strings, and booleans.
* **Encoding:** UTF‑8.
* **Path separators:** `/` only (even on Windows). Paths are always relative to the pack root unless starting with `asset:` or `sha256:`.
* **Comments:** TOML comments with `#`.
* **Case sensitivity:** Keys are case‑sensitive.

> **Canonical projection:** `z1-cli manifest --json` must emit a canonical JSON object following the schema in **A.6** (sorted keys, normalized paths).

## A.2 Top‑level layout

All keys live in fixed top‑level tables. Unknown top‑level keys are a **warning** by default (error in `--strict` mode).

### Required tables

* `[package]`
* `[targets]` (or `targets = [...]` inline)

### Optional tables

* `[entrypoints]`, `[deps]`, `[dicts]`, `[budgets]`, `[capabilities]`, `[policy]`, `[provenance]`, `[security]`, `[fmt]`, `[lint]`, `[test]`, `[profiles.*]`, `[env]`, `[scripts]`, `[extern]`.

---

## A.3 Semantics by table

### `[package]` (required)

| Key           | Type            | Default        | Notes                                                                                         |
| ------------- | --------------- | -------------- | --------------------------------------------------------------------------------------------- |
| `name`        | string          | —              | DNS‑like or dotted name, e.g. `"acme.http"`. Must match `^[a-z][a-z0-9]*(\.[a-z][a-z0-9]*)*$` |
| `version`     | string (SemVer) | `"0.1.0"`      | SemVer, no prerelease required.                                                               |
| `license`     | string          | `"UNLICENSED"` | SPDX ID (e.g., `"Apache-2.0"`) or `"UNLICENSED"`.                                             |
| `description` | string          | `""`           | Short human summary.                                                                          |
| `authors`     | array of string | `[]`           | `"Name <email>"` recommended.                                                                 |
| `repository`  | string (URL)    | `""`           |                                                                                               |
| `homepage`    | string (URL)    | `""`           |                                                                                               |
| `readme`      | string (path)   | `""`           | Optional path to README.                                                                      |

### `targets` (required)

Either as an array or table:

* **Array form**:
  `targets = ["wasm32", "ts", "rust"]`
* **Table form**:

  ```toml
  [targets]
  wasm32 = { opt_level = 2 }
  ts     = { module = "esm" }
  rust   = { edition = "2021", crate = "cdylib" }
  ```

**Supported target ids:** `"wasm32"`, `"ts"`, `"rust"`.
Per‑target options are free‑form maps; unknown keys are warnings unless `--strict`.

### `[entrypoints]` (optional)

Declares root cells (modules) for builds.

| Key     | Type               | Default              | Notes                              |
| ------- | ------------------ | -------------------- | ---------------------------------- |
| `lib`   | string (cell path) | `""`                 | e.g., `"cells/lib.z1c"`.           |
| `bins`  | array of string    | `[]`                 | e.g., `["cells/http.server.z1c"]`. |
| `tests` | array of string    | `["tests/**/*.z1t"]` | Glob patterns for test files.      |

### `[deps]` (optional)

Dependencies on other Z1 packs or standard libraries.

Two forms are allowed under `[deps]`:

1. **Registry/Git deps** as subtables named by package:

```toml
[deps."std"]
version = "0.5.1"
```

or

```toml
[deps."acme.util"]
git = "https://github.com/acme/zero1-util.git"
rev = "3b8d57c"
```

2. **Path deps**:

```toml
[deps."local.math"]
path = "../math-pack"
```

**Rules**

* Exactly one of `version` / `git`+`rev` / `path` is required.
* Optional `checksum` (sha256) may pin a registry tarball.

### `[dicts]` (optional)

Model‑specific compaction dictionaries.

* **Map form:** keys are model ids; values are file paths.

```toml
[dicts]
"llm-x-2025-08" = "dicts/llm-x-2025-08.sdict"
"gpt-neo-5"     = "dicts/gpt-neo-5.sdict"
```

### `[budgets]` (optional)

Context/token budgets.

| Key                | Type    | Default | Notes                                             |
| ------------------ | ------- | ------- | ------------------------------------------------- |
| `ctx.fn_default`   | integer | `256`   | Max tokens per function (estimate).               |
| `ctx.cell_default` | integer | `512`   | Max tokens per cell.                              |
| `ctx.pack_max`     | integer | `65536` | Soft cap across build slices.                     |
| `estimator.model`  | string  | `""`    | If set, use SDict for that model; else heuristic. |

### `[capabilities]` (optional)

Pack‑level capability grants (upper bound; cells must still declare).

| Key     | Type            | Default | Notes                             |
| ------- | --------------- | ------- | --------------------------------- |
| `allow` | array of string | `[]`    | e.g., `["net", "time", "fs.ro"]`  |
| `deny`  | array of string | `[]`    | Deny takes precedence over allow. |

### `[policy]` (optional)

Enforced size/shape constraints (compiler & linter).

| Key                         | Type | Default |
| --------------------------- | ---- | ------- |
| `cell.max_ast_nodes`        | int  | `200`   |
| `cell.max_exports`          | int  | `5`     |
| `deps.max_fanin`            | int  | `10`    |
| `deps.max_fanout`           | int  | `10`    |
| `ctx.max_per_fn`            | int  | `256`   |
| `effects.allow_unsafe`      | bool | `false` |
| `require_inline_provenance` | bool | `false` |
| `deny_shadow_comments`      | bool | `false` |

### `[provenance]` (optional)

Controls provenance capture & signature policy.

| Key                | Type                     | Default    | Notes                                              |
| ------------------ | ------------------------ | ---------- | -------------------------------------------------- |
| `inline`           | bool                     | `true`     | Keep shadow comments in code (ignored by SemHash). |
| `required_signers` | array of string          | `[]`       | Patterns like `"dev:*"`, `"agent:z1-agent/*"`.     |
| `timestamp_source` | `"system"` | `"rfc3161"` | `"system"` | RFC3161 requires TSA config.                       |
| `tsa.url`          | string                   | `""`       | If using RFC3161.                                  |
| `signing.keys`     | array of string          | `[]`       | Paths to Ed25519 public keys allowed to sign.      |

### `[security]` (optional)

| Key                    | Type   | Default     | Notes                                         |
| ---------------------- | ------ | ----------- | --------------------------------------------- |
| `merkle_root`          | string | `""`        | Pinned after release; empty means “unlocked”. |
| `lockfile`             | string | `"z1.lock"` | Dependency lock.                              |
| `allow_unsigned_local` | bool   | `true`      | If false, local builds require signatures.    |

### `[fmt]` (optional)

Formatter rules.

| Key                    | Type   | Default                   |
| ---------------------- | ------ | ------------------------- |
| `line_width`           | int    | `100`                     |
| `indent`               | int    | `2`                       |
| `newline`              | string | `"lf"` (`"crlf"` allowed) |
| `keep_symbolmap_order` | bool   | `true`                    |

### `[lint]` (optional)

Flags for style & structure.

| Key                     | Type | Default |
| ----------------------- | ---- | ------- |
| `single_responsibility` | bool | `true`  |
| `exhaustive_match`      | bool | `true`  |
| `no_wildcard_imports`   | bool | `true`  |

### `[test]` (optional)

Runner configuration (defaults used by `.z1t` files).

| Key             | Type            | Default                 |
| --------------- | --------------- | ----------------------- |
| `timeout_ms`    | int             | `5000`                  |
| `parallel`      | int             | `4`                     |
| `seed`          | int             | `0`                     |
| `include`       | array of string | `["tests/**/*.z1t"]`    |
| `exclude`       | array of string | `[]`                    |
| `tags.include`  | array of string | `[]`                    |
| `tags.exclude`  | array of string | `[]`                    |
| `snapshots.dir` | string          | `"tests/__snapshots__"` |

### `[profiles.*]` (optional)

Overlays by profile name (e.g., `dev`, `ci`, `prod`). Keys inside a profile mirror top‑level sections and **shallow‑merge** over them.

```toml
[profiles.dev.test]
timeout_ms = 15000
parallel   = 8
```

> **Precedence:** base → profile overlay → CLI flags. Array values **replace**; maps shallow‑merge.

### `[env]` (optional)

Key–value map of environment vars provided to builds/tests (non‑secret).

### `[scripts]` (optional)

Named CLI script aliases, e.g.,

```toml
[scripts]
build = "z1c build --target wasm32"
fmt   = "z1fmt --expand cells/*.z1c"
```

### `[extern]` (optional)

Foreign module configuration (host shims, FFI).

| Key                       | Type            | Example                         |
| ------------------------- | --------------- | ------------------------------- |
| `allow`                   | array of string | `["time.now_ms", "net.listen"]` |
| `deny`                    | array of string | `["fs.write"]`                  |
| `shim."time.now_ms".wasm` | string (path)   | `"shims/time_now_ms.wasm"`      |

---

## A.4 Validation rules

* `package.name` unique within workspace; matches regex above.
* `targets` not empty; ids must be known.
* If `[capabilities].deny` intersects `[capabilities].allow`, **deny wins** (warning printed).
* `[provenance].required_signers` non‑empty if `[security].allow_unsigned_local=false`.
* Dict files in `[dicts]` must exist; model ids are free‑form strings.
* Glob patterns use **POSIX glob** (no regex).

---

## A.5 Examples

### Minimal

```toml
[package]
name = "acme.http"
version = "0.1.0"
license = "Apache-2.0"

targets = ["wasm32", "ts"]

[entrypoints]
bins = ["cells/http.server.z1c"]
```

### Full

```toml
[package]
name = "acme.http"
version = "0.3.4"
license = "Apache-2.0"
authors = ["Alice <alice@acme.io>"]
repository = "https://github.com/acme/zero1-http"

[targets]
wasm32 = { opt_level = 2 }
ts     = { module = "esm" }

[entrypoints]
lib  = "cells/lib.z1c"
bins = ["cells/http.server.z1c"]
tests = ["tests/**/*.z1t"]

[deps."std"]
version = "0.5.1"

[dicts]
"llm-x-2025-08" = "dicts/llm-x-2025-08.sdict"

[budgets]
ctx.fn_default = 256
ctx.cell_default = 512
ctx.pack_max = 65536
estimator.model = "llm-x-2025-08"

[capabilities]
allow = ["net", "time"]
deny  = []

[policy]
cell.max_ast_nodes = 200
cell.max_exports   = 5
deps.max_fanin     = 10
deps.max_fanout    = 10
ctx.max_per_fn     = 256
effects.allow_unsafe = false

[provenance]
inline = true
required_signers = ["dev:*", "agent:z1-agent/*"]
timestamp_source = "system"
signing.keys = ["keys/devs.pub"]

[security]
merkle_root = ""
lockfile = "z1.lock"
allow_unsigned_local = true

[fmt]
line_width = 100
indent = 2
newline = "lf"

[lint]
single_responsibility = true
exhaustive_match = true
no_wildcard_imports = true

[test]
timeout_ms = 5000
parallel = 6
seed = 1234
include = ["tests/**/*.z1t"]
exclude = ["tests/slow/**"]
snapshots.dir = "tests/__snapshots__"

[profiles.ci.test]
timeout_ms = 20000
parallel = 8
```

---

## A.6 Canonical JSON schema (for `z1-cli manifest --json`)

This schema (abridged for space) is **normative** for the JSON projection.

```json
{
  "$schema": "https://json-schema.org/draft/2020-12/schema",
  "title": "Zero1 Manifest JSON",
  "type": "object",
  "required": ["package", "targets"],
  "additionalProperties": false,
  "properties": {
    "package": {
      "type": "object",
      "required": ["name", "version", "license"],
      "additionalProperties": false,
      "properties": {
        "name": { "type": "string", "pattern": "^[a-z][a-z0-9]*(\\.[a-z][a-z0-9]*)*$" },
        "version": { "type": "string" },
        "license": { "type": "string" },
        "description": { "type": "string" },
        "authors": { "type": "array", "items": { "type": "string" } },
        "repository": { "type": "string" },
        "homepage": { "type": "string" },
        "readme": { "type": "string" }
      }
    },
    "targets": {
      "oneOf": [
        { "type": "array", "items": { "type": "string" }, "minItems": 1 },
        { "type": "object", "additionalProperties": { "type": "object" } }
      ]
    },
    "entrypoints": {
      "type": "object",
      "additionalProperties": false,
      "properties": {
        "lib": { "type": "string" },
        "bins": { "type": "array", "items": { "type": "string" } },
        "tests": { "type": "array", "items": { "type": "string" } }
      }
    },
    "deps": { "type": "object", "additionalProperties": { "type": "object" } },
    "dicts": { "type": "object", "additionalProperties": { "type": "string" } },
    "budgets": { "type": "object", "additionalProperties": {} },
    "capabilities": { "type": "object", "additionalProperties": false,
      "properties": {
        "allow": { "type": "array", "items": { "type": "string" } },
        "deny":  { "type": "array", "items": { "type": "string" } }
      }
    },
    "policy": { "type": "object", "additionalProperties": {} },
    "provenance": { "type": "object", "additionalProperties": {} },
    "security": { "type": "object", "additionalProperties": {} },
    "fmt": { "type": "object", "additionalProperties": {} },
    "lint": { "type": "object", "additionalProperties": {} },
    "test": { "type": "object", "additionalProperties": {} },
    "profiles": { "type": "object", "additionalProperties": { "type": "object" } },
    "env": { "type": "object", "additionalProperties": { "type": "string" } },
    "scripts": { "type": "object", "additionalProperties": { "type": "string" } },
    "extern": { "type": "object", "additionalProperties": {} }
  }
}
```

