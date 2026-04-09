//! Deploy your first WASM smart contract on the Dytallix testnet.
//! Run with: cargo run --example deploy-contract
//! Run dytallix init first if you have not already.

use dytallix_sdk::keystore::Keystore;

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    let keystore = Keystore::open(Keystore::default_path())?;
    let entry = keystore.active().ok_or_else(|| {
        anyhow::anyhow!(
            "No active wallet. Run dytallix init first.\nDiscord: https://discord.gg/eyVvu5kmPG"
        )
    })?;
    println!("Deploying from: {}", entry.address);
    println!("Run: dytallix contract deploy ./examples/contracts/minimal_contract/target/wasm32-unknown-unknown/release/minimal_contract.wasm");
    println!("Public testnet contract commands use https://dytallix.com/rpc automatically.");
    println!("See: https://dytallix.com/docs/getting-started");
    Ok(())
}
