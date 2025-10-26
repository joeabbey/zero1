//! Function inlining optimization pass
//!
//! This module implements function inlining based on heuristics:
//! - Inline trivial functions (1-2 statements)
//! - Inline small pure functions
//! - Avoid recursive inlining
//! - Don't inline if it significantly increases code size

use crate::{IrBlock, IrExpr, IrFunction, IrModule, IrStmt};
use std::collections::{HashMap, HashSet};

/// Configuration for inlining heuristics
pub struct InlineConfig {
    /// Maximum number of statements to consider for inlining
    pub max_inline_size: usize,
    /// Always inline functions smaller than this
    pub always_inline_threshold: usize,
}

impl Default for InlineConfig {
    fn default() -> Self {
        InlineConfig {
            max_inline_size: 5,
            always_inline_threshold: 2,
        }
    }
}

/// Performs function inlining on an IR module
pub fn inline_functions(module: &mut IrModule) -> usize {
    inline_functions_with_config(module, &InlineConfig::default())
}

/// Performs function inlining with custom configuration
pub fn inline_functions_with_config(module: &mut IrModule, config: &InlineConfig) -> usize {
    let mut inlined_count = 0;

    // Build a map of functions for lookup
    let func_map: HashMap<String, IrFunction> = module
        .functions
        .iter()
        .map(|f| (f.name.clone(), f.clone()))
        .collect();

    // Identify which functions are recursive (don't inline these)
    let recursive_funcs = identify_recursive_functions(&module.functions);

    // Inline in each function
    for func in &mut module.functions {
        inlined_count += inline_in_function(func, &func_map, &recursive_funcs, config);
    }

    inlined_count
}

/// Identifies functions that call themselves (directly or indirectly)
fn identify_recursive_functions(functions: &[IrFunction]) -> HashSet<String> {
    let mut recursive = HashSet::new();

    for func in functions {
        if calls_function(&func.body, &func.name) {
            recursive.insert(func.name.clone());
        }
    }

    // TODO: Could also detect mutually recursive functions
    recursive
}

/// Checks if a block calls a specific function
fn calls_function(block: &IrBlock, target_name: &str) -> bool {
    for stmt in &block.statements {
        if stmt_calls_function(stmt, target_name) {
            return true;
        }
    }
    false
}

/// Checks if a statement calls a specific function
fn stmt_calls_function(stmt: &IrStmt, target_name: &str) -> bool {
    match stmt {
        IrStmt::Let { value, .. } => expr_calls_function(value, target_name),
        IrStmt::Assign { target, value } => {
            expr_calls_function(target, target_name) || expr_calls_function(value, target_name)
        }
        IrStmt::If {
            cond,
            then_block,
            else_block,
        } => {
            expr_calls_function(cond, target_name)
                || calls_function(then_block, target_name)
                || else_block
                    .as_ref()
                    .is_some_and(|b| calls_function(b, target_name))
        }
        IrStmt::While { cond, body } => {
            expr_calls_function(cond, target_name) || calls_function(body, target_name)
        }
        IrStmt::Return { value } => value
            .as_ref()
            .is_some_and(|v| expr_calls_function(v, target_name)),
        IrStmt::Expr(expr) => expr_calls_function(expr, target_name),
    }
}

/// Checks if an expression calls a specific function
fn expr_calls_function(expr: &IrExpr, target_name: &str) -> bool {
    match expr {
        IrExpr::Call { func, args } => {
            if let IrExpr::Var(name) = func.as_ref() {
                if name == target_name {
                    return true;
                }
            }
            args.iter().any(|arg| expr_calls_function(arg, target_name))
        }
        IrExpr::BinOp { left, right, .. } => {
            expr_calls_function(left, target_name) || expr_calls_function(right, target_name)
        }
        IrExpr::UnaryOp { expr: inner, .. } => expr_calls_function(inner, target_name),
        IrExpr::Field { base, .. } => expr_calls_function(base, target_name),
        IrExpr::Record { fields } => fields
            .iter()
            .any(|(_, e)| expr_calls_function(e, target_name)),
        _ => false,
    }
}

/// Performs inlining within a single function
fn inline_in_function(
    func: &mut IrFunction,
    func_map: &HashMap<String, IrFunction>,
    recursive_funcs: &HashSet<String>,
    config: &InlineConfig,
) -> usize {
    let mut inlined_count = 0;

    // Iterative inlining until fixpoint
    loop {
        let before = inlined_count;

        inlined_count += inline_in_block(&mut func.body, func_map, recursive_funcs, config);

        if inlined_count == before {
            break;
        }
    }

    inlined_count
}

