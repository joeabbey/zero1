//! Provenance chain verification.

use crate::signature::verify_signature;
use crate::types::{ProvenanceChain, ProvenanceEntry};
use sha3::{Digest, Sha3_256};
use std::collections::HashMap;
use thiserror::Error;

/// Errors that can occur during provenance verification.
#[derive(Debug, Error)]
pub enum VerificationError {
    #[error("Chain is empty")]
    EmptyChain,

    #[error("Entry {0} has invalid Merkle link: expected {1}, got {2:?}")]
    InvalidMerkleLink(String, String, Option<String>),

    #[error("Entry {0} has no signatures")]
    NoSignatures(String),

    #[error("Entry {0} signature by {1} failed verification")]
    InvalidSignature(String, String),

    #[error("Entry {0} missing required signer {1}")]
    MissingRequiredSigner(String, String),

    #[error("Public key for {0} not provided")]
    MissingPublicKey(String),
}

/// Compute the hash of a provenance entry for Merkle chain linking.
///
/// This is used to verify the `prev` field of subsequent entries.
pub fn compute_entry_hash(entry: &ProvenanceEntry) -> String {
    let json = serde_json::to_string(entry).expect("entry serialization failed");
    let mut hasher = Sha3_256::new();
    hasher.update(json.as_bytes());
    format!("sha3-256:{}", hex::encode(hasher.finalize()))
}

/// Verify the Merkle chain structure of a provenance chain.
///
/// Ensures that each entry's `prev` field correctly references the hash
/// of the previous entry.
pub fn verify_chain(chain: &ProvenanceChain) -> Result<(), VerificationError> {
    if chain.is_empty() {
        return Ok(()); // Empty chain is valid
    }

    let mut prev_hash: Option<String> = None;

    for entry in &chain.entries {
        // Check that prev matches the hash of the previous entry
        if prev_hash != entry.prev {
            return Err(VerificationError::InvalidMerkleLink(
                entry.entry_id.clone(),
                prev_hash.clone().unwrap_or_else(|| "None".to_string()),
                entry.prev.clone(),
            ));
        }

        // Compute hash for next iteration
        prev_hash = Some(compute_entry_hash(entry));
    }

    Ok(())
}

/// Verify all signatures in a provenance chain.
///
/// # Arguments
/// * `chain` - The provenance chain to verify
/// * `public_keys` - Map from signer IDs to their Ed25519 public keys (32 bytes)
/// * `required_signers` - Optional set of signer IDs that must sign every entry
///
/// # Returns
/// `Ok(())` if all signatures are valid, otherwise an error.
pub fn verify_chain_signatures(
    chain: &ProvenanceChain,
    public_keys: &HashMap<String, [u8; 32]>,
    required_signers: Option<&[String]>,
) -> Result<(), VerificationError> {
    for entry in &chain.entries {
        verify_entry_signatures(entry, public_keys, required_signers)?;
    }
    Ok(())
}

