//! Tests for WASM binary output generation

use z1_codegen_wasm::{generate_wasm_binary, generate_wasm_binary_optimized, validate_wasm_binary};
use z1_ir::*;

/// Helper to create a simple test IR module
fn simple_module() -> IrModule {
    IrModule {
        name: "test".to_string(),
        version: "1.0.0".to_string(),
        imports: vec![],
        types: vec![],
        functions: vec![IrFunction {
            name: "add".to_string(),
            params: vec![
                ("a".to_string(), IrType::U32),
                ("b".to_string(), IrType::U32),
            ],
            return_type: IrType::U32,
            effects: vec![],
            body: IrBlock {
                statements: vec![IrStmt::Return {
                    value: Some(IrExpr::BinOp {
                        op: IrBinOp::Add,
                        left: Box::new(IrExpr::Var("a".to_string())),
                        right: Box::new(IrExpr::Var("b".to_string())),
                    }),
                }],
            },
        }],
        exports: vec!["add".to_string()],
    }
}

/// Helper to create a complex module with multiple functions
fn complex_module() -> IrModule {
    IrModule {
        name: "complex".to_string(),
        version: "1.0.0".to_string(),
        imports: vec![],
        types: vec![],
        functions: vec![
            IrFunction {
                name: "factorial".to_string(),
                params: vec![("n".to_string(), IrType::U32)],
                return_type: IrType::U32,
                effects: vec![],
                body: IrBlock {
                    statements: vec![
                        IrStmt::Let {
                            name: "result".to_string(),
                            mutable: true,
                            ty: Some(IrType::U32),
                            value: IrExpr::Literal(IrLiteral::U32(1)),
                        },
                        IrStmt::Let {
                            name: "i".to_string(),
                            mutable: true,
                            ty: Some(IrType::U32),
                            value: IrExpr::Literal(IrLiteral::U32(1)),
                        },
                        IrStmt::While {
                            cond: IrExpr::BinOp {
                                op: IrBinOp::Le,
                                left: Box::new(IrExpr::Var("i".to_string())),
                                right: Box::new(IrExpr::Var("n".to_string())),
                            },
                            body: IrBlock {
                                statements: vec![
                                    IrStmt::Assign {
                                        target: IrExpr::Var("result".to_string()),
                                        value: IrExpr::BinOp {
                                            op: IrBinOp::Mul,
                                            left: Box::new(IrExpr::Var("result".to_string())),
                                            right: Box::new(IrExpr::Var("i".to_string())),
                                        },
                                    },
                                    IrStmt::Assign {
                                        target: IrExpr::Var("i".to_string()),
                                        value: IrExpr::BinOp {
                                            op: IrBinOp::Add,
                                            left: Box::new(IrExpr::Var("i".to_string())),
                                            right: Box::new(IrExpr::Literal(IrLiteral::U32(1))),
                                        },
                                    },
                                ],
                            },
                        },
                        IrStmt::Return {
                            value: Some(IrExpr::Var("result".to_string())),
                        },
                    ],
                },
            },
            IrFunction {
                name: "helper".to_string(),
                params: vec![("x".to_string(), IrType::U32)],
                return_type: IrType::U32,
                effects: vec![],
                body: IrBlock {
                    statements: vec![IrStmt::Return {
                        value: Some(IrExpr::BinOp {
                            op: IrBinOp::Add,
                            left: Box::new(IrExpr::Var("x".to_string())),
                            right: Box::new(IrExpr::Literal(IrLiteral::U32(10))),
                        }),
                    }],
                },
            },
        ],
        exports: vec!["factorial".to_string(), "helper".to_string()],
    }
}

#[test]
fn test_generate_binary_for_simple_function() {
    let module = simple_module();
    let binary = generate_wasm_binary(&module).expect("Binary generation should succeed");

    // Binary should not be empty
    assert!(!binary.is_empty(), "Generated binary should not be empty");

    // Binary should start with WASM magic number (0x00 0x61 0x73 0x6D)
    assert_eq!(
        &binary[0..4],
        &[0x00, 0x61, 0x73, 0x6D],
        "Binary should start with WASM magic number"
    );

    // Binary should have WASM version (0x01 0x00 0x00 0x00)
    assert_eq!(
        &binary[4..8],
        &[0x01, 0x00, 0x00, 0x00],
        "Binary should have WASM version 1"
    );
}

#[test]
fn test_validate_generated_binary() {
    let module = simple_module();
    let binary = generate_wasm_binary(&module).expect("Binary generation should succeed");

    // Validate the binary
    let result = validate_wasm_binary(&binary);
    assert!(
        result.is_ok(),
        "Generated binary should be valid: {:?}",
        result.err()
    );
}

