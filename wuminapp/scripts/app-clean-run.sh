#!/usr/bin/env bash
# 清空缓存 + 重新编译 + 启动手机 App
#
# 默认使用 smoldot 轻节点连接区块链（无需 RPC 服务器）。
# 如需回退到传统 HTTP RPC 模式（开发调试用），设置环境变量：
#   export WUMINAPP_RPC_URL=http://10.92.152.128:9944
set -euo pipefail
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
# 仅在显式设置了 WUMINAPP_RPC_URL 时才传入（回退到 HTTP RPC 模式）
if [[ -n "${WUMINAPP_RPC_URL:-}" ]]; then
  DART_DEFINES+=(--dart-define=WUMINAPP_RPC_URL="$WUMINAPP_RPC_URL")
  echo "[启动模式] HTTP RPC 回退: $WUMINAPP_RPC_URL"
else
  echo "[启动模式] smoldot 轻节点"
fi

echo "==> 清空构建缓存..."
flutter clean
echo "==> 获取依赖..."
flutter pub get
echo "==> 编译并启动 App..."
flutter run "${DART_DEFINES[@]}"
