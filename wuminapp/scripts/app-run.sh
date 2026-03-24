#!/usr/bin/env bash
# 启动手机 App（不清空缓存）
#
# 固定使用 smoldot 轻节点连接区块链（无需 RPC 服务器）。
set -euo pipefail
SCRIPT_DIR="$(cd "$(dirname "$0")" && pwd)"
cd "$(dirname "$0")/.."

ENV_FILE="../sfid/.env.dev.local"
if [[ -f "$ENV_FILE" ]]; then
  set -a
  source "$ENV_FILE"
  set +a
fi

if [[ -z "${WUMINAPP_API_BASE_URL:-}" && -n "${SFID_PUBLIC_BASE_URL:-}" ]]; then
  WUMINAPP_API_BASE_URL="$SFID_PUBLIC_BASE_URL"
fi
if [[ -z "${WUMINAPP_API_BASE_URL:-}" && -n "${SFID_BIND_ADDR:-}" ]]; then
  WUMINAPP_API_BASE_URL="http://${SFID_BIND_ADDR}"
fi

if [[ -z "${WUMINAPP_API_BASE_URL:-}" ]]; then
  echo "Missing WUMINAPP_API_BASE_URL or SFID_BIND_ADDR. Please configure a phone-reachable SFID address."
  exit 1
fi

# 构造 dart-define 参数
DART_DEFINES=(--dart-define=WUMINAPP_API_BASE_URL="$WUMINAPP_API_BASE_URL")
echo "[启动模式] smoldot 轻节点"

# 确保 Rust 原生库已编译（增量构建，未改动时秒过）
DEVICE_LINE=$(flutter devices --machine 2>/dev/null | python3 -c "
import sys, json
try:
    devices = json.load(sys.stdin)
    for d in devices:
        p = d.get('targetPlatform','')
        if 'android' in p:
            print('android'); break
        elif 'ios' in p:
            print('ios'); break
    else:
        print('android')
except:
    print('android')
" 2>/dev/null || echo "android")
"$SCRIPT_DIR/build-smoldot-native.sh" "$DEVICE_LINE"

flutter run "${DART_DEFINES[@]}"
