//! Provenance chain management and Ed25519 signature verification.
//!
//! This crate provides structures and functions for managing Zero1 provenance chains,
//! including cryptographic signatures using Ed25519, Merkle root calculation, and file I/O.
//!
//! # Example
//!
//! ```
//! use z1_prov::{ProvenanceChain, ProvenanceEntry, ProvenanceChainExt};
//! use chrono::Utc;
//!
//! let mut chain = ProvenanceChain::new();
//!
//! let entry = ProvenanceEntry {
//!     entry_id: "cell:example@v1".to_string(),
//!     prev: None,
//!     actor: "agent:test/1.0".to_string(),
//!     model: "llm-test-2025".to_string(),
//!     prompt_sha3: "abc123".to_string(),
//!     prompt_excerpt: "Create example cell".to_string(),
//!     tools: vec!["z1-fmt".to_string()],
//!     diff_sha3: "def456".to_string(),
//!     timestamp: Utc::now(),
//!     signatures: vec![],
//! };
//!
//! let hash = chain.append(entry).unwrap();
//! assert!(!hash.is_empty());
//! ```

mod chain;
mod signature;
mod types;
mod verify;

pub use chain::{compute_entry_hash, ChainError, ProvenanceChainExt};
pub use signature::{keygen, sign_entry, verify_signature};
pub use types::{ProvenanceChain, ProvenanceEntry, Signature};
pub use verify::{verify_chain, verify_chain_signatures, VerificationError};
