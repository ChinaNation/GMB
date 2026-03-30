#!/usr/bin/env bash
# 不清库，继续启动节点
set -euo pipefail

cleanup() {
    echo ""
    echo "==> 正在关闭节点进程..."
    pkill -f "node-bin-\|citizenchain-node\|nodeui-desktop-shell" 2>/dev/null || true
    sleep 1
    pkill -9 -f "node-bin-\|citizenchain-node" 2>/dev/null || true
    echo "    节点已关闭"
}
trap cleanup EXIT INT TERM HUP

SCRIPT_DIR="$(cd "$(dirname "$0")" && pwd)"
REPO_ROOT="$(dirname "$SCRIPT_DIR")"
WASM_DIR="$REPO_ROOT/target/ci-wasm"

# ── 1. 下载最新 CI WASM（必须成功，失败则拒绝启动）──
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

# ── 2. 彻底清除 runtime 编译缓存，强制用最新 CI WASM ──
echo "==> 清除 runtime 编译缓存..."
rm -rf "$REPO_ROOT/target/debug/build/citizenchain-"*
rm -rf "$REPO_ROOT/target/debug/wbuild/citizenchain"
rm -rf "$REPO_ROOT/target/release/build/citizenchain-"*
rm -rf "$REPO_ROOT/target/release/wbuild/citizenchain"
cargo clean --manifest-path "$REPO_ROOT/Cargo.toml" -p citizenchain 2>/dev/null || true
cargo clean --release --manifest-path "$REPO_ROOT/Cargo.toml" -p citizenchain 2>/dev/null || true
echo "    已清除"

# ── 3. 启动 ──
cd "$REPO_ROOT/node"
echo "==> 启动公民链（保留现有链数据，不清库）..."
cargo tauri dev
