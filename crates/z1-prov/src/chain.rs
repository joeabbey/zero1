//! Provenance chain operations including Merkle root calculation and file I/O.

use crate::types::{ProvenanceChain, ProvenanceEntry};
use sha3::{Digest, Sha3_256};
use std::fs;
use std::path::Path;
use thiserror::Error;

/// Errors that can occur during chain operations.
#[derive(Debug, Error)]
pub enum ChainError {
    #[error("I/O error: {0}")]
    Io(#[from] std::io::Error),

    #[error("Serialization error: {0}")]
    Serialization(#[from] serde_json::Error),

    #[error("Invalid entry: {0}")]
    InvalidEntry(String),
}

/// Compute the hash of a provenance entry for Merkle chain linking.
pub fn compute_entry_hash(entry: &ProvenanceEntry) -> String {
    let json = serde_json::to_string(entry).expect("entry serialization failed");
    let mut hasher = Sha3_256::new();
    hasher.update(json.as_bytes());
    hex::encode(hasher.finalize())
}

/// Extension trait for ProvenanceChain with additional operations.
pub trait ProvenanceChainExt {
    /// Append a new entry to the chain.
    ///
    /// Automatically sets the `prev` field to link to the previous entry.
    /// Returns the hash of the appended entry.
    fn append(&mut self, entry: ProvenanceEntry) -> Result<String, ChainError>;

    /// Compute the Merkle root of all entries.
    fn compute_merkle_root(&self) -> String;

    /// Update the merkle_root field by recomputing it.
    fn update_merkle_root(&mut self);

    /// Load a provenance chain from a JSON file.
    fn load_from_file<P: AsRef<Path>>(path: P) -> Result<ProvenanceChain, ChainError>;

    /// Save the provenance chain to a JSON file.
    fn save_to_file<P: AsRef<Path>>(&self, path: P) -> Result<(), ChainError>;

    /// Get an entry by index.
    fn get(&self, index: usize) -> Option<&ProvenanceEntry>;
}

impl ProvenanceChainExt for ProvenanceChain {
    fn append(&mut self, mut entry: ProvenanceEntry) -> Result<String, ChainError> {
        // Validate entry
        if entry.entry_id.is_empty() {
            return Err(ChainError::InvalidEntry(
                "entry_id cannot be empty".to_string(),
            ));
        }
        if entry.prompt_excerpt.len() > 200 {
            return Err(ChainError::InvalidEntry(
                "prompt_excerpt exceeds 200 characters".to_string(),
            ));
        }

        // Set prev to hash of last entry (if any)
        if let Some(last_entry) = self.entries.last() {
            let prev_hash = compute_entry_hash(last_entry);
            entry.prev = Some(prev_hash);
        } else {
            // First entry has no previous
            entry.prev = None;
        }

        // Compute hash of new entry
        let entry_hash = compute_entry_hash(&entry);

        // Add to chain
        self.entries.push(entry);

        // Recompute Merkle root
        self.merkle_root = self.compute_merkle_root();

        Ok(entry_hash)
    }

    fn compute_merkle_root(&self) -> String {
        if self.entries.is_empty() {
            return String::new();
        }

        let mut hasher = Sha3_256::new();

        for entry in &self.entries {
            let entry_hash = compute_entry_hash(entry);
            hasher.update(entry_hash.as_bytes());
        }

        let result = hasher.finalize();
        hex::encode(result)
    }

    fn update_merkle_root(&mut self) {
        self.merkle_root = self.compute_merkle_root();
    }

    fn load_from_file<P: AsRef<Path>>(path: P) -> Result<ProvenanceChain, ChainError> {
        let contents = fs::read_to_string(path)?;
        let chain: ProvenanceChain = serde_json::from_str(&contents)?;
        Ok(chain)
    }

    fn save_to_file<P: AsRef<Path>>(&self, path: P) -> Result<(), ChainError> {
        let json = serde_json::to_string_pretty(&self)?;
        fs::write(path, json)?;
        Ok(())
    }

    fn get(&self, index: usize) -> Option<&ProvenanceEntry> {
        self.entries.get(index)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::types::{ProvenanceChain, ProvenanceEntry, Signature};
    use chrono::Utc;

    fn create_test_entry(id: &str, actor: &str) -> ProvenanceEntry {
        ProvenanceEntry {
            entry_id: id.to_string(),
            prev: None,
            actor: actor.to_string(),
            model: "test-model".to_string(),
            prompt_sha3: "test_prompt_hash".to_string(),
            prompt_excerpt: "Test prompt".to_string(),
            tools: vec!["test-tool".to_string()],
            diff_sha3: "test_diff_hash".to_string(),
            timestamp: Utc::now(),
            signatures: vec![Signature {
                by: actor.to_string(),
                sig: "ed25519:test_sig".to_string(),
            }],
        }
    }

    #[test]
    fn test_append_single_entry() {
        let mut chain = ProvenanceChain::new();
        let entry = create_test_entry("cell:test@v1", "agent:test");

        let hash = chain.append(entry.clone()).unwrap();
        assert!(!hash.is_empty());
        assert_eq!(chain.len(), 1);
        assert!(!chain.merkle_root.is_empty());
    }

    #[test]
    fn test_append_multiple_entries() {
        let mut chain = ProvenanceChain::new();

        let entry1 = create_test_entry("cell:test@v1", "agent:test");
        let entry2 = create_test_entry("cell:test@v2", "agent:test");
        let entry3 = create_test_entry("cell:test@v3", "agent:test");

        chain.append(entry1).unwrap();
        chain.append(entry2).unwrap();
        chain.append(entry3).unwrap();

        assert_eq!(chain.len(), 3);

        // Verify prev links are set
        assert!(chain.entries[0].prev.is_none());
        assert!(chain.entries[1].prev.is_some());
        assert!(chain.entries[2].prev.is_some());
    }

    #[test]
    fn test_compute_merkle_root_empty_chain() {
        let chain = ProvenanceChain::new();
        let root = chain.compute_merkle_root();
        assert_eq!(root, "");
    }

    #[test]
    fn test_compute_merkle_root_deterministic() {
        let mut chain1 = ProvenanceChain::new();
        let mut chain2 = ProvenanceChain::new();

        let entry = create_test_entry("cell:test@v1", "agent:test");

        chain1.append(entry.clone()).unwrap();
        chain2.append(entry).unwrap();

        assert_eq!(chain1.merkle_root, chain2.merkle_root);
    }

    #[test]
    fn test_compute_merkle_root_changes_with_entry() {
        let mut chain = ProvenanceChain::new();
        let entry1 = create_test_entry("cell:test@v1", "agent:test");

        chain.append(entry1).unwrap();
        let root1 = chain.merkle_root.clone();

        let entry2 = create_test_entry("cell:test@v2", "agent:test");
        chain.append(entry2).unwrap();
        let root2 = chain.merkle_root.clone();

        assert_ne!(root1, root2);
    }

    #[test]
    fn test_entry_hash_deterministic() {
        let entry = create_test_entry("cell:test@v1", "agent:test");
        let hash1 = compute_entry_hash(&entry);
        let hash2 = compute_entry_hash(&entry);

        assert_eq!(hash1, hash2);
    }

    #[test]
    fn test_entry_hash_changes_with_content() {
        let entry1 = create_test_entry("cell:test@v1", "agent:test");
        let entry2 = create_test_entry("cell:test@v2", "agent:test");

        let hash1 = compute_entry_hash(&entry1);
        let hash2 = compute_entry_hash(&entry2);

        assert_ne!(hash1, hash2);
    }

    #[test]
    fn test_save_and_load_file() {
        let temp_file = "/tmp/test_provenance_chain.json";

        let mut chain = ProvenanceChain::new();
        let entry = create_test_entry("cell:test@v1", "agent:test");
        chain.append(entry).unwrap();

        chain.save_to_file(temp_file).unwrap();

        let loaded = ProvenanceChain::load_from_file(temp_file).unwrap();
        assert_eq!(chain.merkle_root, loaded.merkle_root);
        assert_eq!(chain.entries.len(), loaded.entries.len());

        fs::remove_file(temp_file).ok();
    }

    #[test]
    fn test_append_to_loaded_chain() {
        let temp_file = "/tmp/test_provenance_append_chain.json";

        let mut chain = ProvenanceChain::new();
        let entry1 = create_test_entry("cell:test@v1", "agent:test");
        chain.append(entry1).unwrap();
        chain.save_to_file(temp_file).unwrap();

        let mut loaded = ProvenanceChain::load_from_file(temp_file).unwrap();
        let entry2 = create_test_entry("cell:test@v2", "agent:test");
        loaded.append(entry2).unwrap();

        assert_eq!(loaded.len(), 2);

        fs::remove_file(temp_file).ok();
    }

    #[test]
    fn test_round_trip_serialization() {
        let mut chain = ProvenanceChain::new();
        let entry = create_test_entry("cell:test@v1", "agent:test");

        chain.append(entry).unwrap();

        let json = serde_json::to_string(&chain).unwrap();
        let deserialized: ProvenanceChain = serde_json::from_str(&json).unwrap();

        assert_eq!(chain.merkle_root, deserialized.merkle_root);
        assert_eq!(chain.entries.len(), deserialized.entries.len());
    }

    #[test]
    fn test_update_merkle_root() {
        let mut chain = ProvenanceChain::new();
        let entry = create_test_entry("cell:test@v1", "agent:test");

        // add_entry automatically updates merkle_root
        chain.add_entry(entry);

        // After add_entry, merkle_root should be set
        assert!(!chain.merkle_root.is_empty());

        chain.update_merkle_root();
        assert!(!chain.merkle_root.is_empty());
    }

    #[test]
    fn test_very_long_chain() {
        let mut chain = ProvenanceChain::new();

        for i in 1..=100 {
            let entry = create_test_entry(&format!("cell:test@v{}", i), "agent:test");
            chain.append(entry).unwrap();
        }

        assert_eq!(chain.len(), 100);
        assert!(!chain.merkle_root.is_empty());
    }

    #[test]
    fn test_validate_entry_empty_id() {
        let mut chain = ProvenanceChain::new();
        let mut entry = create_test_entry("cell:test@v1", "agent:test");
        entry.entry_id = "".to_string();

        assert!(chain.append(entry).is_err());
    }

    #[test]
    fn test_validate_entry_long_excerpt() {
        let mut chain = ProvenanceChain::new();
        let mut entry = create_test_entry("cell:test@v1", "agent:test");
        entry.prompt_excerpt = "a".repeat(201);

        assert!(chain.append(entry).is_err());
    }

    #[test]
    fn test_validate_entry_max_excerpt_length() {
        let mut chain = ProvenanceChain::new();
        let mut entry = create_test_entry("cell:test@v1", "agent:test");
        entry.prompt_excerpt = "a".repeat(200);

        assert!(chain.append(entry).is_ok());
    }

    #[test]
    fn test_chain_get_method() {
        let mut chain = ProvenanceChain::new();
        let entry1 = create_test_entry("cell:test@v1", "agent:test");
        let entry2 = create_test_entry("cell:test@v2", "agent:test");

        chain.append(entry1.clone()).unwrap();
        chain.append(entry2.clone()).unwrap();

        assert_eq!(chain.get(0).unwrap().entry_id, "cell:test@v1");
        assert_eq!(chain.get(1).unwrap().entry_id, "cell:test@v2");
        assert!(chain.get(2).is_none());
    }

    #[test]
    fn test_load_corrupted_file() {
        let temp_file = "/tmp/test_corrupted_chain.json";
        fs::write(temp_file, "not valid json").unwrap();

        let result = ProvenanceChain::load_from_file(temp_file);
        assert!(result.is_err());

        fs::remove_file(temp_file).ok();
    }

    #[test]
    fn test_multiple_signatures_per_entry() {
        let mut entry = create_test_entry("cell:test@v1", "agent:test");
        entry.signatures.push(Signature {
            by: "agent:test2".to_string(),
            sig: "ed25519:sig456".to_string(),
        });

        let hash = compute_entry_hash(&entry);
        assert!(!hash.is_empty());
    }

    #[test]
    fn test_tools_list_in_entry() {
        let mut entry = create_test_entry("cell:test@v1", "agent:test");
        entry.tools = vec![
            "z1-fmt".to_string(),
            "z1-typeck".to_string(),
            "z1-hash".to_string(),
        ];

        let json = serde_json::to_string(&entry).unwrap();
        let deserialized: ProvenanceEntry = serde_json::from_str(&json).unwrap();

        assert_eq!(entry.tools, deserialized.tools);
    }

    #[test]
    fn test_timestamp_serialization() {
        let entry = create_test_entry("cell:test@v1", "agent:test");
        let json = serde_json::to_string(&entry).unwrap();

        // Check that timestamp is serialized
        assert!(json.contains("timestamp"));

        let deserialized: ProvenanceEntry = serde_json::from_str(&json).unwrap();
        assert_eq!(entry.timestamp, deserialized.timestamp);
    }
}
