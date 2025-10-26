//! Tests for std/crypto standard library modules

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
fn test_parse_crypto_hash_relaxed() {
    let path = workspace_root().join("stdlib/crypto/hash.z1r");
    let source = std::fs::read_to_string(&path).unwrap();
    let result = parse_module(&source);
    assert!(
        result.is_ok(),
        "Failed to parse hash.z1r: {:?}",
        result.err()
    );
    let module = result.unwrap();
    assert_eq!(module.path.0, vec!["std", "crypto", "hash"]);
    assert_eq!(module.ctx_budget, Some(256));
    assert_eq!(module.caps, vec!["crypto"]);
}

#[test]
fn test_parse_crypto_hash_compact() {
    let path = workspace_root().join("stdlib/crypto/hash.z1c");
    let source = std::fs::read_to_string(&path).unwrap();
    let result = parse_module(&source);
    assert!(
        result.is_ok(),
        "Failed to parse hash.z1c: {:?}",
        result.err()
    );
}

#[test]
fn test_parse_crypto_hmac_relaxed() {
    let path = workspace_root().join("stdlib/crypto/hmac.z1r");
    let source = std::fs::read_to_string(&path).unwrap();
    let result = parse_module(&source);
    assert!(
        result.is_ok(),
        "Failed to parse hmac.z1r: {:?}",
        result.err()
    );
    let module = result.unwrap();
    assert_eq!(module.path.0, vec!["std", "crypto", "hmac"]);
    assert_eq!(module.ctx_budget, Some(256));
    assert_eq!(module.caps, vec!["crypto"]);
}

#[test]
fn test_parse_crypto_hmac_compact() {
    let path = workspace_root().join("stdlib/crypto/hmac.z1c");
    let source = std::fs::read_to_string(&path).unwrap();
    let result = parse_module(&source);
    assert!(
        result.is_ok(),
        "Failed to parse hmac.z1c: {:?}",
        result.err()
    );
}

#[test]
fn test_parse_crypto_random_relaxed() {
    let path = workspace_root().join("stdlib/crypto/random.z1r");
    let source = std::fs::read_to_string(&path).unwrap();
    let result = parse_module(&source);
    assert!(
        result.is_ok(),
        "Failed to parse random.z1r: {:?}",
        result.err()
    );
    let module = result.unwrap();
    assert_eq!(module.path.0, vec!["std", "crypto", "random"]);
    assert_eq!(module.ctx_budget, Some(256));
    assert_eq!(module.caps, vec!["crypto"]);
}

#[test]
fn test_parse_crypto_random_compact() {
    let path = workspace_root().join("stdlib/crypto/random.z1c");
    let source = std::fs::read_to_string(&path).unwrap();
    let result = parse_module(&source);
    assert!(
        result.is_ok(),
        "Failed to parse random.z1c: {:?}",
        result.err()
    );
}

#[test]
fn test_crypto_hash_has_correct_types() {
    let path = workspace_root().join("stdlib/crypto/hash.z1r");
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

    assert!(
        type_names.contains(&"HashAlgorithm".to_string()),
        "HashAlgorithm type should be defined"
    );
}

#[test]
fn test_crypto_hash_sha256_function_requires_crypto() {
    let path = workspace_root().join("stdlib/crypto/hash.z1r");
    let source = std::fs::read_to_string(&path).unwrap();
    let module = parse_module(&source).unwrap();

    let sha256_fn = module
        .items
        .iter()
        .filter_map(|item| match item {
            z1_ast::Item::Fn(f) if f.name == "sha256" => Some(f),
            _ => None,
        })
        .next();

    assert!(sha256_fn.is_some(), "sha256() function not found");
    let effects_str = format!("{:?}", sha256_fn.unwrap().effects);
    assert!(
        effects_str.contains("Crypto") || effects_str.to_lowercase().contains("crypto"),
        "sha256() should have crypto effect"
    );
}

#[test]
fn test_crypto_hash_sha3_256_function_requires_crypto() {
    let path = workspace_root().join("stdlib/crypto/hash.z1r");
    let source = std::fs::read_to_string(&path).unwrap();
    let module = parse_module(&source).unwrap();

    let sha3_fn = module
        .items
        .iter()
        .filter_map(|item| match item {
            z1_ast::Item::Fn(f) if f.name == "sha3_256" => Some(f),
            _ => None,
        })
        .next();

    assert!(sha3_fn.is_some(), "sha3_256() function not found");
    let effects_str = format!("{:?}", sha3_fn.unwrap().effects);
    assert!(
        effects_str.contains("Crypto") || effects_str.to_lowercase().contains("crypto"),
        "sha3_256() should have crypto effect"
    );
}

#[test]
fn test_crypto_hash_hash_bytes_function_requires_crypto() {
    let path = workspace_root().join("stdlib/crypto/hash.z1r");
    let source = std::fs::read_to_string(&path).unwrap();
    let module = parse_module(&source).unwrap();

    let hash_fn = module
        .items
        .iter()
        .filter_map(|item| match item {
            z1_ast::Item::Fn(f) if f.name == "hashBytes" => Some(f),
            _ => None,
        })
        .next();

    assert!(hash_fn.is_some(), "hashBytes() function not found");
    let effects_str = format!("{:?}", hash_fn.unwrap().effects);
    assert!(
        effects_str.contains("Crypto") || effects_str.to_lowercase().contains("crypto"),
        "hashBytes() should have crypto effect"
    );
}

#[test]
fn test_crypto_hmac_functions_require_crypto() {
    let path = workspace_root().join("stdlib/crypto/hmac.z1r");
    let source = std::fs::read_to_string(&path).unwrap();
    let module = parse_module(&source).unwrap();

    let crypto_fns = vec!["hmacSha256", "verifyHmac"];

    for fn_name in crypto_fns {
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
            effects_str.contains("Crypto") || effects_str.to_lowercase().contains("crypto"),
            "{fn_name} should have crypto effect"
        );
    }
}

#[test]
fn test_crypto_random_functions_require_crypto() {
    let path = workspace_root().join("stdlib/crypto/random.z1r");
    let source = std::fs::read_to_string(&path).unwrap();
    let module = parse_module(&source).unwrap();

    let crypto_fns = vec!["randomBytes", "randomU32", "randomU64", "randomRange"];

    for fn_name in crypto_fns {
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
            effects_str.contains("Crypto") || effects_str.to_lowercase().contains("crypto"),
            "{fn_name} should have crypto effect"
        );
    }
}

#[test]
fn test_round_trip_hash_preserves_semantic_hash() {
    use z1_hash::module_hashes;

    let relaxed_path = workspace_root().join("stdlib/crypto/hash.z1r");
    let compact_path = workspace_root().join("stdlib/crypto/hash.z1c");

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
fn test_round_trip_hmac_preserves_semantic_hash() {
    use z1_hash::module_hashes;

    let relaxed_path = workspace_root().join("stdlib/crypto/hmac.z1r");
    let compact_path = workspace_root().join("stdlib/crypto/hmac.z1c");

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
fn test_round_trip_random_preserves_semantic_hash() {
    use z1_hash::module_hashes;

    let relaxed_path = workspace_root().join("stdlib/crypto/random.z1r");
    let compact_path = workspace_root().join("stdlib/crypto/random.z1c");

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
