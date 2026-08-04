import 'dart:async';
import 'dart:io';

import 'package:flutter/foundation.dart';
import 'package:path_provider/path_provider.dart';

/// 全端唯一日志门面。release 构建（kReleaseMode 为编译期常量）下所有日志为空操作
/// 并被 tree-shake 剥离,不进设备日志、不泄任何字段。
///
/// debug/profile 下:走 debugPrint,并**追加写文件** `files/citizenapp_diag.log`。
/// 原因:本 app 独立启动(非 `flutter run`)时 debugPrint 不落 logcat,真机诊断
/// 只能读文件(`adb exec-out run-as <pkg> cat files/citizenapp_diag.log`)。
/// 新增日志一律走 AppLog,禁止再直接调用 debugPrint。
class AppLog {
  const AppLog._();

  static File? _diagFile;
  static bool _diagResolved = false;
  static Future<void> _tail = Future<void>.value();

  /// 调试日志。签名与 debugPrint 兼容（String? + wrapWidth），既有调用点可原样迁移。
  static void d(String? message, {int? wrapWidth}) {
    if (kReleaseMode) return;
    debugPrint(message, wrapWidth: wrapWidth);
    if (message != null) _appendDiag(message);
  }

  // 串行追加,避免并发写交错;首次解析目录失败(如单测无平台通道)后永久空操作。
  static void _appendDiag(String message) {
    // 单测无平台通道(path_provider 不可用),直接跳过文件写。
    if (Platform.environment.containsKey('FLUTTER_TEST')) return;
    _tail = _tail.then((_) async {
      try {
        if (!_diagResolved) {
          _diagResolved = true;
          final dir = await getApplicationSupportDirectory();
          _diagFile = File('${dir.path}/citizenapp_diag.log');
        }
        final f = _diagFile;
        if (f == null) return;
        await f.writeAsString(
          '${DateTime.now().toIso8601String()} $message\n',
          mode: FileMode.append,
        );
      } on Object {
        // 诊断日志失败绝不影响业务
      }
    });
  }
}
