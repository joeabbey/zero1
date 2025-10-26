//! Tests for std/fs standard library modules

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
fn test_parse_fs_core_relaxed() {
    let path = workspace_root().join("stdlib/fs/core.z1r");
    let source = std::fs::read_to_string(&path).unwrap();
    let result = parse_module(&source);
    assert!(
        result.is_ok(),
        "Failed to parse core.z1r: {:?}",
        result.err()
    );
    let module = result.unwrap();
    assert_eq!(module.path.0, vec!["std", "fs", "core"]);
    assert_eq!(module.ctx_budget, Some(256));
    assert!(module.caps.contains(&"fs.ro".to_string()));
    assert!(module.caps.contains(&"fs.rw".to_string()));
}

#[test]
fn test_parse_fs_core_compact() {
    let path = workspace_root().join("stdlib/fs/core.z1c");
    let source = std::fs::read_to_string(&path).unwrap();
    let result = parse_module(&source);
    assert!(
        result.is_ok(),
        "Failed to parse core.z1c: {:?}",
        result.err()
    );
}

#[test]
fn test_parse_fs_dir_relaxed() {
    let path = workspace_root().join("stdlib/fs/dir.z1r");
    let source = std::fs::read_to_string(&path).unwrap();
    let result = parse_module(&source);
    assert!(
        result.is_ok(),
        "Failed to parse dir.z1r: {:?}",
        result.err()
    );
    let module = result.unwrap();
    assert_eq!(module.path.0, vec!["std", "fs", "dir"]);
}

#[test]
fn test_parse_fs_dir_compact() {
    let path = workspace_root().join("stdlib/fs/dir.z1c");
    let source = std::fs::read_to_string(&path).unwrap();
    let result = parse_module(&source);
    assert!(
        result.is_ok(),
        "Failed to parse dir.z1c: {:?}",
        result.err()
    );
}

#[test]
fn test_parse_fs_path_relaxed() {
    let path = workspace_root().join("stdlib/fs/path.z1r");
    let source = std::fs::read_to_string(&path).unwrap();
    let result = parse_module(&source);
    assert!(
        result.is_ok(),
        "Failed to parse path.z1r: {:?}",
        result.err()
    );
    let module = result.unwrap();
    assert_eq!(module.path.0, vec!["std", "fs", "path"]);
    assert_eq!(module.ctx_budget, Some(128));
    assert_eq!(module.caps.len(), 0);
}

#[test]
fn test_parse_fs_path_compact() {
    let path = workspace_root().join("stdlib/fs/path.z1c");
    let source = std::fs::read_to_string(&path).unwrap();
    let result = parse_module(&source);
    assert!(
        result.is_ok(),
        "Failed to parse path.z1c: {:?}",
        result.err()
    );
}

#[test]
fn test_fs_core_has_correct_types() {
    let path = workspace_root().join("stdlib/fs/core.z1r");
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

    assert!(type_names.contains(&"ReadResult".to_string()));
    assert!(type_names.contains(&"WriteResult".to_string()));
}

#[test]
fn test_fs_dir_has_correct_types() {
    let path = workspace_root().join("stdlib/fs/dir.z1r");
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

    assert!(type_names.contains(&"ListResult".to_string()));
    assert!(type_names.contains(&"DirResult".to_string()));
}

#[test]
fn test_fs_path_has_correct_types() {
    let path = workspace_root().join("stdlib/fs/path.z1r");
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

    assert!(type_names.contains(&"ExtOption".to_string()));
}

#[test]
fn test_fs_core_read_text_has_fs_effect() {
    let path = workspace_root().join("stdlib/fs/core.z1r");
    let source = std::fs::read_to_string(&path).unwrap();
    let module = parse_module(&source).unwrap();

    let read_fn = module
        .items
        .iter()
        .filter_map(|item| match item {
            z1_ast::Item::Fn(f) if f.name == "readText" => Some(f),
            _ => None,
        })
        .next();

    assert!(read_fn.is_some(), "readText() function not found");
    let effects_str = format!("{:?}", read_fn.unwrap().effects);
    assert!(
        effects_str.contains("Fs") || effects_str.to_lowercase().contains("fs"),
        "readText() should have fs effect"
    );
}

