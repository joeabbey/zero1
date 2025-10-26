//! Tests for std/env standard library modules

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
fn test_parse_env_vars_relaxed() {
    let path = workspace_root().join("stdlib/env/vars.z1r");
    let source = std::fs::read_to_string(&path).unwrap();
    let result = parse_module(&source);
    assert!(
        result.is_ok(),
        "Failed to parse vars.z1r: {:?}",
        result.err()
    );
    let module = result.unwrap();
    assert_eq!(module.path.0, vec!["std", "env", "vars"]);
    assert_eq!(module.ctx_budget, Some(256));
    assert_eq!(module.caps, vec!["env"]);
}

#[test]
fn test_parse_env_vars_compact() {
    let path = workspace_root().join("stdlib/env/vars.z1c");
    let source = std::fs::read_to_string(&path).unwrap();
    let result = parse_module(&source);
    assert!(
        result.is_ok(),
        "Failed to parse vars.z1c: {:?}",
        result.err()
    );
}

#[test]
fn test_parse_env_args_relaxed() {
    let path = workspace_root().join("stdlib/env/args.z1r");
    let source = std::fs::read_to_string(&path).unwrap();
    let result = parse_module(&source);
    assert!(
        result.is_ok(),
        "Failed to parse args.z1r: {:?}",
        result.err()
    );
    let module = result.unwrap();
    assert_eq!(module.path.0, vec!["std", "env", "args"]);
}

#[test]
fn test_parse_env_args_compact() {
    let path = workspace_root().join("stdlib/env/args.z1c");
    let source = std::fs::read_to_string(&path).unwrap();
    let result = parse_module(&source);
    assert!(
        result.is_ok(),
        "Failed to parse args.z1c: {:?}",
        result.err()
    );
}

#[test]
fn test_parse_env_process_relaxed() {
    let path = workspace_root().join("stdlib/env/process.z1r");
    let source = std::fs::read_to_string(&path).unwrap();
    let result = parse_module(&source);
    assert!(
        result.is_ok(),
        "Failed to parse process.z1r: {:?}",
        result.err()
    );
    let module = result.unwrap();
    assert_eq!(module.path.0, vec!["std", "env", "process"]);
}

#[test]
fn test_parse_env_process_compact() {
    let path = workspace_root().join("stdlib/env/process.z1c");
    let source = std::fs::read_to_string(&path).unwrap();
    let result = parse_module(&source);
    assert!(
        result.is_ok(),
        "Failed to parse process.z1c: {:?}",
        result.err()
    );
}

#[test]
fn test_env_vars_has_correct_types() {
    let path = workspace_root().join("stdlib/env/vars.z1r");
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

    assert!(type_names.contains(&"EnvVar".to_string()));
}

#[test]
fn test_env_args_has_correct_types() {
    let path = workspace_root().join("stdlib/env/args.z1r");
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

    assert!(type_names.contains(&"Args".to_string()));
}

#[test]
fn test_env_process_has_correct_types() {
    let path = workspace_root().join("stdlib/env/process.z1r");
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

    assert!(type_names.contains(&"ProcessInfo".to_string()));
}

#[test]
fn test_env_vars_get_var_requires_env_capability() {
    let path = workspace_root().join("stdlib/env/vars.z1r");
    let source = std::fs::read_to_string(&path).unwrap();
    let module = parse_module(&source).unwrap();

    let get_var_fn = module
        .items
        .iter()
        .filter_map(|item| match item {
            z1_ast::Item::Fn(f) if f.name == "getVar" => Some(f),
            _ => None,
        })
        .next();

    assert!(get_var_fn.is_some(), "getVar() function not found");
    let effects_str = format!("{:?}", get_var_fn.unwrap().effects);
    assert!(
        effects_str.contains("Env") || effects_str.to_lowercase().contains("env"),
        "getVar() should have env effect"
    );
}

#[test]
fn test_env_args_parse_flags_is_pure() {
    let path = workspace_root().join("stdlib/env/args.z1r");
    let source = std::fs::read_to_string(&path).unwrap();
    let module = parse_module(&source).unwrap();

    let parse_flags_fn = module
        .items
        .iter()
        .filter_map(|item| match item {
            z1_ast::Item::Fn(f) if f.name == "parseFlags" => Some(f),
            _ => None,
        })
        .next();

    assert!(parse_flags_fn.is_some(), "parseFlags() function not found");
    let effects_str = format!("{:?}", parse_flags_fn.unwrap().effects);
    assert!(
        effects_str.contains("Pure") || effects_str.to_lowercase().contains("pure"),
        "parseFlags() should have pure effect"
    );
}

