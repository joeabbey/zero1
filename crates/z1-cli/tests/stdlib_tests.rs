//! Tests for Zero1 standard library modules
//!
//! This test suite verifies that stdlib modules parse, format, and type-check correctly.

use std::fs;
use std::path::PathBuf;

fn workspace_root() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .parent()
        .unwrap()
        .parent()
        .unwrap()
        .to_path_buf()
}

fn read_stdlib_file(path: &str) -> String {
    let full_path = workspace_root().join(path);
    fs::read_to_string(&full_path)
        .unwrap_or_else(|e| panic!("Failed to read {}: {}", full_path.display(), e))
}

// ========== Parse/Format Tests ==========

#[test]
fn test_http_server_compact_parses() {
    let source = read_stdlib_file("stdlib/http/server.z1c");
    let result = z1_parse::parse_module(&source);
    assert!(result.is_ok(), "Failed to parse server.z1c: {:?}", result);

    let module = result.unwrap();
    assert_eq!(module.path.0, vec!["std", "http", "server"]);
    assert_eq!(module.version, Some("1.0".to_string()));
    assert_eq!(module.ctx_budget, Some(512));
    assert_eq!(module.caps, vec!["net"]);
}

#[test]
fn test_http_server_relaxed_parses() {
    let source = read_stdlib_file("stdlib/http/server.z1r");
    let result = z1_parse::parse_module(&source);
    assert!(result.is_ok(), "Failed to parse server.z1r: {:?}", result);

    let module = result.unwrap();
    assert_eq!(module.path.0, vec!["std", "http", "server"]);
    assert_eq!(module.version, Some("1.0".to_string()));
}

#[test]
fn test_http_client_compact_parses() {
    let source = read_stdlib_file("stdlib/http/client.z1c");
    let result = z1_parse::parse_module(&source);
    assert!(result.is_ok(), "Failed to parse client.z1c: {:?}", result);

    let module = result.unwrap();
    assert_eq!(module.path.0, vec!["std", "http", "client"]);
    assert_eq!(module.ctx_budget, Some(256));
}

#[test]
fn test_http_client_relaxed_parses() {
    let source = read_stdlib_file("stdlib/http/client.z1r");
    let result = z1_parse::parse_module(&source);
    assert!(result.is_ok(), "Failed to parse client.z1r: {:?}", result);
}

#[test]
fn test_http_server_round_trip_preserves_hash() {
    let compact = read_stdlib_file("stdlib/http/server.z1c");
    let relaxed = read_stdlib_file("stdlib/http/server.z1r");

    let compact_module = z1_parse::parse_module(&compact).unwrap();
    let relaxed_module = z1_parse::parse_module(&relaxed).unwrap();

    let compact_hashes = z1_hash::module_hashes(&compact_module);
    let relaxed_hashes = z1_hash::module_hashes(&relaxed_module);

    assert_eq!(
        compact_hashes.semantic, relaxed_hashes.semantic,
        "SemHash should be identical for compact and relaxed versions"
    );
}

#[test]
fn test_http_client_round_trip_preserves_hash() {
    let compact = read_stdlib_file("stdlib/http/client.z1c");
    let relaxed = read_stdlib_file("stdlib/http/client.z1r");

    let compact_module = z1_parse::parse_module(&compact).unwrap();
    let relaxed_module = z1_parse::parse_module(&relaxed).unwrap();

    let compact_hashes = z1_hash::module_hashes(&compact_module);
    let relaxed_hashes = z1_hash::module_hashes(&relaxed_module);

    assert_eq!(
        compact_hashes.semantic, relaxed_hashes.semantic,
        "SemHash should be identical for compact and relaxed versions"
    );
}

// ========== Semantic Tests ==========

#[test]
fn test_http_server_has_correct_types() {
    let source = read_stdlib_file("stdlib/http/server.z1c");
    let module = z1_parse::parse_module(&source).unwrap();

    // Count type declarations
    let type_count = module
        .items
        .iter()
        .filter(|item| matches!(item, z1_ast::Item::Type(_)))
        .count();
    assert_eq!(type_count, 3, "Expected 3 type declarations (Req, Res, HS)");

    // Verify type names exist
    let type_names: Vec<String> = module
        .items
        .iter()
        .filter_map(|item| match item {
            z1_ast::Item::Type(t) => Some(t.name.clone()),
            _ => None,
        })
        .collect();

    assert!(type_names.contains(&"Req".to_string()));
    assert!(type_names.contains(&"Res".to_string()));
    assert!(type_names.contains(&"HS".to_string()));
}

