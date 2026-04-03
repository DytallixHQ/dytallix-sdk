//! Send command implementation.

use anyhow::{anyhow, Result};
use clap::{Args, ValueEnum};

use dytallix_sdk::transaction::TransactionBuilder;
use dytallix_sdk::Token;

use crate::commands::{
    active_entry, active_keypair, configured_client, format_number, humanize_sdk_error,
    load_keystore, validate_address,
};
use crate::output;

/// Arguments for the `send` command.
#[derive(Debug, Clone, Args)]
pub struct SendArgs {
    /// The token to send. Defaults to DRT.
    #[arg(long, default_value = "drt")]
    pub token: SendToken,
    /// The destination Dytallix address.
    pub address: String,
    /// The token amount to send.
    pub amount: u128,
}

/// CLI token selector for send operations.
#[derive(Debug, Clone, Copy, ValueEnum)]
pub enum SendToken {
    /// Dytallix Governance Token.
    Dgt,
    /// Dytallix Reward Token.
    Drt,
}

impl From<SendToken> for Token {
    fn from(value: SendToken) -> Self {
        match value {
            SendToken::Dgt => Token::DGT,
            SendToken::Drt => Token::DRT,
        }
    }
}

/// Runs the `send` command.
pub async fn run(args: SendArgs) -> Result<()> {
    let destination = validate_destination_before_network(&args.address)?;

    let keystore = load_keystore()?;
    let sender = active_entry(&keystore)?.address.clone();
    let keypair = active_keypair(&keystore)?;
    let client = configured_client().await?;
    let account = client
        .get_account(&sender)
        .await
        .map_err(humanize_sdk_error)?;

    let token: Token = args.token.into();
    match token {
        Token::DGT if account.balance.dgt < args.amount => {
            return Err(anyhow!(
                "Insufficient balance for DGT. Required: {} DGT. Available: {} DGT.",
                format_number(args.amount),
                format_number(account.balance.dgt)
            ));
        }
        Token::DRT if account.balance.drt < args.amount => {
            return Err(anyhow!(
                "Insufficient balance for DRT. Required: {} DRT. Available: {} DRT.",
                format_number(args.amount),
                format_number(account.balance.drt)
            ));
        }
        _ => {}
    }

    let tx = TransactionBuilder::new()
        .from(sender)
        .to(destination)
        .amount(args.amount, token)
        .nonce(account.nonce)
        .build()
        .map_err(|err| anyhow!(err.to_string()))?;
    let fee = tx.estimate_fee(&client).await.map_err(humanize_sdk_error)?;
    if account.balance.drt < fee.total_cost_drt {
        return Err(anyhow!(
			"Insufficient DRT for gas fees. Required: {} DRT. Available: {} DRT. Run dytallix faucet to get more.",
			format_number(fee.total_cost_drt),
			format_number(account.balance.drt)
		));
    }

    output::fee_breakdown(&fee);
    let signed = tx.sign(&keypair).map_err(humanize_sdk_error)?;
    let receipt = client
        .submit_transaction(&signed)
        .await
        .map_err(humanize_sdk_error)?;
    output::tx_hash(&receipt.hash);
    output::success("Transaction submitted", None);
    Ok(())
}

fn validate_destination_before_network(raw: &str) -> Result<dytallix_core::address::DAddr> {
    validate_address(raw)
}

#[cfg(test)]
mod tests {
    use super::validate_destination_before_network;

    #[test]
    fn send_validates_address_before_network_call() {
        let error = validate_destination_before_network("not-a-dytallix-address")
            .unwrap_err()
            .to_string();
        assert!(error.contains("checksum failed"));
    }
}
