//! Asynchronous HTTP client for Dytallix node APIs.

use std::collections::BTreeMap;
use std::time::Duration;

use reqwest::Url;
use serde::de::DeserializeOwned;
use serde::Deserialize;
use tokio::time::sleep;

use crate::error::SdkError;
use crate::transaction::{SignedTransaction, Transaction};
use crate::{
    AccountState, Balance, Block, BlockId, ChainStatus, Delegation, FeeEstimate,
    TransactionReceipt, TransactionStatus, Validator,
};
use dytallix_core::address::DAddr;

const DEFAULT_PUBLIC_MIN_GAS_PRICE: u64 = 1_000;
const PUBLIC_TESTNET_ENDPOINT: &str = "https://dytallix.com";
const LOCAL_NODE_ENDPOINT: &str = "http://localhost:3030";

/// Asynchronous client for interacting with Dytallix nodes.
#[derive(Debug, Clone)]
pub struct DytallixClient {
    endpoint: String,
    http: reqwest::Client,
}

impl DytallixClient {
    /// Creates a new client targeting the provided node endpoint.
    pub async fn new(endpoint: &str) -> Result<Self, SdkError> {
        let normalized = normalize_endpoint(endpoint)?;
        let http = reqwest::Client::builder()
            .build()
            .map_err(|err| SdkError::Network(err.to_string()))?;

        Ok(Self {
            endpoint: normalized,
            http,
        })
    }

    /// Creates a client for the canonical Dytallix testnet node.
    pub async fn testnet() -> Result<Self, SdkError> {
        Self::new(PUBLIC_TESTNET_ENDPOINT).await
    }

    /// Creates a client for a local Dytallix node.
    pub async fn local() -> Result<Self, SdkError> {
        Self::new(LOCAL_NODE_ENDPOINT).await
    }

    /// Fetches the current account state for the provided address.
    pub async fn get_account(&self, address: &DAddr) -> Result<AccountState, SdkError> {
        let account: AccountResponse = self.get_json(&format!("/account/{address}")).await?;

        Ok(AccountState {
            address: address.clone(),
            pubkey_hash: [0; 32],
            balance: account.balances,
            nonce: account.nonce,
            key_scheme: crate::KeyScheme::MlDsa65,
        })
    }

    /// Fetches the current token balances for the provided address.
    pub async fn get_balance(&self, address: &DAddr) -> Result<Balance, SdkError> {
        let balance: BalanceResponse = self.get_json(&format!("/balance/{address}")).await?;
        Ok(balance.balances)
    }

    /// Fetches a block by number, hash, or chain-relative identifier.
    pub async fn get_block(&self, id: BlockId) -> Result<Block, SdkError> {
        let path = match id {
            BlockId::Number(number) => format!("/block/{number}"),
            BlockId::Hash(hash) => format!("/block/{hash}"),
            BlockId::Latest | BlockId::Finalized => "/block/latest".to_owned(),
        };

        let block: BlockResponse = self.get_json(&path).await?;
        Ok(block.into())
    }

    /// Fetches a transaction receipt by hash.
    pub async fn get_transaction(&self, hash: &str) -> Result<TransactionReceipt, SdkError> {
        let receipt: TransactionReceiptResponse = self.get_json(&format!("/tx/{hash}")).await?;
        Ok(receipt.into())
    }

    /// Fetches the current chain status.
    pub async fn get_chain_status(&self) -> Result<ChainStatus, SdkError> {
        let status: ChainStatusResponse = self.get_json("/status").await?;
        Ok(ChainStatus {
            block_height: status.latest_height,
            epoch: 0,
            slot: 0,
            finalized_checkpoint: status.chain_id,
        })
    }

    /// Submits a signed transaction to the node and returns its receipt.
    pub async fn submit_transaction(
        &self,
        tx: &SignedTransaction,
    ) -> Result<TransactionReceipt, SdkError> {
        let url = self.url(transaction_submit_path(&self.endpoint))?;
        let tx_hash = tx.hash();
        let response = self
            .http
            .post(url.clone())
            .json(&SubmitTransactionBody { signed_tx: tx })
            .send()
            .await
            .map_err(|err| SdkError::NodeUnavailable {
                endpoint: url.to_string(),
                reason: err.to_string(),
            })?;

        if response.status().is_success() {
            let submitted: SubmittedTransaction =
                response.json().await.map_err(serialization_error)?;
            Ok(TransactionReceipt {
                hash: submitted.hash,
                block: 0,
                status: map_transaction_status(&submitted.status, None),
                fee: tx.fee_breakdown().unwrap_or_else(|| tx.tx.fee_estimate()),
            })
        } else {
            let reason = response
                .text()
                .await
                .unwrap_or_else(|_| "request rejected".to_owned());
            if let Some(receipt) = self.lookup_submitted_transaction(&tx_hash).await {
                return Ok(receipt);
            }
            Err(SdkError::TransactionRejected(reason))
        }
    }

