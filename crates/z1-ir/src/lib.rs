//! Zero1 Intermediate Representation (IR)
//!
//! This crate provides a simplified, lower-level representation of Zero1 code
//! optimized for code generation. The IR eliminates syntactic sugar and
//! normalizes the AST into a form that's easier to compile to target languages.

pub mod optimize;

use z1_ast as ast;

/// IR Module - compiled representation of a Z1 cell
#[derive(Debug, Clone, PartialEq)]
pub struct IrModule {
    pub name: String,
    pub version: String,
    pub imports: Vec<IrImport>,
    pub types: Vec<IrTypeDef>,
    pub functions: Vec<IrFunction>,
    pub exports: Vec<String>,
}

#[derive(Debug, Clone, PartialEq)]
pub struct IrImport {
    pub path: String,
    pub alias: Option<String>,
    pub items: Vec<String>,
}

#[derive(Debug, Clone, PartialEq)]
pub struct IrTypeDef {
    pub name: String,
    pub ty: IrType,
}

#[derive(Debug, Clone, PartialEq)]
pub enum IrType {
    Bool,
    Str,
    U16,
    U32,
    U64,
    Unit,
    Named(String),
    Record(Vec<(String, IrType)>),
    Union(Vec<(String, Option<IrType>)>),
    Generic {
        base: Box<IrType>,
        args: Vec<IrType>,
    },
}

#[derive(Debug, Clone, PartialEq)]
pub struct IrFunction {
    pub name: String,
    pub params: Vec<(String, IrType)>,
    pub return_type: IrType,
    pub effects: Vec<String>,
    pub body: IrBlock,
}

#[derive(Debug, Clone, PartialEq)]
pub struct IrBlock {
    pub statements: Vec<IrStmt>,
}

#[derive(Debug, Clone, PartialEq)]
pub enum IrStmt {
    Let {
        name: String,
        mutable: bool,
        ty: Option<IrType>,
        value: IrExpr,
    },
    Assign {
        target: IrExpr,
        value: IrExpr,
    },
    If {
        cond: IrExpr,
        then_block: IrBlock,
        else_block: Option<IrBlock>,
    },
    While {
        cond: IrExpr,
        body: IrBlock,
    },
    Return {
        value: Option<IrExpr>,
    },
    Expr(IrExpr),
}

#[derive(Debug, Clone, PartialEq)]
pub enum IrExpr {
    Var(String),
    Literal(IrLiteral),
    BinOp {
        op: IrBinOp,
        left: Box<IrExpr>,
        right: Box<IrExpr>,
    },
    UnaryOp {
        op: IrUnaryOp,
        expr: Box<IrExpr>,
    },
    Call {
        func: Box<IrExpr>,
        args: Vec<IrExpr>,
    },
    Field {
        base: Box<IrExpr>,
        field: String,
    },
    Record {
        fields: Vec<(String, IrExpr)>,
    },
    Path(Vec<String>),
}

#[derive(Debug, Clone, PartialEq)]
pub enum IrLiteral {
    Bool(bool),
    Str(String),
    U16(u16),
    U32(u32),
    U64(u64),
    Int(i64),
    Unit,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum IrBinOp {
    Add,
    Sub,
    Mul,
    Div,
    Mod,
    Eq,
    Ne,
    Lt,
    Le,
    Gt,
    Ge,
    And,
    Or,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum IrUnaryOp {
    Neg,
    Not,
    Await,
}

/// Error type for IR lowering
#[derive(Debug, Clone, PartialEq)]
pub enum LoweringError {
    UnsupportedTypeExpr(String),
    UnsupportedStmt(String),
    UnsupportedExpr(String),
}

impl std::fmt::Display for LoweringError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            LoweringError::UnsupportedTypeExpr(msg) => {
                write!(f, "Unsupported type expression: {msg}")
            }
            LoweringError::UnsupportedStmt(msg) => write!(f, "Unsupported statement: {msg}"),
            LoweringError::UnsupportedExpr(msg) => write!(f, "Unsupported expression: {msg}"),
        }
    }
}

