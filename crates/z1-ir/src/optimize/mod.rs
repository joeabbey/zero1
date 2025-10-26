//! IR optimization passes
//!
//! This module orchestrates various optimization passes on the IR:
//! - Dead code elimination (DCE)
//! - Constant folding and propagation
//! - Function inlining
//!
//! Optimizations can be run at different levels (O0, O1, O2).

pub mod const_fold;
pub mod dce;
pub mod inline;

use crate::IrModule;

/// Optimization level
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum OptLevel {
    /// No optimizations (for debugging)
    O0,
    /// Basic optimizations (constant folding, simple DCE)
    #[default]
    O1,
    /// Aggressive optimizations (all passes, multiple iterations)
    O2,
}

impl std::str::FromStr for OptLevel {
    type Err = String;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s.to_lowercase().as_str() {
            "0" | "o0" => Ok(OptLevel::O0),
            "1" | "o1" => Ok(OptLevel::O1),
            "2" | "o2" => Ok(OptLevel::O2),
            _ => Err(format!("Invalid optimization level: {s}")),
        }
    }
}

/// Statistics about applied optimizations
#[derive(Debug, Clone, Default)]
pub struct OptStats {
    pub constants_folded: usize,
    pub dead_code_eliminated: usize,
    pub functions_inlined: usize,
    pub total_iterations: usize,
}

impl OptStats {
    /// Returns total number of optimizations applied
    pub fn total_optimizations(&self) -> usize {
        self.constants_folded + self.dead_code_eliminated + self.functions_inlined
    }
}

/// Optimizes an IR module at the specified optimization level
pub fn optimize(module: &mut IrModule, level: OptLevel) -> OptStats {
    match level {
        OptLevel::O0 => OptStats::default(), // No optimizations
        OptLevel::O1 => optimize_basic(module),
        OptLevel::O2 => optimize_aggressive(module),
    }
}

/// Applies basic optimizations (O1 level)
fn optimize_basic(module: &mut IrModule) -> OptStats {
    let mut stats = OptStats::default();

    // Single iteration of each pass
    stats.constants_folded += const_fold::fold_constants(module);
    stats.dead_code_eliminated += dce::eliminate_dead_code(module);

    stats.total_iterations = 1;
    stats
}

/// Applies aggressive optimizations (O2 level)
fn optimize_aggressive(module: &mut IrModule) -> OptStats {
    let mut stats = OptStats::default();

    // Iterate until fixpoint (no more optimizations applied)
    let max_iterations = 10;
    for iteration in 0..max_iterations {
        let before_count = stats.total_optimizations();

        // Run optimization passes in order
        // 1. Constant folding - evaluates constant expressions
        stats.constants_folded += const_fold::fold_constants(module);

        // 2. Dead code elimination - removes unused code
        stats.dead_code_eliminated += dce::eliminate_dead_code(module);

        // 3. Function inlining - replaces calls with function bodies
        stats.functions_inlined += inline::inline_functions(module);

        // 4. Constant folding again - new opportunities from inlining
        stats.constants_folded += const_fold::fold_constants(module);

        // 5. Dead code elimination again - cleanup after inlining
        stats.dead_code_eliminated += dce::eliminate_dead_code(module);

        stats.total_iterations = iteration + 1;

        // Check for fixpoint
        if stats.total_optimizations() == before_count {
            break;
        }
    }

    stats
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{IrBinOp, IrBlock, IrExpr, IrFunction, IrLiteral, IrStmt, IrType};

    #[test]
    fn test_o0_does_nothing() {
        let mut module = create_test_module();
        let original = module.clone();

        let stats = optimize(&mut module, OptLevel::O0);

        assert_eq!(stats.total_optimizations(), 0);
        assert_eq!(module, original);
    }

    #[test]
    fn test_o1_applies_basic_optimizations() {
        let mut module = create_test_module_with_constants();

        let stats = optimize(&mut module, OptLevel::O1);

        assert!(stats.constants_folded > 0 || stats.dead_code_eliminated > 0);
        assert_eq!(stats.total_iterations, 1);
    }

    #[test]
    fn test_o2_applies_aggressive_optimizations() {
        let mut module = create_test_module_with_inlinable_function();

        let stats = optimize(&mut module, OptLevel::O2);

        // Should have done some optimizations
        assert!(stats.total_optimizations() > 0);
    }

    #[test]
    fn test_combined_optimizations_improve_code() {
        let mut module = IrModule {
            name: "test".to_string(),
            version: "1.0.0".to_string(),
            imports: vec![],
            types: vec![],
            functions: vec![
                // Helper: fn get_value() -> U32 { return 5; }
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
                // Main: fn main() -> U32 {
                //   let x = get_value();  // Can be inlined to 5
                //   let y = x + 3;         // Can be folded to 8
                //   let z = 999;           // Unused, can be eliminated
                //   return y;
                // }
                IrFunction {
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
                                value: IrExpr::Call {
                                    func: Box::new(IrExpr::Var("get_value".to_string())),
                                    args: vec![],
                                },
                            },
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

        // Should have inlined, folded constants, and eliminated dead code
        assert!(stats.functions_inlined > 0, "Should have inlined get_value");
        assert!(
            stats.constants_folded > 0,
            "Should have folded some constants"
        );
        assert!(
            stats.dead_code_eliminated > 0,
            "Should have eliminated unused variable z"
        );
    }

    #[test]
    fn test_optimization_levels_parse() {
        assert_eq!("0".parse::<OptLevel>().unwrap(), OptLevel::O0);
        assert_eq!("o0".parse::<OptLevel>().unwrap(), OptLevel::O0);
        assert_eq!("O0".parse::<OptLevel>().unwrap(), OptLevel::O0);

        assert_eq!("1".parse::<OptLevel>().unwrap(), OptLevel::O1);
        assert_eq!("o1".parse::<OptLevel>().unwrap(), OptLevel::O1);

        assert_eq!("2".parse::<OptLevel>().unwrap(), OptLevel::O2);
        assert_eq!("o2".parse::<OptLevel>().unwrap(), OptLevel::O2);

        assert!("invalid".parse::<OptLevel>().is_err());
    }

    fn create_test_module() -> IrModule {
        IrModule {
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
                        value: Some(IrExpr::Literal(IrLiteral::U32(42))),
                    }],
                },
            }],
            exports: vec![],
        }
    }

    fn create_test_module_with_constants() -> IrModule {
        IrModule {
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
        }
    }

    fn create_test_module_with_inlinable_function() -> IrModule {
        IrModule {
            name: "test".to_string(),
            version: "1.0.0".to_string(),
            imports: vec![],
            types: vec![],
            functions: vec![
                IrFunction {
                    name: "helper".to_string(),
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
                                func: Box::new(IrExpr::Var("helper".to_string())),
                                args: vec![],
                            }),
                        }],
                    },
                },
            ],
            exports: vec![],
        }
    }
}