/// Performs inlining within a block
fn inline_in_block(
    block: &mut IrBlock,
    func_map: &HashMap<String, IrFunction>,
    recursive_funcs: &HashSet<String>,
    config: &InlineConfig,
) -> usize {
    let mut inlined_count = 0;
    let mut new_statements = Vec::new();

    for stmt in &block.statements {
        let (new_stmt, count) = inline_in_stmt(stmt, func_map, recursive_funcs, config);
        inlined_count += count;
        new_statements.push(new_stmt);
    }

    block.statements = new_statements;
    inlined_count
}

/// Performs inlining within a statement
fn inline_in_stmt(
    stmt: &IrStmt,
    func_map: &HashMap<String, IrFunction>,
    recursive_funcs: &HashSet<String>,
    config: &InlineConfig,
) -> (IrStmt, usize) {
    let mut inlined_count = 0;

    let new_stmt = match stmt {
        IrStmt::Let {
            name,
            mutable,
            ty,
            value,
        } => {
            let (new_value, count) = inline_in_expr(value, func_map, recursive_funcs, config);
            inlined_count += count;

            IrStmt::Let {
                name: name.clone(),
                mutable: *mutable,
                ty: ty.clone(),
                value: new_value,
            }
        }
        IrStmt::Assign { target, value } => {
            let (new_target, count1) = inline_in_expr(target, func_map, recursive_funcs, config);
            let (new_value, count2) = inline_in_expr(value, func_map, recursive_funcs, config);
            inlined_count += count1 + count2;

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
            let (new_cond, count) = inline_in_expr(cond, func_map, recursive_funcs, config);
            inlined_count += count;

            let mut new_then = then_block.clone();
            inlined_count += inline_in_block(&mut new_then, func_map, recursive_funcs, config);

            let new_else = if let Some(else_blk) = else_block {
                let mut new_else_blk = else_blk.clone();
                inlined_count +=
                    inline_in_block(&mut new_else_blk, func_map, recursive_funcs, config);
                Some(new_else_blk)
            } else {
                None
            };

            IrStmt::If {
                cond: new_cond,
                then_block: new_then,
                else_block: new_else,
            }
        }
        IrStmt::While { cond, body } => {
            let (new_cond, count) = inline_in_expr(cond, func_map, recursive_funcs, config);
            inlined_count += count;

            let mut new_body = body.clone();
            inlined_count += inline_in_block(&mut new_body, func_map, recursive_funcs, config);

            IrStmt::While {
                cond: new_cond,
                body: new_body,
            }
        }
        IrStmt::Return { value } => {
            let new_value = if let Some(val) = value {
                let (new_val, count) = inline_in_expr(val, func_map, recursive_funcs, config);
                inlined_count += count;
                Some(new_val)
            } else {
                None
            };

            IrStmt::Return { value: new_value }
        }
        IrStmt::Expr(expr) => {
            let (new_expr, count) = inline_in_expr(expr, func_map, recursive_funcs, config);
            inlined_count += count;
            IrStmt::Expr(new_expr)
        }
    };

    (new_stmt, inlined_count)
}

/// Performs inlining within an expression
fn inline_in_expr(
    expr: &IrExpr,
    func_map: &HashMap<String, IrFunction>,
    recursive_funcs: &HashSet<String>,
    config: &InlineConfig,
) -> (IrExpr, usize) {
    let mut inlined_count = 0;

    let result = match expr {
        IrExpr::Call { func, args } => {
            // Check if this is a simple function call we can inline
            if let IrExpr::Var(func_name) = func.as_ref() {
                if let Some(target_func) = func_map.get(func_name) {
                    // Check if we should inline this function
                    if should_inline(target_func, recursive_funcs, config) {
                        // Inline the function
                        if let Some(inlined) = try_inline_call(target_func, args) {
                            inlined_count += 1;
                            return (inlined, inlined_count);
                        }
                    }
                }
            }

            // If we didn't inline, recursively process arguments
            let mut new_args = Vec::new();
            for arg in args {
                let (new_arg, count) = inline_in_expr(arg, func_map, recursive_funcs, config);
                inlined_count += count;
                new_args.push(new_arg);
            }

            IrExpr::Call {
                func: func.clone(),
                args: new_args,
            }
        }
        IrExpr::BinOp { op, left, right } => {
            let (new_left, count1) = inline_in_expr(left, func_map, recursive_funcs, config);
            let (new_right, count2) = inline_in_expr(right, func_map, recursive_funcs, config);
            inlined_count += count1 + count2;

            IrExpr::BinOp {
                op: *op,
                left: Box::new(new_left),
                right: Box::new(new_right),
            }
        }
        IrExpr::UnaryOp { op, expr: inner } => {
            let (new_inner, count) = inline_in_expr(inner, func_map, recursive_funcs, config);
            inlined_count += count;

            IrExpr::UnaryOp {
                op: *op,
                expr: Box::new(new_inner),
            }
        }
        IrExpr::Field { base, field } => {
            let (new_base, count) = inline_in_expr(base, func_map, recursive_funcs, config);
            inlined_count += count;

            IrExpr::Field {
                base: Box::new(new_base),
                field: field.clone(),
            }
        }
        IrExpr::Record { fields } => {
            let mut new_fields = Vec::new();
            for (name, field_expr) in fields {
                let (new_expr, count) =
                    inline_in_expr(field_expr, func_map, recursive_funcs, config);
                inlined_count += count;
                new_fields.push((name.clone(), new_expr));
            }

            IrExpr::Record { fields: new_fields }
        }
        _ => expr.clone(),
    };

    (result, inlined_count)
}