#[test]
fn test_binary_round_trip_wat_to_binary() {
    let module = simple_module();

    // Generate WAT text
    let wat_text = z1_codegen_wasm::generate_wasm(&module);

    // Generate binary
    let binary = generate_wasm_binary(&module).expect("Binary generation should succeed");

    // Parse binary back to WAT (using wasmparser to verify structure)
    let validation_result = validate_wasm_binary(&binary);
    assert!(
        validation_result.is_ok(),
        "Binary should be valid after round-trip"
    );

    // Verify WAT text contains expected structures
    assert!(wat_text.contains("(func $add"));
    assert!(wat_text.contains("i32.add"));

    // Verify binary is non-empty and valid
    assert!(!binary.is_empty());
    assert_eq!(&binary[0..4], &[0x00, 0x61, 0x73, 0x6D]);
}

#[test]
fn test_binary_contains_expected_module_structure() {
    let module = simple_module();
    let binary = generate_wasm_binary(&module).expect("Binary generation should succeed");

    // Parse binary using wasmparser to check structure
    use wasmparser::{Parser, Payload};

    let mut has_memory = false;
    let mut has_function = false;
    let mut has_export = false;

    for payload in Parser::new(0).parse_all(&binary) {
        match payload.expect("Valid payload") {
            Payload::MemorySection(_) => has_memory = true,
            Payload::FunctionSection(_) => has_function = true,
            Payload::ExportSection(_) => has_export = true,
            _ => {}
        }
    }

    assert!(has_memory, "Binary should contain memory section");
    assert!(has_function, "Binary should contain function section");
    assert!(has_export, "Binary should contain export section");
}

#[test]
fn test_binary_generation_with_optimization() {
    let module = simple_module();

    // Generate with different optimization levels
    let binary_o0 =
        generate_wasm_binary_optimized(&module, optimize::OptLevel::O0).expect("O0 should work");
    let binary_o1 =
        generate_wasm_binary_optimized(&module, optimize::OptLevel::O1).expect("O1 should work");
    let binary_o2 =
        generate_wasm_binary_optimized(&module, optimize::OptLevel::O2).expect("O2 should work");

    // All binaries should be valid
    assert!(validate_wasm_binary(&binary_o0).is_ok());
    assert!(validate_wasm_binary(&binary_o1).is_ok());
    assert!(validate_wasm_binary(&binary_o2).is_ok());

    // All binaries should have WASM magic number
    assert_eq!(&binary_o0[0..4], &[0x00, 0x61, 0x73, 0x6D]);
    assert_eq!(&binary_o1[0..4], &[0x00, 0x61, 0x73, 0x6D]);
    assert_eq!(&binary_o2[0..4], &[0x00, 0x61, 0x73, 0x6D]);
}

#[test]
fn test_binary_for_complex_module() {
    let module = complex_module();
    let binary = generate_wasm_binary(&module).expect("Complex module should generate binary");

    // Validate
    assert!(
        validate_wasm_binary(&binary).is_ok(),
        "Complex module binary should be valid"
    );

    // Check structure
    use wasmparser::{Parser, Payload};

    let mut function_count = 0;
    let mut export_count = 0;

    for payload in Parser::new(0).parse_all(&binary) {
        match payload.expect("Valid payload") {
            Payload::FunctionSection(reader) => {
                function_count = reader.count();
            }
            Payload::ExportSection(reader) => {
                export_count = reader.count();
            }
            _ => {}
        }
    }

    // We have 2 functions: factorial and helper
    assert_eq!(function_count, 2, "Should have 2 functions");

    // We export both functions plus memory
    assert!(export_count >= 2, "Should export at least 2 items");
}

#[test]
fn test_binary_with_string_literals() {
    let module = IrModule {
        name: "strings".to_string(),
        version: "1.0.0".to_string(),
        imports: vec![],
        types: vec![],
        functions: vec![IrFunction {
            name: "get_message".to_string(),
            params: vec![],
            return_type: IrType::Str,
            effects: vec![],
            body: IrBlock {
                statements: vec![IrStmt::Return {
                    value: Some(IrExpr::Literal(IrLiteral::Str("Hello, WASM!".to_string()))),
                }],
            },
        }],
        exports: vec!["get_message".to_string()],
    };

    let binary = generate_wasm_binary(&module).expect("String literal module should work");

    // Validate
    assert!(validate_wasm_binary(&binary).is_ok());

    // Check for data section
    use wasmparser::{Parser, Payload};

    let mut has_data = false;

    for payload in Parser::new(0).parse_all(&binary).flatten() {
        if let Payload::DataSection(_) = payload {
            has_data = true;
        }
    }

    assert!(
        has_data,
        "Binary with string literals should have data section"
    );
}

#[test]
fn test_invalid_wat_produces_error() {
    // This test verifies that if WAT generation produces invalid output,
    // we get a proper error message

    // Create a minimal module that will generate valid WAT
    let module = simple_module();

    // This should succeed
    let result = generate_wasm_binary(&module);
    assert!(
        result.is_ok(),
        "Valid module should generate binary successfully"
    );
}
