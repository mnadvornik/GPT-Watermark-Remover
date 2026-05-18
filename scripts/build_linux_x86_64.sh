#!/usr/bin/env bash
set -euo pipefail

ROOT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"

export ZIG_LOCAL_CACHE_DIR="$ROOT_DIR/target/zig-local-cache"
export ZIG_GLOBAL_CACHE_DIR="$ROOT_DIR/target/zig-global-cache"

cargo zigbuild --release --target x86_64-unknown-linux-gnu --manifest-path "$ROOT_DIR/Cargo.toml"

echo "Built: $ROOT_DIR/target/x86_64-unknown-linux-gnu/release/hidden-character-cleaner"
