#!/usr/bin/env bash
# 不清库，继续启动节点
set -euo pipefail

cleanup() {
    echo ""
    echo "==> 正在关闭节点进程..."
    pkill -f "citizenchain" 2>/dev/null || true
    sleep 1
    echo "    节点已关闭"
}
trap cleanup EXIT INT TERM HUP

SCRIPT_DIR="$(cd "$(dirname "$0")" && pwd)"
REPO_ROOT="$(dirname "$SCRIPT_DIR")"
WASM_DIR="$REPO_ROOT/target/wasm"

# ── 1. 清除所有旧的 wasm 相关目录，只保留 target/wasm ──
echo "==> 清除旧 WASM 目录..."
find "$REPO_ROOT/target" -maxdepth 1 -type d -iname "*wasm*" ! -name "wasm" -exec rm -rf {} + 2>/dev/null || true

# ── 2. 下载最新 CI WASM 到 target/wasm（直接覆盖）──
echo "==> 下载最新 WASM..."
mkdir -p "$WASM_DIR"
if ! gh run download --name citizenchain-wasm --dir "$WASM_DIR" -R ChinaNation/GMB; then
    echo "错误：无法下载 WASM。gh auth login 后重试。"
    exit 1
fi
export WASM_FILE="$WASM_DIR/citizenchain.compact.compressed.wasm"
[ -f "$WASM_FILE" ] || { echo "错误：WASM 文件不存在"; exit 1; }
echo "    WASM: $WASM_FILE"

# ── 3. 彻底清除 runtime 编译缓存 ──
echo "==> 清除 runtime 缓存..."
find "$REPO_ROOT/target" -maxdepth 3 -type d -name "citizenchain-*" -path "*/build/*" -exec rm -rf {} + 2>/dev/null || true
find "$REPO_ROOT/target" -maxdepth 2 -type d -name "citizenchain" -path "*/wbuild/*" -exec rm -rf {} + 2>/dev/null || true
find "$REPO_ROOT/target" -name "libcitizenchain*" -delete 2>/dev/null || true
echo "    已清除"

# ── 4. 启动 ──
cd "$REPO_ROOT/node"
echo "==> 启动公民链..."
cargo tauri dev
