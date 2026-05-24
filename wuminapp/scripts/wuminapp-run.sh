#!/usr/bin/env bash
# 清空缓存 + 重新编译 + 启动手机 App
#
# 固定使用 smoldot 轻节点连接区块链（无需 RPC 服务器）。
set -euo pipefail
SCRIPT_DIR="$(cd "$(dirname "$0")" && pwd)"
APP_ROOT="$SCRIPT_DIR/.."
TARGET_DIR="$APP_ROOT/target"
TARGET_APK="$TARGET_DIR/公民.apk"
cd "$APP_ROOT"

SFID_DEV_USB_PORT=8899

# 构造 dart-define 参数
DART_DEFINES=(--dart-define=WUMINAPP_SFID_ENV=dev_usb)
ANDROID_TARGET_PLATFORMS=(--target-platform android-arm,android-arm64)
echo "[启动模式] smoldot 轻节点"

# ── chainspec.json 已在创世时冻结，严禁自动重新生成 ──
# 原因：chainspec 决定 genesis hash，genesis hash 决定 libp2p 通知协议名
#   (/<genesis_hash>/block-announces/1 等)。runtime 重编后 wasm 二进制变化，
#   而 wasm 是 genesis state 的一部分 (:code:)，所以重新 build-spec 会让
#   轻节点的 genesis hash 和线上全节点对不上，smoldot gossip 握手直接
#   ProtocolNotAvailable，wuminapp 永远连不上链。
# 正确做法：runtime 升级走链上 system.setCode 交易，chainspec.json 绝不动。
# 详见 memory/07-ai/chainspec-frozen.md
CHAINSPEC_OUT="$SCRIPT_DIR/../assets/chainspec.json"
CHAINSPEC_SHA_FILE="$SCRIPT_DIR/../assets/chainspec.json.sha256"
if [[ ! -s "$CHAINSPEC_OUT" ]]; then
  echo "错误：$CHAINSPEC_OUT 不存在或为空。chainspec 是创世冻结文件，请从 git 恢复。"
  exit 1
fi
if [[ -s "$CHAINSPEC_SHA_FILE" ]]; then
  EXPECTED_SHA="$(awk '{print $1}' "$CHAINSPEC_SHA_FILE")"
  # bootNodes / lightSyncState 不参与 genesis hash，剔除后校验，允许网络层与轻节点 checkpoint 变更。
  ACTUAL_SHA="$(jq -cS 'del(.bootNodes, .lightSyncState)' "$CHAINSPEC_OUT" | shasum -a 256 | awk '{print $1}')"
  if [[ "$ACTUAL_SHA" != "$EXPECTED_SHA" ]]; then
    echo "错误：chainspec.json 哈希不一致！这是创世冻结文件，禁止修改。"
    echo "       期望 $EXPECTED_SHA"
    echo "       实际 $ACTUAL_SHA"
    echo "恢复方法：git checkout -- wuminapp/assets/chainspec.json"
    exit 1
  fi
  echo "==> chainspec.json 完整性校验通过"
else
  echo "警告：未找到 $CHAINSPEC_SHA_FILE，跳过完整性校验"
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

# ── SFID 本地开发路径：只允许 Android USB adb reverse ──
# 中文注释：开发版 App 内部固定访问 http://127.0.0.1:8899；该地址必须由
# adb reverse 转发到本机 SFID 后端，禁止改走局域网 IP 或其他自定义 URL。
if [[ "$DEVICE_LINE" != "android" ]]; then
  echo "错误：wuminapp 本地开发访问 SFID 只支持 Android USB adb reverse。"
  exit 1
fi
ADB_BIN="${ANDROID_HOME:-$HOME/Library/Android/sdk}/platform-tools/adb"
if [[ ! -x "$ADB_BIN" ]]; then
  echo "错误：未找到 adb：$ADB_BIN"
  exit 1
fi
if ! "$ADB_BIN" get-state >/dev/null 2>&1; then
  echo "错误：未检测到可用 Android USB 设备。"
  exit 1
fi
"$ADB_BIN" reverse "tcp:$SFID_DEV_USB_PORT" "tcp:$SFID_DEV_USB_PORT"
echo "==> Android USB: adb reverse tcp:$SFID_DEV_USB_PORT, wuminapp SFID 环境=dev_usb"

# ── 开发期 USB 桥接：自动检测本地诊断节点并打开 ADB reverse + 注入 dart-define ──
# 远端 prczss/nrcgch 偶发 SubstreamReset 时，本地节点 (--listen-addr ws/30334)
# 作为 wuminapp 第三个稳定 peer 兜底。出门后 localhost 不可达 smoldot 自动忽略。
DEV_NODE_RPC="${WUMINAPP_DEV_LOCAL_RPC:-http://localhost:9945}"
DEV_NODE_PORT="${WUMINAPP_DEV_LOCAL_WS_PORT:-30334}"
DEV_NODE_PEER_ID="$(curl -sS --max-time 2 -H 'Content-Type: application/json' \
  -d '{"id":1,"jsonrpc":"2.0","method":"system_localPeerId","params":[]}' \
  "$DEV_NODE_RPC" 2>/dev/null \
  | python3 -c "import json,sys
try:
    print(json.load(sys.stdin)['result'])
except Exception:
    pass" 2>/dev/null || true)"
if [[ -n "$DEV_NODE_PEER_ID" ]]; then
  echo "==> 检测到本地诊断节点 peer_id=$DEV_NODE_PEER_ID (port=$DEV_NODE_PORT)"
  ADB_BIN="${ANDROID_HOME:-$HOME/Library/Android/sdk}/platform-tools/adb"
  if [[ -x "$ADB_BIN" ]]; then
    "$ADB_BIN" reverse "tcp:$DEV_NODE_PORT" "tcp:$DEV_NODE_PORT" >/dev/null 2>&1 || true
    echo "    已配置 adb reverse tcp:$DEV_NODE_PORT -> host:$DEV_NODE_PORT"
  fi
  DART_DEFINES+=(--dart-define=WUMINAPP_DEV_LOCAL_PEER_ID="$DEV_NODE_PEER_ID")
  DART_DEFINES+=(--dart-define=WUMINAPP_DEV_LOCAL_WS_PORT="$DEV_NODE_PORT")
else
  echo "==> 未检测到本地诊断节点 ($DEV_NODE_RPC)，跳过 USB 桥接（仅走远端 bootnode）"
fi

sync_android_artifact() {
  local source_apk="build/app/outputs/flutter-apk/app-debug.apk"
  if [[ -f "$source_apk" ]]; then
    mkdir -p "$TARGET_DIR"
    cp "$source_apk" "$TARGET_APK"
    echo "==> Android 产物已保存: $TARGET_APK"
  fi
}

if [[ "$DEVICE_LINE" == "android" ]]; then
  # 中文注释：启动脚本固定把本地 APK 产物沉淀到项目根 target/，便于离线安装和回滚。
  echo "==> 生成 Android 产物..."
  flutter build apk --debug "${ANDROID_TARGET_PLATFORMS[@]}" "${DART_DEFINES[@]}"
  sync_android_artifact
fi

echo "==> 编译并启动 App..."
flutter run "${DART_DEFINES[@]}"
sync_android_artifact
