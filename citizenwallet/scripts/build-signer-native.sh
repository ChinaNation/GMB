#!/usr/bin/env bash
# 编译 CitizenWallet 冷钱包 sr25519 原生签名库，放到 Flutter 能自动打包的位置。
#
# 实现来自共享 crate citizenchain/crates/citizen-signer（与 CitizenApp 热端同一份
# 源码）；本库只是冷端的 FFI 外壳，产物几百 KB（冷钱包永久离线、不需要链）。
#
# 前置条件：安装 Rust (rustup)
#   curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh
#
# 用法：
#   ./scripts/build-signer-native.sh            # 编译所有平台
#   ./scripts/build-signer-native.sh android    # 仅 Android
#   ./scripts/build-signer-native.sh ios        # 仅 iOS
#   ./scripts/build-signer-native.sh macos      # 仅 macOS（flutter test 用）
set -euo pipefail

SCRIPT_DIR="$(cd "$(dirname "$0")" && pwd)"
WALLET_DIR="$(dirname "$SCRIPT_DIR")"
RUST_DIR="$WALLET_DIR/rust"
LIB_NAME="libcitizenwallet_signer"
TARGET="${1:-all}"

ensure_target() {
  local target="$1"
  if ! rustup target list --installed | grep -q "$target"; then
    echo "安装 Rust 目标: $target"
    rustup target add "$target"
  fi
}

# 导出的 4 个 C 符号必须齐全，否则 Dart 侧 lookupFunction 会在运行时才炸。
# 注意平台差异：ELF 用 -D，Mach-O 用 -g，用错标志会误判为 0。
verify_symbols() {
  local lib="$1"
  local nm_flag="$2"
  local nm_bin
  nm_bin="$(command -v llvm-nm || true)"
  if [ -z "$nm_bin" ]; then
    local sdk_home="${ANDROID_HOME:-$HOME/Library/Android/sdk}"
    nm_bin="$(ls "$sdk_home"/ndk/*/toolchains/llvm/prebuilt/*/bin/llvm-nm 2>/dev/null | tail -1 || true)"
  fi
  if [ -z "$nm_bin" ]; then
    echo "    (跳过符号检查：未找到 llvm-nm)"
    return 0
  fi
  local count
  count="$("$nm_bin" "$nm_flag" "$lib" 2>/dev/null | grep -c 'citizen_sr25519' || true)"
  if [ "$count" != "4" ]; then
    echo "错误: $lib 导出的 citizen_sr25519_* 符号为 $count 个（应为 4）"
    return 1
  fi
  echo "    符号检查通过：4 个 citizen_sr25519_*"
}

build_android() {
  echo ""
  echo "=== 编译 Android (arm64-v8a) ==="
  ensure_target aarch64-linux-android

  local ndk_home="${ANDROID_NDK_HOME:-}"
  if [ -z "$ndk_home" ]; then
    local sdk_home="${ANDROID_HOME:-$HOME/Library/Android/sdk}"
    ndk_home="$(ls -d "$sdk_home/ndk/"* 2>/dev/null | sort -V | tail -1 || true)"
  fi
  if [ -z "$ndk_home" ] || [ ! -d "$ndk_home" ]; then
    echo "错误: 未找到 Android NDK。请设置 ANDROID_NDK_HOME 或通过 Android Studio 安装 NDK。"
    return 1
  fi
  echo "使用 NDK: $ndk_home"

  local toolchain=""
  case "$(uname -s)" in
    Darwin)
      toolchain="$ndk_home/toolchains/llvm/prebuilt/darwin-x86_64"
      if [ ! -d "$toolchain" ]; then
        toolchain="$ndk_home/toolchains/llvm/prebuilt/darwin-aarch64"
      fi
      ;;
    Linux)
      toolchain="$ndk_home/toolchains/llvm/prebuilt/linux-x86_64"
      ;;
    *)
      echo "错误: 当前系统不支持自动定位 Android NDK toolchain: $(uname -s)"
      return 1
      ;;
  esac
  if [ ! -d "$toolchain" ]; then
    echo "错误: 未找到 Android NDK toolchain: $toolchain"
    return 1
  fi

  export CARGO_TARGET_AARCH64_LINUX_ANDROID_LINKER="$toolchain/bin/aarch64-linux-android24-clang"
  export CC_aarch64_linux_android="$toolchain/bin/aarch64-linux-android24-clang"
  export AR_aarch64_linux_android="$toolchain/bin/llvm-ar"

  cd "$RUST_DIR"
  cargo build --release --target aarch64-linux-android

  # CitizenWallet Android 唯一支持 arm64-v8a；禁止重新生成任何 32 位或 x86 ABI。
  local arm64_dest="$WALLET_DIR/android/app/src/main/jniLibs/arm64-v8a"
  mkdir -p "$arm64_dest"
  cp "target/aarch64-linux-android/release/$LIB_NAME.so" "$arm64_dest/"
  echo "Android arm64-v8a: $arm64_dest/$LIB_NAME.so ($(wc -c < "$arm64_dest/$LIB_NAME.so" | tr -d ' ') bytes)"
  verify_symbols "$arm64_dest/$LIB_NAME.so" -D
}

build_ios() {
  echo ""
  echo "=== 编译 iOS (arm64 真机) ==="
  ensure_target aarch64-apple-ios

  cd "$RUST_DIR"
  cargo build --release --target aarch64-apple-ios

  # iOS 用**静态库**而非 dylib：裸 .dylib 需嵌入 + 单独签名，且 App Store 要求
  # 动态库必须包在 .framework 里；静态库直接链进 App 二进制，无这些坑。
  # 符号经 podspec 的 -force_load 保留，Dart 侧用 DynamicLibrary.process() 取。
  local dest="$WALLET_DIR/ios/signer"
  mkdir -p "$dest"
  cp "target/aarch64-apple-ios/release/$LIB_NAME.a" "$dest/"
  echo "iOS arm64: $dest/$LIB_NAME.a ($(wc -c < "$dest/$LIB_NAME.a" | tr -d ' ') bytes)"
  verify_symbols "$dest/$LIB_NAME.a" ""
}

build_host() {
  echo ""
  echo "=== 编译宿主平台动态库 (flutter test 用) ==="
  cd "$RUST_DIR"
  # host 调试库给 Dart FFI / flutter test 直接 dlopen；release profile 已设
  # strip=false，本机 dyld 不会报 LINKEDIT 对齐错误。
  cargo build --release

  # 宿主扩展名：macOS 产 .dylib，Linux 产 .so；Dart 侧 native_sr25519.dart 按同一规则取。
  local host_ext
  case "$(uname -s)" in
    Darwin) host_ext=dylib ;;
    *)      host_ext=so ;;
  esac
  local host_lib="$RUST_DIR/target/release/$LIB_NAME.$host_ext"
  echo "宿主库: $host_lib ($(wc -c < "$host_lib" | tr -d ' ') bytes)"
  verify_symbols "$host_lib" -g
}

case "$TARGET" in
  android) build_android ;;
  ios)     build_ios ;;
  host|macos|linux) build_host ;;
  all)
    build_android
    build_ios
    build_host
    ;;
  *)
    echo "用法: $0 [android|ios|macos|all]"
    exit 1
    ;;
esac

echo ""
echo "=== 编译完成 ==="
echo "flutter build / flutter run 会自动把 native library 打包进 App。"
