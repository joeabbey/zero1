//! Tests for examples/api-server example

use std::path::PathBuf;
use z1_parse::parse_module;

fn workspace_root() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .parent()
        .unwrap()
        .parent()
        .unwrap()
        .to_path_buf()
}

#[test]
fn test_parse_api_server_compact() {
    let path = workspace_root().join("examples/api-server/main.z1c");
    let source = std::fs::read_to_string(&path).unwrap();
    let result = parse_module(&source);
    assert!(
        result.is_ok(),
        "Failed to parse api-server main.z1c: {:?}",
        result.err()
    );
}

#[test]
fn test_parse_api_server_relaxed() {
    let path = workspace_root().join("examples/api-server/main.z1r");
    let source = std::fs::read_to_string(&path).unwrap();
    let result = parse_module(&source);
    assert!(
        result.is_ok(),
        "Failed to parse api-server main.z1r: {:?}",
        result.err()
    );
}

#[test]
fn test_api_server_module_metadata() {
    let path = workspace_root().join("examples/api-server/main.z1r");
    let source = std::fs::read_to_string(&path).unwrap();
    let module = parse_module(&source).unwrap();

    // Verify module path
    assert_eq!(module.path.0, vec!["example", "api", "server"]);

    // Verify version
    assert_eq!(module.version, Some("1.0".to_string()));

    // Verify context budget
    assert_eq!(module.ctx_budget, Some(1024));

    // Verify capabilities
    assert_eq!(module.caps.len(), 2);
    assert!(module.caps.contains(&"net".to_string()));
    assert!(module.caps.contains(&"fs.ro".to_string()));
}

#[test]
fn test_api_server_has_correct_types() {
    let path = workspace_root().join("examples/api-server/main.z1r");
    let source = std::fs::read_to_string(&path).unwrap();
    let module = parse_module(&source).unwrap();

    let type_names: Vec<String> = module
        .items
        .iter()
        .filter_map(|item| match item {
            z1_ast::Item::Type(t) => Some(t.name.clone()),
            _ => None,
        })
        .collect();

    // Should have User, UserList, Response, StatusResponse, Router types
    assert!(
        type_names.len() >= 5,
        "Expected at least 5 type definitions"
    );

    assert!(type_names.contains(&"User".to_string()));
    assert!(type_names.contains(&"UserList".to_string()));
    assert!(type_names.contains(&"Response".to_string()));
    assert!(type_names.contains(&"StatusResponse".to_string()));
    assert!(type_names.contains(&"Router".to_string()));
}

#[test]
fn test_api_server_has_handler_functions() {
    let path = workspace_root().join("examples/api-server/main.z1r");
    let source = std::fs::read_to_string(&path).unwrap();
    let module = parse_module(&source).unwrap();

    let fn_names: Vec<String> = module
        .items
        .iter()
        .filter_map(|item| match item {
            z1_ast::Item::Fn(f) => Some(f.name.clone()),
            _ => None,
        })
        .collect();

    // Core handlers
    assert!(fn_names.contains(&"handleStatus".to_string()));
    assert!(fn_names.contains(&"handleListUsers".to_string()));
    assert!(fn_names.contains(&"handleGetUser".to_string()));
    assert!(fn_names.contains(&"handleCreateUser".to_string()));
    assert!(fn_names.contains(&"handleUpdateUser".to_string()));
    assert!(fn_names.contains(&"handleDeleteUser".to_string()));
    assert!(fn_names.contains(&"handleStatic".to_string()));

    // Routing
    assert!(fn_names.contains(&"routeRequest".to_string()));
    assert!(fn_names.contains(&"handleRequest".to_string()));

    // Entry point
    assert!(fn_names.contains(&"main".to_string()));
}

#[test]
fn test_api_server_imports_http_server() {
    let path = workspace_root().join("examples/api-server/main.z1r");
    let source = std::fs::read_to_string(&path).unwrap();
    let module = parse_module(&source).unwrap();

    let imports: Vec<&z1_ast::Import> = module
        .items
        .iter()
        .filter_map(|item| match item {
            z1_ast::Item::Import(imp) => Some(imp),
            _ => None,
        })
        .collect();

    // Should import std/http/server
    assert_eq!(imports.len(), 1);
    let import = imports[0];
    assert_eq!(import.path, "std/http/server");
    assert_eq!(import.alias, Some("http".to_string()));

    // Should have only clause
    let only_items = &import.only;
    assert!(only_items.contains(&"HttpRequest".to_string()));
    assert!(only_items.contains(&"HttpResponse".to_string()));
    assert!(only_items.contains(&"createServer".to_string()));
    assert!(only_items.contains(&"listen".to_string()));
}

#[test]
fn test_api_server_main_function_effects() {
    let path = workspace_root().join("examples/api-server/main.z1r");
    let source = std::fs::read_to_string(&path).unwrap();
    let module = parse_module(&source).unwrap();

    // Find main function
    let main_fn = module
        .items
        .iter()
        .filter_map(|item| match item {
            z1_ast::Item::Fn(f) => Some(f),
            _ => None,
        })
        .find(|f| f.name == "main")
        .expect("main function not found");

    // Should have net, async, fs effects
    assert!(main_fn.effects.contains(&"net".to_string()));
    assert!(main_fn.effects.contains(&"async".to_string()));
    assert!(main_fn.effects.contains(&"fs".to_string()));
}

#[test]
fn test_api_server_compact_relaxed_semantic_equivalence() {
    let compact_path = workspace_root().join("examples/api-server/main.z1c");
    let relaxed_path = workspace_root().join("examples/api-server/main.z1r");

    let compact_source = std::fs::read_to_string(&compact_path).unwrap();
    let relaxed_source = std::fs::read_to_string(&relaxed_path).unwrap();

    let compact_module = parse_module(&compact_source).unwrap();
    let relaxed_module = parse_module(&relaxed_source).unwrap();

    // Module metadata should match
    assert_eq!(compact_module.path, relaxed_module.path);
    assert_eq!(compact_module.version, relaxed_module.version);
    assert_eq!(compact_module.ctx_budget, relaxed_module.ctx_budget);
    assert_eq!(compact_module.caps, relaxed_module.caps);

    // Item counts should match
    assert_eq!(compact_module.items.len(), relaxed_module.items.len());

    // Type count should match
    let compact_types = compact_module
        .items
        .iter()
        .filter(|item| matches!(item, z1_ast::Item::Type(_)))
        .count();
    let relaxed_types = relaxed_module
        .items
        .iter()
        .filter(|item| matches!(item, z1_ast::Item::Type(_)))
        .count();
    assert_eq!(compact_types, relaxed_types);

    // Function count should match
    let compact_fns = compact_module
        .items
        .iter()
        .filter(|item| matches!(item, z1_ast::Item::Fn(_)))
        .count();
    let relaxed_fns = relaxed_module
        .items
        .iter()
        .filter(|item| matches!(item, z1_ast::Item::Fn(_)))
        .count();
    assert_eq!(compact_fns, relaxed_fns);
}
