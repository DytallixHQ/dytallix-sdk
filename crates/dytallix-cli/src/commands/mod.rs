//! Command modules and shared CLI helpers.

pub mod balance;
pub mod chain;
pub mod config;
pub mod contract;
pub mod crypto;
pub mod dev;
pub mod faucet;
pub mod governance;
pub mod init;
pub mod node;
pub mod send;
pub mod stake;
pub mod wallet;

use std::collections::BTreeMap;
use std::fs;
use std::path::{Path, PathBuf};

use anyhow::{anyhow, Context, Result};
use dytallix_core::address::DAddr;
use dytallix_core::keypair::DytallixKeypair;
use dytallix_sdk::client::DytallixClient;
use dytallix_sdk::error::SdkError;
use dytallix_sdk::faucet::FaucetClient;
use dytallix_sdk::keystore::Keystore;
use dytallix_sdk::{FaucetStatus, KeystoreEntry, Token};
use serde::{Deserialize, Serialize};
use serde_json::Value;

const TESTNET_ENDPOINT: &str = "https://dytallix.com";
const TESTNET_CONTRACT_ENDPOINT: &str = "https://dytallix.com/rpc";
const LOCAL_ENDPOINT: &str = "http://localhost:3030";
const TESTNET_FAUCET: &str = "https://dytallix.com/api/faucet";
const LOCAL_FAUCET: &str = "http://localhost:3004";
const DISCORD_LINK: &str = "https://discord.gg/eyVvu5kmPG";
const EXPLORER_LINK: &str = "https://dytallix.com/build/blockchain";
const ENDPOINT_OVERRIDE_KEY: &str = "endpoint";

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub(crate) struct CliConfig {
    pub(crate) network: NetworkProfile,
    pub(crate) values: BTreeMap<String, String>,
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq, Default)]
#[serde(rename_all = "lowercase")]
pub(crate) enum NetworkProfile {
    #[default]
    Testnet,
    Mainnet,
    Local,
}

impl std::fmt::Display for NetworkProfile {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Testnet => f.write_str("testnet"),
            Self::Mainnet => f.write_str("mainnet"),
            Self::Local => f.write_str("local"),
        }
    }
}

pub(crate) async fn configured_client() -> Result<DytallixClient> {
    let config = load_config()?;
    let endpoint = configured_network_endpoint(&config)?;
    DytallixClient::new(&endpoint)
        .await
        .map_err(humanize_sdk_error)
}

pub(crate) fn configured_faucet() -> Result<FaucetClient> {
    let config = load_config()?;
    let endpoint = match config.network {
        NetworkProfile::Testnet => TESTNET_FAUCET,
        NetworkProfile::Local => LOCAL_FAUCET,
        NetworkProfile::Mainnet => {
            return Err(anyhow!(
				"Faucet is not available on mainnet. Switch to testnet with `dytallix config network testnet`."
			));
        }
    };

    Ok(FaucetClient::new(endpoint))
}

pub(crate) fn faucet_endpoint(profile: NetworkProfile) -> Result<&'static str> {
    match profile {
        NetworkProfile::Testnet => Ok(TESTNET_FAUCET),
        NetworkProfile::Local => Ok(LOCAL_FAUCET),
        NetworkProfile::Mainnet => Err(anyhow!(
            "Faucet is not available on mainnet. Switch to testnet with `dytallix config network testnet`."
        )),
    }
}

pub(crate) fn network_endpoint(profile: NetworkProfile) -> Result<&'static str> {
    match profile {
        NetworkProfile::Testnet => Ok(TESTNET_ENDPOINT),
        NetworkProfile::Local => Ok(LOCAL_ENDPOINT),
        NetworkProfile::Mainnet => Err(anyhow!(
            "Mainnet is not publicly available yet. Switch to testnet with `dytallix config network testnet`."
        )),
    }
}

fn configured_network_endpoint(config: &CliConfig) -> Result<String> {
    if let Ok(endpoint) = std::env::var("DYTALLIX_ENDPOINT") {
        return normalize_endpoint_override(&endpoint);
    }

    if let Some(endpoint) = config.values.get(ENDPOINT_OVERRIDE_KEY) {
        return normalize_endpoint_override(endpoint);
    }

    Ok(network_endpoint(config.network)?.to_owned())
}

