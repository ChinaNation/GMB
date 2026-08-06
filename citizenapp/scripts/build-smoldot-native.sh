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
CITIZENAPP_DIR="$(dirname "$SCRIPT_DIR")"
RUST_DIR="$CITIZENAPP_DIR/rust"
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

  # CitizenApp Android 唯一支持 arm64-v8a；禁止重新生成任何 32 位或 x86 ABI。
  local arm64_dest="$CITIZENAPP_DIR/android/app/src/main/jniLibs/arm64-v8a"
  mkdir -p "$arm64_dest"
  cp target/aarch64-linux-android/release/libsmoldot.so "$arm64_dest/"
  echo "Android arm64-v8a: $arm64_dest/libsmoldot.so ($(wc -c < "$arm64_dest/libsmoldot.so" | tr -d ' ') bytes)"
}

build_ios() {
  echo ""
  echo "=== 编译 iOS (arm64, 静态库) ==="
  ensure_target aarch64-apple-ios

  cd "$RUST_DIR"
  cargo build --release --target aarch64-apple-ios

  # iOS 走静态库直接链进 Runner 主二进制(本地 pod,见 ios/smoldot/smoldot_ffi.podspec):
  # 裸 .dylib 需要嵌入 App 并单独代码签名,App Store 还要求动态库必须包成 .framework;
  # 静态库没有这些坑。Dart 侧经 DynamicLibrary.process() 取符号,与冷端同一套做法。
  local dest="$CITIZENAPP_DIR/ios/smoldot"
  mkdir -p "$dest"
  cp target/aarch64-apple-ios/release/libsmoldot.a "$dest/"

  # 从 .a 实抽 FFI 导出符号清单,供 podspec 逐个生成 -Wl,-u,<符号>。
  # 手写清单必然漂移:漏一个符号 = Release 被 -dead_strip 静默剔除(Debug 正常、
  # Release 找不到符号),所以清单永远从产物现抽、绝不手维护。
  # Mach-O 符号带下划线前缀;llvm-nm 查 Mach-O 用 -g(-D 是 ELF 专用)。
  local nm
  nm="$(xcrun --find llvm-nm)"
  # llvm-nm 对 .a 里个别无符号表的对象会报警且以非零退出,但符号输出本身完整;
  # 在 pipefail 下必须吞掉它的退出码,真正的完整性由下方三族计数把关。
  ("$nm" -g --defined-only "$dest/libsmoldot.a" 2>/dev/null || true) \
    | awk '$2 == "T" { print $3 }' \
    | grep -E '^_(smoldot_|citizen_sr25519_|citizen_chat_mls_)' \
    | sort -u > "$dest/exported_symbols.txt"

  local n_smoldot n_signer n_mls
  n_smoldot=$(grep -c '^_smoldot_' "$dest/exported_symbols.txt" || true)
  n_signer=$(grep -c '^_citizen_sr25519_' "$dest/exported_symbols.txt" || true)
  n_mls=$(grep -c '^_citizen_chat_mls_' "$dest/exported_symbols.txt" || true)
  echo "iOS arm64: $dest/libsmoldot.a ($(wc -c < "$dest/libsmoldot.a" | tr -d ' ') bytes)"
  echo "符号清单: smoldot_=$n_smoldot citizen_sr25519_=$n_signer citizen_chat_mls_=$n_mls"
  if [ "$n_smoldot" -eq 0 ] || [ "$n_signer" -eq 0 ] || [ "$n_mls" -eq 0 ]; then
    echo "错误: 符号清单整族缺失,检查 crate 导出(三族都必须非空)。"
    return 1
  fi
}

build_macos() {
  echo ""
  echo "=== 编译 macOS (arm64，桌面调试用) ==="
  cd "$RUST_DIR"
  # macOS 桌面调试库要给 Dart FFI / flutter test 直接 dlopen。
  # Rust release profile 的 strip=true 会让本机 dyld 报 LINKEDIT 对齐错误，
  # 因此 host 调试库单独禁用 strip；Android/iOS 打包库仍沿用 release profile。
  CARGO_PROFILE_RELEASE_STRIP=false cargo build --release

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