impl std::error::Error for LoweringError {}

/// Convert Z1 AST to IR
pub fn lower_to_ir(module: &ast::Module) -> Result<IrModule, LoweringError> {
    let name = module.path.as_str_vec().join(".");
    let version = module
        .version
        .clone()
        .unwrap_or_else(|| "0.0.0".to_string());

    let imports = lower_imports(&module.items);
    let types = lower_types(&module.items)?;
    let functions = lower_functions(&module.items)?;
    let exports = collect_exports(&module.items);

    Ok(IrModule {
        name,
        version,
        imports,
        types,
        functions,
        exports,
    })
}

fn lower_imports(items: &[ast::Item]) -> Vec<IrImport> {
    items
        .iter()
        .filter_map(|item| {
            if let ast::Item::Import(imp) = item {
                Some(IrImport {
                    path: imp.path.clone(),
                    alias: imp.alias.clone(),
                    items: imp.only.clone(),
                })
            } else {
                None
            }
        })
        .collect()
}

fn lower_types(items: &[ast::Item]) -> Result<Vec<IrTypeDef>, LoweringError> {
    items
        .iter()
        .filter_map(|item| {
            if let ast::Item::Type(type_decl) = item {
                Some(lower_type_decl(type_decl))
            } else {
                None
            }
        })
        .collect()
}

fn lower_type_decl(decl: &ast::TypeDecl) -> Result<IrTypeDef, LoweringError> {
    Ok(IrTypeDef {
        name: decl.name.clone(),
        ty: lower_type_expr(&decl.expr)?,
    })
}

fn lower_type_expr(ty: &ast::TypeExpr) -> Result<IrType, LoweringError> {
    match ty {
        ast::TypeExpr::Path(segments) => {
            if segments.len() == 1 {
                match segments[0].as_str() {
                    "Bool" => Ok(IrType::Bool),
                    "Str" => Ok(IrType::Str),
                    "U16" => Ok(IrType::U16),
                    "U32" => Ok(IrType::U32),
                    "U64" => Ok(IrType::U64),
                    "()" => Ok(IrType::Unit),
                    name => Ok(IrType::Named(name.to_string())),
                }
            } else {
                Ok(IrType::Named(segments.join(".")))
            }
        }
        ast::TypeExpr::Record(fields) => {
            let mut ir_fields = Vec::new();
            for field in fields {
                let field_ty = lower_type_expr(&field.ty)?;
                ir_fields.push((field.name.clone(), field_ty));
            }
            Ok(IrType::Record(ir_fields))
        }
    }
}

fn lower_functions(items: &[ast::Item]) -> Result<Vec<IrFunction>, LoweringError> {
    items
        .iter()
        .filter_map(|item| {
            if let ast::Item::Fn(fn_decl) = item {
                Some(lower_function(fn_decl))
            } else {
                None
            }
        })
        .collect()
}

fn lower_function(fn_decl: &ast::FnDecl) -> Result<IrFunction, LoweringError> {
    let params: Result<Vec<_>, _> = fn_decl
        .params
        .iter()
        .map(|param| {
            let ty = lower_type_expr(&param.ty)?;
            Ok((param.name.clone(), ty))
        })
        .collect();

    let return_type = lower_type_expr(&fn_decl.ret)?;
    let body = lower_block(&fn_decl.body)?;

    Ok(IrFunction {
        name: fn_decl.name.clone(),
        params: params?,
        return_type,
        effects: fn_decl.effects.clone(),
        body,
    })
}

fn lower_block(block: &ast::Block) -> Result<IrBlock, LoweringError> {
    let statements: Result<Vec<_>, _> = block.statements.iter().map(lower_stmt).collect();

    Ok(IrBlock {
        statements: statements?,
    })
}

