//! Contract command implementation.

use std::path::{Path, PathBuf};

use anyhow::{anyhow, Result};
use clap::{Args, Subcommand};

use dytallix_sdk::transaction::TransactionBuilder;
use dytallix_sdk::Token;

use crate::commands::{
    active_entry, active_keypair, configured_client, display_path, humanize_sdk_error,
    load_keystore, raw_get_json, read_bytes, unsupported_public_gateway_read, validate_address,
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
    let sender = active_entry(&keystore)?.address.clone();
    let keypair = active_keypair(&keystore)?;
    let client = configured_client().await?;
    let account = client
        .get_account(&sender)
        .await
        .map_err(humanize_sdk_error)?;

    let mut data = b"contract:deploy:".to_vec();
    data.extend_from_slice(&wasm);
    let tx = TransactionBuilder::new()
        .from(sender.clone())
        .to(sender)
        .amount(0, Token::DRT)
        .nonce(account.nonce)
        .data(data)
        .build()
        .map_err(|err| anyhow!(err.to_string()))?;
    let (tx, fee) = tx
        .with_estimated_fee(&client)
        .await
        .map_err(humanize_sdk_error)?;
    output::fee_breakdown(&fee);
    let signed = tx.sign(&keypair).map_err(humanize_sdk_error)?;
    let tx_hash = signed.hash();
    let receipt = client
        .submit_transaction(&signed)
        .await
        .map_err(humanize_sdk_error)?;

    output::tx_hash(&receipt.hash);
    println!("Contract address: predicted-{}", &tx_hash[..16]);
    output::success("Contract deployment submitted", None);
    Ok(())
}

async fn call(address: String, method: String, args: Vec<String>) -> Result<()> {
    let contract = validate_address(&address)?;
    let keystore = load_keystore()?;
    let sender = active_entry(&keystore)?.address.clone();
    let keypair = active_keypair(&keystore)?;
    let client = configured_client().await?;
    let account = client
        .get_account(&sender)
        .await
        .map_err(humanize_sdk_error)?;

    let tx = TransactionBuilder::new()
        .from(sender)
        .to(contract)
        .amount(0, Token::DRT)
        .nonce(account.nonce)
        .data(format!("contract:call:{method}:{}", args.join(",")).into_bytes())
        .build()
        .map_err(|err| anyhow!(err.to_string()))?;
    let (tx, fee) = tx
        .with_estimated_fee(&client)
        .await
        .map_err(humanize_sdk_error)?;
    output::fee_breakdown(&fee);
    let signed = tx.sign(&keypair).map_err(humanize_sdk_error)?;
    let receipt = client
        .submit_transaction(&signed)
        .await
        .map_err(humanize_sdk_error)?;
    output::tx_hash(&receipt.hash);
    output::success("Contract call submitted", None);
    Ok(())
}

async fn query(address: String, method: String, args: Vec<String>) -> Result<()> {
    let contract = validate_address(&address)?;
    let path = if args.is_empty() {
        format!("/v1/contracts/{contract}/query/{method}")
    } else {
        format!("/v1/contracts/{contract}/query/{method}?args=<hex-encoded>")
    };
    Err(unsupported_public_gateway_read("contract query", &path))
}

async fn info(address: String) -> Result<()> {
    let contract = validate_address(&address)?;
    let value = raw_get_json("/api/contracts").await?;
    let contract_info = value
        .get("contracts")
        .and_then(|contracts| contracts.as_array())
        .and_then(|contracts| {
            contracts.iter().find(|entry| {
                entry.get("address").and_then(|raw| raw.as_str()) == Some(contract.as_str())
            })
        })
        .cloned()
        .ok_or_else(|| {
            anyhow!(
                "Contract {} was not found through the public contracts API.",
                contract.as_str()
            )
        })?;
    output::section("Contract info");
    println!("{}", serde_json::to_string_pretty(&contract_info)?);
    Ok(())
}

async fn events(address: String) -> Result<()> {
    let contract = validate_address(&address)?;
    Err(unsupported_public_gateway_read(
        "contract events",
        &format!("/v1/contracts/{contract}/events"),
    ))
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
