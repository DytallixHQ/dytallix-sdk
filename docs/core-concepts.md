# Core Concepts

[Docs hub](README.md) | [Getting started](getting-started.md) | [SDK reference](sdk-reference.md)

## Identity and Addresses

- Dytallix accounts are PQC-native.
- The standard hot-wallet path uses ML-DSA-65 keypairs.
- A canonical D-Addr is derived from the ML-DSA-65 public key and encoded as
  Bech32m.
- The CLI also supports generating SLH-DSA keys for cryptographic workflows,
  but those keys do not map to a normal Dytallix account address.

Relevant types:

- [`DytallixKeypair`](../crates/dytallix-core/src/keypair.rs)
- [`DAddr`](../crates/dytallix-core/src/address.rs)
- [`verify_mldsa65`](../crates/dytallix-core/src/signature.rs)

## Tokens

The SDK models two canonical tokens:

| Token | Purpose |
| --- | --- |
| `DGT` | Governance and delegation |
| `DRT` | Gas fees, rewards, and burns |

Relevant types:

- [`Token`](../crates/dytallix-sdk/src/lib.rs)
- [`Balance`](../crates/dytallix-sdk/src/lib.rs)

## Accounts and Nonces

An account state includes:

- The canonical address
- The public-key hash
- DGT and DRT balances
- The next transaction nonce
- The key scheme

The SDK exposes this as [`AccountState`](../crates/dytallix-sdk/src/lib.rs).

## Transactions and Fees

Transactions are created with
[`TransactionBuilder`](../crates/dytallix-sdk/src/transaction.rs).

Each transaction includes:

- `from` and `to` addresses
- An amount and token type
- `c_gas_limit` for compute gas
- `b_gas_limit` for bandwidth gas
- A sender nonce
- Optional `data` bytes for contract, staking, and governance payloads

Default behavior:

- Compute gas defaults to `21_000`
- Bandwidth gas defaults to `data.len() as u64`
- Fees are always denominated in DRT

The fee estimate is represented by
[`FeeEstimate`](../crates/dytallix-sdk/src/lib.rs) and split into compute and
bandwidth components.

## Keystore Model

The SDK ships with a file-backed keystore:

- Path: `~/.dytallix/keystore.json`
- Format: JSON
- Behavior: stores named entries and tracks one active wallet

The CLI builds on top of
[`Keystore`](../crates/dytallix-sdk/src/keystore.rs) and treats the active
entry as the default sender for commands such as `balance`, `send`, `stake`,
and `governance`.

## Network Profiles

The CLI supports three network profiles:

| Profile | Node endpoint | Faucet |
| --- | --- | --- |
| `testnet` | `https://dytallix.com` (`https://dytallix.com/rpc` for contract commands) | `https://dytallix.com/api/faucet` |
| `mainnet` | `https://mainnet.dytallix.com` | Not available |
| `local` | `http://localhost:3030` | `http://localhost:3004` |

The current profile is stored in `~/.dytallix/config.json` and can be changed
with `dytallix config network <testnet|mainnet|local>`.

## Contracts, Governance, and Staking

The current CLI submits higher-level operations by encoding intent into the
transaction `data` bytes:

- `stake:*` payloads for delegation flows
- `governance:*` payloads for proposal and vote flows
- `contract:*` payloads for deployment and contract calls

That keeps the SDK transaction model small while the public network API surface
is still taking shape.

## Current Scope

This repository currently focuses on:

- Core cryptographic primitives
- Transaction building and signing
- Optional node and faucet clients
- A developer-oriented CLI for testnet workflows

For the current command surface, see [CLI reference](cli-reference.md). For the
Rust API surface, see [SDK reference](sdk-reference.md).
