# Dytallix SDK

Official Rust SDK and CLI for the Dytallix public testnet.

Keypair, faucet, transfer, and basic contract lifecycle are available for experimentation on the public testnet. Staking, governance, and some advanced or operator paths are not yet production-complete.

This repository contains the Rust workspace for the core cryptography crate,
the application SDK, and the `dytallix` CLI.

## Repository Role

- Role: public SDK and CLI source
- Current publication state: canonical public client source for install,
  onboarding, and runtime capability consumption
- Important boundary: this repository describes and consumes the public surface,
  but it does not replace missing source publication for explorer, website, or
  faucet backend deployments

## Quick Links

- [Docs hub](docs/README.md)
- [Capability manifest](docs/public-capabilities.json)
- [Getting started](docs/getting-started.md)
- [Core concepts](docs/core-concepts.md)
- [SDK reference](docs/sdk-reference.md)
- [CLI reference](docs/cli-reference.md)
- [FAQ](docs/faq.md)
- [Examples](examples/README.md)
- [Releases](https://github.com/DytallixHQ/dytallix-sdk/releases)
- [CI workflow](.github/workflows/ci.yml)
- [Public smoke workflow](.github/workflows/public-smoke.yml)
- [Contributing](CONTRIBUTING.md)
- [Security policy](SECURITY.md)
- [Changelog](CHANGELOG.md)
- [License](LICENSE)

## What Is Here

- [`crates/dytallix-core`](crates/dytallix-core) - cryptographic primitives
- [`crates/dytallix-sdk`](crates/dytallix-sdk) - Rust SDK library
- [`crates/dytallix-cli`](crates/dytallix-cli) - public testnet CLI
- [`docs/`](docs/README.md) - repository documentation and capability notes
- [`examples/`](examples/README.md) - runnable examples for keypair, transfer, and contract flows

All signing uses ML-DSA-65 (FIPS 204). All addresses are canonical Bech32m.
Only PQC-native accounts are supported.

## Install

The SDK is not currently published on crates.io. Use the Git repository:

```bash
cargo add dytallix-sdk --git https://github.com/DytallixHQ/dytallix-sdk.git
cargo add dytallix-sdk --git https://github.com/DytallixHQ/dytallix-sdk.git --features network
cargo install --git https://github.com/DytallixHQ/dytallix-sdk.git dytallix-cli --bin dytallix
```

Build from source:

```bash
cargo build --release --bin dytallix
```

Release tags matching `v*` build downloadable CLI archives for Linux, macOS,
and Windows through GitHub Actions.

## Public Testnet Scope

The default public endpoint is `https://dytallix.com`.

Supported on the public website gateway today:

- keypair and wallet generation
- faucet funding and cooldown/status checks
- balance reads and transfers
- chain status, block, and transaction reads
- basic contract deploy, call, query, info, and events
- governance proposal reads
- staking balance reads

Not public-complete on the default website gateway:

- staking writes such as delegate, undelegate, and claim
- governance writes such as vote and propose
- validator/delegation legacy JSON reads
- advanced or operator-only paths

For local development or direct-node testing, point the CLI at a custom endpoint
with `DYTALLIX_ENDPOINT` or `dytallix config set endpoint ...`.

## First CLI Session

```bash
dytallix init
dytallix balance
dytallix faucet status
dytallix send <daddr> 100
```

If you have a compiled contract artifact, the public gateway also supports:

```bash
dytallix contract deploy <path-to-your-contract.wasm>
```

See [Getting started](docs/getting-started.md) and the
[CLI reference](docs/cli-reference.md) for the full flow.

## Repo Boundaries

This repository ships the Rust SDK, core cryptography crate, and the
`dytallix` CLI.

It does include the client code that talks to the live faucet endpoint, but it
does not contain the faucet backend implementation.

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
- [dytallix-node](https://github.com/DytallixHQ/dytallix-node) - public node and runtime source
- [dytallix-contracts](https://github.com/DytallixHQ/dytallix-contracts) - protocol contracts
- [dytallix-docs](https://github.com/DytallixHQ/dytallix-docs) - broader documentation
- [dytallix-explorer](https://github.com/DytallixHQ/dytallix-explorer) - explorer surface documentation repo
- [dytallix-faucet](https://github.com/DytallixHQ/dytallix-faucet) - docs-only faucet surface documentation, not deployed faucet backend source
- [DytallixHQ](https://github.com/DytallixHQ)

## External Links

- [Website](https://dytallix.com)
- [Documentation site](https://dytallix.com/docs)
- [Discord](https://discord.gg/eyVvu5kmPG)
- [Explorer app](https://dytallix.com/build/blockchain)
- [Faucet API](https://dytallix.com/api/faucet)
