#!/usr/bin/env bash
set -euo pipefail

SCRIPT_DIR="$(cd "$(dirname "$0")" && pwd)"
ROOT="$SCRIPT_DIR/.."
BINARIES_DIR="$ROOT/nodeui/backend/binaries"
BUILD_NAME="node"
SIDECAR_NAME="citizenchain-node"

echo "==> 编译节点二进制 (release)..."
cargo build --release --manifest-path "$ROOT/Cargo.toml" -p node

echo "==> 复制二进制到 nodeui/backend/binaries/..."
cp "$ROOT/target/release/$BUILD_NAME" "$BINARIES_DIR/$SIDECAR_NAME"
cp "$ROOT/target/release/$BUILD_NAME" "$BINARIES_DIR/${SIDECAR_NAME}-aarch64-apple-darwin"

echo "==> 更新 sha256..."
shasum -a 256 "$BINARIES_DIR/$SIDECAR_NAME" | awk '{print $1}' > "$BINARIES_DIR/${SIDECAR_NAME}.sha256"

echo "==> 启动 nodeui..."
cd "$ROOT/nodeui"
cargo tauri dev