fn lower_stmt(stmt: &ast::Stmt) -> Result<IrStmt, LoweringError> {
    match stmt {
        ast::Stmt::Let(let_stmt) => Ok(IrStmt::Let {
            name: let_stmt.name.clone(),
            mutable: let_stmt.mutable,
            ty: if let Some(ty) = &let_stmt.ty {
                Some(lower_type_expr(ty)?)
            } else {
                None
            },
            value: lower_expr(&let_stmt.init)?,
        }),
        ast::Stmt::Assign(assign_stmt) => Ok(IrStmt::Assign {
            target: lower_expr(&assign_stmt.target)?,
            value: lower_expr(&assign_stmt.value)?,
        }),
        ast::Stmt::If(if_stmt) => {
            let else_block = if let Some(else_blk) = &if_stmt.else_block {
                Some(match else_blk.as_ref() {
                    ast::ElseBlock::Block(blk) => lower_block(blk)?,
                    ast::ElseBlock::If(if_stmt) => {
                        // Convert else-if to nested if in block
                        IrBlock {
                            statements: vec![lower_stmt(&ast::Stmt::If(if_stmt.clone()))?],
                        }
                    }
                })
            } else {
                None
            };

            Ok(IrStmt::If {
                cond: lower_expr(&if_stmt.cond)?,
                then_block: lower_block(&if_stmt.then_block)?,
                else_block,
            })
        }
        ast::Stmt::While(while_stmt) => Ok(IrStmt::While {
            cond: lower_expr(&while_stmt.cond)?,
            body: lower_block(&while_stmt.body)?,
        }),
        ast::Stmt::Return(ret_stmt) => Ok(IrStmt::Return {
            value: if let Some(val) = &ret_stmt.value {
                Some(lower_expr(val)?)
            } else {
                None
            },
        }),
        ast::Stmt::Expr(expr_stmt) => Ok(IrStmt::Expr(lower_expr(&expr_stmt.expr)?)),
    }
}

fn lower_expr(expr: &ast::Expr) -> Result<IrExpr, LoweringError> {
    match expr {
        ast::Expr::Ident(name, _) => Ok(IrExpr::Var(name.clone())),
        ast::Expr::Literal(lit, _) => Ok(IrExpr::Literal(lower_literal(lit))),
        ast::Expr::Path(segments, _) => Ok(IrExpr::Path(segments.clone())),
        ast::Expr::Call { func, args, .. } => Ok(IrExpr::Call {
            func: Box::new(lower_expr(func)?),
            args: args.iter().map(lower_expr).collect::<Result<Vec<_>, _>>()?,
        }),
        ast::Expr::Field { base, field, .. } => Ok(IrExpr::Field {
            base: Box::new(lower_expr(base)?),
            field: field.clone(),
        }),
        ast::Expr::Record { fields, .. } => {
            let ir_fields: Result<Vec<_>, _> = fields
                .iter()
                .map(|f| Ok((f.name.clone(), lower_expr(&f.value)?)))
                .collect();
            Ok(IrExpr::Record { fields: ir_fields? })
        }
        ast::Expr::BinOp { lhs, op, rhs, .. } => Ok(IrExpr::BinOp {
            op: lower_binop(op),
            left: Box::new(lower_expr(lhs)?),
            right: Box::new(lower_expr(rhs)?),
        }),
        ast::Expr::UnaryOp { op, expr, .. } => Ok(IrExpr::UnaryOp {
            op: lower_unaryop(op),
            expr: Box::new(lower_expr(expr)?),
        }),
        ast::Expr::Paren(expr, _) => lower_expr(expr),
    }
}

fn lower_literal(lit: &ast::Literal) -> IrLiteral {
    match lit {
        ast::Literal::Bool(b) => IrLiteral::Bool(*b),
        ast::Literal::Str(s) => IrLiteral::Str(s.clone()),
        ast::Literal::U16(n) => IrLiteral::U16(*n),
        ast::Literal::U32(n) => IrLiteral::U32(*n),
        ast::Literal::U64(n) => IrLiteral::U64(*n),
        ast::Literal::Int(n) => IrLiteral::Int(*n),
        ast::Literal::Unit => IrLiteral::Unit,
    }
}