pub(crate) fn configured_contract_endpoint() -> Result<String> {
    let config = load_config()?;
    let endpoint = configured_network_endpoint(&config)?;
    Ok(contract_endpoint_for_base(&endpoint))
}

fn contract_endpoint_for_base(endpoint: &str) -> String {
    if endpoint == TESTNET_ENDPOINT {
        TESTNET_CONTRACT_ENDPOINT.to_owned()
    } else {
        endpoint.to_owned()
    }
}

fn normalize_endpoint_override(raw: &str) -> Result<String> {
    let endpoint = raw.trim().trim_end_matches('/');
    if endpoint.is_empty() {
        return Err(anyhow!(
            "Configured endpoint override is empty. Set a full http:// or https:// base URL."
        ));
    }
    if !endpoint.starts_with("http://") && !endpoint.starts_with("https://") {
        return Err(anyhow!(
            "Configured endpoint override `{endpoint}` must start with http:// or https://."
        ));
    }
    Ok(endpoint.to_string())
}

pub(crate) fn ensure_cli_dir() -> Result<PathBuf> {
    let dir = home_dir().join(".dytallix");
    fs::create_dir_all(&dir)?;
    Ok(dir)
}

pub(crate) fn config_path() -> PathBuf {
    home_dir().join(".dytallix").join("config.json")
}

pub(crate) fn load_config() -> Result<CliConfig> {
    let path = config_path();
    if !path.exists() {
        return Ok(CliConfig::default());
    }

    let contents = fs::read_to_string(&path)?;
    serde_json::from_str(&contents)
        .map_err(|err| anyhow!("Invalid CLI config at {}: {err}", display_path(&path)))
}

pub(crate) fn save_config(config: &CliConfig) -> Result<()> {
    let path = config_path();
    ensure_cli_dir()?;
    let json = serde_json::to_string_pretty(config)?;
    fs::write(path, json)?;
    Ok(())
}

pub(crate) fn load_keystore() -> Result<Keystore> {
    Keystore::open(Keystore::default_path()).map_err(map_keystore_error)
}

pub(crate) fn load_or_create_keystore() -> Result<Keystore> {
    Keystore::open_or_create(Keystore::default_path()).map_err(map_keystore_error)
}

pub(crate) fn active_entry(keystore: &Keystore) -> Result<&KeystoreEntry> {
    keystore.active().ok_or_else(|| {
		anyhow!(
			"No active wallet. Run `dytallix init` to create one, or `dytallix wallet switch NAME` to activate an existing wallet."
		)
	})
}

pub(crate) fn active_keypair(keystore: &Keystore) -> Result<DytallixKeypair> {
    let entry = active_entry(keystore)?;
    keystore
        .get_keypair(&entry.name)
        .map_err(humanize_sdk_error)
}

pub(crate) fn validate_address(raw: &str) -> Result<DAddr> {
    DAddr::from_str(raw)
        .map_err(|_| anyhow!("Invalid address: Bech32m checksum failed — check for typos."))
}

pub(crate) fn format_number(value: u128) -> String {
    let digits = value.to_string();
    let mut out = String::with_capacity(digits.len() + digits.len() / 3);
    let chars = digits.chars().rev().collect::<Vec<char>>();
    for (index, ch) in chars.iter().enumerate() {
        if index > 0 && index % 3 == 0 {
            out.push(',');
        }
        out.push(*ch);
    }
    out.chars().rev().collect()
}

pub(crate) fn display_path(path: &Path) -> String {
    let home = home_dir();
    if let Ok(stripped) = path.strip_prefix(&home) {
        if stripped.as_os_str().is_empty() {
            "~".to_owned()
        } else {
            format!("~/{}", stripped.display())
        }
    } else {
        path.display().to_string()
    }
}

pub(crate) fn short_address(address: &DAddr) -> String {
    let prefix = address.as_str().chars().take(16).collect::<String>();
    format!("{prefix}...")
}

pub(crate) fn read_bytes(path: &Path) -> Result<Vec<u8>> {
    fs::read(path).with_context(|| format!("Failed to read {}", display_path(path)))
}

