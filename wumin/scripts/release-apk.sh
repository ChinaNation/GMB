#!/usr/bin/env bash
# 构建正式签名 APK，并输出统一文件名“公民钱包.apk”。
set -euo pipefail

cd "$(dirname "$0")/.."

output_dir="build/app/outputs/flutter-apk"
default_apk="$output_dir/app-release.apk"
named_apk="$output_dir/公民钱包.apk"

echo "==> 构建正式签名 APK..."
flutter build apk --release

if [[ ! -f "$default_apk" ]]; then
  echo "未找到构建产物: $default_apk" >&2
  exit 1
fi

echo "==> 生成命名产物: $named_apk"
cp "$default_apk" "$named_apk"

echo "==> 完成"
echo "默认产物: $default_apk"
echo "命名产物: $named_apk"
