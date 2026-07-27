// ScreenshotGuard 引用计数单测:修 HIGH「全局单例子页 dispose 误关父页保护」。
// 多页 enable 时平台开关只在计数 0↔1 边界触发;子页 disable 不关闭父页仍需的保护。
import 'package:flutter/services.dart';
import 'package:flutter_test/flutter_test.dart';
import 'package:citizenwallet/util/screenshot_guard.dart';

void main() {
  final binding = TestWidgetsFlutterBinding.ensureInitialized();
  const channel = MethodChannel('org.citizenwallet/security');
  const events = MethodChannel('org.citizenwallet/security_events');
  final calls = <String>[];

  setUp(() {
    calls.clear();
    binding.defaultBinaryMessenger.setMockMethodCallHandler(channel, (call) async {
      calls.add(call.method);
      return null;
    });
    binding.defaultBinaryMessenger
        .setMockMethodCallHandler(events, (call) async => null);
  });

  tearDown(() {
    binding.defaultBinaryMessenger.setMockMethodCallHandler(channel, null);
    binding.defaultBinaryMessenger.setMockMethodCallHandler(events, null);
  });

  test('两页 enable/disable:平台保护仅在计数 0↔1 切换,子页退出不误关', () async {
    void cbA(String e) {}
    void cbB(String e) {}

    await ScreenshotGuard.enable(cbA); // 父页(如钱包详情揭示助记词)
    await ScreenshotGuard.enable(cbB); // 子页(如账户详情揭示私钥)
    expect(calls.where((m) => m == 'enableScreenshotProtection').length, 1,
        reason: '平台保护只应在首次开启一次');
    expect(calls.contains('disableScreenshotProtection'), isFalse);

    await ScreenshotGuard.disable(cbB); // 子页退出:计数 2→1,不应关闭
    expect(calls.contains('disableScreenshotProtection'), isFalse,
        reason: '父页仍在用,子页 disable 不得关闭全局保护');

    await ScreenshotGuard.disable(cbA); // 父页退出:计数 1→0,真正关闭
    expect(calls.where((m) => m == 'disableScreenshotProtection').length, 1);
  });
}
