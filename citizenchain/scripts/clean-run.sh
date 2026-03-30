#!/usr/bin/env bash
# 杀进程 + 清链数据 + 重新编译启动（全新创世）
set -euo pipefail

APP_DATA_DIR="$HOME/Library/Application Support/org.chinanation.citizenchain.desktop"

cleanup() {
    echo ""
    echo "==> 正在关闭节点进程..."
    pkill -f "citizenchain-node" 2>/dev/null || true
    pkill -f "node-bin-" 2>/dev/null || true
    sleep 1
    echo "    节点已关闭"
}
trap cleanup EXIT INT TERM HUP

# ── 1. 杀进程 ──
echo "==> 杀掉所有节点进程..."
pkill -9 -f "citizenchain-node" 2>/dev/null || true
pkill -9 -f "node-bin-" 2>/dev/null || true
sleep 1
echo "    已清理"

# ── 2. 清链数据 ──
echo "==> 清除链数据：$APP_DATA_DIR"
rm -rf "$APP_DATA_DIR"
echo "    已清除"

# ── 3. 下载最新 CI WASM（必须成功）──
CHAIN_ROOT="$(cd "$(dirname "$0")/.." && pwd)"
WASM_DIR="$CHAIN_ROOT/target/ci-wasm"
echo "==> 下载最新 WASM..."
rm -rf "$WASM_DIR"
mkdir -p "$WASM_DIR"
if ! gh run download --name citizenchain-wasm --dir "$WASM_DIR" -R ChinaNation/GMB; then
    echo "错误：无法下载 WASM。gh auth login 后重试。"
    exit 1
fi
export WASM_FILE="$WASM_DIR/citizenchain.compact.compressed.wasm"
[ -f "$WASM_FILE" ] || { echo "错误：WASM 文件不存在"; exit 1; }
echo "    WASM: $WASM_FILE"

# ── 4. 彻底清除所有编译缓存（用 find，不用 glob）──
echo "==> 清除编译缓存..."
find "$CHAIN_ROOT/target" -maxdepth 3 -type d -name "citizenchain-*" -path "*/build/*" -exec rm -rf {} + 2>/dev/null || true
find "$CHAIN_ROOT/target" -maxdepth 2 -type d -name "citizenchain" -path "*/wbuild/*" -exec rm -rf {} + 2>/dev/null || true
find "$CHAIN_ROOT/target" -name "libcitizenchain*" -delete 2>/dev/null || true
find "$CHAIN_ROOT/target" -name "libnode*" -delete 2>/dev/null || true
find "$CHAIN_ROOT/target" -maxdepth 3 -type d -name "node-*" -path "*/build/*" -exec rm -rf {} + 2>/dev/null || true
echo "    已清除"

# ── 5. 启动 ──
cd "$CHAIN_ROOT/node"
echo "==> 启动公民链（全新创世）..."
cargo tauri dev
