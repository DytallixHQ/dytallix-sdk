//! Asynchronous HTTP client for Dytallix node APIs.

use std::collections::BTreeMap;

use reqwest::Url;
use serde::de::DeserializeOwned;

use crate::error::SdkError;
use crate::transaction::{SignedTransaction, Transaction};
use crate::{
    AccountState, Balance, Block, BlockId, ChainStatus, Delegation, FeeEstimate,
    TransactionReceipt, Validator,
};
use dytallix_core::address::DAddr;

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
        Self::new("https://dytallix.com").await
    }

    /// Creates a client for a local Dytallix node.
    pub async fn local() -> Result<Self, SdkError> {
        Self::new("http://localhost:8545").await
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
            BlockId::Latest => "/blocks".to_owned(),
            BlockId::Finalized => "/blocks".to_owned(),
        };

        self.get_json(&path).await
    }

    /// Fetches a transaction receipt by hash.
    pub async fn get_transaction(&self, hash: &str) -> Result<TransactionReceipt, SdkError> {
        self.get_json(&format!("/tx/{hash}")).await
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
        let url = self.url("/v1/transactions")?;
        let response = self
            .http
            .post(url.clone())
            .json(tx)
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
                .unwrap_or_else(|_| "request rejected".to_owned());
            Err(SdkError::TransactionRejected(reason))
        }
    }

    /// Requests a fee simulation for an unsigned transaction.
    pub async fn simulate_transaction(&self, tx: &Transaction) -> Result<FeeEstimate, SdkError> {
        self.post_json("/v1/transactions/simulate", tx).await
    }

    /// Fetches the active validator set.
    pub async fn get_validators(&self) -> Result<Vec<Validator>, SdkError> {
        self.get_json("/v1/validators").await
    }

    /// Fetches delegations for the provided delegator address.
    pub async fn get_delegations(&self, address: &DAddr) -> Result<Vec<Delegation>, SdkError> {
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

    async fn post_json<T, B>(&self, path: &str, body: &B) -> Result<T, SdkError>
    where
        T: DeserializeOwned,
        B: serde::Serialize + ?Sized,
    {
        let url = self.url(path)?;
        let response = self
            .http
            .post(url.clone())
            .json(body)
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
        let joined = format!("{}{}", self.endpoint, path);
        Url::parse(&joined).map_err(|err| SdkError::Network(err.to_string()))
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

fn serialization_error(err: reqwest::Error) -> SdkError {
    SdkError::Serialization(err.to_string())
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
