#!/usr/bin/env bash
# 杀所有节点进程 + 清除所有节点数据 + 重新编译并启动（全新创世）
set -euo pipefail

APP_DATA_DIR="$HOME/Library/Application Support/org.chinanation.citizenchain.desktop"

# ── 退出时杀掉所有节点子进程，防止孤儿进程 ──
cleanup() {
    echo ""
    echo "==> 正在关闭节点进程..."
    pkill -f "node-bin-" 2>/dev/null || true
    pkill -f "citizenchain-node" 2>/dev/null || true
    pkill -f "nodeui-desktop-shell" 2>/dev/null || true
    sleep 1
    if pgrep -f "node-bin-|citizenchain-node" >/dev/null 2>&1; then
        pkill -9 -f "node-bin-|citizenchain-node" 2>/dev/null || true
        sleep 1
    fi
    echo "    节点已关闭"
}
trap cleanup EXIT INT TERM HUP

# ── 1. 杀掉所有相关进程 ──
echo "==> 杀掉所有节点相关进程..."
pkill -9 -f "nodeui-desktop-shell" 2>/dev/null || true
pkill -9 -f "citizenchain-node" 2>/dev/null || true
pkill -9 -f "node-bin-" 2>/dev/null || true
sleep 1

if pgrep -f "nodeui-desktop-shell|citizenchain-node|node-bin-" >/dev/null 2>&1; then
    echo "    仍有残留，再次强杀..."
    pkill -9 -f "nodeui-desktop-shell|citizenchain-node|node-bin-" 2>/dev/null || true
    sleep 1
fi
echo "    进程已全部清理"

# ── 2. 删除所有节点数据（含 runtime-secrets 中的旧二进制副本）──
echo "==> 清除节点数据：$APP_DATA_DIR"
rm -rf "$APP_DATA_DIR"
echo "    已清除"

# ── 3. 清除编译缓存，强制全量重编译 ──
CHAIN_ROOT="$(cd "$(dirname "$0")/.." && pwd)"
echo "==> 清除节点编译缓存..."
cargo clean --release --manifest-path "$CHAIN_ROOT/Cargo.toml" -p citizenchain -p node 2>/dev/null || true
cargo clean --release --manifest-path "$CHAIN_ROOT/nodeui/backend/Cargo.toml" -p nodeui-desktop-shell 2>/dev/null || true
echo "    编译缓存已清除"

# ── 4. 下载预编译 WASM（必须成功，不允许本地编译）──
WASM_DIR="$CHAIN_ROOT/target/ci-wasm"
echo "==> 下载最新 CI 预编译 WASM..."
rm -rf "$WASM_DIR"
mkdir -p "$WASM_DIR"
if ! gh run download --name citizenchain-wasm --dir "$WASM_DIR" -R ChinaNation/GMB; then
    echo "错误：无法下载 WASM artifact。"
    echo "请确认："
    echo "  1. gh CLI 已登录（gh auth login）"
    echo "  2. WASM CI 至少成功运行过一次"
    echo "  3. 网络连接正常"
    exit 1
fi
export WASM_FILE="$WASM_DIR/citizenchain.compact.compressed.wasm"
if [ ! -f "$WASM_FILE" ]; then
    echo "错误：WASM 文件不存在: $WASM_FILE"
    exit 1
fi
echo "    WASM_FILE=$WASM_FILE"

# ── 5. 重新编译并启动 ──
cd "$CHAIN_ROOT/node"
echo "==> 强制重新编译并启动公民链（全新创世）..."
cargo tauri dev
