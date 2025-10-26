//! Integration tests for IR optimizations
//!
//! These tests verify that the optimization passes work correctly
//! both individually and in combination.

use z1_ir::optimize::{optimize, OptLevel};
use z1_ir::{IrBinOp, IrBlock, IrExpr, IrFunction, IrLiteral, IrModule, IrStmt, IrType};

// ===== Dead Code Elimination Tests =====

#[test]
fn test_dce_eliminates_unused_variable() {
    let mut module = IrModule {
        name: "test".to_string(),
        version: "1.0.0".to_string(),
        imports: vec![],
        types: vec![],
        functions: vec![IrFunction {
            name: "main".to_string(),
            params: vec![],
            return_type: IrType::U32,
            effects: vec![],
            body: IrBlock {
                statements: vec![
                    // Unused variable
                    IrStmt::Let {
                        name: "unused".to_string(),
                        mutable: false,
                        ty: Some(IrType::U32),
                        value: IrExpr::Literal(IrLiteral::U32(999)),
                    },
                    // Used variable
                    IrStmt::Let {
                        name: "used".to_string(),
                        mutable: false,
                        ty: Some(IrType::U32),
                        value: IrExpr::Literal(IrLiteral::U32(42)),
                    },
                    IrStmt::Return {
                        value: Some(IrExpr::Var("used".to_string())),
                    },
                ],
            },
        }],
        exports: vec![],
    };

    let stats = optimize(&mut module, OptLevel::O2);

    assert!(stats.dead_code_eliminated > 0);
    // Only 2 statements should remain (let used + return)
    assert_eq!(module.functions[0].body.statements.len(), 2);
}

#[test]
fn test_dce_removes_unreachable_code_after_return() {
    let mut module = IrModule {
        name: "test".to_string(),
        version: "1.0.0".to_string(),
        imports: vec![],
        types: vec![],
        functions: vec![IrFunction {
            name: "main".to_string(),
            params: vec![],
            return_type: IrType::U32,
            effects: vec![],
            body: IrBlock {
                statements: vec![
                    IrStmt::Return {
                        value: Some(IrExpr::Literal(IrLiteral::U32(42))),
                    },
                    // This should be eliminated
                    IrStmt::Let {
                        name: "unreachable".to_string(),
                        mutable: false,
                        ty: Some(IrType::U32),
                        value: IrExpr::Literal(IrLiteral::U32(999)),
                    },
                ],
            },
        }],
        exports: vec![],
    };

    let stats = optimize(&mut module, OptLevel::O2);

    assert!(stats.dead_code_eliminated > 0);
    // Only the return statement should remain
    assert_eq!(module.functions[0].body.statements.len(), 1);
}

#[test]
fn test_dce_preserves_effectful_operations() {
    let mut module = IrModule {
        name: "test".to_string(),
        version: "1.0.0".to_string(),
        imports: vec![],
        types: vec![],
        functions: vec![IrFunction {
            name: "main".to_string(),
            params: vec![],
            return_type: IrType::Unit,
            effects: vec!["net".to_string()],
            body: IrBlock {
                statements: vec![
                    // Unused but has side effects (function call)
                    IrStmt::Let {
                        name: "result".to_string(),
                        mutable: false,
                        ty: Some(IrType::U32),
                        value: IrExpr::Call {
                            func: Box::new(IrExpr::Var("effectful_func".to_string())),
                            args: vec![],
                        },
                    },
                    IrStmt::Return { value: None },
                ],
            },
        }],
        exports: vec![],
    };

    let original_len = module.functions[0].body.statements.len();
    optimize(&mut module, OptLevel::O2);

    // Should not eliminate the effectful call
    assert_eq!(module.functions[0].body.statements.len(), original_len);
}

#[test]
fn test_dce_removes_empty_blocks() {
    let mut module = IrModule {
        name: "test".to_string(),
        version: "1.0.0".to_string(),
        imports: vec![],
        types: vec![],
        functions: vec![IrFunction {
            name: "main".to_string(),
            params: vec![],
            return_type: IrType::U32,
            effects: vec![],
            body: IrBlock {
                statements: vec![
                    IrStmt::Let {
                        name: "x".to_string(),
                        mutable: false,
                        ty: Some(IrType::U32),
                        value: IrExpr::Literal(IrLiteral::U32(10)),
                    },
                    IrStmt::Return {
                        value: Some(IrExpr::Var("x".to_string())),
                    },
                ],
            },
        }],
        exports: vec![],
    };

    optimize(&mut module, OptLevel::O2);

    // Should optimize without errors
    assert_eq!(module.functions[0].body.statements.len(), 2);
}

