//! Tests for examples/processor data processing example

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
fn test_parse_processor_compact() {
    let path = workspace_root().join("examples/processor/main.z1c");
    let source = std::fs::read_to_string(&path).unwrap();
    let result = parse_module(&source);
    assert!(
        result.is_ok(),
        "Failed to parse processor main.z1c: {:?}",
        result.err()
    );
    let module = result.unwrap();
    assert_eq!(module.path.0, vec!["example", "processor"]);
    assert_eq!(module.ctx_budget, Some(2048));
    assert_eq!(module.caps, vec!["fs.ro", "fs.rw", "crypto"]);
}

#[test]
fn test_parse_processor_relaxed() {
    let path = workspace_root().join("examples/processor/main.z1r");
    let source = std::fs::read_to_string(&path).unwrap();
    let result = parse_module(&source);
    assert!(
        result.is_ok(),
        "Failed to parse processor main.z1r: {:?}",
        result.err()
    );
    let module = result.unwrap();
    assert_eq!(module.path.0, vec!["example", "processor"]);
    assert_eq!(module.ctx_budget, Some(2048));
    assert_eq!(module.caps, vec!["fs.ro", "fs.rw", "crypto"]);
}

#[test]
fn test_processor_has_correct_imports() {
    let path = workspace_root().join("examples/processor/main.z1r");
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
    assert_eq!(imports[0].path, "std/fs/core");
    assert_eq!(imports[0].alias, Some("Fs".to_string()));
    assert_eq!(imports[1].path, "std/crypto/hash");
    assert_eq!(imports[1].alias, Some("Hash".to_string()));
}

#[test]
fn test_processor_type_definitions() {
    let path = workspace_root().join("examples/processor/main.z1r");
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

    // Should have DataRow, ProcessResult, Statistics, ProcessingPipeline
    assert_eq!(types.len(), 4);

    // Check DataRow is a record type
    assert_eq!(types[0].name, "DataRow");

    // Check ProcessResult is a sum type (Ok | Err)
    assert_eq!(types[1].name, "ProcessResult");

    // Check Statistics is a record type
    assert_eq!(types[2].name, "Statistics");

    // Check ProcessingPipeline is a record type
    assert_eq!(types[3].name, "ProcessingPipeline");
}

#[test]
fn test_processor_function_definitions() {
    let path = workspace_root().join("examples/processor/main.z1r");
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

    // Should have: parseRow, filterRow, transformRow, computeStats,
    // generateHash, processFile, writeResults, generateReport, main
    assert_eq!(functions.len(), 9);

    // Check main function exists
    let main_fn = functions.iter().find(|f| f.name == "main");
    assert!(main_fn.is_some());

    // Check main has correct effect annotations
    let main_fn = main_fn.unwrap();
    assert_eq!(main_fn.effects, vec!["fs", "crypto"]);
}

#[test]
fn test_processor_effect_annotations() {
    let path = workspace_root().join("examples/processor/main.z1r");
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

    // Check parseRow is pure
    let parse_row = functions.iter().find(|f| f.name == "parseRow").unwrap();
    assert_eq!(parse_row.effects, vec!["pure"]);

    // Check generateHash uses crypto effect
    let generate_hash = functions.iter().find(|f| f.name == "generateHash").unwrap();
    assert_eq!(generate_hash.effects, vec!["crypto"]);

    // Check processFile uses fs and crypto effects
    let process_file = functions.iter().find(|f| f.name == "processFile").unwrap();
    assert_eq!(process_file.effects, vec!["fs", "crypto"]);

    // Check writeResults uses fs effect
    let write_results = functions.iter().find(|f| f.name == "writeResults").unwrap();
    assert_eq!(write_results.effects, vec!["fs"]);
}
