#!/usr/bin/env bash
# 启动手机 App（不清空缓存）
set -euo pipefail
cd "$(dirname "$0")/.."

ENV_FILE="../sfid/.env.dev.local"
if [[ -f "$ENV_FILE" ]]; then
  set -a
  source "$ENV_FILE"
  set +a
fi

WUMINAPP_RPC_URL="${WUMINAPP_RPC_URL:-http://10.92.152.128:9944}"
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

flutter run \
  --dart-define=WUMINAPP_RPC_URL="$WUMINAPP_RPC_URL" \
  --dart-define=WUMINAPP_API_BASE_URL="$WUMINAPP_API_BASE_URL"