/// Verify all signatures on a single provenance entry.
fn verify_entry_signatures(
    entry: &ProvenanceEntry,
    public_keys: &HashMap<String, [u8; 32]>,
    required_signers: Option<&[String]>,
) -> Result<(), VerificationError> {
    // Check for required signers
    if let Some(required) = required_signers {
        for signer_id in required {
            if !entry.signatures.iter().any(|sig| &sig.by == signer_id) {
                return Err(VerificationError::MissingRequiredSigner(
                    entry.entry_id.clone(),
                    signer_id.clone(),
                ));
            }
        }
    }

    // Verify each signature
    for signature in &entry.signatures {
        let public_key = public_keys
            .get(&signature.by)
            .ok_or_else(|| VerificationError::MissingPublicKey(signature.by.clone()))?;

        if !verify_signature(entry, signature, public_key) {
            return Err(VerificationError::InvalidSignature(
                entry.entry_id.clone(),
                signature.by.clone(),
            ));
        }
    }

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::signature::{keygen, sign_entry};
    use chrono::Utc;

    fn make_test_entry(id: &str, prev: Option<String>) -> ProvenanceEntry {
        ProvenanceEntry {
            entry_id: id.to_string(),
            prev,
            actor: "test-actor".to_string(),
            model: "test-model".to_string(),
            prompt_sha3: "abc123".to_string(),
            prompt_excerpt: "test prompt".to_string(),
            tools: vec![],
            diff_sha3: "def456".to_string(),
            timestamp: Utc::now(),
            signatures: vec![],
        }
    }

    #[test]
    fn test_verify_empty_chain() {
        let chain = ProvenanceChain::new();
        assert!(verify_chain(&chain).is_ok());
    }

    #[test]
    fn test_verify_single_entry_chain() {
        let mut chain = ProvenanceChain::new();
        chain.add_entry(make_test_entry("entry1", None));

        assert!(verify_chain(&chain).is_ok());
    }

    #[test]
    fn test_verify_valid_chain() {
        let mut chain = ProvenanceChain::new();

        let entry1 = make_test_entry("entry1", None);
        let hash1 = compute_entry_hash(&entry1);
        chain.add_entry(entry1);

        let entry2 = make_test_entry("entry2", Some(hash1.clone()));
        let hash2 = compute_entry_hash(&entry2);
        chain.add_entry(entry2);

        let entry3 = make_test_entry("entry3", Some(hash2));
        chain.add_entry(entry3);

        assert!(verify_chain(&chain).is_ok());
    }

    #[test]
    fn test_verify_chain_with_invalid_link() {
        let mut chain = ProvenanceChain::new();

        let entry1 = make_test_entry("entry1", None);
        chain.add_entry(entry1);

        // Entry2 has wrong prev hash
        let entry2 = make_test_entry("entry2", Some("sha3-256:wrong".to_string()));
        chain.add_entry(entry2);

        assert!(matches!(
            verify_chain(&chain),
            Err(VerificationError::InvalidMerkleLink(..))
        ));
    }

    #[test]
    fn test_verify_chain_signatures_valid() {
        let (private_key, public_key) = keygen();
        let mut public_keys = HashMap::new();
        public_keys.insert("signer1".to_string(), public_key);

        let mut entry = make_test_entry("entry1", None);
        let sig = sign_entry(&entry, &private_key, "signer1");
        entry.signatures.push(sig);

        let mut chain = ProvenanceChain::new();
        chain.add_entry(entry);

        assert!(verify_chain_signatures(&chain, &public_keys, None).is_ok());
    }

    #[test]
    fn test_verify_chain_signatures_invalid() {
        let (private_key, _public_key) = keygen();
        let (_, wrong_public_key) = keygen();

        let mut public_keys = HashMap::new();
        public_keys.insert("signer1".to_string(), wrong_public_key);

        let mut entry = make_test_entry("entry1", None);
        let sig = sign_entry(&entry, &private_key, "signer1");
        entry.signatures.push(sig);

        let mut chain = ProvenanceChain::new();
        chain.add_entry(entry);

        assert!(matches!(
            verify_chain_signatures(&chain, &public_keys, None),
            Err(VerificationError::InvalidSignature(..))
        ));
    }

    #[test]
    fn test_verify_chain_missing_required_signer() {
        let (private_key, public_key) = keygen();
        let mut public_keys = HashMap::new();
        public_keys.insert("signer1".to_string(), public_key);

        let mut entry = make_test_entry("entry1", None);
        let sig = sign_entry(&entry, &private_key, "signer1");
        entry.signatures.push(sig);

        let mut chain = ProvenanceChain::new();
        chain.add_entry(entry);

        let required = vec!["signer1".to_string(), "signer2".to_string()];

        assert!(matches!(
            verify_chain_signatures(&chain, &public_keys, Some(&required)),
            Err(VerificationError::MissingRequiredSigner(..))
        ));
    }

    #[test]
    fn test_verify_chain_with_multiple_signers() {
        let (private_key1, public_key1) = keygen();
        let (private_key2, public_key2) = keygen();

        let mut public_keys = HashMap::new();
        public_keys.insert("signer1".to_string(), public_key1);
        public_keys.insert("signer2".to_string(), public_key2);

        let mut entry = make_test_entry("entry1", None);
        let sig1 = sign_entry(&entry, &private_key1, "signer1");
        let sig2 = sign_entry(&entry, &private_key2, "signer2");
        entry.signatures.push(sig1);
        entry.signatures.push(sig2);

        let mut chain = ProvenanceChain::new();
        chain.add_entry(entry);

        let required = vec!["signer1".to_string(), "signer2".to_string()];

        assert!(verify_chain_signatures(&chain, &public_keys, Some(&required)).is_ok());
    }

    #[test]
    fn test_verify_missing_public_key() {
        let (private_key, _) = keygen();
        let public_keys = HashMap::new(); // Empty

        let mut entry = make_test_entry("entry1", None);
        let sig = sign_entry(&entry, &private_key, "signer1");
        entry.signatures.push(sig);

        let mut chain = ProvenanceChain::new();
        chain.add_entry(entry);

        assert!(matches!(
            verify_chain_signatures(&chain, &public_keys, None),
            Err(VerificationError::MissingPublicKey(..))
        ));
    }
}
