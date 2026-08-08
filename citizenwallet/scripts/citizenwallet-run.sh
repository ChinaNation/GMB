#!/usr/bin/env bash
# 清空构建缓存 + 生成代码 + 重新编译 + 把**可直接使用**的 CitizenWallet 覆盖升级到签名设备
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

# iOS 必须走 CoreDevice 的原位安装，禁止 `flutter install`：Flutter 3.41 的
# install 命令默认先卸载旧 App，会连带删除 Application Support 中的钱包 Isar 数据库。
# `devicectl` 直接接受 Flutter 返回的硬件 UDID。iOS 更新时允许迁移数据容器并改变
# 绝对路径，因此不能比较 UUID；改为在覆盖前后复读钱包 Isar 文件并核对大小。
install_ios_update() {
  local device_id="$1" expected_bundle_id="$2" app_bundle="$3" wallet_database="$4"
  local actual_bundle_id team_id before_data_container after_data_container
  local before_database_size after_database_size

  [[ -d "$app_bundle" ]] || { echo "iOS App 产物不存在：$app_bundle" >&2; return 1; }
  actual_bundle_id="$(/usr/libexec/PlistBuddy -c 'Print :CFBundleIdentifier' "$app_bundle/Info.plist" 2>/dev/null || true)"
  [[ "$actual_bundle_id" == "$expected_bundle_id" ]] || {
    echo "iOS Bundle ID 不匹配：期望 ${expected_bundle_id}，实际 ${actual_bundle_id:-空}" >&2
    return 1
  }
  team_id="$(codesign -d --verbose=4 "$app_bundle" 2>&1 | sed -n 's/^TeamIdentifier=//p' | head -n 1)"
  [[ -n "$team_id" && "$team_id" != Not\ Set ]] || {
    echo "iOS App 缺少有效签名团队，拒绝覆盖设备中的正式数据" >&2
    return 1
  }

  ios_data_container() {
    xcrun devicectl device info apps --quiet \
      --device "$device_id" \
      --bundle-id "$expected_bundle_id" \
      --include-container-paths \
      --json-output - |
      python3 -c '
import json, sys
apps = json.load(sys.stdin).get("result", {}).get("apps", [])
if len(apps) > 1:
    raise SystemExit("同一 Bundle ID 返回多个 App，拒绝继续")
if apps:
    path = apps[0].get("dataContainerPath", "")
    if not path:
        raise SystemExit("无法读取已安装 App 的数据容器，拒绝覆盖")
    print(path)
'
  }

  ios_wallet_database_size() {
    xcrun devicectl device info files --quiet \
      --device "$device_id" \
      --domain-type appDataContainer \
      --domain-identifier "$expected_bundle_id" \
      --subdirectory 'Library/Application Support' \
      --recurse \
      --json-output - |
      python3 -c '
import json, sys
database = sys.argv[1]
files = json.load(sys.stdin).get("result", {}).get("files", [])
matches = [item for item in files if item.get("name") == database]
if len(matches) > 1:
    raise SystemExit("钱包数据库返回多个同名文件，拒绝继续")
if matches:
    size = matches[0].get("metadata", {}).get("size", 0)
    if not isinstance(size, int) or size <= 0:
        raise SystemExit("钱包数据库大小无效，拒绝继续")
    print(size)
' "$wallet_database"
  }

  before_data_container="$(ios_data_container)"
  if [[ -n "$before_data_container" ]]; then
    before_database_size="$(ios_wallet_database_size)"
  else
    before_database_size=""
  fi
  xcrun devicectl device install app --quiet --timeout 180 \
    --device "$device_id" "$app_bundle"
  after_data_container="$(ios_data_container)"
  [[ -n "$after_data_container" ]] || {
    echo "iOS 覆盖安装后未找到 $expected_bundle_id" >&2
    return 1
  }
  if [[ -n "$before_database_size" ]]; then
    after_database_size="$(ios_wallet_database_size)"
    [[ "$after_database_size" == "$before_database_size" ]] || {
      echo "iOS 覆盖安装后钱包数据库不存在或大小改变，拒绝把本次安装判为成功" >&2
      return 1
    }
  fi
  echo "    iOS 原位覆盖完成：Bundle ID=${expected_bundle_id}，Team ID=${team_id}，钱包数据库已保留"
}

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
echo "==> 编译原生签名库（${PLATFORM}）..."
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

# 「编译」= 把**能直接使用**的最新软件覆盖升级到设备,与 CI / 正式 Release 是两条独立通路。
# 因此一律 build + install,不用 `flutter run`(它只把 App 挂在调试器上跑)。
# 口径与 citizenapp-run.sh 完全一致,详细理由见该脚本同位置注释:
# - iOS 必须 release:iOS 14+ 禁止 Flutter debug 版脱离 flutter tooling / Xcode 启动,
#   装了 debug 版从桌面点图标必然起不来(表现为"一点就闪退")。安装直接走
#   `devicectl device install app`，接受 flutter 返回的硬件 UDID 且不先卸载旧 App；
#   `flutter install` 默认先卸载，会删除钱包数据，永久禁用。
# - Android 用 debug:安卓 debug 版从桌面就能直接使用,冷钱包排障也依赖它。
# 两端交付物都是**可直接使用**的 App —— 这一条完全一致。
echo "==> 编译并安装到设备..."
if [[ "$PLATFORM" == ios ]]; then
  flutter build ios --release
  install_ios_update "$DEVICE_ID" org.citizenwallet build/ios/iphoneos/Runner.app citizenwallet.isar
else
  flutter build apk --debug
  # `-r` 是原位替换，保留应用数据；失败时直接退出，禁止增加 uninstall 回退。
  adb -s "$DEVICE_ID" install -r build/app/outputs/flutter-apk/app-debug.apk
fi

echo ""
echo "==> 安装完成:请在设备桌面点开「公民钱包」。"
