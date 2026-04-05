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

# ── 同步最新 chainspec（从本地节点二进制导出） ──
CHAIN_ROOT="$SCRIPT_DIR/../../citizenchain"
NODE_BIN="$CHAIN_ROOT/target/debug/citizenchain"
WASM_DIR="$CHAIN_ROOT/target/ci-wasm"
CHAINSPEC_OUT="$SCRIPT_DIR/../assets/chainspec.json"

if [[ -x "$NODE_BIN" ]]; then
  # 查找 WASM 文件（CI 下载的或本地编译的）
  WASM_FILE="${WASM_FILE:-}"
  if [[ -z "$WASM_FILE" && -f "$WASM_DIR/citizenchain.compact.compressed.wasm" ]]; then
    WASM_FILE="$WASM_DIR/citizenchain.compact.compressed.wasm"
  fi
  if [[ -n "$WASM_FILE" && -f "$WASM_FILE" ]]; then
    echo "==> 从本地节点导出 chainspec..."
    WASM_FILE="$WASM_FILE" "$NODE_BIN" build-spec --raw 2>/dev/null > "$CHAINSPEC_OUT.tmp"
    if [[ -s "$CHAINSPEC_OUT.tmp" ]]; then
      mv "$CHAINSPEC_OUT.tmp" "$CHAINSPEC_OUT"
      echo "    已更新 assets/chainspec.json"
    else
      rm -f "$CHAINSPEC_OUT.tmp"
      echo "    警告：build-spec 输出为空，保留旧 chainspec"
    fi
  else
    echo "    跳过 chainspec 同步（未找到 WASM 文件，请先运行 citizenchain/scripts/run.sh）"
  fi
else
  echo "    跳过 chainspec 同步（未找到节点二进制 $NODE_BIN，请先编译 citizenchain）"
fi

echo "==> 清除 Rust 编译缓存..."
(cd "rust" && ~/.cargo/bin/cargo clean 2>/dev/null || true)
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
