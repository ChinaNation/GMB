import 'dart:async';

import 'package:flutter/services.dart';

/// 截屏保护工具。
///
/// - Android: 通过 FLAG_SECURE 阻止截屏和屏幕录制。
/// - iOS: 进入后台时添加模糊遮罩；前台截屏时通过事件通知 Flutter
///   隐藏敏感内容；检测到录屏时主动通知隐藏。
class ScreenshotGuard {
  const ScreenshotGuard._();

  static const MethodChannel _channel =
      MethodChannel('com.wuminapp.wumin/security');

  static const EventChannel _eventChannel =
      EventChannel('com.wuminapp.wumin/security_events');

  static StreamSubscription<dynamic>? _eventSubscription;

  /// 截屏/录屏事件回调。
  ///
  /// 事件类型：
  /// - `screenshot_taken`：用户在前台截屏（iOS，截屏已完成）
  /// - `screen_recording_started`：屏幕录制开始（iOS）
  /// - `screen_recording_stopped`：屏幕录制结束（iOS）
  static void Function(String event)? onSecurityEvent;

  /// 启用截屏保护。
  static Future<void> enable() async {
    try {
      await _channel.invokeMethod('enableScreenshotProtection');
    } on PlatformException {
      // 平台不支持，忽略。
    }
    _startListening();
  }

  /// 禁用截屏保护。
  static Future<void> disable() async {
    try {
      await _channel.invokeMethod('disableScreenshotProtection');
    } on PlatformException {
      // 平台不支持，忽略。
    }
    _stopListening();
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

  static void _startListening() {
    if (_eventSubscription != null) return;
    _eventSubscription = _eventChannel.receiveBroadcastStream().listen(
      (event) {
        if (event is String) {
          onSecurityEvent?.call(event);
        }
      },
      onError: (_) {},
    );
  }

  static void _stopListening() {
    _eventSubscription?.cancel();
    _eventSubscription = null;
  }
}
