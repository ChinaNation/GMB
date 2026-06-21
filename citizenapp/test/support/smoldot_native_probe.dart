// libsmoldot native 库探测(IM MLS 与链 RPC 共用同一 .so/.dylib)。
//
// 背景:`flutter test` 跑在 CI 宿主 VM 上,而 `scripts/build-smoldot-native.sh`
// 只给 Android 交叉编译,产不出宿主版 libsmoldot;dlopen 失败 → 依赖 native 的
// 测试在纯 Dart CI 里必挂(IM MLS native 测试、app 启动初始化链 RPC)。
//
// 本探测器让这些测试在「库不可用」时 **skip(带原因)**:不是删测试 —— 真机 /
// APK 集成构建里 .so 随包,库可用时它们照常全跑;CI 单测无库则跳过,由 APK
// 打包 job(会编 native)+ 集成测试覆盖。一轮只 dlopen 探一次,结果缓存。

import 'package:citizenapp/im/crypto/im_mls_native.dart';

bool _probed = false;
String? _reason;

/// libsmoldot native 库的 skip 原因:可加载→`null`(测试照跑);不可加载→文案(skip)。
///
/// 直接传给 `test(..., skip: smoldotNativeSkipReason())` / `testWidgets(..., skip: ...)`。
String? smoldotNativeSkipReason() {
  if (_probed) return _reason;
  _probed = true;
  try {
    // NativeImMlsCrypto() 构造即 dlopen libsmoldot(与链 RPC 同一库);成功=库可用。
    NativeImMlsCrypto();
    _reason = null;
  } on Object catch (_) {
    _reason = 'libsmoldot native 库不可用(纯 Dart CI 无宿主 .so);'
        '真机 / APK 集成构建覆盖';
  }
  return _reason;
}
