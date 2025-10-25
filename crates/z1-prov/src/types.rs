//! Provenance data structures.

use anyhow::{Context, Result};
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use std::fs;
use std::path::Path;

/// A cryptographic signature on a provenance entry.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct Signature {
    /// Identifier of the signer (e.g., "dev:alice@keys/ed25519" or "agent:z1-agent/1.2.3")
    pub by: String,
    /// Ed25519 signature as hex-encoded string (e.g., "ed25519:ab8...2f1")
    pub sig: String,
}

/// A single entry in the provenance chain.
///
/// Each entry records metadata about a code modification, including:
/// - Who made the change (actor)
/// - What model was used
/// - When it happened
/// - What prompted it
/// - Cryptographic signatures
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct ProvenanceEntry {
    /// Unique identifier for this entry (e.g., "cell:http.server@v3")
    pub entry_id: String,

    /// Hash of the previous entry in the chain (forms Merkle chain)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub prev: Option<String>,

    /// Actor who made the change (human or agent)
    pub actor: String,

    /// Model used for generation
    pub model: String,

    /// SHA3-256 hash of the full prompt
    pub prompt_sha3: String,

    /// First 80 chars of prompt for human reference
    pub prompt_excerpt: String,

    /// Tools used during generation
    #[serde(default)]
    pub tools: Vec<String>,

    /// SHA3-256 hash of the diff/changes
    pub diff_sha3: String,

    /// Timestamp of the change
    pub timestamp: DateTime<Utc>,

    /// Cryptographic signatures on this entry
    #[serde(default)]
    pub signatures: Vec<Signature>,
}

/// A complete provenance chain.
///
/// Contains an ordered list of provenance entries forming a Merkle chain.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct ProvenanceChain {
    /// Ordered list of provenance entries (oldest first)
    pub entries: Vec<ProvenanceEntry>,

    /// Merkle root computed over all entries
    pub merkle_root: String,
}

impl ProvenanceChain {
    /// Create a new empty provenance chain.
    pub fn new() -> Self {
        Self {
            entries: Vec::new(),
            merkle_root: String::new(),
        }
    }

    /// Add an entry to the chain and update the merkle root.
    pub fn add_entry(&mut self, entry: ProvenanceEntry) {
        self.entries.push(entry);
        self.update_merkle_root();
    }

    /// Update the merkle root based on current entries.
    fn update_merkle_root(&mut self) {
        if self.entries.is_empty() {
            self.merkle_root = String::new();
            return;
        }
        // For now, use the hash of the last entry as the merkle root
        // In a full implementation, this would compute a proper Merkle tree root
        if let Some(last) = self.entries.last() {
            self.merkle_root = crate::verify::compute_entry_hash(last);
        }
    }

    /// Get the most recent entry in the chain.
    pub fn latest(&self) -> Option<&ProvenanceEntry> {
        self.entries.last()
    }

    /// Get the number of entries in the chain.
    pub fn len(&self) -> usize {
        self.entries.len()
    }

    /// Check if the chain is empty.
    pub fn is_empty(&self) -> bool {
        self.entries.is_empty()
    }

    /// Load a provenance chain from a JSON file.
    pub fn load<P: AsRef<Path>>(path: P) -> Result<Self> {
        let content =
            fs::read_to_string(path.as_ref()).context("failed to read provenance file")?;
        let chain: Self =
            serde_json::from_str(&content).context("failed to parse provenance JSON")?;
        Ok(chain)
    }

    /// Save a provenance chain to a JSON file.
    pub fn save<P: AsRef<Path>>(&self, path: P) -> Result<()> {
        let json =
            serde_json::to_string_pretty(self).context("failed to serialize provenance chain")?;
        fs::write(path.as_ref(), json).context("failed to write provenance file")?;
        Ok(())
    }
}

impl Default for ProvenanceChain {
    fn default() -> Self {
        Self::new()
    }
}