pub(crate) fn bytes_to_hex(bytes: &[u8]) -> String {
    const HEX: &[u8; 16] = b"0123456789abcdef";
    let mut encoded = String::with_capacity(bytes.len() * 2);
    for byte in bytes {
        encoded.push(HEX[(byte >> 4) as usize] as char);
        encoded.push(HEX[(byte & 0x0f) as usize] as char);
    }
    encoded
}

pub(crate) fn hex_to_bytes(raw: &str) -> Result<Vec<u8>> {
    let trimmed = raw.trim();
    if trimmed.len() % 2 != 0 {
        return Err(anyhow!(
            "Invalid hex input. Provide an even number of characters."
        ));
    }

    let mut bytes = Vec::with_capacity(trimmed.len() / 2);
    let chars = trimmed.as_bytes();
    let mut index = 0usize;
    while index < chars.len() {
        let high = decode_hex_nibble(chars[index] as char)?;
        let low = decode_hex_nibble(chars[index + 1] as char)?;
        bytes.push((high << 4) | low);
        index += 2;
    }
    Ok(bytes)
}

pub(crate) async fn raw_get_json(path: &str) -> Result<Value> {
    let config = load_config()?;
    let endpoint = configured_network_endpoint(&config)?;
    raw_get_json_at(&endpoint, path).await
}

pub(crate) async fn raw_get_json_at(endpoint: &str, path: &str) -> Result<Value> {
    let url = format!("{endpoint}{path}");
    let response = reqwest::get(&url)
        .await
        .map_err(|_| anyhow!("Cannot reach {url}. Check your network connection."))?;
    if response.status().is_success() {
        response
            .json()
            .await
            .map_err(|err| anyhow!("Failed to decode response from {url}: {err}"))
    } else {
        let status = response.status();
        let reason = response.text().await.unwrap_or_default();
        Err(anyhow!(
            "Request to {url} failed with status {status}. {reason}"
        ))
    }
}

pub(crate) async fn raw_post_json_at(endpoint: &str, path: &str, payload: &Value) -> Result<Value> {
    let url = format!("{endpoint}{path}");
    let client = reqwest::Client::new();
    let response = client
        .post(&url)
        .json(payload)
        .send()
        .await
        .map_err(|_| anyhow!("Cannot reach {url}. Check your network connection."))?;
    if response.status().is_success() {
        response
            .json()
            .await
            .map_err(|err| anyhow!("Failed to decode response from {url}: {err}"))
    } else {
        let status = response.status();
        let reason = response.text().await.unwrap_or_default();
        Err(anyhow!(
            "Request to {url} failed with status {status}. {reason}"
        ))
    }
}

pub(crate) async fn faucet_request(address: &DAddr, token_type: &str) -> Result<()> {
    let faucet = configured_faucet()?;
    match token_type.to_ascii_lowercase().as_str() {
        "both" => faucet.fund(address).await.map(|_| ()),
        "dgt" => faucet.fund_dgt(address).await.map(|_| ()),
        "drt" => faucet.fund_drt(address).await.map(|_| ()),
        other => Err(SdkError::FaucetUnavailable {
            endpoint: faucet_endpoint(load_config()?.network)?.to_owned(),
            reason: format!("unsupported faucet token selection: {other}"),
        }),
    }
    .map_err(humanize_sdk_error)
}

pub(crate) async fn faucet_status(address: &DAddr) -> Result<FaucetStatus> {
    let faucet = configured_faucet()?;
    match faucet.status(address).await {
        Ok(status) => Ok(status),
        Err(SdkError::FaucetRateLimited {
            retry_after_seconds,
        }) => Ok(FaucetStatus {
            can_request: false,
            retry_after_seconds: Some(retry_after_seconds),
        }),
        Err(error) => Err(humanize_sdk_error(error)),
    }
}

pub(crate) async fn faucet_balance(address: &DAddr) -> Result<dytallix_sdk::Balance> {
    configured_client()
        .await?
        .get_balance(address)
        .await
        .map_err(humanize_sdk_error)
}

