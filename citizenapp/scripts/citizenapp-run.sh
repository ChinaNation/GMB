#!/usr/bin/env bash
# 清空缓存 + 重新编译 + 启动公民
#
# 固定使用 smoldot 轻节点连接区块链（无需 RPC 服务器）。
set -euo pipefail
SCRIPT_DIR="$(cd "$(dirname "$0")" && pwd)"
APP_ROOT="$SCRIPT_DIR/.."
TARGET_DIR="$APP_ROOT/target"
TARGET_APK="$TARGET_DIR/公民.apk"
cd "$APP_ROOT"

CID_DEV_USB_PORT=8899

# 构造 dart-define 参数
DART_DEFINES=(--dart-define=CITIZENAPP_ONCHINA_ENV=dev_usb)
ANDROID_TARGET_PLATFORMS=(--target-platform android-arm,android-arm64)
echo "[启动模式] smoldot 轻节点"

# ── chainspec.json 是从链端 plain SSOT + 创世状态包派生的轻节点创世 ──
# 节点 SSOT = citizenchain/node/chainspecs/citizenchain.plain.json;App 资产只保留
# genesis.stateRootHash 轻形态。正式创世请先跑 citizenchain/scripts/bake-chainspec.sh
# 同步 plain SSOT、App 轻形态和 genesis-state;runtime 升级走链上 system.setCode。
# 详见 memory/07-ai/chainspec-frozen.md
bash "$SCRIPT_DIR/check-chainspec-frozen.sh"

# ── 启动前 adb 健康自检：只在 adb 真正卡死时才重置，健康时绝不触碰 ──
# adb server 是 fork-server 常驻守护进程，脱离终端独立运行；一旦被
# 挂起(^Z)的 adb 客户端把它的连接状态搞坏，后续每次 `adb devices` 都会永久
# 阻塞，且换终端、重开都无效(守护进程常驻)。
# 关键：绝不能无条件强杀 adb——`kill -9` 重启会让 USB 设备短暂重新枚举，正常
# 连接的设备会在窗口内被下方 `adb get-state` 误判为"未检测到"。所以这里只做
# 探测：用 8 秒超时跑一次 `adb devices`，仅当它卡住(超时/失败)才判定 server
# 已卡死并强制重置；健康时整段是只读探测，不动 adb、不动设备连接。
if command -v adb >/dev/null 2>&1; then
  if ! perl -e 'alarm 8; exec @ARGV' adb devices >/dev/null 2>&1; then
    echo "==> adb 无响应(疑似卡死)，强制重置 server..."
    pkill -9 -f 'adb.*fork-server' 2>/dev/null || true
    adb start-server >/dev/null 2>&1 || true
    sleep 2
  fi
fi
# 清掉上一轮残留的 flutter_tools 进程(无残留则空操作，不影响健康运行)。
pkill -9 -f flutter_tools.snapshot 2>/dev/null || true

echo "==> 清除 Rust 编译缓存..."
(cd "rust" && ~/.cargo/bin/cargo clean 2>/dev/null || true)
echo "==> 编译 Rust 原生库..."
# 检测目标平台：通过 flutter devices 判断。
# `flutter devices --machine` 内部会调 `adb devices`，万一仍被卡住
# (例如本地 adb 异常)，用 perl alarm 包 60s 超时强制结束(macOS 自带 perl，
# 无 GNU `timeout`)，避免无限阻塞；超时/失败统一回退到 android 判定。
DEVICE_LINE=$(perl -e 'alarm 60; exec @ARGV' flutter devices --machine 2>/dev/null | python3 -c "
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

# ── OnChina 本地开发路径：只允许 Android USB adb reverse ──
# 开发版 App 内部固定访问 http://127.0.0.1:8899；该地址必须由
# adb reverse 转发到本机 OnChina 后端，禁止改走局域网 IP 或其他自定义 URL。
if [[ "$DEVICE_LINE" != "android" ]]; then
  echo "错误：citizenapp 本地开发访问 OnChina 只支持 Android USB adb reverse。"
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
"$ADB_BIN" reverse "tcp:$CID_DEV_USB_PORT" "tcp:$CID_DEV_USB_PORT"
echo "==> Android USB: adb reverse tcp:$CID_DEV_USB_PORT, citizenapp CID 环境=dev_usb"

# ── 开发期 USB 桥接：自动检测本地诊断节点并打开 ADB reverse + 注入 dart-define ──
# 远端 prczss/nrcgch 偶发 SubstreamReset 时，本地节点 (--listen-addr ws/30334)
# 作为 citizenapp 第三个稳定 peer 兜底。出门后 localhost 不可达 smoldot 自动忽略。
DEV_NODE_RPC="${CITIZENAPP_DEV_LOCAL_RPC:-http://localhost:9945}"
DEV_NODE_PORT="${CITIZENAPP_DEV_LOCAL_WS_PORT:-30334}"
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
  DART_DEFINES+=(--dart-define=CITIZENAPP_DEV_LOCAL_PEER_ID="$DEV_NODE_PEER_ID")
  DART_DEFINES+=(--dart-define=CITIZENAPP_DEV_LOCAL_WS_PORT="$DEV_NODE_PORT")
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
  # 启动脚本固定把本地 APK 产物沉淀到项目根 target/，便于离线安装和回滚。
  echo "==> 生成 Android 产物..."
  flutter build apk --debug "${ANDROID_TARGET_PLATFORMS[@]}" "${DART_DEFINES[@]}"
  sync_android_artifact
fi

echo "==> 编译并启动 App..."
flutter run "${DART_DEFINES[@]}"
sync_android_artifact
