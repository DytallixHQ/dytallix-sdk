//! Cryptographic utility command implementation.

use std::path::PathBuf;

use anyhow::{anyhow, Result};
use clap::{Args, Subcommand, ValueEnum};
use dytallix_core::address::DAddr;
use dytallix_core::keypair::{DytallixKeypair, KeyScheme};
use dytallix_core::signature::{verify_mldsa65, verify_slhdsa};
use dytallix_sdk::keystore::Keystore;

use crate::commands::{active_keypair, bytes_to_hex, display_path, hex_to_bytes, load_keystore};
use crate::output;

/// Arguments for the `crypto` command.
#[derive(Debug, Clone, Args)]
pub struct CryptoArgs {
    /// Crypto subcommand.
    #[command(subcommand)]
    pub command: CryptoCommand,
}

/// Crypto subcommands.
#[derive(Debug, Clone, Subcommand)]
pub enum CryptoCommand {
    /// Generate a keypair.
    Keygen {
        /// Signature scheme for key generation.
        #[arg(long, default_value = "ml-dsa-65")]
        scheme: CryptoScheme,
    },
    /// Sign a message with the active wallet.
    Sign { message: String },
    /// Verify a message, signature, and public key tuple.
    Verify {
        message: String,
        signature: String,
        pubkey: String,
    },
    /// Derive a D-Addr from a public key.
    Address { pubkey: String },
    /// Inspect a keystore file without exposing private keys.
    Inspect { keystore_file: PathBuf },
}

/// CLI key-generation scheme selector.
#[derive(Debug, Clone, Copy, ValueEnum)]
pub enum CryptoScheme {
    /// ML-DSA-65 keypair generation.
    #[value(name = "ml-dsa-65")]
    MlDsa65,
    /// SLH-DSA-SHAKE-192s keypair generation.
    #[value(name = "slh-dsa")]
    SlhDsa,
}

/// Runs the `crypto` command.
pub async fn run(args: CryptoArgs) -> Result<()> {
    match args.command {
        CryptoCommand::Keygen { scheme } => keygen(scheme),
        CryptoCommand::Sign { message } => sign(message),
        CryptoCommand::Verify {
            message,
            signature,
            pubkey,
        } => verify(message, signature, pubkey),
        CryptoCommand::Address { pubkey } => address(pubkey),
        CryptoCommand::Inspect { keystore_file } => inspect(keystore_file),
    }
}

fn keygen(scheme: CryptoScheme) -> Result<()> {
    let keypair = match scheme {
        CryptoScheme::MlDsa65 => DytallixKeypair::generate(),
        CryptoScheme::SlhDsa => DytallixKeypair::generate_slh_dsa(),
    };

    output::section("Generated keypair");
    println!("Scheme:      {:?}", keypair.scheme());
    println!("Public key:  {} bytes", keypair.public_key().len());
    println!("Private key: {} bytes", keypair.private_key().len());
    if keypair.scheme() == KeyScheme::MlDsa65 {
        let address = DAddr::from_public_key(keypair.public_key())?;
        println!("D-Addr:      {address}");
    } else {
        println!("D-Addr:      not applicable for SLH-DSA cold-storage keys");
    }
    Ok(())
}

fn sign(message: String) -> Result<()> {
    let keystore = load_keystore()?;
    let keypair = active_keypair(&keystore)?;
    let signature = keypair.sign(message.as_bytes())?;
    output::section("Signature");
    println!("{}", bytes_to_hex(&signature));
    Ok(())
}

fn verify(message: String, signature: String, pubkey: String) -> Result<()> {
    let signature = hex_to_bytes(&signature)?;
    let pubkey = hex_to_bytes(&pubkey)?;
    let valid = match pubkey.len() {
        1_952 => verify_mldsa65(&pubkey, message.as_bytes(), &signature)?,
        48 => verify_slhdsa(&pubkey, message.as_bytes(), &signature)?,
        other => {
            return Err(anyhow!(
				"Unsupported public key length: {other} bytes. Provide an ML-DSA-65 or SLH-DSA public key."
			));
        }
    };
    if valid {
        output::success("Signature verified", None);
    } else {
        output::error(
            "Signature verification failed. Check the message, signature, and public key.",
        );
    }
    Ok(())
}

fn address(pubkey: String) -> Result<()> {
    let pubkey = hex_to_bytes(&pubkey)?;
    let address = DAddr::from_public_key(&pubkey)?;
    output::section("Derived D-Addr");
    println!("{address}");
    Ok(())
}

fn inspect(keystore_file: PathBuf) -> Result<()> {
    if !keystore_file.exists() {
        return Err(anyhow!(
            "No keystore found at {}. Run dytallix init to create one.",
            display_path(&keystore_file)
        ));
    }

    let keystore = Keystore::open(keystore_file.clone())?;
    output::section("Keystore contents");
    for entry in keystore.list() {
        println!(
            "Name: {}\n  Address: {}\n  Scheme: {:?}\n  Public key: {} bytes\n  Created: {}",
            entry.name,
            entry.address,
            entry.scheme,
            entry.public_key.len(),
            entry.created_at
        );
    }
    Ok(())
}