pub(crate) fn humanize_sdk_error(error: SdkError) -> anyhow::Error {
    match error {
		SdkError::Core(_) => anyhow!("Invalid address: Bech32m checksum failed — check for typos."),
		SdkError::InsufficientBalance {
			token: Token::DRT,
			required,
			available,
		} => anyhow!(
			"Insufficient DRT balance. Required: {} DRT. Available: {} DRT.",
			format_number(required),
			format_number(available)
		),
		SdkError::InsufficientBalance {
			token: Token::DGT,
			required,
			available,
		} => anyhow!(
			"Insufficient DGT for gas fees. Required: {} DGT. Available: {} DGT. Run dytallix faucet to get more.",
			format_number(required),
			format_number(available)
		),
		SdkError::FaucetRateLimited {
			retry_after_seconds,
		} => anyhow!("Faucet rate limit reached. Try again in {retry_after_seconds} seconds."),
		SdkError::FaucetUnavailable { endpoint, .. } => anyhow!(
			"Faucet is not reachable at {endpoint}. Check your network connection or try again later."
		),
		SdkError::NodeUnavailable { endpoint, reason }
            if transaction_api_unavailable(&endpoint, &reason) => anyhow!(
            "The Dytallix testnet transaction API is not available at {endpoint}. Faucet and balance reads may still work, but transaction simulation and submission are not exposed from this endpoint yet."
        ),
		SdkError::NodeUnavailable { endpoint, .. } => anyhow!(
			"Cannot reach the Dytallix testnet at {endpoint}. Check your network connection."
		),
		SdkError::KeystoreNotFound(_) => anyhow!(keystore_not_found_message()),
		SdkError::Network(message) => anyhow!("Network error: {message}"),
		SdkError::Io(err) => anyhow!("I/O error: {err}"),
		SdkError::Serialization(message) => anyhow!("Serialization error: {message}"),
		SdkError::TransactionRejected(message) if looks_like_gateway_html(&message) => anyhow!(
            "The Dytallix testnet transaction API returned a gateway or HTML response instead of transaction JSON. Transaction submission is not usable from the current endpoint."
        ),
		SdkError::TransactionRejected(message) => anyhow!("Transaction rejected: {message}"),
		SdkError::ContractDeployFailed(message) => anyhow!("Contract deployment failed: {message}"),
		SdkError::KeystoreCorrupt(message) => anyhow!("Keystore corrupt: {message}"),
		SdkError::NetworkMismatch(message) => anyhow!("Network mismatch: {message}"),
		SdkError::InsufficientGas { required, provided } => anyhow!(
			"Insufficient gas: required {required} units but only {provided} were provided. Increase the gas limit and try again."
		),
	}
}

fn transaction_api_unavailable(endpoint: &str, reason: &str) -> bool {
    let lower_reason = reason.to_ascii_lowercase();
    (endpoint.contains("/transactions")
        || endpoint.contains("/simulate")
        || endpoint.contains("/api/blockchain/submit"))
        && (lower_reason.contains("405 not allowed")
            || lower_reason.contains("404 not found")
            || lower_reason.contains("cannot post")
            || lower_reason.contains("<html")
            || lower_reason.contains("<!doctype html"))
}

fn looks_like_gateway_html(message: &str) -> bool {
    let lower = message.to_ascii_lowercase();
    lower.contains("<html") || lower.contains("<!doctype html")
}

pub(crate) fn map_keystore_error(error: SdkError) -> anyhow::Error {
    match error {
        SdkError::KeystoreNotFound(_) => anyhow!(keystore_not_found_message()),
        other => humanize_sdk_error(other),
    }
}

pub(crate) fn keystore_not_found_message() -> &'static str {
    "No keystore found at ~/.dytallix/keystore.json. Run dytallix init to create one."
}

pub(crate) fn faucet_balance_timeout(address: &DAddr) -> anyhow::Error {
    anyhow!(
		"Faucet request submitted but balance not confirmed after 45 seconds. Check the explorer at {EXPLORER_LINK} for address {address}. Join Discord at {DISCORD_LINK} if the problem persists."
	)
}

