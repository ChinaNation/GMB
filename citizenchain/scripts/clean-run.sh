#!/usr/bin/env bash
# 杀所有进程 + 清除所有链数据 + 重新编译并启动（全新创世）
set -euo pipefail

APP_DATA_DIR="$HOME/Library/Application Support/org.chinanation.citizenchain.desktop"

cleanup() {
    echo ""
    echo "==> 正在关闭节点进程..."
    pkill -f "node-bin-\|citizenchain-node\|nodeui-desktop-shell" 2>/dev/null || true
    sleep 1
    pkill -9 -f "node-bin-\|citizenchain-node" 2>/dev/null || true
    echo "    节点已关闭"
}
trap cleanup EXIT INT TERM HUP

# ── 1. 杀掉所有相关进程 ──
echo "==> 杀掉所有节点相关进程..."
pkill -9 -f "nodeui-desktop-shell\|citizenchain-node\|node-bin-" 2>/dev/null || true
sleep 1
echo "    进程已全部清理"

# ── 2. 清除所有链数据 ──
echo "==> 清除链数据：$APP_DATA_DIR"
rm -rf "$APP_DATA_DIR"
echo "    已清除"

# ── 3. 下载最新 CI WASM（必须成功，失败则拒绝启动）──
CHAIN_ROOT="$(cd "$(dirname "$0")/.." && pwd)"
WASM_DIR="$CHAIN_ROOT/target/ci-wasm"
echo "==> 下载最新 WASM..."
rm -rf "$WASM_DIR"
mkdir -p "$WASM_DIR"
if ! gh run download --name citizenchain-wasm --dir "$WASM_DIR" -R ChinaNation/GMB; then
    echo "错误：无法下载 WASM artifact。"
    echo "  1. gh auth login"
    echo "  2. WASM CI 至少成功运行过一次"
    exit 1
fi
export WASM_FILE="$WASM_DIR/citizenchain.compact.compressed.wasm"
if [ ! -f "$WASM_FILE" ]; then
    echo "错误：WASM 文件不存在: $WASM_FILE"
    exit 1
fi
echo "    使用 CI WASM: $WASM_FILE"

# ── 4. 彻底清除所有编译缓存，强制用最新 CI WASM ──
echo "==> 清除编译缓存..."
rm -rf "$CHAIN_ROOT/target/debug/build/citizenchain-"*
rm -rf "$CHAIN_ROOT/target/debug/wbuild/citizenchain"
rm -rf "$CHAIN_ROOT/target/release/build/citizenchain-"*
rm -rf "$CHAIN_ROOT/target/release/wbuild/citizenchain"
cargo clean --manifest-path "$CHAIN_ROOT/Cargo.toml" -p citizenchain -p node 2>/dev/null || true
cargo clean --release --manifest-path "$CHAIN_ROOT/Cargo.toml" -p citizenchain -p node 2>/dev/null || true
echo "    已清除"

# ── 5. 启动 ──
cd "$CHAIN_ROOT/node"
echo "==> 启动公民链（全新创世）..."
cargo tauri dev
