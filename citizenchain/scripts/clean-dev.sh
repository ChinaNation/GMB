#!/usr/bin/env bash
# 【开发模式】杀所有节点进程 + 清除所有节点数据 + 重新编译并启动（全新创世）
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

# ── 3. 重新编译并启动 ──
cd "$(dirname "$0")/../nodeui"
echo "==> 重新编译并启动 nodeui（开发链：30秒出块，全新创世）..."
cargo tauri dev --features dev-chain