pub(crate) fn open_url(url: &str) -> Result<()> {
    #[cfg(target_os = "macos")]
    let command = ("open", vec![url]);
    #[cfg(target_os = "linux")]
    let command = ("xdg-open", vec![url]);
    #[cfg(target_os = "windows")]
    let command = ("cmd", vec!["/C", "start", url]);

    let status = std::process::Command::new(command.0)
        .args(command.1)
        .status()
        .with_context(|| format!("Failed to open {url}"))?;

    if status.success() {
        Ok(())
    } else {
        Err(anyhow!(
            "Failed to open {url}. Open it manually in your browser."
        ))
    }
}

fn home_dir() -> PathBuf {
    std::env::var_os("HOME")
        .map(PathBuf::from)
        .unwrap_or_else(|| PathBuf::from("."))
}

fn decode_hex_nibble(ch: char) -> Result<u8> {
    match ch {
        '0'..='9' => Ok((ch as u8) - b'0'),
        'a'..='f' => Ok((ch as u8) - b'a' + 10),
        'A'..='F' => Ok((ch as u8) - b'A' + 10),
        _ => Err(anyhow!("Invalid hex input. `{ch}` is not a hex character.")),
    }
}

#[cfg(test)]
mod tests {
    use dytallix_sdk::error::SdkError;

    use super::{
        contract_endpoint_for_base, faucet_balance_timeout, faucet_endpoint,
        humanize_sdk_error, keystore_not_found_message, network_endpoint,
        normalize_endpoint_override, NetworkProfile, LOCAL_ENDPOINT,
        TESTNET_CONTRACT_ENDPOINT, TESTNET_ENDPOINT, TESTNET_FAUCET,
    };
    use dytallix_core::address::DAddr;
    use dytallix_core::keypair::DytallixKeypair;

    #[test]
    fn error_messages_are_correct() {
        let rate_limited = humanize_sdk_error(SdkError::FaucetRateLimited {
            retry_after_seconds: 17,
        })
        .to_string();
        assert!(rate_limited.contains("Try again in 17 seconds"));

        let node_unavailable = humanize_sdk_error(SdkError::NodeUnavailable {
            endpoint: "https://dytallix.com".to_owned(),
            reason: "offline".to_owned(),
        })
        .to_string();
        assert!(node_unavailable.contains("Check your network connection"));

        let tx_api_unavailable = humanize_sdk_error(SdkError::NodeUnavailable {
            endpoint: "https://dytallix.com/api/blockchain/submit".to_owned(),
            reason: "<html><h1>405 Not Allowed</h1></html>".to_owned(),
        })
        .to_string();
        assert!(tx_api_unavailable.contains("transaction API is not available"));

        assert!(keystore_not_found_message().contains("Run dytallix init"));

        let address = DAddr::from_public_key(DytallixKeypair::generate().public_key()).unwrap();
        let timeout = faucet_balance_timeout(&address).to_string();
        assert!(timeout.contains("discord.gg/eyVvu5kmPG"));
    }

    #[test]
    fn network_profiles_use_public_surface_defaults() {
        assert_eq!(
            network_endpoint(NetworkProfile::Testnet).unwrap(),
            TESTNET_ENDPOINT
        );
        assert_eq!(
            network_endpoint(NetworkProfile::Local).unwrap(),
            LOCAL_ENDPOINT
        );
        assert_eq!(
            faucet_endpoint(NetworkProfile::Testnet).unwrap(),
            TESTNET_FAUCET
        );
    }

    #[test]
    fn endpoint_override_is_normalized() {
        assert_eq!(
            normalize_endpoint_override("https://rpc.example.test/").unwrap(),
            "https://rpc.example.test"
        );
        assert!(normalize_endpoint_override("rpc.example.test").is_err());
    }

    #[test]
    fn public_contract_commands_use_rpc_gateway() {
        assert_eq!(
            contract_endpoint_for_base(TESTNET_ENDPOINT),
            TESTNET_CONTRACT_ENDPOINT
        );
        assert_eq!(
            contract_endpoint_for_base(LOCAL_ENDPOINT),
            LOCAL_ENDPOINT
        );
        assert_eq!(
            contract_endpoint_for_base("https://rpc.example.test"),
            "https://rpc.example.test"
        );
    }
}
