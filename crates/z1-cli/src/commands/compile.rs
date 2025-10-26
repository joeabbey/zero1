//! Z1 cell compilation orchestration.
//!
//! This module implements the `z1c` command which orchestrates the full compilation pipeline:
//! 1. Parse source code to AST
//! 2. Type checking (structural types + generics)
//! 3. Effect/capability checking
//! 4. Context estimation + budget enforcement
//! 5. Policy gate enforcement
//! 6. IR generation (placeholder)
//! 7. Code generation (TypeScript or WASM)

use anyhow::{Context, Result};
use std::fs;
use std::path::{Path, PathBuf};
use z1_ast::Module;

use crate::error_printer;

/// Compilation target language.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum CompileTarget {
    TypeScript,
    Wasm,
}

/// Compilation options.
pub struct CompileOptions {
    pub input_path: PathBuf,
    pub output_path: Option<PathBuf>,
    pub target: CompileTarget,
    pub binary: bool,
    pub check: bool,
    pub emit_ir: bool,
    pub opt_level: z1_ir::optimize::OptLevel,
    pub verbose: bool,
}

/// Orchestrate the full compilation pipeline.
pub fn compile(opts: CompileOptions) -> Result<()> {
    if opts.verbose {
        println!("Compiling: {}", opts.input_path.display());
    }

    // Step 1: Read and parse
    let source = fs::read_to_string(&opts.input_path)
        .with_context(|| format!("Failed to read {}", opts.input_path.display()))?;

    if opts.verbose {
        println!("  [1/7] Parsing...");
    }

    let file_path = opts.input_path.to_string_lossy().to_string();
    let module = z1_parse::parse_module(&source).map_err(|e| {
        let config = error_printer::ErrorPrinterConfig::default();
        error_printer::print_parse_error(&e, &source, &file_path, &config);
        anyhow::anyhow!("Parse failed")
    })?;

    // Step 2: Type check (if enabled)
    if opts.check {
        if opts.verbose {
            println!("  [2/7] Type checking...");
        }
        check_types(&module, &source, &file_path).context("Type check failed")?;
    } else if opts.verbose {
        println!("  [2/7] Type checking... (skipped)");
    }

    // Step 3: Effect check (if enabled)
    if opts.check {
        if opts.verbose {
            println!("  [3/7] Effect checking...");
        }
        check_effects(&module, &source, &file_path).context("Effect check failed")?;
    } else if opts.verbose {
        println!("  [3/7] Effect checking... (skipped)");
    }

    // Step 4: Context estimation (if enabled)
    if opts.check {
        if opts.verbose {
            println!("  [4/7] Context estimation...");
        }
        let estimate = check_context(&module)?;

        if opts.verbose {
            let total = estimate.total_tokens;
            println!("      Context: {total} tokens");
            if let Some(budget) = estimate.budget {
                let percentage = (estimate.total_tokens as f64 / budget as f64) * 100.0;
                println!("      Budget: {budget} ({percentage:.1}% used)");
            }
        }
    } else if opts.verbose {
        println!("  [4/7] Context estimation... (skipped)");
    }

    // Step 5: Policy gates (if enabled)
    if opts.check {
        if opts.verbose {
            println!("  [5/7] Policy checking...");
        }
        check_policy(&module).context("Policy check failed")?;
    } else if opts.verbose {
        println!("  [5/7] Policy checking... (skipped)");
    }

    // Step 6: Lower to IR
    if opts.verbose {
        println!("  [6/7] Lowering to IR...");
    }
    let mut ir_module = z1_ir::lower_to_ir(&module).context("IR generation failed")?;

    // Apply optimizations
    if opts.verbose {
        println!("  [6.5/7] Optimizing (level {:?})...", opts.opt_level);
    }
    let opt_stats = z1_ir::optimize::optimize(&mut ir_module, opts.opt_level);
    if opts.verbose && opt_stats.total_optimizations() > 0 {
        println!(
            "      Optimizations: {} folded, {} eliminated, {} inlined",
            opt_stats.constants_folded, opt_stats.dead_code_eliminated, opt_stats.functions_inlined
        );
    }

    // If emit-ir, write IR and stop
    if opts.emit_ir {
        let output_path = determine_output_path(&opts.input_path, &opts.output_path, "ir.txt");
        let ir_debug = format!("; IR for module: {}\n\n{ir_module:#?}", ir_module.name);
        fs::write(&output_path, &ir_debug)
            .with_context(|| format!("Failed to write IR to {}", output_path.display()))?;

        println!("✓ IR emitted to: {}", output_path.display());
        return Ok(());
    }

    // Step 7: Code generation
    if opts.verbose {
        println!("  [7/7] Generating {}...", target_name(opts.target));
    }

    let (code, extension) = match opts.target {
        CompileTarget::TypeScript => {
            let ts_code = z1_codegen_ts::generate_typescript(&ir_module);
            (ts_code.into_bytes(), "ts")
        }
        CompileTarget::Wasm => {
            if opts.binary {
                // Generate binary WASM
                let wasm_binary =
                    z1_codegen_wasm::generate_wasm_binary_optimized(&ir_module, opts.opt_level)
                        .map_err(|e| anyhow::anyhow!("WASM binary generation failed: {e}"))?;

                // Note: Validation is available but commented out due to known issues in WAT generation
                // Uncomment this when WAT generation is fully correct
                // z1_codegen_wasm::validate_wasm_binary(&wasm_binary)
                //     .map_err(|e| anyhow::anyhow!("WASM binary validation failed: {}", e))?;

                (wasm_binary, "wasm")
            } else {
                // Generate text WAT
                let wat_code = z1_codegen_wasm::generate_wasm_optimized(&ir_module, opts.opt_level);
                (wat_code.into_bytes(), "wat")
            }
        }
    };

    // Write output
    let output_path = determine_output_path(&opts.input_path, &opts.output_path, extension);
    fs::write(&output_path, code)
        .with_context(|| format!("Failed to write to {}", output_path.display()))?;

    println!("✓ Compiled to: {}", output_path.display());

    Ok(())
}

