#!/usr/bin/env bash
set -euo pipefail

# 中文注释：本地控制台只监听回环地址；启动前编译 Touch ID 辅助程序。
DEPLOY_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
RUNTIME_DIR="$DEPLOY_DIR/.runtime"
mkdir -p "$RUNTIME_DIR"
chmod 700 "$RUNTIME_DIR"
xcrun swiftc "$DEPLOY_DIR/touchid.swift" -o "$RUNTIME_DIR/touchid-auth"
chmod 700 "$RUNTIME_DIR/touchid-auth"
# 中文注释：手动启动时同步编译按需唤醒器；launchd 自身会使用已经安装好的这一份二进制。
xcrun swiftc "$DEPLOY_DIR/socket-launcher.swift" -o "$RUNTIME_DIR/socket-launcher"
chmod 700 "$RUNTIME_DIR/socket-launcher"
# 中文注释：launchd 使用精简 PATH；优先用当前 PATH，否则从本机 NVM 目录定位 Node。
NODE_BIN="$(command -v node || true)"
if [[ -z "$NODE_BIN" ]]; then
  for candidate in "$HOME"/.nvm/versions/node/*/bin/node; do
    [[ -x "$candidate" ]] && NODE_BIN="$candidate"
  done
fi
[[ -n "$NODE_BIN" ]] || { echo '未找到 Node.js，无法启动部署控制台' >&2; exit 1; }
exec "$NODE_BIN" "$DEPLOY_DIR/server.mjs"