/// Determines if a function should be inlined
fn should_inline(
    func: &IrFunction,
    recursive_funcs: &HashSet<String>,
    config: &InlineConfig,
) -> bool {
    // Don't inline recursive functions
    if recursive_funcs.contains(&func.name) {
        return false;
    }

    // Count statements in the function
    let stmt_count = count_statements(&func.body);

    // Always inline very small functions
    if stmt_count <= config.always_inline_threshold {
        return true;
    }

    // Inline if within size threshold
    stmt_count <= config.max_inline_size
}

/// Counts the number of statements in a block
fn count_statements(block: &IrBlock) -> usize {
    let mut count = 0;

    for stmt in &block.statements {
        count += 1;
        count += match stmt {
            IrStmt::If {
                then_block,
                else_block,
                ..
            } => {
                count_statements(then_block)
                    + else_block.as_ref().map_or(0, count_statements)
            }
            IrStmt::While { body, .. } => count_statements(body),
            _ => 0,
        };
    }

    count
}

/// Attempts to inline a function call
fn try_inline_call(func: &IrFunction, args: &[IrExpr]) -> Option<IrExpr> {
    // For now, only inline single-expression functions (very simple case)
    if func.params.len() != args.len() {
        return None;
    }

    // Build parameter substitution map
    let mut subst_map: HashMap<String, IrExpr> = HashMap::new();
    for (param, arg) in func.params.iter().zip(args.iter()) {
        subst_map.insert(param.0.clone(), arg.clone());
    }

    // Try to inline if it's a single return statement
    if func.body.statements.len() == 1 {
        if let IrStmt::Return {
            value: Some(ret_expr),
        } = &func.body.statements[0]
        {
            return Some(substitute_expr(ret_expr, &subst_map));
        }
    }

    None
}

