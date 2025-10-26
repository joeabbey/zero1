//! Tests for examples/scheduler task scheduling example

use std::path::PathBuf;
use z1_ast::Item;
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
fn test_parse_scheduler_compact() {
    let path = workspace_root().join("examples/scheduler/main.z1c");
    let source = std::fs::read_to_string(&path).unwrap();
    let result = parse_module(&source);
    assert!(
        result.is_ok(),
        "Failed to parse scheduler main.z1c: {:?}",
        result.err()
    );
    let module = result.unwrap();
    assert_eq!(module.path.0, vec!["example", "scheduler"]);
    assert_eq!(module.ctx_budget, Some(1024));
    assert_eq!(module.caps, vec!["time", "async"]);
}

#[test]
fn test_parse_scheduler_relaxed() {
    let path = workspace_root().join("examples/scheduler/main.z1r");
    let source = std::fs::read_to_string(&path).unwrap();
    let result = parse_module(&source);
    assert!(
        result.is_ok(),
        "Failed to parse scheduler main.z1r: {:?}",
        result.err()
    );
    let module = result.unwrap();
    assert_eq!(module.path.0, vec!["example", "scheduler"]);
    assert_eq!(module.ctx_budget, Some(1024));
    assert_eq!(module.caps, vec!["time", "async"]);
}

#[test]
fn test_scheduler_has_correct_imports() {
    let path = workspace_root().join("examples/scheduler/main.z1r");
    let source = std::fs::read_to_string(&path).unwrap();
    let module = parse_module(&source).unwrap();

    let imports: Vec<_> = module
        .items
        .iter()
        .filter_map(|item| {
            if let Item::Import(import) = item {
                Some(import)
            } else {
                None
            }
        })
        .collect();

    assert_eq!(imports.len(), 2);
    assert_eq!(imports[0].path, "std/time/core");
    assert_eq!(imports[0].alias, Some("Time".to_string()));
    assert_eq!(imports[1].path, "std/time/timer");
    assert_eq!(imports[1].alias, Some("Timer".to_string()));
}

#[test]
fn test_scheduler_type_definitions() {
    let path = workspace_root().join("examples/scheduler/main.z1r");
    let source = std::fs::read_to_string(&path).unwrap();
    let module = parse_module(&source).unwrap();

    let types: Vec<_> = module
        .items
        .iter()
        .filter_map(|item| {
            if let Item::Type(type_decl) = item {
                Some(type_decl)
            } else {
                None
            }
        })
        .collect();

    // Should have TaskStatus, TaskPriority, Task, ScheduledTask, TaskScheduler
    assert_eq!(types.len(), 5);

    // Check TaskStatus is a sum type
    assert_eq!(types[0].name, "TaskStatus");

    // Check TaskPriority is a sum type
    assert_eq!(types[1].name, "TaskPriority");

    // Check Task is a record type
    assert_eq!(types[2].name, "Task");

    // Check ScheduledTask is a record type
    assert_eq!(types[3].name, "ScheduledTask");

    // Check TaskScheduler is a record type
    assert_eq!(types[4].name, "TaskScheduler");
}

#[test]
fn test_scheduler_function_definitions() {
    let path = workspace_root().join("examples/scheduler/main.z1r");
    let source = std::fs::read_to_string(&path).unwrap();
    let module = parse_module(&source).unwrap();

    let functions: Vec<_> = module
        .items
        .iter()
        .filter_map(|item| {
            if let Item::Fn(fn_decl) = item {
                Some(fn_decl)
            } else {
                None
            }
        })
        .collect();

    // Should have: createTask, scheduleOnce, scheduleRecurring, cancelTask,
    // executeTask, checkTask, runScheduler, main
    assert_eq!(functions.len(), 8);

    // Check main function exists
    let main_fn = functions.iter().find(|f| f.name == "main");
    assert!(main_fn.is_some());

    // Check main has correct effect annotations
    let main_fn = main_fn.unwrap();
    assert_eq!(main_fn.effects, vec!["time", "async"]);
}

#[test]
fn test_scheduler_effect_annotations() {
    let path = workspace_root().join("examples/scheduler/main.z1r");
    let source = std::fs::read_to_string(&path).unwrap();
    let module = parse_module(&source).unwrap();

    let functions: Vec<_> = module
        .items
        .iter()
        .filter_map(|item| {
            if let Item::Fn(fn_decl) = item {
                Some(fn_decl)
            } else {
                None
            }
        })
        .collect();

    // Check createTask is pure
    let create_task = functions.iter().find(|f| f.name == "createTask").unwrap();
    assert_eq!(create_task.effects, vec!["pure"]);

    // Check executeTask uses time effect
    let execute_task = functions.iter().find(|f| f.name == "executeTask").unwrap();
    assert_eq!(execute_task.effects, vec!["time"]);

    // Check runScheduler uses time and async effects
    let run_scheduler = functions.iter().find(|f| f.name == "runScheduler").unwrap();
    assert_eq!(run_scheduler.effects, vec!["time", "async"]);
}
