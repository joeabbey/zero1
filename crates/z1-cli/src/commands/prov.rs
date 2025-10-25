//! Provenance CLI commands.

use anyhow::{Context, Result};
use clap::{Args, Subcommand};
use colored::Colorize;
use std::collections::HashMap;
use std::fs;
use std::path::PathBuf;
use z1_prov::{keygen, verify_chain, verify_chain_signatures, ProvenanceChain, ProvenanceChainExt};

#[derive(Debug, Args)]
pub struct ProvArgs {
    #[command(subcommand)]
    pub command: ProvCommand,
}

#[derive(Debug, Subcommand)]
pub enum ProvCommand {
    /// Display a provenance chain in human-readable format
    Log {
        /// Path to the provenance chain file (.z1p)
        file: PathBuf,
    },
    /// Verify the integrity of a provenance chain
    Verify {
        /// Path to the provenance chain file (.z1p)
        file: PathBuf,
        /// Optional path to JSON file mapping signer IDs to public keys (hex-encoded)
        #[arg(long)]
        keys: Option<PathBuf>,
    },
    /// Generate a new Ed25519 keypair
    Keygen {
        /// Optional output path for the keypair (default: prints to stdout)
        output: Option<PathBuf>,
    },
}

/// Handle the z1prov log command.
pub fn cmd_log(file: PathBuf) -> Result<()> {
    let chain = ProvenanceChain::load_from_file(&file)
        .with_context(|| format!("failed to load provenance chain from {}", file.display()))?;

    if chain.is_empty() {
        println!("{}", "Chain is empty".yellow());
        return Ok(());
    }

    println!("{}", "Provenance Chain".bold().underline());
    println!("{}: {}", "File".bold(), file.display());
    println!("{}: {}", "Entries".bold(), chain.len());
    println!("{}: {}", "Merkle Root".bold(), chain.merkle_root);
    println!();

    for (idx, entry) in chain.entries.iter().enumerate() {
        println!("{} {}", "Entry".bold().cyan(), (idx + 1).to_string().cyan());
        println!("  {}: {}", "ID".bold(), entry.entry_id);
        if let Some(prev) = &entry.prev {
            println!("  {}: {}", "Previous".bold(), prev);
        }
        println!("  {}: {}", "Actor".bold(), entry.actor);
        println!("  {}: {}", "Model".bold(), entry.model);
        println!("  {}: {}", "Timestamp".bold(), entry.timestamp);
        println!("  {}: {}", "Prompt SHA3".bold(), entry.prompt_sha3);
        println!(
            "  {}: \"{}\"",
            "Prompt Excerpt".bold(),
            entry.prompt_excerpt
        );
        if !entry.tools.is_empty() {
            println!("  {}: [{}]", "Tools".bold(), entry.tools.join(", "));
        }
        println!("  {}: {}", "Diff SHA3".bold(), entry.diff_sha3);

        if !entry.signatures.is_empty() {
            println!("  {}:", "Signatures".bold());
            for sig in &entry.signatures {
                let sig_preview = if sig.sig.len() > 20 {
                    format!("{}...", &sig.sig[..20])
                } else {
                    sig.sig.clone()
                };
                println!("    - {}: {}", sig.by.green(), sig_preview);
            }
        } else {
            println!("  {}: {}", "Signatures".bold(), "none".yellow());
        }
        println!();
    }

    Ok(())
}

/// Handle the z1prov verify command.
pub fn cmd_verify(file: PathBuf, keys_file: Option<PathBuf>) -> Result<()> {
    let chain = ProvenanceChain::load_from_file(&file)
        .with_context(|| format!("failed to load provenance chain from {}", file.display()))?;

    // Verify Merkle chain structure
    verify_chain(&chain).context("Merkle chain verification failed")?;

    println!("{} Merkle chain structure valid", "✓".green().bold());

    // If public keys provided, verify signatures
    if let Some(keys_path) = keys_file {
        let keys_json = fs::read_to_string(&keys_path).context("failed to read keys file")?;

        let keys_map: HashMap<String, String> =
            serde_json::from_str(&keys_json).context("failed to parse keys JSON")?;

        let mut public_keys = HashMap::new();
        for (signer_id, hex_key) in keys_map {
            let key_bytes = hex::decode(&hex_key)
                .with_context(|| format!("invalid hex key for {signer_id}"))?;
            if key_bytes.len() != 32 {
                anyhow::bail!("public key for {signer_id} must be 32 bytes");
            }
            let mut key_array = [0u8; 32];
            key_array.copy_from_slice(&key_bytes);
            public_keys.insert(signer_id, key_array);
        }

        verify_chain_signatures(&chain, &public_keys, None)
            .context("signature verification failed")?;

        let sig_count: usize = chain.entries.iter().map(|e| e.signatures.len()).sum();
        println!("{} {} signatures verified", "✓".green().bold(), sig_count);
    }

    println!();
    println!("{}", "Summary:".bold().underline());
    println!("  {}: {}", "Entries".bold(), chain.len());
    println!("  {}: {}", "Status".bold(), "VALID".green().bold());

    Ok(())
}

/// Handle the z1prov keygen command.
pub fn cmd_keygen(output: Option<PathBuf>) -> Result<()> {
    let (private_key, public_key) = keygen();

    let private_hex = hex::encode(private_key);
    let public_hex = hex::encode(public_key);

    if let Some(path) = output {
        let keypair = serde_json::json!({
            "private_key": private_hex,
            "public_key": public_hex,
        });
        let json = serde_json::to_string_pretty(&keypair)?;
        fs::write(&path, json)
            .with_context(|| format!("failed to write keypair to {}", path.display()))?;
        println!("{} Keypair written to {}", "✓".green(), path.display());
    } else {
        println!("{}", "Generated Ed25519 Keypair".bold().underline());
        println!("{}: {}", "Private Key".bold().red(), private_hex);
        println!("{}: {}", "Public Key".bold().green(), public_hex);
        println!();
        println!(
            "{}",
            "WARNING: Keep the private key secret!".yellow().bold()
        );
    }

    Ok(())
}