/// Substitutes parameters in an expression
fn substitute_expr(expr: &IrExpr, subst_map: &HashMap<String, IrExpr>) -> IrExpr {
    match expr {
        IrExpr::Var(name) => subst_map.get(name).cloned().unwrap_or_else(|| expr.clone()),
        IrExpr::BinOp { op, left, right } => IrExpr::BinOp {
            op: *op,
            left: Box::new(substitute_expr(left, subst_map)),
            right: Box::new(substitute_expr(right, subst_map)),
        },
        IrExpr::UnaryOp { op, expr: inner } => IrExpr::UnaryOp {
            op: *op,
            expr: Box::new(substitute_expr(inner, subst_map)),
        },
        IrExpr::Call { func, args } => IrExpr::Call {
            func: Box::new(substitute_expr(func, subst_map)),
            args: args.iter().map(|a| substitute_expr(a, subst_map)).collect(),
        },
        IrExpr::Field { base, field } => IrExpr::Field {
            base: Box::new(substitute_expr(base, subst_map)),
            field: field.clone(),
        },
        IrExpr::Record { fields } => IrExpr::Record {
            fields: fields
                .iter()
                .map(|(n, e)| (n.clone(), substitute_expr(e, subst_map)))
                .collect(),
        },
        _ => expr.clone(),
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{IrBinOp, IrLiteral, IrType};

    #[test]
    fn test_inline_trivial_function() {
        let mut module = IrModule {
            name: "test".to_string(),
            version: "1.0.0".to_string(),
            imports: vec![],
            types: vec![],
            functions: vec![
                // Helper function: fn get_ten() -> U32 { return 10; }
                IrFunction {
                    name: "get_ten".to_string(),
                    params: vec![],
                    return_type: IrType::U32,
                    effects: vec!["pure".to_string()],
                    body: IrBlock {
                        statements: vec![IrStmt::Return {
                            value: Some(IrExpr::Literal(IrLiteral::U32(10))),
                        }],
                    },
                },
                // Main function: fn main() -> U32 { return get_ten(); }
                IrFunction {
                    name: "main".to_string(),
                    params: vec![],
                    return_type: IrType::U32,
                    effects: vec![],
                    body: IrBlock {
                        statements: vec![IrStmt::Return {
                            value: Some(IrExpr::Call {
                                func: Box::new(IrExpr::Var("get_ten".to_string())),
                                args: vec![],
                            }),
                        }],
                    },
                },
            ],
            exports: vec![],
        };

        let inlined = inline_functions(&mut module);
        assert_eq!(inlined, 1);

        // Check that get_ten() was inlined
        let main_func = &module.functions[1];
        match &main_func.body.statements[0] {
            IrStmt::Return {
                value: Some(IrExpr::Literal(IrLiteral::U32(10))),
            } => (),
            _ => panic!("Expected inlined constant 10"),
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
                // Helper: fn double(x: U32) -> U32 { return x * 2; }
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
                // Main: fn main() -> U32 { return double(5); }
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

        let inlined = inline_functions(&mut module);
        assert_eq!(inlined, 1);

        // Check that double(5) was inlined to 5 * 2
        let main_func = &module.functions[1];
        match &main_func.body.statements[0] {
            IrStmt::Return {
                value: Some(IrExpr::BinOp { op, left, right }),
            } => {
                assert_eq!(*op, IrBinOp::Mul);
                assert!(matches!(**left, IrExpr::Literal(IrLiteral::U32(5))));
                assert!(matches!(**right, IrExpr::Literal(IrLiteral::U32(2))));
            }
            _ => panic!("Expected inlined expression 5 * 2"),
        }
    }

    #[test]
    fn test_dont_inline_large_function() {
        let config = InlineConfig {
            max_inline_size: 2,
            always_inline_threshold: 1,
        };

        let mut module = IrModule {
            name: "test".to_string(),
            version: "1.0.0".to_string(),
            imports: vec![],
            types: vec![],
            functions: vec![
                // Large function with 3+ statements
                IrFunction {
                    name: "large".to_string(),
                    params: vec![],
                    return_type: IrType::U32,
                    effects: vec![],
                    body: IrBlock {
                        statements: vec![
                            IrStmt::Let {
                                name: "x".to_string(),
                                mutable: false,
                                ty: Some(IrType::U32),
                                value: IrExpr::Literal(IrLiteral::U32(1)),
                            },
                            IrStmt::Let {
                                name: "y".to_string(),
                                mutable: false,
                                ty: Some(IrType::U32),
                                value: IrExpr::Literal(IrLiteral::U32(2)),
                            },
                            IrStmt::Return {
                                value: Some(IrExpr::BinOp {
                                    op: IrBinOp::Add,
                                    left: Box::new(IrExpr::Var("x".to_string())),
                                    right: Box::new(IrExpr::Var("y".to_string())),
                                }),
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

        let inlined = inline_functions_with_config(&mut module, &config);
        assert_eq!(inlined, 0); // Should not inline large function
    }

    #[test]
    fn test_dont_inline_recursive_function() {
        let mut module = IrModule {
            name: "test".to_string(),
            version: "1.0.0".to_string(),
            imports: vec![],
            types: vec![],
            functions: vec![
                // Recursive: fn fact(n: U32) -> U32 { ... fact(n-1) ... }
                IrFunction {
                    name: "fact".to_string(),
                    params: vec![("n".to_string(), IrType::U32)],
                    return_type: IrType::U32,
                    effects: vec![],
                    body: IrBlock {
                        statements: vec![IrStmt::Return {
                            value: Some(IrExpr::Call {
                                func: Box::new(IrExpr::Var("fact".to_string())),
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
                                func: Box::new(IrExpr::Var("fact".to_string())),
                                args: vec![IrExpr::Literal(IrLiteral::U32(5))],
                            }),
                        }],
                    },
                },
            ],
            exports: vec![],
        };

        let inlined = inline_functions(&mut module);
        assert_eq!(inlined, 0); // Should not inline recursive function
    }
}
