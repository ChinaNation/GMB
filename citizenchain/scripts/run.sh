#!/usr/bin/env bash
# 不清库，继续启动节点
set -euo pipefail

# ── 退出时杀掉所有节点子进程，防止孤儿进程 ──
cleanup() {
    echo ""
    echo "==> 正在关闭节点进程..."
    pkill -f "node-bin-" 2>/dev/null || true
    pkill -f "citizenchain-node" 2>/dev/null || true
    pkill -f "nodeui-desktop-shell" 2>/dev/null || true
    sleep 1
    # 确认无残留，强杀
    if pgrep -f "node-bin-|citizenchain-node" >/dev/null 2>&1; then
        pkill -9 -f "node-bin-|citizenchain-node" 2>/dev/null || true
        sleep 1
    fi
    echo "    节点已关闭"
}
trap cleanup EXIT INT TERM HUP

SCRIPT_DIR="$(cd "$(dirname "$0")" && pwd)"
REPO_ROOT="$(dirname "$SCRIPT_DIR")"
WASM_DIR="$REPO_ROOT/target/ci-wasm"

# ── 下载最新 CI 编译的 WASM，确保本地用的和链上一致 ──
echo "==> 下载最新 WASM..."
mkdir -p "$WASM_DIR"
if gh run download --name citizenchain-wasm --dir "$WASM_DIR" -R ChinaNation/GMB 2>/dev/null; then
    export WASM_FILE="$WASM_DIR/citizenchain.compact.compressed.wasm"
    echo "    使用 CI WASM: $WASM_FILE"
else
    echo "    ⚠ 未能下载 WASM artifact，将从本地源码编译（可能与链上不一致）"
fi

cd "$REPO_ROOT/nodeui"
echo "==> 启动 nodeui（保留现有链数据，不清库）..."
cargo tauri dev
