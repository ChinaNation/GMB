// libsmoldot native 库探测（Chat MLS 与链 RPC 共用同一 .so/.dylib）。
//
// 背景:`flutter test` 跑在 CI 宿主 VM 上,CI 不编 native,宿主无 libsmoldot;
// dlopen 失败 → 依赖 native 的测试在纯 Dart CI 里必挂（Chat MLS native 测试、
// App 启动初始化链 RPC）。
//
// 本探测器让这些测试在「库不可用」时 **skip(带原因)**:不是删测试 —— 真机 /
// APK 集成构建里 .so 随包,库可用时它们照常全跑;CI 单测无库则跳过,由 APK
// 打包 job(会编 native)+ 集成测试覆盖。一轮只 dlopen 探一次,结果缓存。
//
// **本地必须真跑一遍**:skip 只是「这轮没验」,不是「验过了」。改 FFI 两侧任一
// (`rust/src/chat_mls.rs` / `lib/chat/crypto/mls_native.dart`)后,必须先产宿主库
// 再跑,否则跨语言字段名漂移会被 skip 静默掩盖 —— 2026-08-04 定位的
// `device_public_key_hex` 读侧断链(设备公钥恒空,Chat 首启必抛)就是这么潜伏的:
//
//   ./scripts/build-smoldot-native.sh macos   # 产 rust/target/release/libsmoldot.dylib
//   flutter test test/chat/mls_native_test.dart test/chat/mls_native_session_test.dart
//
// **宿主库会被跑 App 反复清掉,不是灵异现象**:`scripts/citizenapp-run.sh` 编译前先
// `cargo clean`(整个 rust/target 连宿主 dylib 一起没),随后只编 `$PLATFORM` 单平台。
// 所以每跑一次 App(安卓/iOS),这些测试就自动回到 skip —— 重跑上面那条 macos 命令即可。

import 'package:citizenapp/chat/crypto/mls_native.dart';

bool _probed = false;
String? _reason;

/// libsmoldot native 库的 skip 原因:可加载→`null`(测试照跑);不可加载→文案(skip)。
///
/// 直接传给 `test(..., skip: smoldotNativeSkipReason())` / `testWidgets(..., skip: ...)`。
String? smoldotNativeSkipReason() {
  if (_probed) return _reason;
  _probed = true;
  try {
    // NativeMlsCrypto() 构造即 dlopen libsmoldot(与链 RPC 同一库);成功=库可用。
    NativeMlsCrypto();
    _reason = null;
  } on Object catch (_) {
    _reason = 'libsmoldot native 库不可用(纯 Dart CI 无宿主 .so);'
        '真机 / APK 集成构建覆盖';
  }
  return _reason;
}