    /// Requests a fee simulation for an unsigned transaction.
    pub async fn simulate_transaction(&self, tx: &Transaction) -> Result<FeeEstimate, SdkError> {
        let gas = self.get_network_gas_params().await?;
        Ok(tx.fee_estimate_with_gas_price(gas.min_gas_price))
    }

    /// Fetches the active validator set.
    pub async fn get_validators(&self) -> Result<Vec<Validator>, SdkError> {
        if self.uses_public_testnet_gateway() {
            return Err(
                self.legacy_public_read_unavailable("/v1/validators", "validator-set reads")
            );
        }
        self.get_json("/v1/validators").await
    }

    /// Fetches delegations for the provided delegator address.
    pub async fn get_delegations(&self, address: &DAddr) -> Result<Vec<Delegation>, SdkError> {
        if self.uses_public_testnet_gateway() {
            return Err(self.legacy_public_read_unavailable(
                &format!("/v1/delegations/{address}"),
                "delegation reads",
            ));
        }
        self.get_json(&format!("/v1/delegations/{address}")).await
    }

    async fn get_json<T>(&self, path: &str) -> Result<T, SdkError>
    where
        T: DeserializeOwned,
    {
        let url = self.url(path)?;
        let response =
            self.http
                .get(url.clone())
                .send()
                .await
                .map_err(|err| SdkError::NodeUnavailable {
                    endpoint: url.to_string(),
                    reason: err.to_string(),
                })?;

        if response.status().is_success() {
            response.json().await.map_err(serialization_error)
        } else {
            let reason = response
                .text()
                .await
                .unwrap_or_else(|_| "request failed".to_owned());
            Err(SdkError::NodeUnavailable {
                endpoint: url.to_string(),
                reason,
            })
        }
    }

    fn url(&self, path: &str) -> Result<Url, SdkError> {
        let joined = format!("{}{}", self.endpoint, public_gateway_path(&self.endpoint, path));
        Url::parse(&joined).map_err(|err| SdkError::Network(err.to_string()))
    }

    async fn get_network_gas_params(&self) -> Result<NetworkGasParams, SdkError> {
        let status: ChainStatusResponse = self.get_json("/status").await?;
        let gas = status.gas.unwrap_or_default();
        Ok(NetworkGasParams {
            min_gas_price: gas.min_gas_price.max(DEFAULT_PUBLIC_MIN_GAS_PRICE),
        })
    }

    fn uses_public_testnet_gateway(&self) -> bool {
        self.endpoint == PUBLIC_TESTNET_ENDPOINT
    }

    fn legacy_public_read_unavailable(&self, path: &str, feature: &str) -> SdkError {
        SdkError::NodeUnavailable {
            endpoint: format!("{}{}", self.endpoint, path),
            reason: format!(
                "{feature} are not exposed as public JSON routes on the website gateway. Connect the SDK to a direct node endpoint that serves `{path}` or use the documented public routes at https://dytallix.com/docs."
            ),
        }
    }

    async fn lookup_submitted_transaction(&self, hash: &str) -> Option<TransactionReceipt> {
        for _ in 0..3 {
            sleep(Duration::from_millis(750)).await;
            if let Ok(receipt) = self.get_transaction(hash).await {
                return Some(receipt);
            }
        }
        None
    }
}

fn normalize_endpoint(endpoint: &str) -> Result<String, SdkError> {
    let parsed = Url::parse(endpoint).map_err(|err| SdkError::Network(err.to_string()))?;
    let mut normalized = parsed.to_string();
    while normalized.ends_with('/') {
        normalized.pop();
    }
    Ok(normalized)
}

fn public_gateway_path(endpoint: &str, path: &str) -> String {
    if endpoint == PUBLIC_TESTNET_ENDPOINT && !path.starts_with("/api/blockchain/") {
        format!("/api/blockchain{path}")
    } else {
        path.to_owned()
    }
}

fn transaction_submit_path(endpoint: &str) -> &'static str {
    if endpoint == PUBLIC_TESTNET_ENDPOINT {
        "/api/blockchain/submit"
    } else {
        "/submit"
    }
}

fn serialization_error(err: reqwest::Error) -> SdkError {
    SdkError::Serialization(err.to_string())
}

fn map_transaction_status(status: &str, error: Option<String>) -> TransactionStatus {
    match status.to_ascii_lowercase().as_str() {
        "pending" => TransactionStatus::Pending,
        "success" | "confirmed" => TransactionStatus::Confirmed,
        _ => TransactionStatus::Failed(
            error.unwrap_or_else(|| format!("transaction failed with status `{status}`")),
        ),
    }
}

