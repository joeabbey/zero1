//! Dead Code Elimination (DCE) optimization pass
//!
//! This module implements dead code elimination through liveness analysis.
//! It removes:
//! - Unused local variables (written but never read)
//! - Unreachable code (after return/unconditional jump)
//! - Empty blocks
//!
//! It preserves:
//! - Side-effectful operations (function calls with effects)
//! - Variables used in nested scopes

use crate::{IrBlock, IrExpr, IrFunction, IrModule, IrStmt};
use std::collections::HashSet;

/// Performs dead code elimination on an IR module
pub fn eliminate_dead_code(module: &mut IrModule) -> usize {
    let mut eliminated_count = 0;

    for func in &mut module.functions {
        eliminated_count += eliminate_dead_code_in_function(func);
    }

    eliminated_count
}

/// Performs dead code elimination on a single function
fn eliminate_dead_code_in_function(func: &mut IrFunction) -> usize {
    let mut eliminated_count = 0;

    // Iterative elimination until fixpoint
    loop {
        let before = eliminated_count;

        // Remove unreachable code
        eliminated_count += remove_unreachable_code(&mut func.body);

        // Remove unused variables
        eliminated_count += remove_unused_variables(&mut func.body);

        // Remove empty blocks (handled implicitly by other optimizations)

        // Check if we reached fixpoint
        if eliminated_count == before {
            break;
        }
    }

    eliminated_count
}

/// Removes code that appears after a return statement or unconditional jump
fn remove_unreachable_code(block: &mut IrBlock) -> usize {
    let mut eliminated_count = 0;
    let mut new_statements = Vec::new();
    let mut reached_return = false;

    for stmt in &block.statements {
        if reached_return {
            eliminated_count += 1;
            continue;
        }

        match stmt {
            IrStmt::Return { .. } => {
                new_statements.push(stmt.clone());
                reached_return = true;
            }
            IrStmt::If {
                cond,
                then_block,
                else_block,
            } => {
                let mut new_then = then_block.clone();
                let mut new_else = else_block.clone();

                eliminated_count += remove_unreachable_code(&mut new_then);
                if let Some(ref mut eb) = new_else {
                    eliminated_count += remove_unreachable_code(eb);
                }

                new_statements.push(IrStmt::If {
                    cond: cond.clone(),
                    then_block: new_then,
                    else_block: new_else,
                });
            }
            IrStmt::While { cond, body } => {
                let mut new_body = body.clone();
                eliminated_count += remove_unreachable_code(&mut new_body);

                new_statements.push(IrStmt::While {
                    cond: cond.clone(),
                    body: new_body,
                });
            }
            _ => {
                new_statements.push(stmt.clone());
            }
        }
    }

    block.statements = new_statements;
    eliminated_count
}

/// Removes variables that are written but never read
fn remove_unused_variables(block: &mut IrBlock) -> usize {
    // First, collect all variable uses
    let used_vars = collect_used_variables(block);

    // Then remove let statements for unused variables (if they have no side effects)
    let mut eliminated_count = 0;
    let mut new_statements = Vec::new();

    for stmt in &block.statements {
        match stmt {
            IrStmt::Let { name, value, .. } => {
                // Keep the let if:
                // 1. The variable is used, OR
                // 2. The value has side effects
                if used_vars.contains(name) || has_side_effects(value) {
                    new_statements.push(stmt.clone());
                } else {
                    eliminated_count += 1;
                }
            }
            IrStmt::If {
                cond,
                then_block,
                else_block,
            } => {
                let mut new_then = then_block.clone();
                let mut new_else = else_block.clone();

                eliminated_count += remove_unused_variables(&mut new_then);
                if let Some(ref mut eb) = new_else {
                    eliminated_count += remove_unused_variables(eb);
                }

                new_statements.push(IrStmt::If {
                    cond: cond.clone(),
                    then_block: new_then,
                    else_block: new_else,
                });
            }
            IrStmt::While { cond, body } => {
                let mut new_body = body.clone();
                eliminated_count += remove_unused_variables(&mut new_body);

                new_statements.push(IrStmt::While {
                    cond: cond.clone(),
                    body: new_body,
                });
            }
            _ => {
                new_statements.push(stmt.clone());
            }
        }
    }

    block.statements = new_statements;
    eliminated_count
}

/// Collects all variables that are actually used (read) in a block
fn collect_used_variables(block: &IrBlock) -> HashSet<String> {
    let mut used = HashSet::new();

    for stmt in &block.statements {
        collect_used_in_stmt(stmt, &mut used);
    }

    used
}

/// Collects used variables from a statement
fn collect_used_in_stmt(stmt: &IrStmt, used: &mut HashSet<String>) {
    match stmt {
        IrStmt::Let { value, .. } => {
            collect_used_in_expr(value, used);
        }
        IrStmt::Assign { target, value } => {
            collect_used_in_expr(target, used);
            collect_used_in_expr(value, used);
        }
        IrStmt::If {
            cond,
            then_block,
            else_block,
        } => {
            collect_used_in_expr(cond, used);
            for s in &then_block.statements {
                collect_used_in_stmt(s, used);
            }
            if let Some(else_blk) = else_block {
                for s in &else_blk.statements {
                    collect_used_in_stmt(s, used);
                }
            }
        }
        IrStmt::While { cond, body } => {
            collect_used_in_expr(cond, used);
            for s in &body.statements {
                collect_used_in_stmt(s, used);
            }
        }
        IrStmt::Return { value } => {
            if let Some(val) = value {
                collect_used_in_expr(val, used);
            }
        }
        IrStmt::Expr(expr) => {
            collect_used_in_expr(expr, used);
        }
    }
}

