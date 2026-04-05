//! Send your first transaction on the Dytallix testnet.
//! Run with: cargo run --example first-transaction

use dytallix_core::keypair::DytallixKeypair;
use dytallix_sdk::client::DytallixClient;
use dytallix_sdk::error::SdkError;
use dytallix_sdk::faucet::FaucetClient;
use dytallix_sdk::transaction::TransactionBuilder;
use dytallix_sdk::Token;
use tokio::time::{sleep, Duration};

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    let keypair = DytallixKeypair::generate();
    let addr = dytallix_core::address::DAddr::from_public_key(keypair.public_key())?;
    println!("Address: {addr}");

    let faucet = FaucetClient::testnet();
    let balance = faucet.fund(&addr).await?;
    println!("{balance}");

    let client = DytallixClient::testnet().await?;
    let account = client.get_account(&addr).await?;
    let tx = TransactionBuilder::new()
        .from(addr.clone())
        .to(addr.clone())
        .amount(1, Token::DRT)
        .nonce(account.nonce)
        .build()?;
    let (tx, fee) = tx
        .with_estimated_fee(&client)
        .await
        .map_err(humanize_transaction_error)?;
    println!("{fee}");

    let signed = tx.sign(&keypair)?;
    let receipt = client
        .submit_transaction(&signed)
        .await
        .map_err(humanize_transaction_error)?;
    println!("Transaction: {}", receipt.hash);
    let receipt = wait_for_receipt(&client, &receipt.hash).await?;
    println!("Status: {:?}", receipt.status);
    Ok(())
}

fn humanize_transaction_error(error: SdkError) -> anyhow::Error {
    anyhow::anyhow!(error.to_string())
}

async fn wait_for_receipt(
    client: &DytallixClient,
    hash: &str,
) -> anyhow::Result<dytallix_sdk::TransactionReceipt> {
    for _ in 0..15 {
        let receipt = client.get_transaction(hash).await?;
        match receipt.status {
            dytallix_sdk::TransactionStatus::Pending => sleep(Duration::from_secs(1)).await,
            _ => return Ok(receipt),
        }
    }
    client.get_transaction(hash).await.map_err(Into::into)
}