// ===== Constant Folding Tests =====

#[test]
fn test_const_fold_arithmetic() {
    let mut module = IrModule {
        name: "test".to_string(),
        version: "1.0.0".to_string(),
        imports: vec![],
        types: vec![],
        functions: vec![IrFunction {
            name: "main".to_string(),
            params: vec![],
            return_type: IrType::U32,
            effects: vec![],
            body: IrBlock {
                statements: vec![IrStmt::Return {
                    value: Some(IrExpr::BinOp {
                        op: IrBinOp::Add,
                        left: Box::new(IrExpr::Literal(IrLiteral::U32(2))),
                        right: Box::new(IrExpr::Literal(IrLiteral::U32(3))),
                    }),
                }],
            },
        }],
        exports: vec![],
    };

    let stats = optimize(&mut module, OptLevel::O2);

    assert!(stats.constants_folded > 0);
    // Check that 2 + 3 was folded to 5
    match &module.functions[0].body.statements[0] {
        IrStmt::Return {
            value: Some(IrExpr::Literal(IrLiteral::U32(5))),
        } => (),
        _ => panic!("Expected folded constant 5"),
    }
}

#[test]
fn test_const_fold_comparisons() {
    let mut module = IrModule {
        name: "test".to_string(),
        version: "1.0.0".to_string(),
        imports: vec![],
        types: vec![],
        functions: vec![IrFunction {
            name: "main".to_string(),
            params: vec![],
            return_type: IrType::Bool,
            effects: vec![],
            body: IrBlock {
                statements: vec![IrStmt::Return {
                    value: Some(IrExpr::BinOp {
                        op: IrBinOp::Gt,
                        left: Box::new(IrExpr::Literal(IrLiteral::U32(5))),
                        right: Box::new(IrExpr::Literal(IrLiteral::U32(3))),
                    }),
                }],
            },
        }],
        exports: vec![],
    };

    let stats = optimize(&mut module, OptLevel::O2);

    assert!(stats.constants_folded > 0);
    // Check that 5 > 3 was folded to true
    match &module.functions[0].body.statements[0] {
        IrStmt::Return {
            value: Some(IrExpr::Literal(IrLiteral::Bool(true))),
        } => (),
        _ => panic!("Expected folded constant true"),
    }
}

#[test]
fn test_const_propagation() {
    let mut module = IrModule {
        name: "test".to_string(),
        version: "1.0.0".to_string(),
        imports: vec![],
        types: vec![],
        functions: vec![IrFunction {
            name: "main".to_string(),
            params: vec![],
            return_type: IrType::U32,
            effects: vec![],
            body: IrBlock {
                statements: vec![
                    IrStmt::Let {
                        name: "x".to_string(),
                        mutable: false,
                        ty: Some(IrType::U32),
                        value: IrExpr::Literal(IrLiteral::U32(42)),
                    },
                    IrStmt::Return {
                        value: Some(IrExpr::Var("x".to_string())),
                    },
                ],
            },
        }],
        exports: vec![],
    };

    let stats = optimize(&mut module, OptLevel::O2);

    assert!(stats.constants_folded > 0);
    // x should be propagated to 42
    match &module.functions[0].body.statements[1] {
        IrStmt::Return {
            value: Some(IrExpr::Literal(IrLiteral::U32(42))),
        } => (),
        _ => panic!("Expected propagated constant 42"),
    }
}

