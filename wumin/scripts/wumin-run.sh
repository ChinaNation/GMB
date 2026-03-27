#!/usr/bin/env bash
# 清空缓存 + 生成代码 + 重新编译 + 启动 Wumin 签名设备
set -euo pipefail
SCRIPT_DIR="$(cd "$(dirname "$0")" && pwd)"
WUMIN_DIR="$SCRIPT_DIR/.."
REPO_ROOT="$SCRIPT_DIR/../.."
cd "$WUMIN_DIR"

echo "==> 同步 runtime spec_version..."
SPEC=$(grep 'spec_version:' "$REPO_ROOT/citizenchain/runtime/src/lib.rs" | grep -o '[0-9]*')
sed -i '' "s/supportedSpecVersions = {[^}]*}/supportedSpecVersions = {$SPEC}/" lib/signer/pallet_registry.dart
echo "    冷钱包 spec_version 已同步为 {$SPEC}"

echo "==> 清空构建缓存..."
flutter clean
echo "==> 获取依赖..."
flutter pub get
echo "==> 生成 Isar 代码..."
flutter pub run build_runner build --delete-conflicting-outputs
echo "==> 编译并启动 App..."
flutter run
