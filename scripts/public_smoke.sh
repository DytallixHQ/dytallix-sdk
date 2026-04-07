#!/usr/bin/env bash

set -euo pipefail

ROOT="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
MODE="${1:-}"

case "$MODE" in
  first-keypair)
    cargo run -p dytallix-sdk --example first-keypair
    ;;
  first-transaction)
    cargo run -p dytallix-sdk --features network --example first-transaction
    ;;
  contract-deploy)
    export RUSTUP_TOOLCHAIN="${RUSTUP_TOOLCHAIN:-stable}"
    rustup target add wasm32-unknown-unknown
    cargo build --bin dytallix
    cargo build \
      --manifest-path "$ROOT/examples/contracts/minimal_contract/Cargo.toml" \
      --target wasm32-unknown-unknown \
      --release
    smoke_home="$(mktemp -d)"
    trap 'rm -rf "$smoke_home"' EXIT
    HOME="$smoke_home" "$ROOT/target/debug/dytallix" init
    HOME="$smoke_home" "$ROOT/target/debug/dytallix" contract deploy \
      "$ROOT/examples/contracts/minimal_contract/target/wasm32-unknown-unknown/release/minimal_contract.wasm"
    ;;
  contract-lifecycle)
    endpoint="${DYTALLIX_ENDPOINT:-https://www.dytallix.com}"
    export RUSTUP_TOOLCHAIN="${RUSTUP_TOOLCHAIN:-stable}"
    rustup target add wasm32-unknown-unknown
    cargo build --bin dytallix
    cargo build \
      --manifest-path "$ROOT/examples/contracts/minimal_contract/Cargo.toml" \
      --target wasm32-unknown-unknown \
      --release
    smoke_home="$(mktemp -d)"
    trap 'rm -rf "$smoke_home"' EXIT
    HOME="$smoke_home" "$ROOT/target/debug/dytallix" init
    HOME="$smoke_home" "$ROOT/target/debug/dytallix" config set endpoint "$endpoint"
    deploy_output="$(
      HOME="$smoke_home" "$ROOT/target/debug/dytallix" contract deploy \
        "$ROOT/examples/contracts/minimal_contract/target/wasm32-unknown-unknown/release/minimal_contract.wasm"
    )"
    printf '%s\n' "$deploy_output"
    address="$(printf '%s\n' "$deploy_output" | sed -n 's/^Contract address: //p' | tail -n 1)"
    if [[ -z "$address" ]]; then
      echo "Failed to parse contract address from deploy output" >&2
      exit 1
    fi
    HOME="$smoke_home" "$ROOT/target/debug/dytallix" contract info "$address"
    HOME="$smoke_home" "$ROOT/target/debug/dytallix" contract query "$address" ping
    HOME="$smoke_home" "$ROOT/target/debug/dytallix" contract call "$address" ping
    HOME="$smoke_home" "$ROOT/target/debug/dytallix" contract events "$address"
    ;;
  *)
    echo "usage: $0 <first-keypair|first-transaction|contract-deploy|contract-lifecycle>" >&2
    exit 1
    ;;
esac