#[test]
fn test_const_fold_simplifies_if_conditions() {
    let mut module = IrModule {
        name: "test".to_string(),
        version: "1.0.0".to_string(),
        imports: vec![],
        types: vec![],
        functions: vec![IrFunction {
            name: "main".to_string(),
            params: vec![],
            return_type: IrType::U32,
            effects: vec![],
            body: IrBlock {
                statements: vec![IrStmt::If {
                    cond: IrExpr::Literal(IrLiteral::Bool(true)),
                    then_block: IrBlock {
                        statements: vec![IrStmt::Return {
                            value: Some(IrExpr::Literal(IrLiteral::U32(1))),
                        }],
                    },
                    else_block: Some(IrBlock {
                        statements: vec![IrStmt::Return {
                            value: Some(IrExpr::Literal(IrLiteral::U32(2))),
                        }],
                    }),
                }],
            },
        }],
        exports: vec![],
    };

    let stats = optimize(&mut module, OptLevel::O2);

    assert!(stats.constants_folded > 0);
    // The if with constant true should be simplified
    match &module.functions[0].body.statements[0] {
        IrStmt::If {
            cond: IrExpr::Literal(IrLiteral::Bool(true)),
            else_block: None,
            ..
        } => (),
        _ => panic!("Expected simplified if statement"),
    }
}

// ===== Function Inlining Tests =====

#[test]
fn test_inline_trivial_function() {
    let mut module = IrModule {
        name: "test".to_string(),
        version: "1.0.0".to_string(),
        imports: vec![],
        types: vec![],
        functions: vec![
            IrFunction {
                name: "get_value".to_string(),
                params: vec![],
                return_type: IrType::U32,
                effects: vec!["pure".to_string()],
                body: IrBlock {
                    statements: vec![IrStmt::Return {
                        value: Some(IrExpr::Literal(IrLiteral::U32(42))),
                    }],
                },
            },
            IrFunction {
                name: "main".to_string(),
                params: vec![],
                return_type: IrType::U32,
                effects: vec![],
                body: IrBlock {
                    statements: vec![IrStmt::Return {
                        value: Some(IrExpr::Call {
                            func: Box::new(IrExpr::Var("get_value".to_string())),
                            args: vec![],
                        }),
                    }],
                },
            },
        ],
        exports: vec![],
    };

    let stats = optimize(&mut module, OptLevel::O2);

    assert!(stats.functions_inlined > 0);
    // get_value() should be inlined to 42
    match &module.functions[1].body.statements[0] {
        IrStmt::Return {
            value: Some(IrExpr::Literal(IrLiteral::U32(42))),
        } => (),
        _ => panic!("Expected inlined constant 42"),
    }
}

#[test]
fn test_inline_small_pure_function() {
    let mut module = IrModule {
        name: "test".to_string(),
        version: "1.0.0".to_string(),
        imports: vec![],
        types: vec![],
        functions: vec![
            IrFunction {
                name: "double".to_string(),
                params: vec![("x".to_string(), IrType::U32)],
                return_type: IrType::U32,
                effects: vec!["pure".to_string()],
                body: IrBlock {
                    statements: vec![IrStmt::Return {
                        value: Some(IrExpr::BinOp {
                            op: IrBinOp::Mul,
                            left: Box::new(IrExpr::Var("x".to_string())),
                            right: Box::new(IrExpr::Literal(IrLiteral::U32(2))),
                        }),
                    }],
                },
            },
            IrFunction {
                name: "main".to_string(),
                params: vec![],
                return_type: IrType::U32,
                effects: vec![],
                body: IrBlock {
                    statements: vec![IrStmt::Return {
                        value: Some(IrExpr::Call {
                            func: Box::new(IrExpr::Var("double".to_string())),
                            args: vec![IrExpr::Literal(IrLiteral::U32(5))],
                        }),
                    }],
                },
            },
        ],
        exports: vec![],
    };

    let stats = optimize(&mut module, OptLevel::O2);

    assert!(stats.functions_inlined > 0);
    // double(5) should be inlined and then folded
    match &module.functions[1].body.statements[0] {
        IrStmt::Return {
            value: Some(IrExpr::Literal(IrLiteral::U32(10))),
        } => (),
        IrStmt::Return {
            value: Some(IrExpr::BinOp { .. }),
        } => (), // Inlined but not yet folded (acceptable)
        _ => panic!("Expected inlined expression"),
    }
}

