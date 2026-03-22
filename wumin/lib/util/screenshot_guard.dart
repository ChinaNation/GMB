import 'package:flutter/services.dart';

/// 截屏保护工具。
///
/// - Android: 通过 FLAG_SECURE 阻止截屏和屏幕录制。
/// - iOS: 进入后台时添加模糊遮罩，检测截屏事件。
class ScreenshotGuard {
  const ScreenshotGuard._();

  static const MethodChannel _channel =
      MethodChannel('com.wuminapp.wumin/security');

  /// 启用截屏保护。
  static Future<void> enable() async {
    try {
      await _channel.invokeMethod('enableScreenshotProtection');
    } on PlatformException {
      // 平台不支持，忽略。
    }
  }

  /// 禁用截屏保护。
  static Future<void> disable() async {
    try {
      await _channel.invokeMethod('disableScreenshotProtection');
    } on PlatformException {
      // 平台不支持，忽略。
    }
  }

  /// 检测设备是否已 root（Android）或越狱（iOS）。
  static Future<bool> isDeviceRooted() async {
    try {
      final result = await _channel.invokeMethod<bool>('isDeviceRooted');
      return result ?? false;
    } on PlatformException {
      return false;
    }
  }
}