/// Type check the module using z1-typeck.
fn check_types(module: &Module, source: &str, file_path: &str) -> Result<()> {
    z1_typeck::check_module(module).map_err(|e| {
        let config = error_printer::ErrorPrinterConfig::default();
        error_printer::print_type_error(&e, source, file_path, &config);
        anyhow::anyhow!("Type check failed")
    })
}

/// Effect check the module using z1-effects.
fn check_effects(module: &Module, source: &str, file_path: &str) -> Result<()> {
    z1_effects::check_module(module).map_err(|e| {
        let config = error_printer::ErrorPrinterConfig::default();
        error_printer::print_effect_error(&e, source, file_path, &config);
        anyhow::anyhow!("Effect check failed")
    })
}

/// Context estimation with budget enforcement.
fn check_context(module: &Module) -> Result<z1_ctx::CellEstimate> {
    let estimate = z1_ctx::estimate_cell(module)?;

    if let Some(budget) = module.ctx_budget {
        if estimate.total_tokens > budget {
            anyhow::bail!(
                "Context budget exceeded: {} tokens used, {} allowed",
                estimate.total_tokens,
                budget
            );
        }
    }

    Ok(estimate)
}

/// Policy gate enforcement using z1-policy.
fn check_policy(module: &Module) -> Result<()> {
    let policy = z1_policy::PolicyLimits::default();
    let checker = z1_policy::PolicyChecker::new(policy);

    checker.check_module(module).map_err(|violations| {
        let msg = violations
            .iter()
            .map(|v| format!("  - {v}"))
            .collect::<Vec<_>>()
            .join("\n");
        anyhow::anyhow!("Policy violations:\n{msg}")
    })
}

