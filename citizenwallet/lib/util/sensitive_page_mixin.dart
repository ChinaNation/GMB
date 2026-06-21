import 'package:flutter/material.dart';

import 'screenshot_guard.dart';

/// 敏感页面 mixin：自动启用截屏保护，响应截屏/录屏事件隐藏内容。
///
/// 使用方式：
/// ```dart
/// class _MyPageState extends State<MyPage> with SensitivePageMixin {
///   @override
///   Widget build(BuildContext context) {
///     if (sensitiveContentHidden) {
///       return buildHiddenPlaceholder(); // mixin 提供的遮罩
///     }
///     return ... // 正常内容
///   }
/// }
/// ```
mixin SensitivePageMixin<T extends StatefulWidget> on State<T> {
  /// 敏感内容是否应被隐藏（截屏或录屏触发）。
  bool sensitiveContentHidden = false;

  @override
  void initState() {
    super.initState();
    ScreenshotGuard.enable();
    ScreenshotGuard.onSecurityEvent = _onSecurityEvent;
  }

  @override
  void dispose() {
    ScreenshotGuard.onSecurityEvent = null;
    ScreenshotGuard.disable();
    super.dispose();
  }

  void _onSecurityEvent(String event) {
    if (!mounted) return;
    if (event == 'screenshot_taken') {
      // 截屏已发生，隐藏内容并提醒用户
      setState(() => sensitiveContentHidden = true);
      ScaffoldMessenger.of(context).showSnackBar(
        const SnackBar(
          content: Text('检测到截屏，敏感信息已隐藏。请勿截屏保存密钥信息。'),
          duration: Duration(seconds: 3),
        ),
      );
    } else if (event == 'screen_recording_started') {
      // 录屏开始，立即隐藏
      setState(() => sensitiveContentHidden = true);
      ScaffoldMessenger.of(context).showSnackBar(
        const SnackBar(
          content: Text('检测到屏幕录制，敏感信息已隐藏'),
          duration: Duration(seconds: 3),
        ),
      );
    } else if (event == 'screen_recording_stopped') {
      // 录屏结束，可恢复显示（用户需重新点击查看）
      // 不自动恢复，保持隐藏状态
    }
  }

  /// 敏感内容被隐藏时的占位 Widget。
  Widget buildHiddenPlaceholder({String message = '敏感信息已隐藏'}) {
    return Scaffold(
      appBar: AppBar(
        title: const Text('安全提醒'),
        centerTitle: true,
      ),
      body: Center(
        child: Column(
          mainAxisAlignment: MainAxisAlignment.center,
          children: [
            Icon(Icons.shield, size: 64, color: Colors.orange.shade400),
            const SizedBox(height: 16),
            Text(
              message,
              style: const TextStyle(fontSize: 16, fontWeight: FontWeight.w600),
            ),
            const SizedBox(height: 8),
            const Text(
              '请返回上一页重新操作',
              style: TextStyle(color: Colors.grey),
            ),
            const SizedBox(height: 24),
            FilledButton(
              onPressed: () => Navigator.of(context).pop(),
              child: const Text('返回'),
            ),
          ],
        ),
      ),
    );
  }
}
