//! Ed25519 signature generation and verification.

use crate::types::{ProvenanceEntry, Signature};
use ed25519_dalek::{Signer, SigningKey, Verifier, VerifyingKey};
use sha3::{Digest, Sha3_256};

/// Generate a new Ed25519 keypair.
///
/// Returns (private_key_bytes, public_key_bytes).
pub fn keygen() -> ([u8; 32], [u8; 32]) {
    // Generate 32 random bytes for the private key using getrandom
    let mut private_bytes = [0u8; 32];
    getrandom::fill(&mut private_bytes).expect("failed to generate random bytes");

    let signing_key = SigningKey::from_bytes(&private_bytes);
    let verifying_key = signing_key.verifying_key();
    let public_bytes = verifying_key.to_bytes();

    (private_bytes, public_bytes)
}

/// Compute the canonical hash of a provenance entry for signing.
///
/// This produces a deterministic hash by:
/// 1. Serializing the entry to canonical JSON (sorted keys, no whitespace)
/// 2. Computing SHA3-256 of the serialized bytes
///
/// The signatures field is excluded from the hash to avoid circular dependencies.
fn entry_hash(entry: &ProvenanceEntry) -> [u8; 32] {
    // Create a copy without signatures for hashing
    let mut hashable = entry.clone();
    hashable.signatures.clear();

    // Serialize to canonical JSON (sorted keys)
    let json = serde_json::to_string(&hashable).expect("entry serialization failed");

    // Hash the canonical representation
    let mut hasher = Sha3_256::new();
    hasher.update(json.as_bytes());
    hasher.finalize().into()
}

/// Sign a provenance entry with an Ed25519 private key.
///
/// # Arguments
/// * `entry` - The provenance entry to sign
/// * `private_key` - 32-byte Ed25519 private key
/// * `signer_id` - Identifier for the signer (e.g., "dev:alice@keys/ed25519")
///
/// # Returns
/// A Signature struct containing the signer ID and hex-encoded signature.
pub fn sign_entry(entry: &ProvenanceEntry, private_key: &[u8; 32], signer_id: &str) -> Signature {
    let signing_key = SigningKey::from_bytes(private_key);
    let hash = entry_hash(entry);
    let signature = signing_key.sign(&hash);

    Signature {
        by: signer_id.to_string(),
        sig: format!("ed25519:{}", hex::encode(signature.to_bytes())),
    }
}

/// Verify a signature on a provenance entry.
///
/// # Arguments
/// * `entry` - The provenance entry that was signed
/// * `signature` - The signature to verify
/// * `public_key` - 32-byte Ed25519 public key of the expected signer
///
/// # Returns
/// `true` if the signature is valid, `false` otherwise.
pub fn verify_signature(
    entry: &ProvenanceEntry,
    signature: &Signature,
    public_key: &[u8; 32],
) -> bool {
    // Parse the signature (expect "ed25519:..." format)
    let sig_hex = match signature.sig.strip_prefix("ed25519:") {
        Some(hex_str) => hex_str,
        None => return false,
    };

    let sig_bytes = match hex::decode(sig_hex) {
        Ok(bytes) => bytes,
        Err(_) => return false,
    };

    if sig_bytes.len() != 64 {
        return false;
    }

    let mut sig_array = [0u8; 64];
    sig_array.copy_from_slice(&sig_bytes);

    let signature_obj = ed25519_dalek::Signature::from_bytes(&sig_array);

    let verifying_key = match VerifyingKey::from_bytes(public_key) {
        Ok(key) => key,
        Err(_) => return false,
    };

    let hash = entry_hash(entry);
    verifying_key.verify(&hash, &signature_obj).is_ok()
}

#[cfg(test)]
mod tests {
    use super::*;
    use chrono::Utc;

    fn make_test_entry() -> ProvenanceEntry {
        ProvenanceEntry {
            entry_id: "cell:test@v1".to_string(),
            prev: None,
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
    fn test_keygen_generates_valid_keypair() {
        let (private_key, public_key) = keygen();
        assert_eq!(private_key.len(), 32);
        assert_eq!(public_key.len(), 32);
    }

    #[test]
    fn test_sign_and_verify_succeeds() {
        let (private_key, public_key) = keygen();
        let entry = make_test_entry();

        let signature = sign_entry(&entry, &private_key, "test:signer");

        assert_eq!(signature.by, "test:signer");
        assert!(signature.sig.starts_with("ed25519:"));

        let is_valid = verify_signature(&entry, &signature, &public_key);
        assert!(is_valid);
    }

    #[test]
    fn test_verify_with_wrong_public_key_fails() {
        let (private_key, _) = keygen();
        let (_, wrong_public_key) = keygen();
        let entry = make_test_entry();

        let signature = sign_entry(&entry, &private_key, "test:signer");
        let is_valid = verify_signature(&entry, &signature, &wrong_public_key);

        assert!(!is_valid);
    }

    #[test]
    fn test_verify_modified_entry_fails() {
        let (private_key, public_key) = keygen();
        let mut entry = make_test_entry();

        let signature = sign_entry(&entry, &private_key, "test:signer");

        // Modify the entry after signing
        entry.actor = "different-actor".to_string();

        let is_valid = verify_signature(&entry, &signature, &public_key);
        assert!(!is_valid);
    }

    #[test]
    fn test_sign_with_multiple_keys() {
        let (private_key1, public_key1) = keygen();
        let (private_key2, public_key2) = keygen();
        let entry = make_test_entry();

        let sig1 = sign_entry(&entry, &private_key1, "signer1");
        let sig2 = sign_entry(&entry, &private_key2, "signer2");

        assert!(verify_signature(&entry, &sig1, &public_key1));
        assert!(verify_signature(&entry, &sig2, &public_key2));
        assert!(!verify_signature(&entry, &sig1, &public_key2));
        assert!(!verify_signature(&entry, &sig2, &public_key1));
    }

    #[test]
    fn test_verify_signature_with_invalid_format() {
        let entry = make_test_entry();
        let (_, public_key) = keygen();

        let bad_sig = Signature {
            by: "test".to_string(),
            sig: "invalid-format".to_string(),
        };

        assert!(!verify_signature(&entry, &bad_sig, &public_key));
    }

    #[test]
    fn test_verify_signature_with_empty_key() {
        let entry = make_test_entry();

        // Generate a valid keypair
        let (valid_pub, _) = keygen();

        // Create a signature with the wrong/different content (all zeros)
        let wrong_sig = Signature {
            by: "test".to_string(),
            sig: "ed25519:".to_string() + &"00".repeat(64),
        };

        // Verification should fail because the signature doesn't match the entry
        assert!(!verify_signature(&entry, &wrong_sig, &valid_pub));
    }
}
