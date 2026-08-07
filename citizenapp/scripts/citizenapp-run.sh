#!/usr/bin/env bash
# 清空缓存 + 重新编译 + 把**可直接使用**的公民安装到设备
# (iOS=release,Android=debug;不用 flutter run,理由见文件末尾安装段注释)
#
# 用法：citizenapp-run.sh <ios|android>
#
# 目标平台是必填参数，不做任何自动探测：探测总要在失败时选一个回落，
# 而回落的那一端会被当成用户想编的那一端——「以为编了 iOS、实际编的 Android」
# 就是这么来的。控制台的「编译iOS端 / 编译Android端」两个按钮各自传死这个参数。
#
# 本脚本不产出任何留存产物：编译产物只在 GitHub（CI / Release）。
# 固定使用 smoldot 轻节点连接区块链（无需 RPC 服务器）。
set -euo pipefail
SCRIPT_DIR="$(cd "$(dirname "$0")" && pwd)"
APP_ROOT="$SCRIPT_DIR/.."
PLATFORM="${1:?缺少目标平台，用法：$0 <ios|android>}"
[[ "$PLATFORM" == ios || "$PLATFORM" == android ]] \
  || { echo "目标平台只接受 ios 或 android：$PLATFORM" >&2; exit 1; }
cd "$APP_ROOT"

# 构造 dart-define 参数
DART_DEFINES=()
echo "[启动模式] smoldot 轻节点 · 目标平台 $PLATFORM"

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
#
# 只在 android 目标下做：adb 与 iOS 编译毫无关系，iOS 没必要白等这 8 秒探测。
if [[ "$PLATFORM" == android ]] && command -v adb >/dev/null 2>&1; then
  if ! perl -e 'alarm 8; exec @ARGV' adb devices >/dev/null 2>&1; then
    echo "==> adb 无响应(疑似卡死)，强制重置 server..."
    pkill -9 -f 'adb.*fork-server' 2>/dev/null || true
    adb start-server >/dev/null 2>&1 || true
    sleep 2
  fi
fi
# 这里曾有一句 `pkill -9 -f flutter_tools.snapshot`，用途是清掉上一轮残留的 flutter。
# 已删除：`-f` 匹配全命令行，而 `flutter_tools.snapshot` 是每一个 flutter 命令的实际执行体，
# 那一枪不区分产品、不区分平台、也不区分是不是本次运行的——公民钱包正在跑的编译、
# 乃至你自己在终端里手敲的 flutter，都会一起被 SIGKILL（现象是 `Killed: 9`）。
# 它要解决的残留问题已经由控制台承接：所有动作子进程都在独立进程组里启动，
# 「停止」与控制台退出都按进程组终止整棵进程树，不会再留下脱缰的 flutter。

echo "==> 清除 Rust 编译缓存..."
(cd "rust" && ~/.cargo/bin/cargo clean 2>/dev/null || true)
echo "==> 编译 Rust 原生库（$PLATFORM）..."
"$SCRIPT_DIR/build-smoldot-native.sh" "$PLATFORM"

echo "==> 清空构建缓存..."
flutter clean
echo "==> 获取依赖..."
flutter pub get

# 按目标平台挑一台设备，把 id 显式传给安装命令。
# 不传设备 id 时 flutter 自己挑：同时连着安卓机和 iPhone 就无从决定，而控制台日志面板
# 没有输入框，它的选择提示在那里根本回答不了。挑不到就报错退出，绝不改编另一端。
# `flutter devices --machine` 内部会调 `adb devices`，万一 adb 异常会永久阻塞，
# 故用 perl alarm 包 60s 超时（macOS 自带 perl，无 GNU `timeout`）。
echo "==> 选择 $PLATFORM 设备..."
DEVICE_ID="$(perl -e 'alarm 60; exec @ARGV' flutter devices --machine 2>/dev/null | python3 -c "
import sys, json
want = sys.argv[1]
try:
    for device in json.load(sys.stdin):
        if want in device.get('targetPlatform', ''):
            print(device['id']); break
except Exception:
    pass
" "$PLATFORM" || true)"
[[ -n "$DEVICE_ID" ]] || {
  echo "未检测到 $PLATFORM 设备：请连接设备（或启动模拟器）后重试。" >&2
  exit 1
}
echo "    设备: $DEVICE_ID"

# 「编译」= 把**能直接使用**的软件装进设备,与 CI / 正式 Release 是两条独立通路。
# 因此一律 build + install,不用 `flutter run`(它只把 App 挂在调试器上跑):
#
# - iOS 必须 release。iOS 14+ 禁止 Flutter debug 版脱离 flutter tooling / Xcode 启动;
#   debug 版装进手机后从桌面点图标必然起不来(系统提示 "Cannot create a FlutterEngine
#   instance in debug mode",随后 signal 11)——表现就是"一点就闪退"。
#   iOS 安装走 `flutter install`:它认 flutter 自己的设备 id(硬件 UDID),而 devicectl
#   用的是另一套内部标识,两者不通用,直接把 flutter 的 id 传给 devicectl 会找不到设备。
# - Android 用 debug:安卓 debug 版本来就能从桌面直接使用,且保留 AppLog 落盘诊断
#   (release 下 kReleaseMode 让它变成空操作),排障成本低得多。
#
# 两端构建模式不同是 iOS 系统能力的客观差异,按「iOS/Android 两端必须一致」铁律在此
# 显式登记:**两端交付物都是可直接使用的 App**,这一条完全一致。
# 要把 Android 也统一成 release,把下面 android 分支的 --debug 改成 --release、
# APK 路径改成 app-release.apk 即可(代价:丢失落盘诊断日志)。
echo "==> 编译并安装到设备..."
if [[ "$PLATFORM" == ios ]]; then
  flutter build ios --release ${DART_DEFINES[@]+"${DART_DEFINES[@]}"}
  flutter install --release -d "$DEVICE_ID"
else
  flutter build apk --debug ${DART_DEFINES[@]+"${DART_DEFINES[@]}"}
  adb -s "$DEVICE_ID" install -r build/app/outputs/flutter-apk/app-debug.apk
fi

echo ""
echo "==> 安装完成:请在设备桌面点开「公民」。"
