#!/usr/bin/env bash
# 清空缓存 + 重新编译 + 启动手机 App
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

# ── 强制更新 chainspec ──
# 每次运行都从 Linux CI artifact 下载最新 chainspec，删除旧的，用新的覆盖。
CHAINSPEC_PATH="assets/chainspec.json"
echo "==> 下载最新 chainspec.json ..."
rm -f "$CHAINSPEC_PATH"
DOWNLOAD_DIR=$(mktemp -d)
if gh run download --name citizenchain-chainspec --dir "$DOWNLOAD_DIR" -R ChinaNation/GMB 2>/dev/null; then
  if [[ -f "$DOWNLOAD_DIR/chainspec.json" ]]; then
    cp "$DOWNLOAD_DIR/chainspec.json" "$CHAINSPEC_PATH"
    echo "    已更新 chainspec.json ($(wc -c < "$CHAINSPEC_PATH" | tr -d ' ') bytes)"
  else
    echo "ERROR: artifact 下载成功但未找到 chainspec.json" >&2
    rm -rf "$DOWNLOAD_DIR"
    exit 1
  fi
else
  echo "ERROR: 无法从 CI 下载 chainspec artifact，请确认 gh 已登录且 citizenchain Linux CI 至少成功运行过一次" >&2
  rm -rf "$DOWNLOAD_DIR"
  exit 1
fi
rm -rf "$DOWNLOAD_DIR"

echo "==> 编译 Rust 原生库..."
# 检测目标平台：通过 flutter devices 判断
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
echo "    目标平台: $DEVICE_LINE"
"$SCRIPT_DIR/build-smoldot-native.sh" "$DEVICE_LINE"

echo "==> 清空构建缓存..."
flutter clean
echo "==> 获取依赖..."
flutter pub get
echo "==> 编译并启动 App..."
flutter run "${DART_DEFINES[@]}"
