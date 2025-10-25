# z1-prov: Provenance Chain Management

Provenance chain management and Ed25519 signature verification for Zero1 cells.

## Overview

This crate provides structures and functions for managing Zero1 provenance chains, including:

- Append-only provenance chain with Merkle linking
- SHA3-256 based entry hashing
- Ed25519 cryptographic signatures
- Chain integrity verification
- File I/O for `.z1p` files
- Merkle root calculation

## Features

### Provenance Entries

Each `ProvenanceEntry` records metadata about a code modification:

- **entry_id**: Unique identifier (e.g., `"cell:http.server@v3"`)
- **prev**: Hash of previous entry (forms Merkle chain)
- **actor**: Who made the change (human or agent)
- **model**: LLM model used for generation
- **prompt_sha3**: SHA3-256 hash of the full prompt
- **prompt_excerpt**: First 80 characters of prompt
- **tools**: List of tools used during generation
- **diff_sha3**: SHA3-256 hash of the changes
- **timestamp**: When the change occurred
- **signatures**: Cryptographic signatures on the entry

### Provenance Chain

A `ProvenanceChain` maintains an ordered list of entries with:

- Automatic Merkle linking (each entry's `prev` field)
- Merkle root calculation over all entries
- Chain integrity verification
- JSON serialization/deserialization

## Usage Examples

### Creating and Appending Entries

```rust
use z1_prov::{ProvenanceChain, ProvenanceEntry, ProvenanceChainExt, Signature};
use chrono::Utc;

// Create a new chain
let mut chain = ProvenanceChain::new();

// Create an entry
let entry = ProvenanceEntry {
    entry_id: "cell:example@v1".to_string(),
    prev: None,  // Will be set automatically
    actor: "agent:z1-agent/1.2.3".to_string(),
    model: "llm-x-2025-08".to_string(),
    prompt_sha3: "abc123...".to_string(),
    prompt_excerpt: "Create HTTP server".to_string(),
    tools: vec!["z1-fmt".to_string(), "z1-typeck".to_string()],
    diff_sha3: "def456...".to_string(),
    timestamp: Utc::now(),
    signatures: vec![],
};

// Append to chain (automatically sets prev and updates merkle root)
let entry_hash = chain.append(entry).unwrap();
println!("Entry hash: {}", entry_hash);
println!("Merkle root: {}", chain.merkle_root);
```

### Signing Entries

```rust
use z1_prov::{keygen, sign_entry, verify_signature};

// Generate a keypair
let (private_key, public_key) = keygen();

// Sign an entry
let signature = sign_entry(&entry, &private_key, "dev:alice@keys/ed25519");

// Add signature to entry
entry.signatures.push(signature.clone());

// Verify signature
assert!(verify_signature(&entry, &signature, &public_key));
```

### Verifying Chains

```rust
use z1_prov::{verify_chain, verify_chain_signatures};
use std::collections::HashMap;

// Verify Merkle chain structure
verify_chain(&chain).unwrap();

// Verify all signatures
let mut public_keys = HashMap::new();
public_keys.insert("dev:alice@keys/ed25519".to_string(), public_key);

verify_chain_signatures(&chain, &public_keys, None).unwrap();
```

### File I/O

```rust
use z1_prov::ProvenanceChainExt;

// Save chain to file
chain.save_to_file("provenance.z1p").unwrap();

// Load chain from file
let loaded_chain = ProvenanceChain::load_from_file("provenance.z1p").unwrap();
```

### Computing Merkle Roots

```rust
use z1_prov::ProvenanceChainExt;

// Compute merkle root
let root = chain.compute_merkle_root();

// Update stored merkle root
chain.update_merkle_root();
```

## API Reference

### Core Types

- `ProvenanceEntry`: A single entry in the provenance chain
- `ProvenanceChain`: An ordered list of provenance entries
- `Signature`: A cryptographic signature on an entry

### Traits

- `ProvenanceChainExt`: Extension trait providing chain operations

### Functions

#### Chain Operations

- `append(&mut self, entry) -> Result<String>`: Append entry and return hash
- `compute_merkle_root(&self) -> String`: Calculate Merkle root
- `update_merkle_root(&mut self)`: Update stored Merkle root
- `get(&self, index) -> Option<&ProvenanceEntry>`: Get entry by index

#### File I/O

- `load_from_file(path) -> Result<ProvenanceChain>`: Load from JSON file
- `save_to_file(&self, path) -> Result<()>`: Save to JSON file

#### Verification

- `verify_chain(chain) -> Result<()>`: Verify Merkle chain structure
- `verify_chain_signatures(chain, keys, required) -> Result<()>`: Verify signatures

#### Signatures

- `keygen() -> ([u8; 32], [u8; 32])`: Generate Ed25519 keypair
- `sign_entry(entry, private_key, signer_id) -> Signature`: Sign an entry
- `verify_signature(entry, signature, public_key) -> bool`: Verify signature

#### Hashing

- `compute_entry_hash(entry) -> String`: Compute SHA3-256 hash of entry

## Error Handling

The crate defines several error types:

- `ChainError`: Errors during chain operations (validation, I/O)
- `VerificationError`: Errors during chain verification

## Design Notes

### Merkle Chain

Each entry's `prev` field contains the SHA3-256 hash of the previous entry,
forming a cryptographic chain. The first entry has `prev: None`.

### Merkle Root

The Merkle root is computed by hashing all entry hashes concatenated together.
This provides a single hash representing the entire chain state.

### Deterministic Serialization

Entry hashes use canonical JSON serialization (sorted keys, no whitespace)
to ensure deterministic hash values across systems.

### Signature Exclusion

When hashing entries for signatures, the `signatures` field is excluded
to avoid circular dependencies.

## Testing

Run the test suite:

```bash
cargo test -p z1-prov
```

The crate includes 36 comprehensive tests covering:

- Entry hashing and determinism
- Chain append operations
- Merkle root calculation
- File I/O and serialization
- Signature generation and verification
- Chain integrity verification
- Edge cases and error handling

## License

Apache-2.0
