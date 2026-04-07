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
  *)
    echo "usage: $0 <first-keypair|first-transaction|contract-deploy>" >&2
    exit 1
    ;;
esac
