# Dytallix SDK

Official SDK and CLI for Dytallix, a PQC-native Layer 1 blockchain.

This repository contains the Rust workspace for the core cryptography crate,
the application SDK, and the `dytallix` CLI.

## Quick Links

- [Docs hub](docs/README.md)
- [Getting started](docs/getting-started.md)
- [Core concepts](docs/core-concepts.md)
- [SDK reference](docs/sdk-reference.md)
- [CLI reference](docs/cli-reference.md)
- [FAQ](docs/faq.md)
- [Examples](examples/README.md)
- [Releases](https://github.com/DytallixHQ/dytallix-sdk/releases)
- [CI workflow](.github/workflows/ci.yml)
- [Release workflow](.github/workflows/release.yml)
- [Contributing](CONTRIBUTING.md)
- [Security policy](SECURITY.md)
- [Changelog](CHANGELOG.md)
- [License](LICENSE)

## What is Dytallix

Dytallix is a Layer 1 blockchain engineered for the post-quantum era.

All signing uses ML-DSA-65 (FIPS 204). All addresses are canonical Bech32m.

Only PQC-native accounts are supported. No hybrid mode. No legacy accounts.

## Workspace

- [`crates/dytallix-core`](crates/dytallix-core) - cryptographic primitives
- [`crates/dytallix-sdk`](crates/dytallix-sdk) - SDK library for building on Dytallix
- [`crates/dytallix-cli`](crates/dytallix-cli) - CLI for interacting with the Dytallix testnet
- [`docs/`](docs/README.md) - repository documentation and reference pages
- [`examples/`](examples/README.md) - runnable examples for first-keypair, transfer, and contract flows

## Status

- `dytallix-core` - in progress
- `dytallix-sdk` - in progress
- `dytallix-cli` - in progress

## Install and Download

- SDK from Git: `cargo add dytallix-sdk --git https://github.com/DytallixHQ/dytallix-sdk.git`
- SDK with network client and faucet support: `cargo add dytallix-sdk --git https://github.com/DytallixHQ/dytallix-sdk.git --features network`
- CLI from Git: `cargo install --git https://github.com/DytallixHQ/dytallix-sdk.git dytallix-cli --bin dytallix`
- Build from source: `cargo build --release --bin dytallix`
- [GitHub Releases](https://github.com/DytallixHQ/dytallix-sdk/releases)

Release tags matching `v*` build downloadable CLI archives for Linux, macOS
(Intel and Apple Silicon), and Windows through GitHub Actions.

The SDK is not currently published on crates.io, so the documented install path
is the Git repository.

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
cargo add dytallix-sdk --git https://github.com/DytallixHQ/dytallix-sdk.git
```

For network client and faucet support, enable the `network` feature:

```bash
cargo add dytallix-sdk --git https://github.com/DytallixHQ/dytallix-sdk.git --features network
```

Repository examples:

```bash
cargo run -p dytallix-sdk --example first-keypair
cargo run -p dytallix-sdk --features network --example first-transaction
```

## First CLI Session

The CLI keeps its state under `~/.dytallix/` and is designed around a funded
testnet flow:

```bash
dytallix init
dytallix balance
dytallix faucet status
```

After the wallet is funded, common next steps are:

```bash
dytallix send <daddr> 100
dytallix contract deploy ./my_contract.wasm
```

See [Getting started](docs/getting-started.md) and the
[CLI reference](docs/cli-reference.md) for the full flow.

## Documentation Map

- [Docs hub](docs/README.md) - overview of every repo documentation page
- [Getting started](docs/getting-started.md) - install, first keypair, first CLI session
- [Core concepts](docs/core-concepts.md) - tokens, addresses, gas, keystore, network profiles
- [SDK reference](docs/sdk-reference.md) - crate surface and common Rust workflows
- [CLI reference](docs/cli-reference.md) - command map and examples
- [FAQ](docs/faq.md) - operational and product questions
- [Examples](examples/README.md) - runnable examples and prerequisites

## DytallixHQ Repositories

- [dytallix-sdk](https://github.com/DytallixHQ/dytallix-sdk) - this repository
- [dytallix-contracts](https://github.com/DytallixHQ/dytallix-contracts) - protocol contracts
- [dytallix-docs](https://github.com/DytallixHQ/dytallix-docs) - broader documentation
- [dytallix-explorer](https://github.com/DytallixHQ/dytallix-explorer) - explorer codebase
- [dytallix-faucet](https://github.com/DytallixHQ/dytallix-faucet) - faucet codebase

## External Links

- [Website](https://dytallix.com)
- [Documentation site](https://dytallix.com/docs)
- [Whitepapers](https://dytallix.com)
- [Discord](https://discord.gg/eyVvu5kmPG)
- [Explorer app](https://explorer.dytallix.com)
- [Faucet API](https://dytallix.com/api/faucet)
- [GitHub organization](https://github.com/DytallixHQ)