#[derive(Debug, serde::Deserialize)]
struct AccountResponse {
    #[serde(default = "default_balance", deserialize_with = "deserialize_balances")]
    balances: Balance,
    #[serde(default)]
    nonce: u64,
}

#[derive(Debug, serde::Deserialize)]
struct BalanceResponse {
    #[serde(default = "default_balance", deserialize_with = "deserialize_balances")]
    balances: Balance,
}

#[derive(Debug, serde::Deserialize)]
struct ChainStatusResponse {
    chain_id: String,
    latest_height: u64,
    #[serde(default)]
    gas: Option<ChainGasResponse>,
}

#[derive(Debug, Clone, Copy, Default, Deserialize)]
struct ChainGasResponse {
    #[serde(default)]
    min_gas_price: u64,
}

#[derive(Debug, Clone, Copy)]
struct NetworkGasParams {
    min_gas_price: u64,
}

#[derive(Debug, serde::Serialize)]
struct SubmitTransactionBody<'a> {
    signed_tx: &'a SignedTransaction,
}

#[derive(Debug, serde::Deserialize)]
struct SubmittedTransaction {
    hash: String,
    status: String,
}

#[derive(Debug, serde::Deserialize)]
struct TransactionReceiptResponse {
    #[serde(alias = "hash", rename = "tx_hash")]
    hash: String,
    #[serde(
        rename = "block_height",
        default,
        deserialize_with = "deserialize_u64_or_default"
    )]
    block: u64,
    status: String,
    #[serde(default)]
    error: Option<String>,
    #[serde(default, deserialize_with = "deserialize_u128_string")]
    fee: u128,
}

impl From<TransactionReceiptResponse> for TransactionReceipt {
    fn from(value: TransactionReceiptResponse) -> Self {
        Self {
            hash: value.hash,
            block: value.block,
            status: map_transaction_status(&value.status, value.error),
            fee: FeeEstimate {
                c_gas: 0,
                c_gas_cost_drt: 0,
                b_gas: 0,
                b_gas_cost_drt: value.fee,
                total_cost_drt: value.fee,
            },
        }
    }
}

#[derive(Debug, serde::Deserialize)]
struct BlockResponse {
    #[serde(alias = "number", alias = "height")]
    number: u64,
    hash: String,
    #[serde(default, alias = "parent", alias = "parent_hash")]
    parent_hash: String,
    #[serde(default)]
    proposer: Option<DAddr>,
    #[serde(default)]
    slot: u64,
    #[serde(default)]
    epoch: u64,
    #[serde(default)]
    c_gas_used: u64,
    #[serde(default)]
    b_gas_used: u64,
    #[serde(default)]
    timestamp: u64,
    #[serde(default)]
    txs: Vec<serde_json::Value>,
}

impl From<BlockResponse> for Block {
    fn from(value: BlockResponse) -> Self {
        Self {
            number: value.number,
            hash: value.hash,
            parent_hash: value.parent_hash,
            proposer: value.proposer.unwrap_or_else(unknown_block_proposer),
            slot: value.slot,
            epoch: value.epoch,
            tx_count: value.txs.len(),
            c_gas_used: value.c_gas_used,
            b_gas_used: value.b_gas_used,
            timestamp: value.timestamp,
        }
    }
}

fn unknown_block_proposer() -> DAddr {
    DAddr::from_public_key(&[0u8; 1_952]).expect("fixed placeholder proposer key is valid")
}

fn deserialize_balances<'de, D>(deserializer: D) -> Result<Balance, D::Error>
where
    D: serde::Deserializer<'de>,
{
    let balances =
        <BTreeMap<String, serde_json::Value> as serde::Deserialize>::deserialize(deserializer)?;
    Ok(Balance {
        dgt: decode_micro_balance(&balances, "udgt"),
        drt: decode_micro_balance(&balances, "udrt"),
    })
}

fn default_balance() -> Balance {
    Balance { dgt: 0, drt: 0 }
}

fn decode_micro_balance(balances: &BTreeMap<String, serde_json::Value>, denom: &str) -> u128 {
    match balances.get(denom) {
        Some(serde_json::Value::Number(number)) => {
            number.as_u64().map(u128::from).unwrap_or(0) / 1_000_000
        }
        Some(serde_json::Value::String(value)) => value.parse::<u128>().unwrap_or(0) / 1_000_000,
        Some(serde_json::Value::Object(value)) => {
            value
                .get("balance")
                .and_then(|amount| match amount {
                    serde_json::Value::Number(number) => number.as_u64().map(u128::from),
                    serde_json::Value::String(raw) => raw.parse::<u128>().ok(),
                    _ => None,
                })
                .unwrap_or(0)
                / 1_000_000
        }
        _ => 0,
    }
}

