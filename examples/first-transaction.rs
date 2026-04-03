//! Send your first transaction on the Dytallix testnet.
//! Run with: cargo run --example first-transaction

use dytallix_core::keypair::DytallixKeypair;
use dytallix_sdk::client::DytallixClient;
use dytallix_sdk::faucet::FaucetClient;
use dytallix_sdk::transaction::TransactionBuilder;
use dytallix_sdk::Token;

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
    let fee = tx.estimate_fee(&client).await?;
    println!("{fee}");

    let signed = tx.sign(&keypair)?;
    let receipt = client.submit_transaction(&signed).await?;
    println!("Transaction: {}", receipt.hash);
    println!("Status: {:?}", receipt.status);
    Ok(())
}
