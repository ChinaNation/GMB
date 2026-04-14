#!/usr/bin/env bash
# 不清库，继续启动节点
set -euo pipefail

cleanup() {
    echo ""
    echo "==> 正在关闭节点进程..."
    pkill -f "citizenchain" 2>/dev/null || true
    # vite dev server 进程名是 node，pkill citizenchain 杀不到，需按端口清理
    lsof -ti:5173 2>/dev/null | xargs kill -9 2>/dev/null || true
    sleep 1
    echo "    节点已关闭"
}
trap cleanup EXIT INT TERM HUP

SCRIPT_DIR="$(cd "$(dirname "$0")" && pwd)"
REPO_ROOT="$(dirname "$SCRIPT_DIR")"
WASM_DIR="$REPO_ROOT/target/wasm"

# ── 1. 清空 target/wasm/ 下所有文件 ──
echo "==> 清空 target/wasm/..."
rm -rf "$WASM_DIR"/*

# ── 2. 下载最新 CI WASM 到 target/wasm ──
echo "==> 下载最新 WASM..."
mkdir -p "$WASM_DIR"
if ! gh run download --name citizenchain-wasm --dir "$WASM_DIR" -R ChinaNation/GMB; then
    echo "错误：无法下载 WASM。gh auth login 后重试。"
    exit 1
fi
export WASM_FILE="$WASM_DIR/citizenchain.compact.compressed.wasm"
[ -f "$WASM_FILE" ] || { echo "错误：WASM 文件不存在"; exit 1; }
echo "    WASM: $WASM_FILE"


# ── 3. 启动 ──
cd "$REPO_ROOT/node"
echo "==> 启动公民链..."
cargo tauri dev
