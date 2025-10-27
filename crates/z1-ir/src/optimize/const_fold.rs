//! Constant folding and propagation optimization pass
//!
//! This module implements compile-time evaluation of constant expressions.
//! It handles:
//! - Arithmetic operations (with overflow checks)
//! - Comparison operations
//! - Boolean operations
//! - Constant propagation through assignments
//! - Simplification of conditional branches with constant conditions

use crate::{IrBinOp, IrBlock, IrExpr, IrFunction, IrLiteral, IrModule, IrStmt, IrUnaryOp};
use std::collections::HashMap;

/// Performs constant folding on an IR module
pub fn fold_constants(module: &mut IrModule) -> usize {
    let mut folded_count = 0;

    for func in &mut module.functions {
        folded_count += fold_constants_in_function(func);
    }

    folded_count
}

/// Performs constant folding on a single function
fn fold_constants_in_function(func: &mut IrFunction) -> usize {
    let mut folded_count = 0;

    // Iterative folding until fixpoint
    loop {
        let before = folded_count;

        // Track known constant values
        let mut const_map = HashMap::new();

        folded_count += fold_constants_in_block(&mut func.body, &mut const_map);

        // Check if we reached fixpoint
        if folded_count == before {
            break;
        }
    }

    folded_count
}

/// Performs constant folding in a block
fn fold_constants_in_block(
    block: &mut IrBlock,
    const_map: &mut HashMap<String, IrLiteral>,
) -> usize {
    let mut folded_count = 0;
    let mut new_statements = Vec::new();

    for stmt in &block.statements {
        let (new_stmt, count) = fold_constants_in_stmt(stmt, const_map);
        folded_count += count;
        new_statements.push(new_stmt);
    }

    block.statements = new_statements;
    folded_count
}

/// Performs constant folding in a statement
fn fold_constants_in_stmt(
    stmt: &IrStmt,
    const_map: &mut HashMap<String, IrLiteral>,
) -> (IrStmt, usize) {
    let mut folded_count = 0;

    let new_stmt = match stmt {
        IrStmt::Let {
            name,
            mutable,
            ty,
            value,
        } => {
            let (new_value, count) = fold_expr(value, const_map);
            folded_count += count;

            // Track constant values for non-mutable variables
            if !mutable {
                if let IrExpr::Literal(lit) = &new_value {
                    const_map.insert(name.clone(), lit.clone());
                }
            }

            IrStmt::Let {
                name: name.clone(),
                mutable: *mutable,
                ty: ty.clone(),
                value: new_value,
            }
        }
        IrStmt::Assign { target, value } => {
            let (new_target, count1) = fold_expr(target, const_map);
            let (new_value, count2) = fold_expr(value, const_map);
            folded_count += count1 + count2;

            // Invalidate constant tracking for assigned variables
            if let IrExpr::Var(name) = &new_target {
                const_map.remove(name);
            }

            IrStmt::Assign {
                target: new_target,
                value: new_value,
            }
        }
        IrStmt::If {
            cond,
            then_block,
            else_block,
        } => {
            let (new_cond, count) = fold_expr(cond, const_map);
            folded_count += count;

            // If condition is a constant, we can simplify
            match &new_cond {
                IrExpr::Literal(IrLiteral::Bool(true)) => {
                    // Condition is always true, execute then block
                    let mut then_map = const_map.clone();
                    let mut new_then = then_block.clone();
                    folded_count += fold_constants_in_block(&mut new_then, &mut then_map);

                    // Only count as a simplification if we actually removed the else block
                    let simplified = else_block.is_some();
                    if simplified {
                        folded_count += 1;
                    }

                    // Return the then block statements inline
                    // For now, keep the if structure but mark it as optimizable
                    IrStmt::If {
                        cond: new_cond,
                        then_block: new_then,
                        else_block: None,
                    }
                }
                IrExpr::Literal(IrLiteral::Bool(false)) => {
                    // Condition is always false, execute else block if present
                    if let Some(else_blk) = else_block {
                        let mut else_map = const_map.clone();
                        let mut new_else = else_blk.clone();
                        folded_count += fold_constants_in_block(&mut new_else, &mut else_map);

                        // Only count as a simplification if we actually removed the then block
                        let simplified = !then_block.statements.is_empty();
                        if simplified {
                            folded_count += 1;
                        }

                        // Return the else block statements inline
                        IrStmt::If {
                            cond: new_cond,
                            then_block: IrBlock { statements: vec![] },
                            else_block: Some(new_else),
                        }
                    } else {
                        // Count the if elimination only once
                        folded_count += 1;
                        // No else block, this if does nothing
                        IrStmt::Expr(IrExpr::Literal(IrLiteral::Unit))
                    }
                }
                _ => {
                    // Non-constant condition, fold both branches
                    let mut then_map = const_map.clone();
                    let mut new_then = then_block.clone();
                    folded_count += fold_constants_in_block(&mut new_then, &mut then_map);

                    let new_else = if let Some(else_blk) = else_block {
                        let mut else_map = const_map.clone();
                        let mut new_else = else_blk.clone();
                        folded_count += fold_constants_in_block(&mut new_else, &mut else_map);
                        Some(new_else)
                    } else {
                        None
                    };

                    IrStmt::If {
                        cond: new_cond,
                        then_block: new_then,
                        else_block: new_else,
                    }
                }
            }
        }
        IrStmt::While { cond, body } => {
            let (new_cond, count1) = fold_expr(cond, const_map);
            folded_count += count1;

            // For while loops, we can't propagate constants from inside the loop
            let mut loop_map = const_map.clone();
            let mut new_body = body.clone();
            folded_count += fold_constants_in_block(&mut new_body, &mut loop_map);

            IrStmt::While {
                cond: new_cond,
                body: new_body,
            }
        }
        IrStmt::Return { value } => {
            let new_value = if let Some(val) = value {
                let (folded, count) = fold_expr(val, const_map);
                folded_count += count;
                Some(folded)
            } else {
                None
            };

            IrStmt::Return { value: new_value }
        }
        IrStmt::Expr(expr) => {
            let (new_expr, count) = fold_expr(expr, const_map);
            folded_count += count;
            IrStmt::Expr(new_expr)
        }
    };

    (new_stmt, folded_count)
}