#[test]
fn test_fs_core_write_text_has_fs_effect() {
    let path = workspace_root().join("stdlib/fs/core.z1r");
    let source = std::fs::read_to_string(&path).unwrap();
    let module = parse_module(&source).unwrap();

    let write_fn = module
        .items
        .iter()
        .filter_map(|item| match item {
            z1_ast::Item::Fn(f) if f.name == "writeText" => Some(f),
            _ => None,
        })
        .next();

    assert!(write_fn.is_some(), "writeText() function not found");
    let effects_str = format!("{:?}", write_fn.unwrap().effects);
    assert!(
        effects_str.contains("Fs") || effects_str.to_lowercase().contains("fs"),
        "writeText() should have fs effect"
    );
}

#[test]
fn test_fs_dir_list_has_fs_effect() {
    let path = workspace_root().join("stdlib/fs/dir.z1r");
    let source = std::fs::read_to_string(&path).unwrap();
    let module = parse_module(&source).unwrap();

    let list_fn = module
        .items
        .iter()
        .filter_map(|item| match item {
            z1_ast::Item::Fn(f) if f.name == "list" => Some(f),
            _ => None,
        })
        .next();

    assert!(list_fn.is_some(), "list() function not found");
    let effects_str = format!("{:?}", list_fn.unwrap().effects);
    assert!(
        effects_str.contains("Fs") || effects_str.to_lowercase().contains("fs"),
        "list() should have fs effect"
    );
}

#[test]
fn test_fs_path_pure_functions_marked_correctly() {
    let path = workspace_root().join("stdlib/fs/path.z1r");
    let source = std::fs::read_to_string(&path).unwrap();
    let module = parse_module(&source).unwrap();

    let pure_fns = vec!["join", "basename", "dirname", "extension"];

    for fn_name in pure_fns {
        let func = module
            .items
            .iter()
            .filter_map(|item| match item {
                z1_ast::Item::Fn(f) if f.name == fn_name => Some(f),
                _ => None,
            })
            .next();

        assert!(func.is_some(), "{fn_name} function not found");
        let effects_str = format!("{:?}", func.unwrap().effects);
        assert!(
            effects_str.contains("Pure") || effects_str.to_lowercase().contains("pure"),
            "{fn_name} should have pure effect"
        );
    }
}

#[test]
fn test_fs_core_capabilities_include_fs_ro() {
    let path = workspace_root().join("stdlib/fs/core.z1r");
    let source = std::fs::read_to_string(&path).unwrap();
    let module = parse_module(&source).unwrap();

    assert!(
        module.caps.contains(&"fs.ro".to_string()),
        "Module should require fs.ro capability"
    );
}

#[test]
fn test_fs_core_capabilities_include_fs_rw() {
    let path = workspace_root().join("stdlib/fs/core.z1r");
    let source = std::fs::read_to_string(&path).unwrap();
    let module = parse_module(&source).unwrap();

    assert!(
        module.caps.contains(&"fs.rw".to_string()),
        "Module should require fs.rw capability"
    );
}

#[test]
fn test_fs_path_requires_no_capabilities() {
    let path = workspace_root().join("stdlib/fs/path.z1r");
    let source = std::fs::read_to_string(&path).unwrap();
    let module = parse_module(&source).unwrap();

    assert_eq!(
        module.caps.len(),
        0,
        "Path module should require no capabilities (pure functions)"
    );
}

// Note: Round-trip hash preservation tests are omitted for MVP stdlib modules.
// The compact and relaxed versions are hand-written and semantically equivalent,
// but may have minor differences in type alias usage that affect semantic hashing.
// This is acceptable for an MVP standard library.
