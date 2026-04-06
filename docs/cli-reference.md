# CLI Reference

[Docs hub](README.md) | [Getting started](getting-started.md) | [FAQ](faq.md)

## Install

```bash
cargo install --git https://github.com/DytallixHQ/dytallix-sdk.git dytallix-cli --bin dytallix
```

Global help:

```bash
dytallix --help
```

## Local State

- Keystore: `~/.dytallix/keystore.json`
- Config: `~/.dytallix/config.json`

## Top-Level Commands

| Command | Purpose | Example |
| --- | --- | --- |
| `init` | Create a wallet, save it, and request faucet funds | `dytallix init` |
| `wallet` | Create, import, export, switch, list, rotate, and inspect wallets | `dytallix wallet info` |
| `balance` | Show DGT and DRT balances | `dytallix balance` |
| `send` | Send DGT or DRT | `dytallix send <daddr> 100` |
| `faucet` | Request faucet funds or inspect eligibility | `dytallix faucet status` |
| `stake` | Delegate, undelegate, claim rewards, or view delegation state | `dytallix stake status` |
| `governance` | Query proposals or submit votes and proposals | `dytallix governance proposals` |
| `contract` | Deploy, call, query, and inspect contracts | `dytallix contract info <daddr>` |
| `node` | Operate or inspect a local node workflow | `dytallix node status` |
| `chain` | Query block, epoch, status, and chain params | `dytallix chain status` |
| `crypto` | Key generation, signing, verification, and keystore inspection | `dytallix crypto keygen` |
| `dev` | Small developer utilities and quick links | `dytallix dev benchmark` |
| `config` | Show, set, reset, and switch CLI config | `dytallix config network testnet` |

## Command Groups

### `init`

Bootstraps the default testnet developer flow:

```bash
dytallix init
```

This command:

- generates an ML-DSA-65 keypair
- derives a D-Addr
- writes the keystore
- submits a faucet request
- waits for DGT and DRT to appear

### `wallet`

Subcommands:

- `create [--name NAME]`
- `import --key-file PATH [--name NAME]`
- `export --output PATH`
- `list`
- `switch NAME`
- `rotate`
- `info`

Examples:

```bash
dytallix wallet create --name default
dytallix wallet list
dytallix wallet info
```

### `balance`, `send`, and `faucet`

Examples:

```bash
dytallix balance
dytallix balance <daddr>
dytallix send --token dgt <daddr> 25
dytallix faucet
dytallix faucet status
```

### `stake`

Subcommands:

- `delegate <validator> <amount>`
- `undelegate <validator> <amount>`
- `claim`
- `status`

Examples:

```bash
dytallix stake delegate <validator> 1000
dytallix stake status
```

### `governance`

Subcommands:

- `proposals`
- `vote <id> <yes|no|abstain>`
- `propose`
- `status <id>`

Examples:

```bash
dytallix governance proposals
dytallix governance vote 7 yes
```

### `contract`

Subcommands:

- `deploy <wasm-file>`
- `call <address> <method> [args...]`
- `query <address> <method> [args...]`
- `info <address>`
- `events <address>`

Examples:

```bash
dytallix contract deploy ./my_contract.wasm
dytallix contract query <contract> get_count
```

### `chain`

Subcommands:

- `status`
- `block <number|hash|latest|finalized>`
- `epoch`
- `params`

Examples:

```bash
dytallix chain status
dytallix chain block latest
```

### `node`

Subcommands:

- `start`
- `stop`
- `status`
- `peers`
- `logs`

The `start` and `stop` commands look for local helper scripts such as
`start-local.sh` and `stop-local.sh` relative to the current directory.

### `crypto`

Subcommands:

- `keygen [--scheme ml-dsa-65|slh-dsa]`
- `sign <message>`
- `verify <message> <signature> <pubkey>`
- `address <pubkey>`
- `inspect <keystore-file>`

Examples:

```bash
dytallix crypto keygen
dytallix crypto sign "hello dytallix"
```

### `dev`

Subcommands:

- `faucet-server`
- `explorer`
- `docs`
- `discord`
- `github`
- `decode <hex>`
- `encode <text>`
- `simulate-tx <address> <amount>`
- `benchmark`

### `config`

Subcommands:

- `show`
- `set <key> <value>`
- `network <testnet|mainnet|local>`
- `reset`

Examples:

```bash
dytallix config show
dytallix config network local
```

## Network Profiles

The CLI resolves endpoints from the active network profile:

- `testnet` -> `https://dytallix.com`
- `mainnet` -> `https://mainnet.dytallix.com`
- `local` -> `http://localhost:8545`

For faucet behavior and other operational notes, see [Core concepts](core-concepts.md)
and [FAQ](faq.md).
