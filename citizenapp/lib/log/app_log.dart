import 'package:flutter/foundation.dart';

/// 全端唯一日志门面。release 构建（kReleaseMode 为编译期常量）下所有日志为空操作
/// 并被 tree-shake 剥离，不进设备日志、不泄任何字段；debug/profile 下走 debugPrint。
/// 新增日志一律走 AppLog，禁止再直接调用 debugPrint。
class AppLog {
  const AppLog._();

  /// 调试日志。签名与 debugPrint 兼容（String? + wrapWidth），既有调用点可原样迁移。
  static void d(String? message, {int? wrapWidth}) {
    if (kReleaseMode) return;
    debugPrint(message, wrapWidth: wrapWidth);
  }
}