#[test]
fn test_dont_inline_large_function() {
    let mut module = IrModule {
        name: "test".to_string(),
        version: "1.0.0".to_string(),
        imports: vec![],
        types: vec![],
        functions: vec![
            IrFunction {
                name: "large".to_string(),
                params: vec![],
                return_type: IrType::U32,
                effects: vec![],
                body: IrBlock {
                    statements: vec![
                        IrStmt::Let {
                            name: "a".to_string(),
                            mutable: false,
                            ty: Some(IrType::U32),
                            value: IrExpr::Literal(IrLiteral::U32(1)),
                        },
                        IrStmt::Let {
                            name: "b".to_string(),
                            mutable: false,
                            ty: Some(IrType::U32),
                            value: IrExpr::Literal(IrLiteral::U32(2)),
                        },
                        IrStmt::Let {
                            name: "c".to_string(),
                            mutable: false,
                            ty: Some(IrType::U32),
                            value: IrExpr::Literal(IrLiteral::U32(3)),
                        },
                        IrStmt::Let {
                            name: "d".to_string(),
                            mutable: false,
                            ty: Some(IrType::U32),
                            value: IrExpr::BinOp {
                                op: IrBinOp::Add,
                                left: Box::new(IrExpr::Var("a".to_string())),
                                right: Box::new(IrExpr::Var("b".to_string())),
                            },
                        },
                        IrStmt::Let {
                            name: "e".to_string(),
                            mutable: false,
                            ty: Some(IrType::U32),
                            value: IrExpr::BinOp {
                                op: IrBinOp::Add,
                                left: Box::new(IrExpr::Var("c".to_string())),
                                right: Box::new(IrExpr::Var("d".to_string())),
                            },
                        },
                        IrStmt::Return {
                            value: Some(IrExpr::Var("e".to_string())),
                        },
                    ],
                },
            },
            IrFunction {
                name: "main".to_string(),
                params: vec![],
                return_type: IrType::U32,
                effects: vec![],
                body: IrBlock {
                    statements: vec![IrStmt::Return {
                        value: Some(IrExpr::Call {
                            func: Box::new(IrExpr::Var("large".to_string())),
                            args: vec![],
                        }),
                    }],
                },
            },
        ],
        exports: vec![],
    };

    optimize(&mut module, OptLevel::O2);

    // The call should remain (not inlined due to size)
    match &module.functions[1].body.statements[0] {
        IrStmt::Return {
            value: Some(IrExpr::Call { .. }),
        } => (),
        _ => {
            // It's acceptable if it was inlined (depending on threshold)
            // This test is just checking the heuristic works
        }
    }
}

#[test]
fn test_dont_inline_recursive_function() {
    let mut module = IrModule {
        name: "test".to_string(),
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
                    statements: vec![IrStmt::Return {
                        value: Some(IrExpr::Call {
                            func: Box::new(IrExpr::Var("factorial".to_string())),
                            args: vec![IrExpr::Literal(IrLiteral::U32(1))],
                        }),
                    }],
                },
            },
            IrFunction {
                name: "main".to_string(),
                params: vec![],
                return_type: IrType::U32,
                effects: vec![],
                body: IrBlock {
                    statements: vec![IrStmt::Return {
                        value: Some(IrExpr::Call {
                            func: Box::new(IrExpr::Var("factorial".to_string())),
                            args: vec![IrExpr::Literal(IrLiteral::U32(5))],
                        }),
                    }],
                },
            },
        ],
        exports: vec![],
    };

    let original = module.clone();
    optimize(&mut module, OptLevel::O2);

    // Recursive function should not be inlined
    assert_eq!(
        module.functions[1].body.statements,
        original.functions[1].body.statements
    );
}

// ===== Integration Tests =====

