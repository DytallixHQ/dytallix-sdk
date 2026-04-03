//! Transaction building, signing, and fee estimation for Dytallix.

use dytallix_core::address::DAddr;
use dytallix_core::hash::blake3_hash;
use dytallix_core::keypair::DytallixKeypair;

use crate::client::DytallixClient;
use crate::error::SdkError;
use crate::{FeeEstimate, KeyScheme, Token};

const DEFAULT_C_GAS: u64 = 21_000;

/// Builder for Dytallix token transfer transactions.
#[derive(Debug, Clone, Default)]
pub struct TransactionBuilder {
    from: Option<DAddr>,
    to: Option<DAddr>,
    amount: Option<u128>,
    token: Option<Token>,
    c_gas: Option<u64>,
    b_gas: Option<u64>,
    nonce: Option<u64>,
    data: Option<Vec<u8>>,
}

impl TransactionBuilder {
    /// Creates a new empty builder.
    pub fn new() -> Self {
        Self::default()
    }

    /// Sets the sender address.
    pub fn from(mut self, address: DAddr) -> Self {
        self.from = Some(address);
        self
    }

    /// Sets the recipient address.
    pub fn to(mut self, address: DAddr) -> Self {
        self.to = Some(address);
        self
    }

    /// Sets the token amount and token type for the transfer.
    pub fn amount(mut self, value: u128, token: Token) -> Self {
        self.amount = Some(value);
        self.token = Some(token);
        self
    }

    /// Sets the compute and bandwidth gas limits.
    pub fn gas_limit(mut self, c_gas: u64, b_gas: u64) -> Self {
        self.c_gas = Some(c_gas);
        self.b_gas = Some(b_gas);
        self
    }

    /// Sets the sender nonce.
    pub fn nonce(mut self, nonce: u64) -> Self {
        self.nonce = Some(nonce);
        self
    }

    /// Sets arbitrary transaction payload bytes.
    pub fn data(mut self, bytes: Vec<u8>) -> Self {
        self.data = Some(bytes);
        self
    }

    /// Builds the final transaction after validation.
    pub fn build(self) -> Result<Transaction, SdkError> {
        let from = self
            .from
            .ok_or_else(|| SdkError::TransactionRejected("missing from address".to_owned()))?;
        let to = self
            .to
            .ok_or_else(|| SdkError::TransactionRejected("missing to address".to_owned()))?;
        let amount = self
            .amount
            .ok_or_else(|| SdkError::TransactionRejected("missing amount and token".to_owned()))?;
        let token = self
            .token
            .ok_or_else(|| SdkError::TransactionRejected("missing amount and token".to_owned()))?;
        let data = self.data.unwrap_or_default();

        Ok(Transaction {
            from,
            to,
            amount,
            token,
            c_gas_limit: self.c_gas.unwrap_or(DEFAULT_C_GAS),
            b_gas_limit: self.b_gas.unwrap_or(data.len() as u64),
            nonce: self.nonce.unwrap_or(0),
            data,
        })
    }
}

/// An unsigned Dytallix transaction.
#[derive(Debug, Clone, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
pub struct Transaction {
    /// The sender address.
    pub from: DAddr,
    /// The recipient address.
    pub to: DAddr,
    /// The token amount to transfer.
    pub amount: u128,
    /// The token being transferred.
    pub token: Token,
    /// The compute gas limit, always charged in DRT.
    pub c_gas_limit: u64,
    /// The bandwidth gas limit, always charged in DRT.
    pub b_gas_limit: u64,
    /// The sender nonce.
    pub nonce: u64,
    /// Optional transaction payload bytes.
    pub data: Vec<u8>,
}

impl Transaction {
    /// Signs the transaction with the provided Dytallix keypair.
    pub fn sign(self, keypair: &DytallixKeypair) -> Result<SignedTransaction, SdkError> {
        let payload =
            serde_json::to_vec(&self).map_err(|err| SdkError::Serialization(err.to_string()))?;
        let signature = keypair.sign(&payload)?;
        let public_key = keypair.public_key().to_vec();
        let scheme = keypair.scheme();
        let hash = transaction_hash(&payload, &signature, &public_key);

        Ok(SignedTransaction {
            transaction: self,
            signature,
            public_key,
            scheme,
            fee: None,
            tx_hash: hash,
        })
    }

    /// Requests a fee estimate for this transaction from the provided client.
    pub async fn estimate_fee(&self, client: &DytallixClient) -> Result<FeeEstimate, SdkError> {
        client.simulate_transaction(self).await
    }
}

/// A signed Dytallix transaction ready for submission.
#[derive(Debug, Clone, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
pub struct SignedTransaction {
    /// The unsigned transaction payload.
    pub transaction: Transaction,
    /// The detached signature bytes.
    pub signature: Vec<u8>,
    /// The raw public key used to sign the transaction.
    pub public_key: Vec<u8>,
    /// The key scheme used to sign the transaction.
    pub scheme: KeyScheme,
    /// Optional fee estimate captured alongside the transaction.
    pub fee: Option<FeeEstimate>,
    /// The canonical signed-transaction hash.
    pub tx_hash: String,
}

impl SignedTransaction {
    /// Returns the optional DRT fee breakdown attached to the signed transaction.
    pub fn fee_breakdown(&self) -> Option<FeeEstimate> {
        self.fee.clone()
    }

    /// Returns the canonical signed-transaction hash.
    pub fn hash(&self) -> String {
        self.tx_hash.clone()
    }
}

fn transaction_hash(payload: &[u8], signature: &[u8], public_key: &[u8]) -> String {
    let mut buffer = Vec::with_capacity(payload.len() + signature.len() + public_key.len());
    buffer.extend_from_slice(payload);
    buffer.extend_from_slice(signature);
    buffer.extend_from_slice(public_key);
    encode_hex(&blake3_hash(&buffer))
}

fn encode_hex(bytes: &[u8]) -> String {
    const HEX: &[u8; 16] = b"0123456789abcdef";
    let mut encoded = String::with_capacity(bytes.len() * 2);
    for byte in bytes {
        encoded.push(HEX[(byte >> 4) as usize] as char);
        encoded.push(HEX[(byte & 0x0f) as usize] as char);
    }
    encoded
}
