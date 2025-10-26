//! Integration tests for z1-ir crate

use z1_ast as ast;
use z1_ir::*;

#[test]
fn test_end_to_end_http_server_lowering() {
    // Manually construct the http_server AST (simplified version)
    // In a real scenario, this would come from the parser

    let module = ast::Module::new(
        ast::ModulePath::from_parts(vec!["http".to_string(), "server".to_string()]),
        Some("1.0".to_string()),
        Some(128),
        vec!["net".to_string()],
        vec![
            // Import: use "std/http" as H only [listen, Req, Res]
            ast::Item::Import(ast::Import {
                path: "std/http".to_string(),
                alias: Some("H".to_string()),
                only: vec!["listen".to_string(), "Req".to_string(), "Res".to_string()],
                span: ast::Span::new(0, 0),
            }),
            // Type: Health = { ok: Bool, msg: Str }
            ast::Item::Type(ast::TypeDecl {
                name: "Health".to_string(),
                expr: ast::TypeExpr::Record(vec![
                    ast::RecordField {
                        name: "ok".to_string(),
                        ty: Box::new(ast::TypeExpr::Path(vec!["Bool".to_string()])),
                        span: ast::Span::new(0, 0),
                    },
                    ast::RecordField {
                        name: "msg".to_string(),
                        ty: Box::new(ast::TypeExpr::Path(vec!["Str".to_string()])),
                        span: ast::Span::new(0, 0),
                    },
                ]),
                span: ast::Span::new(0, 0),
            }),
            // Function: handler
            ast::Item::Fn(ast::FnDecl {
                name: "handler".to_string(),
                params: vec![ast::Param {
                    name: "q".to_string(),
                    ty: ast::TypeExpr::Path(vec!["H".to_string(), "Req".to_string()]),
                    span: ast::Span::new(0, 0),
                }],
                ret: ast::TypeExpr::Path(vec!["H".to_string(), "Res".to_string()]),
                effects: vec!["pure".to_string()],
                body: ast::Block {
                    raw: String::new(),
                    statements: vec![ast::Stmt::Return(ast::ReturnStmt {
                        value: Some(ast::Expr::Record {
                            fields: vec![
                                ast::RecordInit {
                                    name: "status".to_string(),
                                    value: ast::Expr::Literal(
                                        ast::Literal::U32(200),
                                        ast::Span::new(0, 0),
                                    ),
                                    span: ast::Span::new(0, 0),
                                },
                                ast::RecordInit {
                                    name: "body".to_string(),
                                    value: ast::Expr::Literal(
                                        ast::Literal::Str("ok".to_string()),
                                        ast::Span::new(0, 0),
                                    ),
                                    span: ast::Span::new(0, 0),
                                },
                            ],
                            span: ast::Span::new(0, 0),
                        }),
                        span: ast::Span::new(0, 0),
                    })],
                    span: ast::Span::new(0, 0),
                },
                span: ast::Span::new(0, 0),
            }),
            // Function: serve
            ast::Item::Fn(ast::FnDecl {
                name: "serve".to_string(),
                params: vec![ast::Param {
                    name: "p".to_string(),
                    ty: ast::TypeExpr::Path(vec!["U16".to_string()]),
                    span: ast::Span::new(0, 0),
                }],
                ret: ast::TypeExpr::Path(vec!["()".to_string()]),
                effects: vec!["net".to_string()],
                body: ast::Block {
                    raw: String::new(),
                    statements: vec![ast::Stmt::Expr(ast::ExprStmt {
                        expr: ast::Expr::Call {
                            func: Box::new(ast::Expr::Path(
                                vec!["H".to_string(), "listen".to_string()],
                                ast::Span::new(0, 0),
                            )),
                            args: vec![
                                ast::Expr::Ident("p".to_string(), ast::Span::new(0, 0)),
                                ast::Expr::Ident("h".to_string(), ast::Span::new(0, 0)),
                            ],
                            span: ast::Span::new(0, 0),
                        },
                        span: ast::Span::new(0, 0),
                    })],
                    span: ast::Span::new(0, 0),
                },
                span: ast::Span::new(0, 0),
            }),
        ],
        ast::Span::new(0, 0),
    );

    // Lower to IR
    let ir = lower_to_ir(&module).expect("Failed to lower to IR");

    // Verify IR structure
    assert_eq!(ir.name, "http.server");
    assert_eq!(ir.version, "1.0");

    // Verify imports
    assert_eq!(ir.imports.len(), 1);
    assert_eq!(ir.imports[0].path, "std/http");
    assert_eq!(ir.imports[0].alias, Some("H".to_string()));
    assert_eq!(ir.imports[0].items.len(), 3);

    // Verify types
    assert_eq!(ir.types.len(), 1);
    assert_eq!(ir.types[0].name, "Health");
    match &ir.types[0].ty {
        IrType::Record(fields) => {
            assert_eq!(fields.len(), 2);
            assert_eq!(fields[0].0, "ok");
            assert_eq!(fields[0].1, IrType::Bool);
            assert_eq!(fields[1].0, "msg");
            assert_eq!(fields[1].1, IrType::Str);
        }
        _ => panic!("Expected record type"),
    }

    // Verify functions
    assert_eq!(ir.functions.len(), 2);

    // Check handler function
    assert_eq!(ir.functions[0].name, "handler");
    assert_eq!(ir.functions[0].params.len(), 1);
    assert_eq!(ir.functions[0].params[0].0, "q");
    assert_eq!(
        ir.functions[0].params[0].1,
        IrType::Named("H.Req".to_string())
    );
    assert_eq!(ir.functions[0].effects, vec!["pure"]);

    // Check serve function
    assert_eq!(ir.functions[1].name, "serve");
    assert_eq!(ir.functions[1].params.len(), 1);
    assert_eq!(ir.functions[1].params[0].0, "p");
    assert_eq!(ir.functions[1].params[0].1, IrType::U16);
    assert_eq!(ir.functions[1].effects, vec!["net"]);

    // Verify exports
    assert_eq!(ir.exports.len(), 3);
    assert!(ir.exports.contains(&"Health".to_string()));
    assert!(ir.exports.contains(&"handler".to_string()));
    assert!(ir.exports.contains(&"serve".to_string()));
}
