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

cd "$(dirname "$0")/../nodeui"
echo "==> 启动 nodeui（保留现有链数据，不清库）..."
cargo tauri dev
