#!/usr/bin/env bash
# 启动手机 App（不清空缓存）
set -euo pipefail
cd "$(dirname "$0")/../wuminapp"

# 本机局域网 IP（自动检测）
LAN_IP=$(ipconfig getifaddr en0 2>/dev/null || ipconfig getifaddr en1 2>/dev/null || echo "127.0.0.1")

flutter run \
  --dart-define=WUMINAPP_RPC_URL=http://${LAN_IP}:9944 \
  --dart-define=WUMINAPP_API_BASE_URL=http://${LAN_IP}:8899
