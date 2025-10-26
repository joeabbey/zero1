//! Tests for examples/cli-tool example

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
fn test_parse_cli_tool_compact() {
    let path = workspace_root().join("examples/cli-tool/main.z1c");
    let source = std::fs::read_to_string(&path).unwrap();
    let result = parse_module(&source);
    assert!(
        result.is_ok(),
        "Failed to parse cli-tool main.z1c: {:?}",
        result.err()
    );
}

#[test]
fn test_parse_cli_tool_relaxed() {
    let path = workspace_root().join("examples/cli-tool/main.z1r");
    let source = std::fs::read_to_string(&path).unwrap();
    let result = parse_module(&source);
    assert!(
        result.is_ok(),
        "Failed to parse cli-tool main.z1r: {:?}",
        result.err()
    );
}

#[test]
fn test_cli_tool_module_metadata() {
    let path = workspace_root().join("examples/cli-tool/main.z1r");
    let source = std::fs::read_to_string(&path).unwrap();
    let module = parse_module(&source).unwrap();

    // Verify module path
    assert_eq!(module.path.0, vec!["example", "cli", "processor"]);

    // Verify version
    assert_eq!(module.version, Some("1.0".to_string()));

    // Verify context budget
    assert_eq!(module.ctx_budget, Some(768));

    // Verify capabilities
    assert_eq!(module.caps.len(), 3);
    assert!(module.caps.contains(&"env".to_string()));
    assert!(module.caps.contains(&"fs.ro".to_string()));
    assert!(module.caps.contains(&"fs.rw".to_string()));
}

#[test]
fn test_cli_tool_has_correct_types() {
    let path = workspace_root().join("examples/cli-tool/main.z1r");
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

    // Should have Config, ProcessStats, ProcessResult types
    assert!(
        type_names.len() >= 3,
        "Expected at least 3 type definitions"
    );

    assert!(type_names.contains(&"Config".to_string()));
    assert!(type_names.contains(&"ProcessStats".to_string()));
    assert!(type_names.contains(&"ProcessResult".to_string()));
}

#[test]
fn test_cli_tool_has_processing_functions() {
    let path = workspace_root().join("examples/cli-tool/main.z1r");
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

    // Argument and config handling
    assert!(fn_names.contains(&"parseArgs".to_string()));
    assert!(fn_names.contains(&"loadConfig".to_string()));
    assert!(fn_names.contains(&"validateConfig".to_string()));

    // File processing
    assert!(fn_names.contains(&"processFile".to_string()));
    assert!(fn_names.contains(&"processLine".to_string()));
    assert!(fn_names.contains(&"transformText".to_string()));

    // I/O operations
    assert!(fn_names.contains(&"writeOutput".to_string()));

    // Utilities
    assert!(fn_names.contains(&"countLines".to_string()));
    assert!(fn_names.contains(&"printStats".to_string()));
    assert!(fn_names.contains(&"printHelp".to_string()));

    // Entry point
    assert!(fn_names.contains(&"main".to_string()));
}

#[test]
fn test_cli_tool_imports_stdlib_modules() {
    let path = workspace_root().join("examples/cli-tool/main.z1r");
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

    // Should import 4 stdlib modules
    assert_eq!(imports.len(), 4);

    let import_paths: Vec<_> = imports.iter().map(|i| i.path.as_str()).collect();
    assert!(import_paths.contains(&"std/env/args"));
    assert!(import_paths.contains(&"std/env/vars"));
    assert!(import_paths.contains(&"std/env/process"));
    assert!(import_paths.contains(&"std/fs/core"));

    // Verify aliases
    let args_import = imports.iter().find(|i| i.path == "std/env/args").unwrap();
    assert_eq!(args_import.alias, Some("args".to_string()));

    let fs_import = imports.iter().find(|i| i.path == "std/fs/core").unwrap();
    assert_eq!(fs_import.alias, Some("fs".to_string()));
}

#[test]
fn test_cli_tool_main_function_effects() {
    let path = workspace_root().join("examples/cli-tool/main.z1r");
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

    // Should have env, fs, unsafe effects
    assert!(main_fn.effects.contains(&"env".to_string()));
    assert!(main_fn.effects.contains(&"fs".to_string()));
    assert!(main_fn.effects.contains(&"unsafe".to_string()));
}

#[test]
fn test_cli_tool_process_file_effects() {
    let path = workspace_root().join("examples/cli-tool/main.z1r");
    let source = std::fs::read_to_string(&path).unwrap();
    let module = parse_module(&source).unwrap();

    // Find processFile function
    let process_fn = module
        .items
        .iter()
        .filter_map(|item| match item {
            z1_ast::Item::Fn(f) => Some(f),
            _ => None,
        })
        .find(|f| f.name == "processFile")
        .expect("processFile function not found");

    // Should have fs effect for file I/O
    assert!(process_fn.effects.contains(&"fs".to_string()));
}

#[test]
fn test_cli_tool_pure_transformations() {
    let path = workspace_root().join("examples/cli-tool/main.z1r");
    let source = std::fs::read_to_string(&path).unwrap();
    let module = parse_module(&source).unwrap();

    // Transformation functions should be pure
    let pure_functions = ["processLine", "transformText", "printHelp", "printStats"];

    let functions: Vec<&z1_ast::FnDecl> = module
        .items
        .iter()
        .filter_map(|item| match item {
            z1_ast::Item::Fn(f) => Some(f),
            _ => None,
        })
        .collect();

    for fn_name in &pure_functions {
        let func = functions
            .iter()
            .find(|f| f.name == *fn_name)
            .unwrap_or_else(|| panic!("Function {fn_name} not found"));

        assert!(
            func.effects.contains(&"pure".to_string()),
            "Function {fn_name} should have pure effect"
        );
    }
}

#[test]
fn test_cli_tool_compact_relaxed_semantic_equivalence() {
    let compact_path = workspace_root().join("examples/cli-tool/main.z1c");
    let relaxed_path = workspace_root().join("examples/cli-tool/main.z1r");

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

    // Import count should match
    let compact_imports = compact_module
        .items
        .iter()
        .filter(|item| matches!(item, z1_ast::Item::Import(_)))
        .count();
    let relaxed_imports = relaxed_module
        .items
        .iter()
        .filter(|item| matches!(item, z1_ast::Item::Import(_)))
        .count();
    assert_eq!(compact_imports, relaxed_imports);
}