/// Performs constant folding on an expression
fn fold_expr(expr: &IrExpr, const_map: &HashMap<String, IrLiteral>) -> (IrExpr, usize) {
    let mut folded_count = 0;

    let result = match expr {
        IrExpr::Var(name) => {
            // Replace with constant if known
            if let Some(lit) = const_map.get(name) {
                folded_count += 1;
                IrExpr::Literal(lit.clone())
            } else {
                expr.clone()
            }
        }
        IrExpr::BinOp { op, left, right } => {
            let (new_left, count1) = fold_expr(left, const_map);
            let (new_right, count2) = fold_expr(right, const_map);
            folded_count += count1 + count2;

            // Try to fold if both operands are literals
            if let (IrExpr::Literal(l), IrExpr::Literal(r)) = (&new_left, &new_right) {
                if let Some(result) = fold_binop(*op, l, r) {
                    folded_count += 1;
                    IrExpr::Literal(result)
                } else {
                    IrExpr::BinOp {
                        op: *op,
                        left: Box::new(new_left),
                        right: Box::new(new_right),
                    }
                }
            } else {
                IrExpr::BinOp {
                    op: *op,
                    left: Box::new(new_left),
                    right: Box::new(new_right),
                }
            }
        }
        IrExpr::UnaryOp { op, expr: inner } => {
            let (new_inner, count) = fold_expr(inner, const_map);
            folded_count += count;

            if let IrExpr::Literal(lit) = &new_inner {
                if let Some(result) = fold_unaryop(*op, lit) {
                    folded_count += 1;
                    IrExpr::Literal(result)
                } else {
                    IrExpr::UnaryOp {
                        op: *op,
                        expr: Box::new(new_inner),
                    }
                }
            } else {
                IrExpr::UnaryOp {
                    op: *op,
                    expr: Box::new(new_inner),
                }
            }
        }
        IrExpr::Call { func, args } => {
            let (new_func, count1) = fold_expr(func, const_map);
            folded_count += count1;

            let mut new_args = Vec::new();
            for arg in args {
                let (new_arg, count) = fold_expr(arg, const_map);
                folded_count += count;
                new_args.push(new_arg);
            }

            IrExpr::Call {
                func: Box::new(new_func),
                args: new_args,
            }
        }
        IrExpr::Field { base, field } => {
            let (new_base, count) = fold_expr(base, const_map);
            folded_count += count;

            IrExpr::Field {
                base: Box::new(new_base),
                field: field.clone(),
            }
        }
        IrExpr::Record { fields } => {
            let mut new_fields = Vec::new();
            for (name, field_expr) in fields {
                let (new_expr, count) = fold_expr(field_expr, const_map);
                folded_count += count;
                new_fields.push((name.clone(), new_expr));
            }

            IrExpr::Record { fields: new_fields }
        }
        _ => expr.clone(),
    };

    (result, folded_count)
}

