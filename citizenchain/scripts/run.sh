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
TARGET_DIR="$REPO_ROOT/target"

# 本地启动脚本只使用当前源码构建 runtime WASM。
# runtime 正式升级走链上 setCode，桌面端启动不再从 GitHub CI 下载 wasm 产物。
unset WASM_FILE
mkdir -p "$TARGET_DIR"
echo "==> 使用本地源码构建 runtime WASM，不下载 GitHub CI WASM..."
echo "    节点启动产物目录: $TARGET_DIR"

# ── 启动 ──
cd "$REPO_ROOT/node"
echo "==> 启动公民链..."
cargo tauri dev
