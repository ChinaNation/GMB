#!/usr/bin/env bash
# 清空缓存 + 生成代码 + 重新编译 + 把**可直接使用**的 CitizenWallet 安装到签名设备
# (iOS=release,Android=debug;不用 flutter run,理由见文件末尾安装段注释)
#
# 用法：citizenwallet-run.sh <ios|android>
#
# 目标平台是必填参数，不做任何自动探测：探测总要在失败时选一个回落，
# 而回落的那一端会被当成用户想编的那一端。控制台的「编译iOS端 / 编译Android端」
# 两个按钮各自传死这个参数。与 citizenapp-run.sh 同口径。
#
# 本脚本不产出任何留存产物：编译产物只在 GitHub（CI / Release）。
set -euo pipefail
SCRIPT_DIR="$(cd "$(dirname "$0")" && pwd)"
CITIZENWALLET_DIR="$SCRIPT_DIR/.."
REPO_ROOT="$SCRIPT_DIR/../.."
PLATFORM="${1:?缺少目标平台，用法：$0 <ios|android>}"
[[ "$PLATFORM" == ios || "$PLATFORM" == android ]] \
  || { echo "目标平台只接受 ios 或 android：$PLATFORM" >&2; exit 1; }
cd "$CITIZENWALLET_DIR"

# 索引同步的唯一实现；CI 的两个 job 调的是同一个脚本，三处不会漂移。
"$SCRIPT_DIR/sync-pallet-registry.sh" "$REPO_ROOT"

echo "==> 清空构建缓存..."
flutter clean
echo "==> 获取依赖..."
flutter pub get
echo "==> 生成 Isar 代码..."
flutter pub run build_runner build --delete-conflicting-outputs

# sr25519 原生签名库(schnorrkel)。签名、派生、验签全走它，缺库会在运行时才炸，
# 所以必须先于 flutter build 产出；实现来自 citizenchain/crates/citizen-signer，
# 与 CitizenApp 热端同一份源码。
echo "==> 编译原生签名库（$PLATFORM）..."
# 必须用绝对路径 SCRIPT_DIR:上方已 cd 进 CITIZENWALLET_DIR,而控制台以相对路径
# 调本脚本时 $0 是相对串,$(dirname "$0") 会拼在新 cwd 上多套一层目录。
"$SCRIPT_DIR/build-signer-native.sh" "$PLATFORM"

# 按目标平台挑一台设备，把 id 显式传给安装命令。与 citizenapp-run.sh 同口径：
# 不传设备 id 时 flutter 自己挑，同时连着安卓机和 iPhone 就无从决定，而控制台日志面板
# 没有输入框，它的选择提示回答不了。挑不到就报错退出，绝不改编另一端。
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
# 因此一律 build + install,不用 `flutter run`(它只把 App 挂在调试器上跑)。
# 口径与 citizenapp-run.sh 完全一致,详细理由见该脚本同位置注释:
# - iOS 必须 release:iOS 14+ 禁止 Flutter debug 版脱离 flutter tooling / Xcode 启动,
#   装了 debug 版从桌面点图标必然起不来(表现为"一点就闪退")。安装走
#   `flutter install`(认 flutter 设备 id;devicectl 用的是另一套标识,不通用)。
# - Android 用 debug:安卓 debug 版从桌面就能直接使用,冷钱包排障也依赖它。
# 两端交付物都是**可直接使用**的 App —— 这一条完全一致。
echo "==> 编译并安装到设备..."
if [[ "$PLATFORM" == ios ]]; then
  flutter build ios --release
  flutter install --release -d "$DEVICE_ID"
else
  flutter build apk --debug
  adb -s "$DEVICE_ID" install -r build/app/outputs/flutter-apk/app-debug.apk
fi

echo ""
echo "==> 安装完成:请在设备桌面点开「公民钱包」。"
