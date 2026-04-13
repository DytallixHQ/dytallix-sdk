# Getting Started

[Docs hub](README.md) | [Project README](../README.md) | [Examples](../examples/README.md)

## Install Paths

The SDK is currently consumed from Git, not crates.io.

Add the library crate:

```bash
cargo add dytallix-sdk --git https://github.com/DytallixHQ/dytallix-sdk.git
```

Add the library crate with the network client and faucet support:

```bash
cargo add dytallix-sdk --git https://github.com/DytallixHQ/dytallix-sdk.git --features network
```

Install the CLI:

```bash
cargo install --git https://github.com/DytallixHQ/dytallix-sdk.git dytallix-cli --bin dytallix
```

Build from a local clone:

```bash
cargo build --all
```

## First Keypair

Generate an ML-DSA-65 keypair and print a D-Addr:

```rust
use dytallix_sdk::{DAddr, DytallixKeypair};

fn main() {
    let keypair = DytallixKeypair::generate();
    let addr = DAddr::from_public_key(keypair.public_key()).unwrap();
    println!("{addr}");
}
```

If you cloned this repository, you can run the same flow directly:

```bash
cargo run -p dytallix-sdk --example first-keypair
```

## Network Features

```bash
cargo add dytallix-sdk --git https://github.com/DytallixHQ/dytallix-sdk.git --features network
```

If you cloned this repository, the network example runs with:

```bash
cargo run -p dytallix-sdk --features network --example first-transaction
```

Minimal networked flow in Rust:

```rust
use dytallix_sdk::client::DytallixClient;
use dytallix_sdk::faucet::FaucetClient;

let client = DytallixClient::testnet().await?;
let faucet = FaucetClient::testnet();
```

The public testnet surface is still evolving, but the SDK now targets the live
read, faucet, and transaction submission routes exposed from
`https://dytallix.com`.

## CLI Quickstart

The CLI stores its state under `~/.dytallix/`.

Create a wallet, persist it to the keystore, and request faucet funds:

```bash
dytallix init
```

Inspect the active wallet:

```bash
dytallix wallet info
dytallix balance
```

When you use the default public endpoint at `https://dytallix.com`, manual
checks can use root routes such as `/status`, `/balance/<daddr>`,
`/account/<daddr>`, and `/submit`. Compatibility aliases are also available on
`/api/status` and `/api/blockchain/...`.

Check faucet eligibility:

```bash
dytallix faucet status
```

Send a test transfer:

```bash
dytallix send <daddr> 100
```

Prepare a first contract deployment:

```bash
dytallix contract deploy ./my_contract.wasm
```

Contract deploy uses `POST /contracts/deploy` on the active endpoint:

```bash
dytallix config set endpoint http://localhost:3030
```

To run a local node from this repository checkout:

```bash
./start-local.sh
dytallix config network local
```

After deploy, verify the indexed contract metadata with:

```bash
dytallix contract info <contract-address>
```

On the public testnet gateway, `dytallix contract info <contract-address>` is the
canonical verification path if `/tx/<hash>` indexing lags behind contract metadata.

## Local Files

- Keystore: `~/.dytallix/keystore.json`
- CLI config: `~/.dytallix/config.json`

## Next Steps

- Read [Core concepts](core-concepts.md) for tokens, addresses, gas, and network profiles.
- Read [SDK reference](sdk-reference.md) for the Rust API layout.
- Read [CLI reference](cli-reference.md) for command-by-command examples.
- Read [FAQ](faq.md) for current operational caveats.
