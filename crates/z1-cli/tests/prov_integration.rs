//! Integration tests for provenance CLI commands.

use chrono::Utc;
use std::fs;
use std::path::PathBuf;
use std::process::Command;
use tempfile::TempDir;
use z1_prov::{keygen, sign_entry, ProvenanceChain, ProvenanceChainExt, ProvenanceEntry};

/// Get the path to the z1-cli binary.
fn cli_bin() -> PathBuf {
    let mut path = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
    path.pop(); // Go up to workspace root
    path.pop();
    path.push("target");
    path.push("debug");
    path.push("z1-cli");
    path
}

/// Create a test provenance chain with signatures.
fn create_test_chain_with_signatures(dir: &TempDir) -> (PathBuf, PathBuf) {
    let (private_key, public_key) = keygen();

    let mut chain = ProvenanceChain::new();

    let entry = ProvenanceEntry {
        entry_id: "cell:test@v1".to_string(),
        prev: None,
        actor: "agent:test/1.0".to_string(),
        model: "test-model-2025".to_string(),
        prompt_sha3: "a".repeat(64),
        prompt_excerpt: "Test prompt for integration test".to_string(),
        tools: vec!["z1-fmt".to_string()],
        diff_sha3: "b".repeat(64),
        timestamp: Utc::now(),
        signatures: vec![],
    };

    chain.append(entry.clone()).unwrap();

    // Add signature to the entry
    let sig = sign_entry(&entry, &private_key, "test:signer");
    chain.entries[0].signatures.push(sig);
    chain.update_merkle_root();

    let chain_path = dir.path().join("test_chain.z1p");
    chain.save_to_file(&chain_path).unwrap();

    // Save public key
    let keys_path = dir.path().join("keys.json");
    let keys_json = serde_json::json!({
        "test:signer": hex::encode(public_key)
    });
    fs::write(
        &keys_path,
        serde_json::to_string_pretty(&keys_json).unwrap(),
    )
    .unwrap();

    (chain_path, keys_path)
}

#[test]
fn test_prov_keygen() {
    let output = Command::new(cli_bin())
        .args(["prov", "keygen"])
        .output()
        .expect("failed to execute z1-cli");

    assert!(output.status.success());
    let stdout = String::from_utf8_lossy(&output.stdout);
    assert!(stdout.contains("Generated Ed25519 Keypair"));
    assert!(stdout.contains("Private Key"));
    assert!(stdout.contains("Public Key"));
}

#[test]
fn test_prov_keygen_to_file() {
    let dir = TempDir::new().unwrap();
    let output_path = dir.path().join("keypair.json");

    let output = Command::new(cli_bin())
        .args(["prov", "keygen", output_path.to_str().unwrap()])
        .output()
        .expect("failed to execute z1-cli");

    assert!(output.status.success());
    assert!(output_path.exists());

    let content = fs::read_to_string(&output_path).unwrap();
    let keypair: serde_json::Value = serde_json::from_str(&content).unwrap();
    assert!(keypair["private_key"].is_string());
    assert!(keypair["public_key"].is_string());
}

#[test]
fn test_prov_log() {
    let dir = TempDir::new().unwrap();
    let (chain_path, _) = create_test_chain_with_signatures(&dir);

    let output = Command::new(cli_bin())
        .args(["prov", "log", chain_path.to_str().unwrap()])
        .output()
        .expect("failed to execute z1-cli");

    assert!(output.status.success());
    let stdout = String::from_utf8_lossy(&output.stdout);
    assert!(stdout.contains("Provenance Chain"));
    assert!(stdout.contains("Entry 1"));
    assert!(stdout.contains("cell:test@v1"));
    assert!(stdout.contains("test:signer"));
}

#[test]
fn test_prov_verify_valid_chain() {
    let dir = TempDir::new().unwrap();
    let (chain_path, keys_path) = create_test_chain_with_signatures(&dir);

    let output = Command::new(cli_bin())
        .args([
            "prov",
            "verify",
            chain_path.to_str().unwrap(),
            "--keys",
            keys_path.to_str().unwrap(),
        ])
        .output()
        .expect("failed to execute z1-cli");

    assert!(output.status.success());
    let stdout = String::from_utf8_lossy(&output.stdout);
    assert!(stdout.contains("Merkle chain structure valid"));
    assert!(stdout.contains("VALID"));
}

#[test]
fn test_prov_verify_without_keys() {
    let dir = TempDir::new().unwrap();
    let (chain_path, _) = create_test_chain_with_signatures(&dir);

    let output = Command::new(cli_bin())
        .args(["prov", "verify", chain_path.to_str().unwrap()])
        .output()
        .expect("failed to execute z1-cli");

    assert!(output.status.success());
    let stdout = String::from_utf8_lossy(&output.stdout);
    assert!(stdout.contains("Merkle chain structure valid"));
    assert!(stdout.contains("VALID"));
}

#[test]
fn test_prov_verify_missing_file() {
    let output = Command::new(cli_bin())
        .args(["prov", "verify", "/nonexistent/file.z1p"])
        .output()
        .expect("failed to execute z1-cli");

    assert!(!output.status.success());
}

#[test]
fn test_prov_log_empty_chain() {
    let dir = TempDir::new().unwrap();
    let chain_path = dir.path().join("empty_chain.z1p");

    let chain = ProvenanceChain::new();
    chain.save_to_file(&chain_path).unwrap();

    let output = Command::new(cli_bin())
        .args(["prov", "log", chain_path.to_str().unwrap()])
        .output()
        .expect("failed to execute z1-cli");

    assert!(output.status.success());
    let stdout = String::from_utf8_lossy(&output.stdout);
    assert!(stdout.contains("Chain is empty"));
}