fn lower_binop(op: &ast::BinOp) -> IrBinOp {
    match op {
        ast::BinOp::Add => IrBinOp::Add,
        ast::BinOp::Sub => IrBinOp::Sub,
        ast::BinOp::Mul => IrBinOp::Mul,
        ast::BinOp::Div => IrBinOp::Div,
        ast::BinOp::Mod => IrBinOp::Mod,
        ast::BinOp::Eq => IrBinOp::Eq,
        ast::BinOp::Ne => IrBinOp::Ne,
        ast::BinOp::Lt => IrBinOp::Lt,
        ast::BinOp::Le => IrBinOp::Le,
        ast::BinOp::Gt => IrBinOp::Gt,
        ast::BinOp::Ge => IrBinOp::Ge,
        ast::BinOp::And => IrBinOp::And,
        ast::BinOp::Or => IrBinOp::Or,
    }
}

fn lower_unaryop(op: &ast::UnaryOp) -> IrUnaryOp {
    match op {
        ast::UnaryOp::Neg => IrUnaryOp::Neg,
        ast::UnaryOp::Not => IrUnaryOp::Not,
        ast::UnaryOp::Await => IrUnaryOp::Await,
    }
}

fn collect_exports(items: &[ast::Item]) -> Vec<String> {
    items
        .iter()
        .filter_map(|item| match item {
            ast::Item::Type(td) => Some(td.name.clone()),
            ast::Item::Fn(fd) => Some(fd.name.clone()),
            _ => None,
        })
        .collect()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_lower_simple_module() {
        let module = ast::Module::new(
            ast::ModulePath::from_parts(vec!["test".to_string()]),
            Some("1.0.0".to_string()),
            None,
            vec![],
            vec![],
            ast::Span::new(0, 0),
        );

        let ir = lower_to_ir(&module).unwrap();
        assert_eq!(ir.name, "test");
        assert_eq!(ir.version, "1.0.0");
        assert_eq!(ir.imports.len(), 0);
        assert_eq!(ir.types.len(), 0);
        assert_eq!(ir.functions.len(), 0);
    }

    #[test]
    fn test_lower_type_definitions() {
        let type_decl = ast::TypeDecl {
            name: "Point".to_string(),
            expr: ast::TypeExpr::Record(vec![
                ast::RecordField {
                    name: "x".to_string(),
                    ty: Box::new(ast::TypeExpr::Path(vec!["U32".to_string()])),
                    span: ast::Span::new(0, 0),
                },
                ast::RecordField {
                    name: "y".to_string(),
                    ty: Box::new(ast::TypeExpr::Path(vec!["U32".to_string()])),
                    span: ast::Span::new(0, 0),
                },
            ]),
            span: ast::Span::new(0, 0),
        };

        let module = ast::Module::new(
            ast::ModulePath::from_parts(vec!["test".to_string()]),
            None,
            None,
            vec![],
            vec![ast::Item::Type(type_decl)],
            ast::Span::new(0, 0),
        );

        let ir = lower_to_ir(&module).unwrap();
        assert_eq!(ir.types.len(), 1);
        assert_eq!(ir.types[0].name, "Point");
        match &ir.types[0].ty {
            IrType::Record(fields) => {
                assert_eq!(fields.len(), 2);
                assert_eq!(fields[0].0, "x");
                assert_eq!(fields[0].1, IrType::U32);
                assert_eq!(fields[1].0, "y");
                assert_eq!(fields[1].1, IrType::U32);
            }
            _ => panic!("Expected record type"),
        }
    }

    #[test]
    fn test_lower_function_with_params() {
        let fn_decl = ast::FnDecl {
            name: "add".to_string(),
            params: vec![
                ast::Param {
                    name: "a".to_string(),
                    ty: ast::TypeExpr::Path(vec!["U32".to_string()]),
                    span: ast::Span::new(0, 0),
                },
                ast::Param {
                    name: "b".to_string(),
                    ty: ast::TypeExpr::Path(vec!["U32".to_string()]),
                    span: ast::Span::new(0, 0),
                },
            ],
            ret: ast::TypeExpr::Path(vec!["U32".to_string()]),
            effects: vec!["pure".to_string()],
            body: ast::Block {
                raw: String::new(),
                statements: vec![],
                span: ast::Span::new(0, 0),
            },
            span: ast::Span::new(0, 0),
        };

        let module = ast::Module::new(
            ast::ModulePath::from_parts(vec!["test".to_string()]),
            None,
            None,
            vec![],
            vec![ast::Item::Fn(fn_decl)],
            ast::Span::new(0, 0),
        );

        let ir = lower_to_ir(&module).unwrap();
        assert_eq!(ir.functions.len(), 1);
        assert_eq!(ir.functions[0].name, "add");
        assert_eq!(ir.functions[0].params.len(), 2);
        assert_eq!(ir.functions[0].params[0].0, "a");
        assert_eq!(ir.functions[0].params[0].1, IrType::U32);
        assert_eq!(ir.functions[0].return_type, IrType::U32);
        assert_eq!(ir.functions[0].effects, vec!["pure"]);
    }

    #[test]
    fn test_lower_let_statement() {
        let let_stmt = ast::Stmt::Let(ast::LetStmt {
            mutable: false,
            name: "x".to_string(),
            ty: Some(ast::TypeExpr::Path(vec!["U32".to_string()])),
            init: ast::Expr::Literal(ast::Literal::U32(42), ast::Span::new(0, 0)),
            span: ast::Span::new(0, 0),
        });

        let ir_stmt = lower_stmt(&let_stmt).unwrap();
        match ir_stmt {
            IrStmt::Let {
                name,
                mutable,
                ty,
                value,
            } => {
                assert_eq!(name, "x");
                assert!(!mutable);
                assert_eq!(ty, Some(IrType::U32));
                assert_eq!(value, IrExpr::Literal(IrLiteral::U32(42)));
            }
            _ => panic!("Expected let statement"),
        }
    }

    #[test]
    fn test_lower_if_statement() {
        let if_stmt = ast::Stmt::If(ast::IfStmt {
            cond: ast::Expr::Literal(ast::Literal::Bool(true), ast::Span::new(0, 0)),
            then_block: ast::Block {
                raw: String::new(),
                statements: vec![],
                span: ast::Span::new(0, 0),
            },
            else_block: None,
            span: ast::Span::new(0, 0),
        });

        let ir_stmt = lower_stmt(&if_stmt).unwrap();
        match ir_stmt {
            IrStmt::If {
                cond,
                then_block,
                else_block,
            } => {
                assert_eq!(cond, IrExpr::Literal(IrLiteral::Bool(true)));
                assert_eq!(then_block.statements.len(), 0);
                assert!(else_block.is_none());
            }
            _ => panic!("Expected if statement"),
        }
    }

    #[test]
    fn test_lower_return_statement() {
        let ret_stmt = ast::Stmt::Return(ast::ReturnStmt {
            value: Some(ast::Expr::Literal(
                ast::Literal::U32(42),
                ast::Span::new(0, 0),
            )),
            span: ast::Span::new(0, 0),
        });

        let ir_stmt = lower_stmt(&ret_stmt).unwrap();
        match ir_stmt {
            IrStmt::Return { value } => {
                assert_eq!(value, Some(IrExpr::Literal(IrLiteral::U32(42))));
            }
            _ => panic!("Expected return statement"),
        }
    }

    #[test]
    fn test_lower_binary_operation() {
        let expr = ast::Expr::BinOp {
            lhs: Box::new(ast::Expr::Literal(
                ast::Literal::U32(1),
                ast::Span::new(0, 0),
            )),
            op: ast::BinOp::Add,
            rhs: Box::new(ast::Expr::Literal(
                ast::Literal::U32(2),
                ast::Span::new(0, 0),
            )),
            span: ast::Span::new(0, 0),
        };

        let ir_expr = lower_expr(&expr).unwrap();
        match ir_expr {
            IrExpr::BinOp { op, left, right } => {
                assert_eq!(op, IrBinOp::Add);
                assert_eq!(*left, IrExpr::Literal(IrLiteral::U32(1)));
                assert_eq!(*right, IrExpr::Literal(IrLiteral::U32(2)));
            }
            _ => panic!("Expected binary operation"),
        }
    }

    #[test]
    fn test_lower_function_call() {
        let expr = ast::Expr::Call {
            func: Box::new(ast::Expr::Ident("foo".to_string(), ast::Span::new(0, 0))),
            args: vec![
                ast::Expr::Literal(ast::Literal::U32(1), ast::Span::new(0, 0)),
                ast::Expr::Literal(ast::Literal::U32(2), ast::Span::new(0, 0)),
            ],
            span: ast::Span::new(0, 0),
        };

        let ir_expr = lower_expr(&expr).unwrap();
        match ir_expr {
            IrExpr::Call { func, args } => {
                assert_eq!(*func, IrExpr::Var("foo".to_string()));
                assert_eq!(args.len(), 2);
                assert_eq!(args[0], IrExpr::Literal(IrLiteral::U32(1)));
                assert_eq!(args[1], IrExpr::Literal(IrLiteral::U32(2)));
            }
            _ => panic!("Expected function call"),
        }
    }

    #[test]
    fn test_lower_field_access() {
        let expr = ast::Expr::Field {
            base: Box::new(ast::Expr::Ident("obj".to_string(), ast::Span::new(0, 0))),
            field: "x".to_string(),
            span: ast::Span::new(0, 0),
        };

        let ir_expr = lower_expr(&expr).unwrap();
        match ir_expr {
            IrExpr::Field { base, field } => {
                assert_eq!(*base, IrExpr::Var("obj".to_string()));
                assert_eq!(field, "x");
            }
            _ => panic!("Expected field access"),
        }
    }

    #[test]
    fn test_lower_record_literal() {
        let expr = ast::Expr::Record {
            fields: vec![
                ast::RecordInit {
                    name: "x".to_string(),
                    value: ast::Expr::Literal(ast::Literal::U32(1), ast::Span::new(0, 0)),
                    span: ast::Span::new(0, 0),
                },
                ast::RecordInit {
                    name: "y".to_string(),
                    value: ast::Expr::Literal(ast::Literal::U32(2), ast::Span::new(0, 0)),
                    span: ast::Span::new(0, 0),
                },
            ],
            span: ast::Span::new(0, 0),
        };

        let ir_expr = lower_expr(&expr).unwrap();
        match ir_expr {
            IrExpr::Record { fields } => {
                assert_eq!(fields.len(), 2);
                assert_eq!(fields[0].0, "x");
                assert_eq!(fields[0].1, IrExpr::Literal(IrLiteral::U32(1)));
                assert_eq!(fields[1].0, "y");
                assert_eq!(fields[1].1, IrExpr::Literal(IrLiteral::U32(2)));
            }
            _ => panic!("Expected record literal"),
        }
    }

    #[test]
    fn test_ir_preserves_function_effects() {
        let fn_decl = ast::FnDecl {
            name: "async_fn".to_string(),
            params: vec![],
            ret: ast::TypeExpr::Path(vec!["()".to_string()]),
            effects: vec!["async".to_string(), "net".to_string()],
            body: ast::Block {
                raw: String::new(),
                statements: vec![],
                span: ast::Span::new(0, 0),
            },
            span: ast::Span::new(0, 0),
        };

        let module = ast::Module::new(
            ast::ModulePath::from_parts(vec!["test".to_string()]),
            None,
            None,
            vec![],
            vec![ast::Item::Fn(fn_decl)],
            ast::Span::new(0, 0),
        );

        let ir = lower_to_ir(&module).unwrap();
        assert_eq!(ir.functions[0].effects, vec!["async", "net"]);
    }

    #[test]
    fn test_ir_preserves_imports() {
        let import = ast::Import {
            path: "std/http".to_string(),
            alias: Some("H".to_string()),
            only: vec!["listen".to_string(), "Req".to_string()],
            span: ast::Span::new(0, 0),
        };

        let module = ast::Module::new(
            ast::ModulePath::from_parts(vec!["test".to_string()]),
            None,
            None,
            vec![],
            vec![ast::Item::Import(import)],
            ast::Span::new(0, 0),
        );

        let ir = lower_to_ir(&module).unwrap();
        assert_eq!(ir.imports.len(), 1);
        assert_eq!(ir.imports[0].path, "std/http");
        assert_eq!(ir.imports[0].alias, Some("H".to_string()));
        assert_eq!(ir.imports[0].items, vec!["listen", "Req"]);
    }

    #[test]
    fn test_ir_collects_exports() {
        let module = ast::Module::new(
            ast::ModulePath::from_parts(vec!["test".to_string()]),
            None,
            None,
            vec![],
            vec![
                ast::Item::Type(ast::TypeDecl {
                    name: "Point".to_string(),
                    expr: ast::TypeExpr::Path(vec!["U32".to_string()]),
                    span: ast::Span::new(0, 0),
                }),
                ast::Item::Fn(ast::FnDecl {
                    name: "foo".to_string(),
                    params: vec![],
                    ret: ast::TypeExpr::Path(vec!["()".to_string()]),
                    effects: vec![],
                    body: ast::Block {
                        raw: String::new(),
                        statements: vec![],
                        span: ast::Span::new(0, 0),
                    },
                    span: ast::Span::new(0, 0),
                }),
            ],
            ast::Span::new(0, 0),
        );

        let ir = lower_to_ir(&module).unwrap();
        assert_eq!(ir.exports, vec!["Point", "foo"]);
    }

    #[test]
    fn test_complex_nested_expressions() {
        // (a + b) * (c - d)
        let expr = ast::Expr::BinOp {
            lhs: Box::new(ast::Expr::BinOp {
                lhs: Box::new(ast::Expr::Ident("a".to_string(), ast::Span::new(0, 0))),
                op: ast::BinOp::Add,
                rhs: Box::new(ast::Expr::Ident("b".to_string(), ast::Span::new(0, 0))),
                span: ast::Span::new(0, 0),
            }),
            op: ast::BinOp::Mul,
            rhs: Box::new(ast::Expr::BinOp {
                lhs: Box::new(ast::Expr::Ident("c".to_string(), ast::Span::new(0, 0))),
                op: ast::BinOp::Sub,
                rhs: Box::new(ast::Expr::Ident("d".to_string(), ast::Span::new(0, 0))),
                span: ast::Span::new(0, 0),
            }),
            span: ast::Span::new(0, 0),
        };

        let ir_expr = lower_expr(&expr).unwrap();
        match ir_expr {
            IrExpr::BinOp { op, left, right } => {
                assert_eq!(op, IrBinOp::Mul);

                // Check left side (a + b)
                if let IrExpr::BinOp {
                    op: left_op,
                    left: ll,
                    right: lr,
                } = *left
                {
                    assert_eq!(left_op, IrBinOp::Add);
                    assert_eq!(*ll, IrExpr::Var("a".to_string()));
                    assert_eq!(*lr, IrExpr::Var("b".to_string()));
                } else {
                    panic!("Expected binary operation on left");
                }

                // Check right side (c - d)
                if let IrExpr::BinOp {
                    op: right_op,
                    left: rl,
                    right: rr,
                } = *right
                {
                    assert_eq!(right_op, IrBinOp::Sub);
                    assert_eq!(*rl, IrExpr::Var("c".to_string()));
                    assert_eq!(*rr, IrExpr::Var("d".to_string()));
                } else {
                    panic!("Expected binary operation on right");
                }
            }
            _ => panic!("Expected binary operation"),
        }
    }
}
