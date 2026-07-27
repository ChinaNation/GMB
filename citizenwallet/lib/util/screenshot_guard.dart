import 'dart:async';

import 'package:flutter/services.dart';

/// 截屏保护工具（进程级全局资源，引用计数管理）。
///
/// - Android: 通过 FLAG_SECURE 阻止截屏和屏幕录制。
/// - iOS: 进入后台时添加模糊遮罩；前台截屏时通过事件通知 Flutter
///   隐藏敏感内容；检测到录屏时主动通知隐藏。
///
/// 多个敏感页可能同时需要保护（如钱包详情揭示助记词后进入账户详情揭示私钥）。
/// 因平台侧的 FLAG_SECURE 是**全局单一开关**,这里用**引用计数**管理:每个页面
/// `enable(cb)` 计数 +1、`disable(cb)` 计数 -1,只有计数归零才真正关闭平台保护;
/// 回调用**集合**保存,截屏事件广播给所有在用页面——避免子页 dispose 单方面关掉
/// 父页仍在用的保护。
class ScreenshotGuard {
  const ScreenshotGuard._();

  static const MethodChannel _channel =
      MethodChannel('org.citizenwallet/security');

  static const EventChannel _eventChannel =
      EventChannel('org.citizenwallet/security_events');

  static StreamSubscription<dynamic>? _eventSubscription;

  /// 当前持有保护的页面数(引用计数)。
  static int _refCount = 0;

  /// 截屏/录屏事件监听器集合(多页共存时每个在用页各注册一份)。
  ///
  /// 事件类型：
  /// - `screenshot_taken`：用户在前台截屏（iOS，截屏已完成）
  /// - `screen_recording_started`：屏幕录制开始（iOS）
  /// - `screen_recording_stopped`：屏幕录制结束（iOS）
  static final Set<void Function(String event)> _listeners = {};

  /// 启用截屏保护;可选传入本页的事件回调。幂等叠加(引用计数 +1)。
  static Future<void> enable([void Function(String event)? onEvent]) async {
    if (onEvent != null) _listeners.add(onEvent);
    _refCount++;
    if (_refCount == 1) {
      try {
        await _channel.invokeMethod('enableScreenshotProtection');
      } on PlatformException {
        // 平台不支持，忽略。
      }
      _startListening();
    }
  }

  /// 关闭截屏保护;传入的回调从集合移除。引用计数 -1,归零才真正关闭平台保护。
  static Future<void> disable([void Function(String event)? onEvent]) async {
    if (onEvent != null) _listeners.remove(onEvent);
    if (_refCount > 0) _refCount--;
    if (_refCount == 0) {
      try {
        await _channel.invokeMethod('disableScreenshotProtection');
      } on PlatformException {
        // 平台不支持，忽略。
      }
      _stopListening();
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

  static void _startListening() {
    if (_eventSubscription != null) return;
    _eventSubscription = _eventChannel.receiveBroadcastStream().listen(
      (event) {
        if (event is String) {
          // 广播给所有在用页面(拷贝一份,避免回调内修改集合)。
          for (final listener in List.of(_listeners)) {
            listener(event);
          }
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
