#!/usr/bin/env bash
# 清空缓存 + 重新编译 + 启动手机 App
set -euo pipefail
cd "$(dirname "$0")/../wuminapp"
echo "==> 清空构建缓存..."
flutter clean
echo "==> 获取依赖..."
flutter pub get
echo "==> 编译并启动 App..."
flutter run --dart-define=WUMINAPP_RPC_URL=http://10.92.152.128:9944
