# Examples

These examples are the fastest way to validate the current SDK and CLI flows
from the repository root.

## Available Examples

- [`first-keypair.rs`](first-keypair.rs) - generate an ML-DSA-65 keypair,
  derive a D-Addr, sign a message, and verify the signature
- [`first-transaction.rs`](first-transaction.rs) - request faucet funds and
  attempt a signed transfer against the configured public testnet endpoints
- [`deploy-contract.rs`](deploy-contract.rs) - use the CLI keystore and prepare
  a first contract deployment flow

## Run

```bash
cargo run -p dytallix-sdk --example first-keypair
cargo run -p dytallix-sdk --features network --example first-transaction
cargo run -p dytallix-cli --example deploy-contract
```

## Related Docs

- [Getting started](../docs/getting-started.md)
- [SDK reference](../docs/sdk-reference.md)
- [CLI reference](../docs/cli-reference.md)
