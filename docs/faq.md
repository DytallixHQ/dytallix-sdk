# FAQ

[Docs hub](README.md) | [Getting started](getting-started.md) | [CLI reference](cli-reference.md)

## How do I install the SDK if it is not on crates.io yet?

Use the Git repository directly:

```bash
cargo add dytallix-sdk --git https://github.com/DytallixHQ/dytallix-sdk.git
```

Enable network support with:

```bash
cargo add dytallix-sdk --git https://github.com/DytallixHQ/dytallix-sdk.git --features network
```

## What is the difference between DGT and DRT?

- `DGT` is the governance and delegation token.
- `DRT` is used for gas fees, rewards, and burns.

The SDK models both through the `Token` enum.

## Why does wallet rotation not preserve the same address?

The D-Addr is derived from the ML-DSA-65 public key. Rotating the key changes
the public key, which changes the derived address. The CLI surfaces that
directly and recommends creating a new wallet instead of pretending rotation can
keep the same identity.

## Why do I need the `network` feature?

Without `network`, the SDK stays small and supports offline flows such as:

- key generation
- address derivation
- signing and verification
- keystore operations
- transaction construction

Enable `network` when you need the async node client or faucet client.

## Where does the CLI store keys and config?

Under `~/.dytallix/`:

- `keystore.json` stores named key entries and the active wallet
- `config.json` stores the selected network profile and free-form config values

## Can I use an SLH-DSA keypair as a normal Dytallix wallet?

No. The CLI can generate SLH-DSA keys for cryptographic workflows, but the
normal Dytallix account path and D-Addr derivation are ML-DSA-65 based.

## Why can faucet funding work while transaction submission still fails?

The public testnet API surface is still evolving. The repository example
[`first-transaction.rs`](../examples/first-transaction.rs) already handles the
case where faucet funding is available but transaction simulation or submission
endpoints are not exposed from the current public endpoint.

## How do I switch between testnet, mainnet, and local development?

Use:

```bash
dytallix config network testnet
dytallix config network mainnet
dytallix config network local
```

The active profile controls which node and faucet endpoints the CLI uses.

## Where should I ask for help or report issues?

- General issues or feature requests: open a GitHub issue in the repository
- Security issues: follow the private process in [SECURITY.md](../SECURITY.md)
- Community questions: join [Discord](https://discord.gg/eyVvu5kmPG)
