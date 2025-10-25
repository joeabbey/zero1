# ADR 001: Codegen Strategy - Direct AST-to-Target vs LLVM

## Status

Proposed

## Context

Zero1 needs to generate code for two targets:
1. **TypeScript** - For Node.js/browser runtimes
2. **WebAssembly** - For sandboxed, deterministic execution

We need to decide between:
- **Option A**: Use LLVM as a backend (wasm32 target)
- **Option B**: Direct codegen from AST to each target
- **Option C**: Simple custom IR, then target-specific backends

## Decision Drivers

### Zero1's Unique Characteristics

1. **Cell size limits**: Max 200 AST nodes, ~800 compact tokens
2. **Context efficiency paramount**: Token count matters more than runtime perf
3. **Deterministic builds**: Required for provenance chain integrity
4. **Agent workflows**: Fast iteration cycles, clear diagnostics critical
5. **Dual targets**: TS and WASM both needed
6. **Simplicity philosophy**: "Tiny pieces loosely joined"

### Codegen Requirements

- Generate valid TypeScript with type annotations
- Generate valid WebAssembly (text or binary format)
- Preserve effect annotations in generated code (metadata)
- Fast compile times (agent iteration loops)
- Deterministic output (same input → same binary)
- Clear error messages referencing source cells

## Options Analyzed

### Option A: LLVM Backend

**How it works:**
- Define LLVM IR lowering from Z1 AST
- Use LLVM's wasm32 target for WASM
- Still need separate TS codegen (LLVM doesn't help here)

**Pros:**
- Battle-tested optimization infrastructure
- Excellent WASM support built-in
- Could leverage existing optimizations
- Well-documented ecosystem

**Cons:**
- **Massive dependency**: ~500MB added to binary, 5-10min added to build
- **Overkill for small cells**: 200 AST nodes don't need heavy optimization
- **TS target unaffected**: Half the codegen gets no benefit
- **Complexity**: LLVM IR learning curve for contributors
- **Determinism concerns**: LLVM can have platform-specific behavior
- **Slow iteration**: Against Zero1's agent-friendly philosophy
- **Binary bloat**: z1-cli would be 100MB+ instead of <10MB

**Verdict:** ❌ **Not recommended** - Costs outweigh benefits for Zero1's use case

### Option B: Direct AST-to-Target Codegen

**How it works:**
- `z1-codegen-ts`: AST → TypeScript (string generation)
- `z1-codegen-wasm`: AST → WASM (using `wasm-encoder` or `walrus`)
- No intermediate IR, direct translation

**Pros:**
- **Simple and transparent**: Easy to understand and debug
- **Fast builds**: No heavyweight dependencies
- **Small binaries**: z1-cli stays <10MB
- **Deterministic**: Full control over output
- **Contributor-friendly**: TypeScript and WASM generation is straightforward
- **Fast iteration**: Compile in milliseconds, not minutes

**Cons:**
- Code duplication between backends (but cells are tiny)
- Manual optimization (but cells are too small to need much)
- Have to implement WASM encoding ourselves

**Verdict:** ✅ **Recommended** - Best fit for Zero1's philosophy

### Option C: Custom Simple IR + Backends

**How it works:**
- Define tiny SSA-style IR (as sketched in design.md)
- Lower Z1 AST → IR
- IR → TS and IR → WASM backends

**Pros:**
- Share optimization logic between backends
- Still lightweight (IR is just Rust enums)
- More structured than direct codegen
- Room to grow if needed

**Cons:**
- More complexity than Option B
- Another layer to debug
- May be premature for current scope

**Verdict:** 🟡 **Acceptable alternative** - Good middle ground

## Decision

**Choose Option B (Direct Codegen) for MVP, with Option C as future enhancement.**

### Rationale

1. **Zero1 cells are tiny** (200 AST nodes): Traditional compiler optimizations don't help
2. **Fast iteration is critical**: Agent workflows can't wait minutes for builds
3. **Deterministic builds**: Direct control ensures provenance chain integrity
4. **Binary size matters**: Small CLI is more deployable
5. **TypeScript target**: LLVM provides zero value here (50% of use case)
6. **Simplicity**: Direct codegen is easier for contributors to understand

### Implementation Plan

**For TypeScript (z1-codegen-ts):**
```rust
pub fn emit_ts(module: &Module) -> String {
    // Direct AST → TypeScript string generation
    // Preserve type annotations
    // Add effect metadata as JSDoc comments
}
```

**For WebAssembly (z1-codegen-wasm):**
```rust
// Use lightweight wasm-encoder crate
pub fn emit_wasm(module: &Module) -> Vec<u8> {
    let mut wasm = wasm_encoder::Module::new();
    // Direct AST → WASM instructions
    // Cap-based imports (net, time, etc.)
}
```

**Optimization Strategy:**
- Cells are too small to need aggressive optimization
- Focus on clear, idiomatic output code
- Let TS/WASM runtimes handle optimization
- If needed later: add peephole optimizations, not full compiler

## Consequences

### Positive

- Fast compile times (milliseconds, not minutes)
- Small z1-cli binary (<10MB vs 100MB+)
- Easy to understand and contribute to
- Full control over deterministic output
- No massive dependencies to manage

### Negative

- Code duplication between TS and WASM backends (mitigated by small cells)
- Limited optimization opportunities (acceptable for tiny cells)
- Manual WASM encoding (but wasm-encoder makes this easy)

### Neutral

- Can revisit if cells grow larger (but hard limits prevent this)
- Could add custom IR layer later without full rewrite
- Cranelift is an option if WASM optimization becomes critical

## Alternatives Considered

### Cranelift (Lightweight Code Generator)

**What it is:** Modern code generator used by Wasmtime, much smaller than LLVM

**Analysis:**
- Still adds ~50MB to binary (vs LLVM's 500MB)
- Only helps WASM target, not TypeScript
- Adds complexity for minimal benefit given cell size limits
- **Verdict:** Better than LLVM, but still unnecessary for Zero1

### Binaryen

**What it is:** WASM optimizer and toolchain

**Analysis:**
- Could use for post-processing generated WASM
- Adds dependency but smaller than LLVM
- Optimization not critical for tiny cells
- **Verdict:** Could add later if needed, skip for MVP

## References

- [LLVM wasm32 target documentation](https://llvm.org/docs/CommandGuide/llc.html)
- [wasm-encoder crate](https://crates.io/crates/wasm-encoder) - Lightweight WASM generation
- [Cranelift](https://cranelift.dev/) - Lightweight alternative to LLVM
- Zero1 design.md section 10: Codegen (M3)
- Zero1 vision.md: "Tiny pieces loosely joined"

## Notes

This decision aligns with Zero1's core philosophy:
- **Context efficiency**: Small binaries, fast builds
- **Auditability**: Deterministic output, clear provenance
- **Composability by enforcement**: Simple tools for simple cells
- **Agent-friendly**: Fast iteration, no heavyweight deps

If Zero1 evolves to support larger cells or compute-intensive workloads, we can:
1. Add custom IR layer (Option C)
2. Consider Cranelift for WASM optimization
3. Add opt-in LLVM backend for specialized use cases

But for the core use case (LLM-generated tiny cells), direct codegen is optimal.
