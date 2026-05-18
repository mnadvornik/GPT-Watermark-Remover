#!/usr/bin/env bash
set -euo pipefail

ROOT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"

if ! rustup target list --installed | grep -qx "x86_64-pc-windows-gnu"; then
  rustup target add x86_64-pc-windows-gnu
fi
cargo build --release --target x86_64-pc-windows-gnu --manifest-path "$ROOT_DIR/Cargo.toml"

echo "Built: $ROOT_DIR/target/x86_64-pc-windows-gnu/release/hidden-character-cleaner.exe"
