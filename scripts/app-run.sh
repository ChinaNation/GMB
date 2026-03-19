#!/usr/bin/env bash
# 启动手机 App（不清空缓存）
set -euo pipefail
cd "$(dirname "$0")/../wuminapp"
flutter run --dart-define=WUMINAPP_RPC_URL=http://10.92.152.128:9944