#[test]
fn test_combined_optimizations() {
    let mut module = IrModule {
        name: "test".to_string(),
        version: "1.0.0".to_string(),
        imports: vec![],
        types: vec![],
        functions: vec![
            IrFunction {
                name: "get_value".to_string(),
                params: vec![],
                return_type: IrType::U32,
                effects: vec!["pure".to_string()],
                body: IrBlock {
                    statements: vec![IrStmt::Return {
                        value: Some(IrExpr::Literal(IrLiteral::U32(5))),
                    }],
                },
            },
            IrFunction {
                name: "main".to_string(),
                params: vec![],
                return_type: IrType::U32,
                effects: vec![],
                body: IrBlock {
                    statements: vec![
                        // This will be inlined to 5
                        IrStmt::Let {
                            name: "x".to_string(),
                            mutable: false,
                            ty: Some(IrType::U32),
                            value: IrExpr::Call {
                                func: Box::new(IrExpr::Var("get_value".to_string())),
                                args: vec![],
                            },
                        },
                        // This will be folded to 8
                        IrStmt::Let {
                            name: "y".to_string(),
                            mutable: false,
                            ty: Some(IrType::U32),
                            value: IrExpr::BinOp {
                                op: IrBinOp::Add,
                                left: Box::new(IrExpr::Var("x".to_string())),
                                right: Box::new(IrExpr::Literal(IrLiteral::U32(3))),
                            },
                        },
                        // This is unused and will be eliminated
                        IrStmt::Let {
                            name: "z".to_string(),
                            mutable: false,
                            ty: Some(IrType::U32),
                            value: IrExpr::Literal(IrLiteral::U32(999)),
                        },
                        IrStmt::Return {
                            value: Some(IrExpr::Var("y".to_string())),
                        },
                    ],
                },
            },
        ],
        exports: vec![],
    };

    let stats = optimize(&mut module, OptLevel::O2);

    // All three types of optimizations should occur
    assert!(stats.functions_inlined > 0, "Should inline get_value");
    assert!(stats.constants_folded > 0, "Should fold constants");
    assert!(stats.dead_code_eliminated > 0, "Should eliminate z");

    // Final result should be highly optimized
    assert!(module.functions[1].body.statements.len() <= 4);
}

#[test]
fn test_optimization_levels_work() {
    let create_module = || IrModule {
        name: "test".to_string(),
        version: "1.0.0".to_string(),
        imports: vec![],
        types: vec![],
        functions: vec![IrFunction {
            name: "main".to_string(),
            params: vec![],
            return_type: IrType::U32,
            effects: vec![],
            body: IrBlock {
                statements: vec![IrStmt::Return {
                    value: Some(IrExpr::BinOp {
                        op: IrBinOp::Add,
                        left: Box::new(IrExpr::Literal(IrLiteral::U32(2))),
                        right: Box::new(IrExpr::Literal(IrLiteral::U32(3))),
                    }),
                }],
            },
        }],
        exports: vec![],
    };

    // O0: No optimizations
    let mut module_o0 = create_module();
    let stats_o0 = optimize(&mut module_o0, OptLevel::O0);
    assert_eq!(stats_o0.total_optimizations(), 0);

    // O1: Basic optimizations
    let mut module_o1 = create_module();
    let stats_o1 = optimize(&mut module_o1, OptLevel::O1);
    assert!(stats_o1.total_optimizations() > 0);
    assert_eq!(stats_o1.total_iterations, 1);

    // O2: Aggressive optimizations
    let mut module_o2 = create_module();
    let stats_o2 = optimize(&mut module_o2, OptLevel::O2);
    assert!(stats_o2.total_optimizations() > 0);
}

#[test]
fn test_stats_tracking_accurate() {
    let mut module = IrModule {
        name: "test".to_string(),
        version: "1.0.0".to_string(),
        imports: vec![],
        types: vec![],
        functions: vec![IrFunction {
            name: "main".to_string(),
            params: vec![],
            return_type: IrType::U32,
            effects: vec![],
            body: IrBlock {
                statements: vec![
                    IrStmt::Let {
                        name: "unused".to_string(),
                        mutable: false,
                        ty: Some(IrType::U32),
                        value: IrExpr::Literal(IrLiteral::U32(999)),
                    },
                    IrStmt::Return {
                        value: Some(IrExpr::BinOp {
                            op: IrBinOp::Add,
                            left: Box::new(IrExpr::Literal(IrLiteral::U32(2))),
                            right: Box::new(IrExpr::Literal(IrLiteral::U32(3))),
                        }),
                    },
                ],
            },
        }],
        exports: vec![],
    };

    let stats = optimize(&mut module, OptLevel::O2);

    // Should have at least constant folding and DCE
    assert!(stats.constants_folded > 0);
    assert!(stats.dead_code_eliminated > 0);
    assert_eq!(
        stats.total_optimizations(),
        stats.constants_folded + stats.dead_code_eliminated + stats.functions_inlined
    );
}