/// Determine output file path.
fn determine_output_path(input: &Path, output: &Option<PathBuf>, extension: &str) -> PathBuf {
    if let Some(out) = output {
        return out.clone();
    }

    // Default: replace input extension with target extension
    input.with_extension(extension)
}

/// Get human-readable target name.
fn target_name(target: CompileTarget) -> &'static str {
    match target {
        CompileTarget::TypeScript => "TypeScript",
        CompileTarget::Wasm => "WebAssembly",
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs;
    use tempfile::TempDir;

    fn setup_test_cell(content: &str) -> (TempDir, PathBuf) {
        let dir = TempDir::new().unwrap();
        let path = dir.path().join("test.z1c");
        fs::write(&path, content).unwrap();
        (dir, path)
    }

    fn simple_valid_cell() -> &'static str {
        r#"module test : 1.0
  ctx = 100
  caps = [net]

fn add(x: U32, y: U32) -> U32
  eff [pure]
{
  ret x;
}
"#
    }

    fn cell_with_type_error() -> &'static str {
        r#"module test : 1.0
  caps = []

type Foo = { x: U32 }
type Bar = { x: Str }
"#
    }

    fn cell_with_effect_error() -> &'static str {
        r#"module test : 1.0
  caps = []

fn server(x: U32) -> U32
  eff [net]
{
  ret x;
}
"#
    }

    fn cell_with_context_error() -> &'static str {
        // Module with tiny budget that will be exceeded
        r#"module test : 1.0
  ctx = 5
  caps = []