#[test]
fn test_http_client_has_correct_types() {
    let source = read_stdlib_file("stdlib/http/client.z1c");
    let module = z1_parse::parse_module(&source).unwrap();

    let type_count = module
        .items
        .iter()
        .filter(|item| matches!(item, z1_ast::Item::Type(_)))
        .count();
    assert_eq!(type_count, 2, "Expected 2 type declarations (M, C)");
}

#[test]
fn test_listen_function_requires_net_capability() {
    let source = read_stdlib_file("stdlib/http/server.z1c");
    let module = z1_parse::parse_module(&source).unwrap();

    // Find listen function
    let listen_fn = module.items.iter().find_map(|item| match item {
        z1_ast::Item::Fn(f) if f.name == "l" => Some(f),
        _ => None,
    });

    assert!(listen_fn.is_some(), "listen function not found");
    let listen_fn = listen_fn.unwrap();

    // Verify effects include net and async
    assert!(
        listen_fn.effects.contains(&"net".to_string()),
        "listen function should have net effect"
    );
    assert!(
        listen_fn.effects.contains(&"async".to_string()),
        "listen function should have async effect"
    );
}

#[test]
fn test_all_pure_functions_marked_correctly() {
    let source = read_stdlib_file("stdlib/http/server.z1c");
    let module = z1_parse::parse_module(&source).unwrap();

    // Functions that should be pure
    let pure_functions = vec!["cs", "gm", "gp", "ss", "sb"];

    for fn_name in pure_functions {
        let func = module.items.iter().find_map(|item| match item {
            z1_ast::Item::Fn(f) if f.name == fn_name => Some(f),
            _ => None,
        });

        assert!(func.is_some(), "Function {} not found", fn_name);
        let func = func.unwrap();
        assert!(
            func.effects.contains(&"pure".to_string()),
            "Function {} should be marked as pure",
            fn_name
        );
    }
}

#[test]
fn test_example_application_parses() {
    let source = read_stdlib_file("examples/http-hello/main.z1c");
    let result = z1_parse::parse_module(&source);
    assert!(result.is_ok(), "Failed to parse example application: {:?}", result);

    let module = result.unwrap();
    assert_eq!(module.path.0, vec!["hello", "http"]);
    assert_eq!(module.caps, vec!["net"]);
}

#[test]
fn test_http_server_functions_count() {
    let source = read_stdlib_file("stdlib/http/server.z1c");
    let module = z1_parse::parse_module(&source).unwrap();

    let fn_count = module
        .items
        .iter()
        .filter(|item| matches!(item, z1_ast::Item::Fn(_)))
        .count();

    assert_eq!(fn_count, 7, "Expected 7 function declarations in HTTP server");
}

#[test]
fn test_http_client_functions_count() {
    let source = read_stdlib_file("stdlib/http/client.z1c");
    let module = z1_parse::parse_module(&source).unwrap();

    let fn_count = module
        .items
        .iter()
        .filter(|item| matches!(item, z1_ast::Item::Fn(_)))
        .count();

    assert_eq!(fn_count, 5, "Expected 5 function declarations in HTTP client");
}

#[test]
fn test_http_server_symbol_map_exists() {
    let source = read_stdlib_file("stdlib/http/server.z1c");
    let module = z1_parse::parse_module(&source).unwrap();

    let has_symbol_map = module
        .items
        .iter()
        .any(|item| matches!(item, z1_ast::Item::Symbol(_)));

    assert!(has_symbol_map, "HTTP server should have a symbol map");
}

#[test]
fn test_http_client_all_functions_require_net() {
    let source = read_stdlib_file("stdlib/http/client.z1c");
    let module = z1_parse::parse_module(&source).unwrap();

    // All HTTP client functions should require net capability
    let client_functions = vec!["g", "p", "pu", "d", "ft"];

    for fn_name in client_functions {
        let func = module.items.iter().find_map(|item| match item {
            z1_ast::Item::Fn(f) if f.name == fn_name => Some(f),
            _ => None,
        });

        assert!(func.is_some(), "Function {} not found", fn_name);
        let func = func.unwrap();
        assert!(
            func.effects.contains(&"net".to_string()),
            "Function {} should have net effect",
            fn_name
        );
        assert!(
            func.effects.contains(&"async".to_string()),
            "Function {} should have async effect",
            fn_name
        );
    }
}

#[test]
fn test_http_server_context_budget() {
    let source = read_stdlib_file("stdlib/http/server.z1c");
    let module = z1_parse::parse_module(&source).unwrap();

    assert_eq!(
        module.ctx_budget,
        Some(512),
        "HTTP server should have context budget of 512"
    );
}

#[test]
fn test_http_client_context_budget() {
    let source = read_stdlib_file("stdlib/http/client.z1c");
    let module = z1_parse::parse_module(&source).unwrap();

    assert_eq!(
        module.ctx_budget,
        Some(256),
        "HTTP client should have context budget of 256"
    );
}
