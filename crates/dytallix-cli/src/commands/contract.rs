//! Contract command implementation.

use std::path::{Path, PathBuf};

use anyhow::{anyhow, Result};
use clap::{Args, Subcommand};
use serde_json::json;

use crate::commands::{
    active_entry, bytes_to_hex, display_path, hex_to_bytes, load_keystore, raw_get_json,
    raw_post_json, read_bytes,
};
use crate::output;

/// Arguments for the `contract` command.
#[derive(Debug, Clone, Args)]
pub struct ContractArgs {
    /// Contract subcommand.
    #[command(subcommand)]
    pub command: ContractCommand,
}

/// Contract subcommands.
#[derive(Debug, Clone, Subcommand)]
pub enum ContractCommand {
    /// Deploy a WASM contract.
    Deploy { wasm_file: PathBuf },
    /// Call a contract method by submitting a transaction.
    Call {
        address: String,
        method: String,
        args: Vec<String>,
    },
    /// Query a contract method without submitting a transaction.
    Query {
        address: String,
        method: String,
        args: Vec<String>,
    },
    /// Show contract metadata.
    Info { address: String },
    /// Show contract events.
    Events { address: String },
}

/// Runs the `contract` command.
pub async fn run(args: ContractArgs) -> Result<()> {
    match args.command {
        ContractCommand::Deploy { wasm_file } => deploy(wasm_file).await,
        ContractCommand::Call {
            address,
            method,
            args,
        } => call(address, method, args).await,
        ContractCommand::Query {
            address,
            method,
            args,
        } => query(address, method, args).await,
        ContractCommand::Info { address } => info(address).await,
        ContractCommand::Events { address } => events(address).await,
    }
}

async fn deploy(wasm_file: PathBuf) -> Result<()> {
    let wasm = validated_wasm_bytes(&wasm_file)?;
    let keystore = load_keystore()?;
    let sender = active_entry(&keystore)?.address.to_string();
    let value = raw_post_json(
        "/contracts/deploy",
        &json!({
            "deployer": sender,
            "code": bytes_to_hex(&wasm),
            "gas_limit": 1_000_000u64,
        }),
    )
    .await?;
    if let Some(tx_hash) = value.get("tx_hash").and_then(|raw| raw.as_str()) {
        output::tx_hash(tx_hash);
    }
    if let Some(address) = value.get("address").and_then(|raw| raw.as_str()) {
        println!("Contract address: {address}");
    }
    output::success("Contract deployment submitted", None);
    Ok(())
}

async fn call(address: String, method: String, args: Vec<String>) -> Result<()> {
    let contract = validate_contract_address(&address)?;
    let value = raw_post_json(
        "/contracts/call",
        &json!({
            "address": contract,
            "method": method,
            "args": encode_contract_args(&args),
            "gas_limit": 1_000_000u64,
        }),
    )
    .await?;
    if let Some(tx_hash) = value.get("tx_hash").and_then(|raw| raw.as_str()) {
        output::tx_hash(tx_hash);
    }
    output::section("Contract call");
    println!("{}", serde_json::to_string_pretty(&value)?);
    output::success("Contract call executed", None);
    Ok(())
}

async fn query(address: String, method: String, args: Vec<String>) -> Result<()> {
    let contract = validate_contract_address(&address)?;
    let mut path = format!("/api/contracts/{contract}/query/{method}");
    let encoded_args = encode_contract_args(&args);
    if !encoded_args.is_empty() {
        path.push_str(&format!("?args={encoded_args}"));
    }
    let value = raw_get_json(&path).await?;
    output::section("Contract query");
    println!("{}", serde_json::to_string_pretty(&value)?);
    Ok(())
}

async fn info(address: String) -> Result<()> {
    let contract = validate_contract_address(&address)?;
    let contract_info = raw_get_json(&format!("/api/contracts/{contract}")).await?;
    output::section("Contract info");
    println!("{}", serde_json::to_string_pretty(&contract_info)?);
    Ok(())
}

async fn events(address: String) -> Result<()> {
    let contract = validate_contract_address(&address)?;
    let value = raw_get_json(&format!("/api/contracts/{contract}/events")).await?;
    output::section("Contract events");
    println!("{}", serde_json::to_string_pretty(&value)?);
    Ok(())
}

fn validated_wasm_bytes(path: &Path) -> Result<Vec<u8>> {
    if !path.exists() {
        return Err(anyhow!(
            "Contract file not found at {}. Check the path and try again.",
            display_path(path)
        ));
    }

    let bytes = read_bytes(path)?;
    if bytes.len() < 4 || &bytes[..4] != b"\0asm" {
        return Err(anyhow!(
            "File at {} is not a valid WASM binary.",
            display_path(path)
        ));
    }
    Ok(bytes)
}

fn validate_contract_address(raw: &str) -> Result<String> {
    let trimmed = raw.trim();
    let hex = trimmed.strip_prefix("0x").unwrap_or(trimmed);
    if hex.len() != 40 {
        return Err(anyhow!(
            "Invalid contract address `{trimmed}`. Expected a 0x-prefixed 20-byte hex address."
        ));
    }
    hex_to_bytes(hex)?;
    Ok(format!("0x{hex}"))
}

fn encode_contract_args(args: &[String]) -> String {
    if args.is_empty() {
        String::new()
    } else {
        bytes_to_hex(args.join(",").as_bytes())
    }
}