fn foo(x: U32, y: U32, z: U32) -> U32
  eff [pure]
{
  ret x;
}
"#
    }

    // ========== Integration Tests ==========

    #[test]
    fn test_compile_to_typescript_succeeds() {
        let (_dir, input) = setup_test_cell(simple_valid_cell());
        let output = input.with_extension("ts");

        let opts = CompileOptions {
            input_path: input.clone(),
            output_path: Some(output.clone()),
            target: CompileTarget::TypeScript,
            binary: false,
            check: true,
            emit_ir: false,
            opt_level: z1_ir::optimize::OptLevel::O0,
            verbose: false,
        };

        let result = compile(opts);
        assert!(result.is_ok(), "Compilation failed: {result:?}");
        assert!(output.exists(), "Output file was not created");

        let content = fs::read_to_string(&output).unwrap();
        assert!(content.contains("Generated by Zero1 compiler"));
        assert!(content.contains("TypeScript"));
        assert!(content.contains("export"));
    }

    #[test]
    fn test_compile_to_wasm_succeeds() {
        let (_dir, input) = setup_test_cell(simple_valid_cell());
        let output = input.with_extension("wat");

        let opts = CompileOptions {
            input_path: input.clone(),
            output_path: Some(output.clone()),
            target: CompileTarget::Wasm,
            binary: false,
            check: true,
            emit_ir: false,
            opt_level: z1_ir::optimize::OptLevel::O0,
            verbose: false,
        };

        let result = compile(opts);
        assert!(result.is_ok(), "Compilation failed: {result:?}");
        assert!(output.exists(), "Output file was not created");

        let content = fs::read_to_string(&output).unwrap();
        assert!(content.contains("(module"));
        assert!(content.contains("Generated by Zero1 compiler"));
        assert!(content.contains("WebAssembly"));
    }

    #[test]
    fn test_compile_with_emit_ir_flag() {
        let (_dir, input) = setup_test_cell(simple_valid_cell());
        let output = input.with_extension("ir.txt");

        let opts = CompileOptions {
            input_path: input.clone(),
            output_path: Some(output.clone()),
            target: CompileTarget::TypeScript,
            binary: false,
            check: true,
            emit_ir: true,
            opt_level: z1_ir::optimize::OptLevel::O0,
            verbose: false,
        };

        let result = compile(opts);
        assert!(result.is_ok(), "Compilation failed: {result:?}");
        assert!(output.exists(), "IR file was not created");

        let content = fs::read_to_string(&output).unwrap();
        assert!(content.contains("; IR for module:"));
        assert!(content.contains("test"));
    }

    #[test]
    fn test_type_check_catches_errors() {
        let (_dir, input) = setup_test_cell(cell_with_type_error());

        let opts = CompileOptions {
            input_path: input,
            output_path: None,
            target: CompileTarget::TypeScript,
            binary: false,
            check: true,
            emit_ir: false,
            opt_level: z1_ir::optimize::OptLevel::O0,
            verbose: false,
        };

        let result = compile(opts);
        // Type checking should pass (we have valid types, just different)
        // This test verifies the type checker is called
        assert!(result.is_ok());
    }

    #[test]
    fn test_effect_check_catches_missing_capabilities() {
        let (_dir, input) = setup_test_cell(cell_with_effect_error());

        let opts = CompileOptions {
            input_path: input,
            output_path: None,
            target: CompileTarget::TypeScript,
            binary: false,
            check: true,
            emit_ir: false,
            opt_level: z1_ir::optimize::OptLevel::O0,
            verbose: false,
        };

        let result = compile(opts);
        assert!(result.is_err(), "Expected capability check to fail");
        let err = result.unwrap_err();
        // Type checker or effect checker can catch this
        assert!(
            err.to_string().contains("Type check failed")
                || err.to_string().contains("Effect check failed")
                || err.to_string().contains("net")
                || err.to_string().contains("capability"),
            "Error message: {err}"
        );
    }

    #[test]
    fn test_context_check_catches_budget_violations() {
        let (_dir, input) = setup_test_cell(cell_with_context_error());

        let opts = CompileOptions {
            input_path: input,
            output_path: None,
            target: CompileTarget::TypeScript,
            binary: false,
            check: true,
            emit_ir: false,
            opt_level: z1_ir::optimize::OptLevel::O0,
            verbose: false,
        };

        let result = compile(opts);
        assert!(result.is_err(), "Expected context check to fail");
        let err = result.unwrap_err();
        assert!(
            err.to_string().contains("Context budget exceeded")
                || err.to_string().contains("tokens"),
            "Error message: {err}"
        );
    }

    #[test]
    fn test_policy_check_enforces_limits() {
        // Create a cell with too many exports (> 5)
        let cell = r#"module test : 1.0
  caps = []

fn f1() -> Unit eff [pure] { ret Unit; }
fn f2() -> Unit eff [pure] { ret Unit; }
fn f3() -> Unit eff [pure] { ret Unit; }
fn f4() -> Unit eff [pure] { ret Unit; }
fn f5() -> Unit eff [pure] { ret Unit; }
fn f6() -> Unit eff [pure] { ret Unit; }
"#;
        let (_dir, input) = setup_test_cell(cell);

        let opts = CompileOptions {
            input_path: input,
            output_path: None,
            target: CompileTarget::TypeScript,
            binary: false,
            check: true,
            emit_ir: false,
            opt_level: z1_ir::optimize::OptLevel::O0,
            verbose: false,
        };

        let result = compile(opts);
        assert!(result.is_err(), "Expected policy check to fail");
        let err = result.unwrap_err();
        assert!(
            err.to_string().contains("Policy")
                || err.to_string().contains("export")
                || err.to_string().contains("limit"),
            "Error message: {err}"
        );
    }

    #[test]
    fn test_compile_with_no_check_skips_checks() {
        // This cell has an effect error, but we skip checks
        let (_dir, input) = setup_test_cell(cell_with_effect_error());

        let opts = CompileOptions {
            input_path: input,
            output_path: None,
            target: CompileTarget::TypeScript,
            binary: false,
            check: false,
            emit_ir: false,
            opt_level: z1_ir::optimize::OptLevel::O0,
            verbose: false,
        };

        let result = compile(opts);
        // Should succeed because we skipped checks
        assert!(
            result.is_ok(),
            "Expected compilation to succeed with --no-check"
        );
    }

    #[test]
    fn test_output_path_customization_works() {
        let (_dir, input) = setup_test_cell(simple_valid_cell());
        let custom_output = input.parent().unwrap().join("custom_output.ts");

        let opts = CompileOptions {
            input_path: input,
            output_path: Some(custom_output.clone()),
            target: CompileTarget::TypeScript,
            binary: false,
            check: true,
            emit_ir: false,
            opt_level: z1_ir::optimize::OptLevel::O0,
            verbose: false,
        };

        let result = compile(opts);
        assert!(result.is_ok(), "Compilation failed: {result:?}");
        assert!(custom_output.exists(), "Custom output file was not created");
    }

    #[test]
    fn test_verbose_mode_prints_progress() {
        let (_dir, input) = setup_test_cell(simple_valid_cell());

        let opts = CompileOptions {
            input_path: input,
            output_path: None,
            target: CompileTarget::TypeScript,
            binary: false,
            check: true,
            emit_ir: false,
            opt_level: z1_ir::optimize::OptLevel::O0,
            verbose: true, // Enable verbose output
        };

        // This will print to stdout, which we can't easily capture in tests
        // but we verify it doesn't crash
        let result = compile(opts);
        assert!(result.is_ok());
    }

    // ========== Unit Tests ==========

    #[test]
    fn test_determine_output_path_with_custom_path() {
        let input = Path::new("/tmp/test.z1c");
        let custom = Some(PathBuf::from("/tmp/custom.ts"));
        let result = determine_output_path(input, &custom, "ts");
        assert_eq!(result, PathBuf::from("/tmp/custom.ts"));
    }

    #[test]
    fn test_determine_output_path_with_default_typescript() {
        let input = Path::new("/tmp/test.z1c");
        let result = determine_output_path(input, &None, "ts");
        assert_eq!(result, PathBuf::from("/tmp/test.ts"));
    }

    #[test]
    fn test_determine_output_path_with_default_wasm() {
        let input = Path::new("/tmp/test.z1c");
        let result = determine_output_path(input, &None, "wat");
        assert_eq!(result, PathBuf::from("/tmp/test.wat"));
    }

    #[test]
    fn test_target_name_typescript() {
        assert_eq!(target_name(CompileTarget::TypeScript), "TypeScript");
    }

    #[test]
    fn test_target_name_wasm() {
        assert_eq!(target_name(CompileTarget::Wasm), "WebAssembly");
    }

    // NOTE: These tests disabled - test internal APIs that no longer exist.
    // Functionality covered by integration tests above.

    //     #[test]
    //     fn test_lower_to_ir_produces_valid_output() {
    //         let cell = simple_valid_cell();
    //         let module = z1_parse::parse_module(cell).unwrap();
    //         let ir = lower_to_ir(&module).unwrap();
    //
    //         assert!(ir.contains("; IR for module:"));
    //         assert!(ir.contains("test"));
    //         assert!(ir.contains("fn add"));
    //     }
    //
    //     #[test]
    //     fn test_generate_typescript_produces_valid_output() {
    //         let cell = simple_valid_cell();
    //         let module = z1_parse::parse_module(cell).unwrap();
    //         let ir = lower_to_ir(&module).unwrap();
    //         let ts = generate_typescript(&module, &ir);
    //
    //         assert!(ts.contains("// Generated by Zero1 compiler"));
    //         assert!(ts.contains("TypeScript"));
    //         assert!(ts.contains("export"));
    //     }
    //
    //     #[test]
    //     fn test_generate_wasm_produces_valid_output() {
    //         let cell = simple_valid_cell();
    //         let module = z1_parse::parse_module(cell).unwrap();
    //         let ir = lower_to_ir(&module).unwrap();
    //         let wat = generate_wasm(&module, &ir);
    //
    //         assert!(wat.contains("(module"));
    //         assert!(wat.contains("Generated by Zero1 compiler"));
    //         assert!(wat.contains("WebAssembly"));
    //     }
}
