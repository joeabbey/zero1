//! Tests for std/time standard library modules

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
fn test_parse_time_core_relaxed() {
    let path = workspace_root().join("stdlib/time/core.z1r");
    let source = std::fs::read_to_string(&path).unwrap();
    let result = parse_module(&source);
    assert!(
        result.is_ok(),
        "Failed to parse core.z1r: {:?}",
        result.err()
    );
    let module = result.unwrap();
    assert_eq!(module.path.0, vec!["std", "time", "core"]);
    assert_eq!(module.ctx_budget, Some(256));
    assert_eq!(module.caps, vec!["time"]);
}

#[test]
fn test_parse_time_core_compact() {
    let path = workspace_root().join("stdlib/time/core.z1c");
    let source = std::fs::read_to_string(&path).unwrap();
    let result = parse_module(&source);
    assert!(
        result.is_ok(),
        "Failed to parse core.z1c: {:?}",
        result.err()
    );
}

#[test]
fn test_parse_time_timer_relaxed() {
    let path = workspace_root().join("stdlib/time/timer.z1r");
    let source = std::fs::read_to_string(&path).unwrap();
    let result = parse_module(&source);
    assert!(
        result.is_ok(),
        "Failed to parse timer.z1r: {:?}",
        result.err()
    );
    let module = result.unwrap();
    assert_eq!(module.path.0, vec!["std", "time", "timer"]);
}

#[test]
fn test_parse_time_timer_compact() {
    let path = workspace_root().join("stdlib/time/timer.z1c");
    let source = std::fs::read_to_string(&path).unwrap();
    let result = parse_module(&source);
    assert!(
        result.is_ok(),
        "Failed to parse timer.z1c: {:?}",
        result.err()
    );
}

#[test]
fn test_time_core_has_correct_types() {
    let path = workspace_root().join("stdlib/time/core.z1r");
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

    assert!(type_names.contains(&"Timestamp".to_string()));
    assert!(type_names.contains(&"Duration".to_string()));
    assert!(type_names.contains(&"DateTime".to_string()));
}

#[test]
fn test_time_timer_has_correct_types() {
    let path = workspace_root().join("stdlib/time/timer.z1r");
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

    assert!(type_names.contains(&"Timer".to_string()));
}

#[test]
fn test_time_core_now_function_requires_time_capability() {
    let path = workspace_root().join("stdlib/time/core.z1r");
    let source = std::fs::read_to_string(&path).unwrap();
    let module = parse_module(&source).unwrap();

    let now_fn = module
        .items
        .iter()
        .filter_map(|item| match item {
            z1_ast::Item::Fn(f) if f.name == "now" => Some(f),
            _ => None,
        })
        .next();

    assert!(now_fn.is_some(), "now() function not found");
    let effects_str = format!("{:?}", now_fn.unwrap().effects);
    assert!(
        effects_str.contains("Time") || effects_str.to_lowercase().contains("time"),
        "now() should have time effect"
    );
}

#[test]
fn test_time_core_sleep_requires_async() {
    let path = workspace_root().join("stdlib/time/core.z1r");
    let source = std::fs::read_to_string(&path).unwrap();
    let module = parse_module(&source).unwrap();

    let sleep_fn = module
        .items
        .iter()
        .filter_map(|item| match item {
            z1_ast::Item::Fn(f) if f.name == "sleep" => Some(f),
            _ => None,
        })
        .next();

    assert!(sleep_fn.is_some(), "sleep() function not found");
    let effects_str = format!("{:?}", sleep_fn.unwrap().effects);
    assert!(
        effects_str.contains("Async") || effects_str.to_lowercase().contains("async"),
        "sleep() should have async effect"
    );
}

#[test]
fn test_time_core_pure_functions_marked_correctly() {
    let path = workspace_root().join("stdlib/time/core.z1r");
    let source = std::fs::read_to_string(&path).unwrap();
    let module = parse_module(&source).unwrap();

    let pure_fns = vec![
        "add",
        "subtract",
        "fromMillis",
        "toMillis",
        "format",
        "parse",
    ];

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
fn test_example_time_demo_parses() {
    let path = workspace_root().join("examples/time-demo/main.z1r");
    let source = std::fs::read_to_string(&path).unwrap();
    let result = parse_module(&source);
    assert!(
        result.is_ok(),
        "Failed to parse time-demo example: {:?}",
        result.err()
    );
    let module = result.unwrap();
    assert_eq!(module.path.0, vec!["demo", "time"]);
    assert_eq!(module.caps, vec!["time"]);
}

#[test]
fn test_round_trip_core_preserves_hash() {
    use z1_hash::module_hashes;

    let relaxed_path = workspace_root().join("stdlib/time/core.z1r");
    let compact_path = workspace_root().join("stdlib/time/core.z1c");

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
fn test_round_trip_timer_preserves_hash() {
    use z1_hash::module_hashes;

    let relaxed_path = workspace_root().join("stdlib/time/timer.z1r");
    let compact_path = workspace_root().join("stdlib/time/timer.z1c");

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
