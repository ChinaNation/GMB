#!/usr/bin/env bash
# 编译 smoldot native library 并放置到 Flutter 能自动打包的位置。
#
# 编译完成后 flutter build / flutter run 会自动将 .so / .dylib 打包进 App，
# 不需要额外操作。
#
# 前置条件：安装 Rust (rustup)
#   curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh
#
# 用法：
#   ./scripts/build-smoldot-native.sh           # 编译所有平台
#   ./scripts/build-smoldot-native.sh android    # 仅 Android
#   ./scripts/build-smoldot-native.sh ios        # 仅 iOS
set -euo pipefail

SCRIPT_DIR="$(cd "$(dirname "$0")" && pwd)"
WUMINAPP_DIR="$(dirname "$SCRIPT_DIR")"
RUST_DIR="$WUMINAPP_DIR/rust"
TARGET="${1:-all}"

# 确保 Rust 交叉编译目标已安装
ensure_target() {
  local target="$1"
  if ! rustup target list --installed | grep -q "$target"; then
    echo "安装 Rust 目标: $target"
    rustup target add "$target"
  fi
}

build_android() {
  echo ""
  echo "=== 编译 Android (arm64-v8a) ==="
  ensure_target aarch64-linux-android

  # 自动检测 NDK
  local ndk_home="${ANDROID_NDK_HOME:-}"
  if [ -z "$ndk_home" ]; then
    # 从 Android SDK 中查找
    local sdk_home="${ANDROID_HOME:-$HOME/Library/Android/sdk}"
    ndk_home="$(ls -d "$sdk_home/ndk/"* 2>/dev/null | sort -V | tail -1 || true)"
  fi
  if [ -z "$ndk_home" ] || [ ! -d "$ndk_home" ]; then
    echo "错误: 未找到 Android NDK。请设置 ANDROID_NDK_HOME 或通过 Android Studio 安装 NDK。"
    return 1
  fi
  echo "使用 NDK: $ndk_home"

  local toolchain="$ndk_home/toolchains/llvm/prebuilt/darwin-x86_64"
  if [ ! -d "$toolchain" ]; then
    toolchain="$ndk_home/toolchains/llvm/prebuilt/darwin-aarch64"
  fi

  export CARGO_TARGET_AARCH64_LINUX_ANDROID_LINKER="$toolchain/bin/aarch64-linux-android24-clang"
  export CC_aarch64_linux_android="$toolchain/bin/aarch64-linux-android24-clang"
  export AR_aarch64_linux_android="$toolchain/bin/llvm-ar"

  cd "$RUST_DIR"
  cargo build --release --target aarch64-linux-android

  # 放到 Flutter 自动打包的位置
  local dest="$WUMINAPP_DIR/android/app/src/main/jniLibs/arm64-v8a"
  mkdir -p "$dest"
  cp target/aarch64-linux-android/release/libsmoldot.so "$dest/"
  echo "Android arm64: $dest/libsmoldot.so ($(wc -c < "$dest/libsmoldot.so" | tr -d ' ') bytes)"
}

build_ios() {
  echo ""
  echo "=== 编译 iOS (arm64) ==="
  ensure_target aarch64-apple-ios

  cd "$RUST_DIR"
  cargo build --release --target aarch64-apple-ios

  local dest="$WUMINAPP_DIR/ios/Frameworks"
  mkdir -p "$dest"
  cp target/aarch64-apple-ios/release/libsmoldot.dylib "$dest/"
  echo "iOS arm64: $dest/libsmoldot.dylib ($(wc -c < "$dest/libsmoldot.dylib" | tr -d ' ') bytes)"
}

build_macos() {
  echo ""
  echo "=== 编译 macOS (arm64，桌面调试用) ==="
  cd "$RUST_DIR"
  cargo build --release

  echo "macOS arm64: $RUST_DIR/target/release/libsmoldot.dylib ($(wc -c < "$RUST_DIR/target/release/libsmoldot.dylib" | tr -d ' ') bytes)"
}

case "$TARGET" in
  android)
    build_android
    ;;
  ios)
    build_ios
    ;;
  macos)
    build_macos
    ;;
  all)
    build_android
    build_ios
    build_macos
    ;;
  *)
    echo "用法: $0 [android|ios|macos|all]"
    exit 1
    ;;
esac

echo ""
echo "=== 编译完成 ==="
echo "flutter build / flutter run 会自动将 native library 打包进 App。"
