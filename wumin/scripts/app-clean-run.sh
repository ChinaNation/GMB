#!/usr/bin/env bash
# 清空缓存 + 生成代码 + 重新编译 + 启动 Wumin 签名设备
set -euo pipefail
cd "$(dirname "$0")/.."

echo "==> 清空构建缓存..."
flutter clean
echo "==> 获取依赖..."
flutter pub get
echo "==> 生成 Isar 代码..."
flutter pub run build_runner build --delete-conflicting-outputs
echo "==> 编译并启动 App..."
flutter run