/// Folds a binary operation on two literals
fn fold_binop(op: IrBinOp, left: &IrLiteral, right: &IrLiteral) -> Option<IrLiteral> {
    match (op, left, right) {
        // Arithmetic on U32
        (IrBinOp::Add, IrLiteral::U32(a), IrLiteral::U32(b)) => {
            a.checked_add(*b).map(IrLiteral::U32)
        }
        (IrBinOp::Sub, IrLiteral::U32(a), IrLiteral::U32(b)) => {
            a.checked_sub(*b).map(IrLiteral::U32)
        }
        (IrBinOp::Mul, IrLiteral::U32(a), IrLiteral::U32(b)) => {
            a.checked_mul(*b).map(IrLiteral::U32)
        }
        (IrBinOp::Div, IrLiteral::U32(a), IrLiteral::U32(b)) => {
            if *b == 0 {
                None // Don't fold division by zero
            } else {
                Some(IrLiteral::U32(a / b))
            }
        }
        (IrBinOp::Mod, IrLiteral::U32(a), IrLiteral::U32(b)) => {
            if *b == 0 {
                None
            } else {
                Some(IrLiteral::U32(a % b))
            }
        }

        // Arithmetic on Int
        (IrBinOp::Add, IrLiteral::Int(a), IrLiteral::Int(b)) => {
            a.checked_add(*b).map(IrLiteral::Int)
        }
        (IrBinOp::Sub, IrLiteral::Int(a), IrLiteral::Int(b)) => {
            a.checked_sub(*b).map(IrLiteral::Int)
        }
        (IrBinOp::Mul, IrLiteral::Int(a), IrLiteral::Int(b)) => {
            a.checked_mul(*b).map(IrLiteral::Int)
        }

        // Comparisons on U32
        (IrBinOp::Eq, IrLiteral::U32(a), IrLiteral::U32(b)) => Some(IrLiteral::Bool(a == b)),
        (IrBinOp::Ne, IrLiteral::U32(a), IrLiteral::U32(b)) => Some(IrLiteral::Bool(a != b)),
        (IrBinOp::Lt, IrLiteral::U32(a), IrLiteral::U32(b)) => Some(IrLiteral::Bool(a < b)),
        (IrBinOp::Le, IrLiteral::U32(a), IrLiteral::U32(b)) => Some(IrLiteral::Bool(a <= b)),
        (IrBinOp::Gt, IrLiteral::U32(a), IrLiteral::U32(b)) => Some(IrLiteral::Bool(a > b)),
        (IrBinOp::Ge, IrLiteral::U32(a), IrLiteral::U32(b)) => Some(IrLiteral::Bool(a >= b)),

        // Boolean operations
        (IrBinOp::And, IrLiteral::Bool(a), IrLiteral::Bool(b)) => Some(IrLiteral::Bool(*a && *b)),
        (IrBinOp::Or, IrLiteral::Bool(a), IrLiteral::Bool(b)) => Some(IrLiteral::Bool(*a || *b)),
        (IrBinOp::Eq, IrLiteral::Bool(a), IrLiteral::Bool(b)) => Some(IrLiteral::Bool(a == b)),
        (IrBinOp::Ne, IrLiteral::Bool(a), IrLiteral::Bool(b)) => Some(IrLiteral::Bool(a != b)),

        _ => None,
    }
}

/// Folds a unary operation on a literal
fn fold_unaryop(op: IrUnaryOp, operand: &IrLiteral) -> Option<IrLiteral> {
    match (op, operand) {
        (IrUnaryOp::Not, IrLiteral::Bool(b)) => Some(IrLiteral::Bool(!b)),
        (IrUnaryOp::Neg, IrLiteral::Int(i)) => i.checked_neg().map(IrLiteral::Int),
        _ => None,
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::IrType;

    #[test]
    fn test_fold_arithmetic_constants() {
        let mut func = IrFunction {
            name: "test".to_string(),
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
        };

        let folded = fold_constants_in_function(&mut func);
        assert!(folded > 0);

        // Check that the addition was folded
        match &func.body.statements[0] {
            IrStmt::Return {
                value: Some(IrExpr::Literal(IrLiteral::U32(5))),
            } => (),
            _ => panic!("Expected folded constant 5"),
        }
    }

    #[test]
    fn test_fold_comparison_constants() {
        let mut func = IrFunction {
            name: "test".to_string(),
            params: vec![],
            return_type: IrType::Bool,
            effects: vec![],
            body: IrBlock {
                statements: vec![IrStmt::Return {
                    value: Some(IrExpr::BinOp {
                        op: IrBinOp::Gt,
                        left: Box::new(IrExpr::Literal(IrLiteral::U32(10))),
                        right: Box::new(IrExpr::Literal(IrLiteral::U32(5))),
                    }),
                }],
            },
        };

        let folded = fold_constants_in_function(&mut func);
        assert!(folded > 0);

        // Check that the comparison was folded
        match &func.body.statements[0] {
            IrStmt::Return {
                value: Some(IrExpr::Literal(IrLiteral::Bool(true))),
            } => (),
            _ => panic!("Expected folded constant true"),
        }
    }

    #[test]
    fn test_propagate_constants() {
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
                    IrStmt::Return {
                        value: Some(IrExpr::Var("x".to_string())),
                    },
                ],
            },
        };

        let folded = fold_constants_in_function(&mut func);
        assert!(folded > 0);

        // Check that x was replaced with 42
        match &func.body.statements[1] {
            IrStmt::Return {
                value: Some(IrExpr::Literal(IrLiteral::U32(42))),
            } => (),
            _ => panic!("Expected propagated constant 42"),
        }
    }

    #[test]
    fn test_simplify_if_with_constant_condition() {
        let mut func = IrFunction {
            name: "test".to_string(),
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
        };

        let folded = fold_constants_in_function(&mut func);
        assert!(folded > 0);

        // The if should still be there but simplified (else removed)
        match &func.body.statements[0] {
            IrStmt::If {
                cond: IrExpr::Literal(IrLiteral::Bool(true)),
                then_block: _,
                else_block,
            } => {
                assert!(else_block.is_none());
            }
            _ => panic!("Expected simplified if statement"),
        }
    }
}
