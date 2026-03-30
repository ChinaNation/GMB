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

# ── 下载最新 CI 编译的 WASM（必须成功，不允许本地编译）──
echo "==> 下载最新 WASM..."
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
echo "    使用 CI WASM: $WASM_FILE"

cd "$REPO_ROOT/node"
echo "==> 启动公民链（保留现有链数据，不清库）..."
cargo tauri dev
