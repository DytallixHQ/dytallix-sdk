# Dytallix SDK

Official SDK and CLI for Dytallix — a PQC-native Layer 1 blockchain.

Testnet active. SDK under construction.

## What is Dytallix

Dytallix is a Layer 1 blockchain engineered for the post-quantum era.

All signing uses ML-DSA-65 (FIPS 204). All addresses are canonical Bech32m.

Only PQC-native accounts are supported. No hybrid mode. No legacy accounts.

## Structure

- crates/dytallix-core — cryptographic primitives
- crates/dytallix-sdk — SDK library for building on Dytallix
- crates/dytallix-cli — CLI for interacting with the Dytallix testnet

## Status

- dytallix-core — in progress
- dytallix-sdk — in progress
- dytallix-cli — in progress

## Download

- GitHub Releases: https://github.com/DytallixHQ/dytallix-sdk/releases
- Build from source: `cargo build --release --bin dytallix`
- Install from GitHub: `cargo install --git https://github.com/DytallixHQ/dytallix-sdk.git dytallix-cli --bin dytallix`

Release tags matching `v*` build downloadable CLI archives for Linux, macOS
(Intel and Apple Silicon), and Windows through GitHub Actions.

## First Keypair

The default `dytallix-sdk` crate now supports the shortest path to a real
post-quantum identity:

```rust
use dytallix_sdk::{DAddr, DytallixKeypair};

fn main() {
    let keypair = DytallixKeypair::generate();
    let addr = DAddr::from_public_key(keypair.public_key()).unwrap();
    println!("{addr}");
}
```

Add it to a project with:

```bash
cargo add dytallix-sdk
```

For network client and faucet support, enable the `network` feature:

```bash
cargo add dytallix-sdk --features network
```

Repository examples:

```bash
cargo run -p dytallix-sdk --example first-keypair
cargo run -p dytallix-sdk --features network --example first-transaction
```

## DytallixHQ Repositories

- dytallix-sdk — this repository
- dytallix-contracts — protocol contracts
- dytallix-docs — documentation
- dytallix-explorer — block explorer
- dytallix-faucet — testnet faucet

## Links

- Website: https://dytallix.com
- Documentation: https://github.com/DytallixHQ/dytallix-docs
- Whitepapers: https://dytallix.com
- Discord: https://discord.gg/eyVvu5kmPG
- Explorer: https://github.com/DytallixHQ/dytallix-explorer
- Faucet: https://github.com/DytallixHQ/dytallix-faucet