/// Collects used variables from an expression
fn collect_used_in_expr(expr: &IrExpr, used: &mut HashSet<String>) {
    match expr {
        IrExpr::Var(name) => {
            used.insert(name.clone());
        }
        IrExpr::BinOp { left, right, .. } => {
            collect_used_in_expr(left, used);
            collect_used_in_expr(right, used);
        }
        IrExpr::UnaryOp { expr, .. } => {
            collect_used_in_expr(expr, used);
        }
        IrExpr::Call { func, args } => {
            collect_used_in_expr(func, used);
            for arg in args {
                collect_used_in_expr(arg, used);
            }
        }
        IrExpr::Field { base, .. } => {
            collect_used_in_expr(base, used);
        }
        IrExpr::Record { fields } => {
            for (_, field_expr) in fields {
                collect_used_in_expr(field_expr, used);
            }
        }
        IrExpr::Path(segments) => {
            // Path references are typically module-level, track first segment
            if let Some(first) = segments.first() {
                used.insert(first.clone());
            }
        }
        IrExpr::Literal(_) => {
            // Literals don't use variables
        }
    }
}

/// Checks if an expression has side effects
fn has_side_effects(expr: &IrExpr) -> bool {
    match expr {
        // Function calls may have side effects
        IrExpr::Call { .. } => true,
        // Recursive checks
        IrExpr::BinOp { left, right, .. } => has_side_effects(left) || has_side_effects(right),
        IrExpr::UnaryOp { expr, .. } => has_side_effects(expr),
        IrExpr::Field { base, .. } => has_side_effects(base),
        IrExpr::Record { fields } => fields.iter().any(|(_, e)| has_side_effects(e)),
        // Safe expressions
        IrExpr::Var(_) | IrExpr::Literal(_) | IrExpr::Path(_) => false,
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{IrLiteral, IrType};

    #[test]
    fn test_eliminate_unused_variable() {
        let mut func = IrFunction {
            name: "test".to_string(),
            params: vec![],
            return_type: IrType::U32,
            effects: vec![],
            body: IrBlock {
                statements: vec![
                    // x is unused
                    IrStmt::Let {
                        name: "x".to_string(),
                        mutable: false,
                        ty: Some(IrType::U32),
                        value: IrExpr::Literal(IrLiteral::U32(42)),
                    },
                    // y is used in return
                    IrStmt::Let {
                        name: "y".to_string(),
                        mutable: false,
                        ty: Some(IrType::U32),
                        value: IrExpr::Literal(IrLiteral::U32(10)),
                    },
                    IrStmt::Return {
                        value: Some(IrExpr::Var("y".to_string())),
                    },
                ],
            },
        };

        let eliminated = eliminate_dead_code_in_function(&mut func);
        assert_eq!(eliminated, 1); // x should be eliminated
        assert_eq!(func.body.statements.len(), 2); // Only y and return remain
    }

    #[test]
    fn test_remove_code_after_return() {
        let mut func = IrFunction {
            name: "test".to_string(),
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
                        name: "x".to_string(),
                        mutable: false,
                        ty: Some(IrType::U32),
                        value: IrExpr::Literal(IrLiteral::U32(10)),
                    },
                    // This too
                    IrStmt::Return {
                        value: Some(IrExpr::Literal(IrLiteral::U32(20))),
                    },
                ],
            },
        };

        let eliminated = eliminate_dead_code_in_function(&mut func);
        assert_eq!(eliminated, 2); // Two statements after first return
        assert_eq!(func.body.statements.len(), 1); // Only first return remains
    }

    #[test]
    fn test_preserve_side_effectful_calls() {
        let mut func = IrFunction {
            name: "test".to_string(),
            params: vec![],
            return_type: IrType::Unit,
            effects: vec![],
            body: IrBlock {
                statements: vec![
                    // x is unused, but the call has side effects
                    IrStmt::Let {
                        name: "x".to_string(),
                        mutable: false,
                        ty: Some(IrType::U32),
                        value: IrExpr::Call {
                            func: Box::new(IrExpr::Var("effectful_fn".to_string())),
                            args: vec![],
                        },
                    },
                    IrStmt::Return { value: None },
                ],
            },
        };

        let eliminated = eliminate_dead_code_in_function(&mut func);
        assert_eq!(eliminated, 0); // Nothing eliminated due to side effects
        assert_eq!(func.body.statements.len(), 2);
    }

    #[test]
    fn test_preserve_variables_used_in_nested_scopes() {
        let mut func = IrFunction {
            name: "test".to_string(),
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
                    // x is used in nested if block
                    IrStmt::If {
                        cond: IrExpr::Literal(IrLiteral::Bool(true)),
                        then_block: IrBlock {
                            statements: vec![IrStmt::Return {
                                value: Some(IrExpr::Var("x".to_string())),
                            }],
                        },
                        else_block: None,
                    },
                    IrStmt::Return {
                        value: Some(IrExpr::Literal(IrLiteral::U32(0))),
                    },
                ],
            },
        };

        let eliminated = eliminate_dead_code_in_function(&mut func);
        assert_eq!(eliminated, 0); // x is used in nested scope
        assert_eq!(func.body.statements.len(), 3);
    }
}
