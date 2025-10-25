use z1_ast::{FnDecl, Import, Item, Module, ModulePath, Param, Span, TypeDecl, TypeExpr};
use z1_typeck::{check_module, Type, TypeError};

fn make_span() -> Span {
    Span::new(0, 0)
}

fn make_module(items: Vec<Item>) -> Module {
    Module::new(
        ModulePath::from_parts(vec!["test".to_string()]),
        Some("1.0".to_string()),
        Some(128),
        vec!["net".to_string()],
        items,
        make_span(),
    )
}

#[test]
fn test_simple_module_with_type_decl() {
    let type_decl = TypeDecl {
        name: "Point".to_string(),
        expr: TypeExpr::Record(vec![
            z1_ast::RecordField {
                name: "x".to_string(),
                ty: Box::new(TypeExpr::Path(vec!["U32".to_string()])),
                span: make_span(),
            },
            z1_ast::RecordField {
                name: "y".to_string(),
                ty: Box::new(TypeExpr::Path(vec!["U32".to_string()])),
                span: make_span(),
            },
        ]),
        span: make_span(),
    };

    let module = make_module(vec![Item::Type(type_decl)]);

    assert!(check_module(&module).is_ok());
}

#[test]
fn test_function_with_pure_effect() {
    let fn_decl = FnDecl {
        name: "add".to_string(),
        params: vec![
            Param {
                name: "a".to_string(),
                ty: TypeExpr::Path(vec!["U32".to_string()]),
                span: make_span(),
            },
            Param {
                name: "b".to_string(),
                ty: TypeExpr::Path(vec!["U32".to_string()]),
                span: make_span(),
            },
        ],
        ret: TypeExpr::Path(vec!["U32".to_string()]),
        effects: vec!["pure".to_string()],
        body: z1_ast::Block::default(),
        span: make_span(),
    };

    let module = make_module(vec![Item::Fn(fn_decl)]);

    assert!(check_module(&module).is_ok());
}

#[test]
fn test_function_requires_capability() {
    let fn_decl = FnDecl {
        name: "fetch".to_string(),
        params: vec![],
        ret: TypeExpr::Path(vec!["Unit".to_string()]),
        effects: vec!["net".to_string()],
        body: z1_ast::Block::default(),
        span: make_span(),
    };

    let module = make_module(vec![Item::Fn(fn_decl)]);

    // Module has net capability, should succeed
    assert!(check_module(&module).is_ok());
}

#[test]
fn test_function_missing_capability() {
    let fn_decl = FnDecl {
        name: "read_file".to_string(),
        params: vec![],
        ret: TypeExpr::Path(vec!["Unit".to_string()]),
        effects: vec!["fs".to_string()],
        body: z1_ast::Block::default(),
        span: make_span(),
    };

    // Module only has net capability, not fs
    let module = make_module(vec![Item::Fn(fn_decl)]);

    let result = check_module(&module);
    assert!(result.is_err());

    // Check that it's the right error type
    if let Err(TypeError::CapabilityNotGranted { cap }) = result {
        assert_eq!(cap, "fs");
    } else {
        panic!("Expected CapabilityNotGranted error");
    }
}

#[test]
fn test_import_with_alias() {
    let import = Import {
        path: "std/http".to_string(),
        alias: Some("H".to_string()),
        only: vec!["Req".to_string(), "Res".to_string()],
        span: make_span(),
    };

    let module = make_module(vec![Item::Import(import)]);

    // Import processing should succeed
    assert!(check_module(&module).is_ok());
}

#[test]
fn test_function_with_imported_types() {
    // First import the types
    let import = Import {
        path: "std/http".to_string(),
        alias: Some("H".to_string()),
        only: vec!["Req".to_string(), "Res".to_string()],
        span: make_span(),
    };

    // Then use them in a function
    let fn_decl = FnDecl {
        name: "handler".to_string(),
        params: vec![Param {
            name: "req".to_string(),
            ty: TypeExpr::Path(vec!["H".to_string(), "Req".to_string()]),
            span: make_span(),
        }],
        ret: TypeExpr::Path(vec!["H".to_string(), "Res".to_string()]),
        effects: vec!["pure".to_string()],
        body: z1_ast::Block::default(),
        span: make_span(),
    };

    let module = make_module(vec![Item::Import(import), Item::Fn(fn_decl)]);

    // Should succeed - imported types are treated as opaque for MVP
    assert!(check_module(&module).is_ok());
}

#[test]
fn test_structural_record_types() {
    use std::collections::BTreeMap;

    // Create two structurally equal record types
    let mut fields1 = BTreeMap::new();
    fields1.insert("x".to_string(), Box::new(Type::U32));
    fields1.insert("y".to_string(), Box::new(Type::Bool));

    let mut fields2 = BTreeMap::new();
    fields2.insert("y".to_string(), Box::new(Type::Bool));
    fields2.insert("x".to_string(), Box::new(Type::U32));

    let rec1 = Type::Record(fields1);
    let rec2 = Type::Record(fields2);

    // They should be structurally equal (order-independent)
    assert!(rec1.structural_eq(&rec2));
}

#[test]
fn test_http_server_example() {
    // Recreate the http_server.z1c example
    let import = Import {
        path: "std/http".to_string(),
        alias: Some("H".to_string()),
        only: vec!["listen".to_string(), "Req".to_string(), "Res".to_string()],
        span: make_span(),
    };

    let health_type = TypeDecl {
        name: "Health".to_string(),
        expr: TypeExpr::Record(vec![
            z1_ast::RecordField {
                name: "ok".to_string(),
                ty: Box::new(TypeExpr::Path(vec!["Bool".to_string()])),
                span: make_span(),
            },
            z1_ast::RecordField {
                name: "msg".to_string(),
                ty: Box::new(TypeExpr::Path(vec!["Str".to_string()])),
                span: make_span(),
            },
        ]),
        span: make_span(),
    };

    let handler_fn = FnDecl {
        name: "handler".to_string(),
        params: vec![Param {
            name: "q".to_string(),
            ty: TypeExpr::Path(vec!["H".to_string(), "Req".to_string()]),
            span: make_span(),
        }],
        ret: TypeExpr::Path(vec!["H".to_string(), "Res".to_string()]),
        effects: vec!["pure".to_string()],
        body: z1_ast::Block {
            raw: "ret H.Res{ status:200, body:\"ok\" };".to_string(),
            span: make_span(),
        },
        span: make_span(),
    };

    let serve_fn = FnDecl {
        name: "serve".to_string(),
        params: vec![Param {
            name: "p".to_string(),
            ty: TypeExpr::Path(vec!["U16".to_string()]),
            span: make_span(),
        }],
        ret: TypeExpr::Path(vec!["Unit".to_string()]),
        effects: vec!["net".to_string()],
        body: z1_ast::Block {
            raw: "H.listen(p, handler);".to_string(),
            span: make_span(),
        },
        span: make_span(),
    };

    let module = make_module(vec![
        Item::Import(import),
        Item::Type(health_type),
        Item::Fn(handler_fn),
        Item::Fn(serve_fn),
    ]);

    // Should type check successfully
    let result = check_module(&module);
    assert!(result.is_ok(), "Expected Ok but got: {:?}", result);
}