#[test]
fn test_env_process_exit_requires_unsafe() {
    let path = workspace_root().join("stdlib/env/process.z1r");
    let source = std::fs::read_to_string(&path).unwrap();
    let module = parse_module(&source).unwrap();

    let exit_fn = module
        .items
        .iter()
        .filter_map(|item| match item {
            z1_ast::Item::Fn(f) if f.name == "exit" => Some(f),
            _ => None,
        })
        .next();

    assert!(exit_fn.is_some(), "exit() function not found");
    let effects_str = format!("{:?}", exit_fn.unwrap().effects);
    assert!(
        effects_str.contains("Unsafe") || effects_str.to_lowercase().contains("unsafe"),
        "exit() should have unsafe effect"
    );
}

#[test]
fn test_env_process_get_pid_requires_env() {
    let path = workspace_root().join("stdlib/env/process.z1r");
    let source = std::fs::read_to_string(&path).unwrap();
    let module = parse_module(&source).unwrap();

    let get_pid_fn = module
        .items
        .iter()
        .filter_map(|item| match item {
            z1_ast::Item::Fn(f) if f.name == "getPid" => Some(f),
            _ => None,
        })
        .next();

    assert!(get_pid_fn.is_some(), "getPid() function not found");
    let effects_str = format!("{:?}", get_pid_fn.unwrap().effects);
    assert!(
        effects_str.contains("Env") || effects_str.to_lowercase().contains("env"),
        "getPid() should have env effect"
    );
}

#[test]
fn test_round_trip_vars_preserves_hash() {
    use z1_hash::module_hashes;

    let relaxed_path = workspace_root().join("stdlib/env/vars.z1r");
    let compact_path = workspace_root().join("stdlib/env/vars.z1c");

    let relaxed = std::fs::read_to_string(&relaxed_path).unwrap();
    let compact = std::fs::read_to_string(&compact_path).unwrap();

    let relaxed_module = parse_module(&relaxed).unwrap();
    let compact_module = parse_module(&compact).unwrap();

    let relaxed_hashes = module_hashes(&relaxed_module);
    let compact_hashes = module_hashes(&compact_module);

    assert_eq!(
        relaxed_hashes.semantic, compact_hashes.semantic,
        "Semantic hash should be identical for relaxed and compact formats"
    );
}

#[test]
fn test_round_trip_args_preserves_hash() {
    use z1_hash::module_hashes;

    let relaxed_path = workspace_root().join("stdlib/env/args.z1r");
    let compact_path = workspace_root().join("stdlib/env/args.z1c");

    let relaxed = std::fs::read_to_string(&relaxed_path).unwrap();
    let compact = std::fs::read_to_string(&compact_path).unwrap();

    let relaxed_module = parse_module(&relaxed).unwrap();
    let compact_module = parse_module(&compact).unwrap();

    let relaxed_hashes = module_hashes(&relaxed_module);
    let compact_hashes = module_hashes(&compact_module);

    assert_eq!(
        relaxed_hashes.semantic, compact_hashes.semantic,
        "Semantic hash should be identical for relaxed and compact formats"
    );
}

#[test]
fn test_round_trip_process_preserves_hash() {
    use z1_hash::module_hashes;

    let relaxed_path = workspace_root().join("stdlib/env/process.z1r");
    let compact_path = workspace_root().join("stdlib/env/process.z1c");

    let relaxed = std::fs::read_to_string(&relaxed_path).unwrap();
    let compact = std::fs::read_to_string(&compact_path).unwrap();

    let relaxed_module = parse_module(&relaxed).unwrap();
    let compact_module = parse_module(&compact).unwrap();

    let relaxed_hashes = module_hashes(&relaxed_module);
    let compact_hashes = module_hashes(&compact_module);

    assert_eq!(
        relaxed_hashes.semantic, compact_hashes.semantic,
        "Semantic hash should be identical for relaxed and compact formats"
    );
}
