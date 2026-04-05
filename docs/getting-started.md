# Getting Started

## First keypair

Add the SDK:

```bash
cargo add dytallix-sdk
```

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

## Network features

The default crate is optimized for the shortest first-keypair path. If you need
the testnet client or faucet client, enable the `network` feature:

```bash
cargo add dytallix-sdk --features network
```

If you cloned this repository, the network example runs with:

```bash
cargo run -p dytallix-sdk --features network --example first-transaction
```

## CLI

Install the CLI:

```bash
cargo install --git https://github.com/DytallixHQ/dytallix-sdk.git dytallix-cli --bin dytallix
```

Create a wallet and request faucet funds:

```bash
dytallix init
```

Website: https://dytallix.com
Discord: https://discord.gg/eyVvu5kmPG