fn deserialize_u64_or_default<'de, D>(deserializer: D) -> Result<u64, D::Error>
where
    D: serde::Deserializer<'de>,
{
    let value = serde_json::Value::deserialize(deserializer)?;
    Ok(match value {
        serde_json::Value::Number(number) => number.as_u64().unwrap_or_default(),
        serde_json::Value::String(raw) => raw.parse::<u64>().unwrap_or_default(),
        _ => 0,
    })
}

fn deserialize_u128_string<'de, D>(deserializer: D) -> Result<u128, D::Error>
where
    D: serde::Deserializer<'de>,
{
    let value = serde_json::Value::deserialize(deserializer)?;
    Ok(match value {
        serde_json::Value::Number(number) => number.as_u64().map(u128::from).unwrap_or_default(),
        serde_json::Value::String(raw) => raw.parse::<u128>().unwrap_or_default(),
        _ => 0,
    })
}

#[cfg(test)]
mod tests {
    use super::{
        normalize_endpoint, public_gateway_path, transaction_submit_path, BlockResponse,
        DytallixClient, LOCAL_NODE_ENDPOINT, PUBLIC_TESTNET_ENDPOINT,
    };
    use crate::error::SdkError;
    use dytallix_core::address::DAddr;
    use dytallix_core::keypair::DytallixKeypair;

    #[test]
    fn normalize_endpoint_trims_trailing_slash() {
        assert_eq!(
            normalize_endpoint("https://dytallix.com/").unwrap(),
            PUBLIC_TESTNET_ENDPOINT
        );
        assert_eq!(
            normalize_endpoint("http://localhost:3030///").unwrap(),
            LOCAL_NODE_ENDPOINT
        );
    }

    #[test]
    fn public_gateway_paths_are_prefixed() {
        assert_eq!(
            public_gateway_path(PUBLIC_TESTNET_ENDPOINT, "/balance/demo"),
            "/api/blockchain/balance/demo"
        );
        assert_eq!(
            public_gateway_path(PUBLIC_TESTNET_ENDPOINT, "/status"),
            "/api/blockchain/status"
        );
        assert_eq!(
            public_gateway_path(PUBLIC_TESTNET_ENDPOINT, "/api/blockchain/submit"),
            "/api/blockchain/submit"
        );
        assert_eq!(
            public_gateway_path(LOCAL_NODE_ENDPOINT, "/status"),
            "/status"
        );
    }

    #[test]
    fn submit_path_uses_direct_node_route_when_needed() {
        assert_eq!(
            transaction_submit_path(PUBLIC_TESTNET_ENDPOINT),
            "/api/blockchain/submit"
        );
        assert_eq!(transaction_submit_path(LOCAL_NODE_ENDPOINT), "/submit");
        assert_eq!(transaction_submit_path("http://127.0.0.1:43030"), "/submit");
    }

    #[test]
    fn block_response_maps_minimal_node_payload() {
        let parsed: BlockResponse = serde_json::from_str(
            r#"{
                "asset_hashes": [],
                "hash": "0xabc",
                "height": 42,
                "parent": "0xdef",
                "timestamp": 1776025359,
                "txs": []
            }"#,
        )
        .unwrap();

        let block: crate::Block = parsed.into();
        assert_eq!(block.number, 42);
        assert_eq!(block.hash, "0xabc");
        assert_eq!(block.parent_hash, "0xdef");
        assert_eq!(block.timestamp, 1776025359);
        assert_eq!(block.tx_count, 0);
        assert_eq!(block.slot, 0);
        assert_eq!(block.epoch, 0);
    }

    #[tokio::test]
    async fn public_gateway_legacy_reads_fail_fast() {
        let client = DytallixClient::testnet().await.unwrap();
        let address = DAddr::from_public_key(DytallixKeypair::generate().public_key()).unwrap();

        let validators = client.get_validators().await.unwrap_err();
        let delegations = client.get_delegations(&address).await.unwrap_err();

        match validators {
            SdkError::NodeUnavailable { endpoint, reason } => {
                assert!(endpoint.ends_with("/v1/validators"));
                assert!(reason.contains("not exposed as public JSON routes"));
            }
            other => panic!("unexpected validator error: {other:?}"),
        }

        match delegations {
            SdkError::NodeUnavailable { endpoint, reason } => {
                assert!(endpoint.contains("/v1/delegations/"));
                assert!(reason.contains("direct node endpoint"));
            }
            other => panic!("unexpected delegation error: {other:?}"),
        }
    }
}
