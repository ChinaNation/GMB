import 'package:flutter/services.dart';
import 'package:flutter_test/flutter_test.dart';
import 'package:wuminapp_mobile/main.dart';

void main() {
  TestWidgetsFlutterBinding.ensureInitialized();

  const secureStorageChannel =
      MethodChannel('plugins.it_nomads.com/flutter_secure_storage');
  const localAuthChannel = MethodChannel('plugins.flutter.io/local_auth');

  setUp(() {
    // Mock secure storage — return null for all reads (no PIN set, no device lock).
    TestDefaultBinaryMessengerBinding.instance.defaultBinaryMessenger
        .setMockMethodCallHandler(secureStorageChannel, (call) async {
      switch (call.method) {
        case 'read':
          return null;
        case 'write':
        case 'delete':
        case 'deleteAll':
          return null;
        case 'containsKey':
          return false;
        case 'readAll':
          return <String, String>{};
        default:
          return null;
      }
    });

    // Mock local_auth — device not supported (skips device lock gate).
    TestDefaultBinaryMessengerBinding.instance.defaultBinaryMessenger
        .setMockMethodCallHandler(localAuthChannel, (call) async {
      switch (call.method) {
        case 'isDeviceSupported':
        case 'deviceSupportsBiometrics':
          return false;
        case 'getAvailableBiometrics':
          return const <String>[];
        case 'authenticate':
          return true;
        default:
          return null;
      }
    });
  });

  tearDown(() {
    TestDefaultBinaryMessengerBinding.instance.defaultBinaryMessenger
        .setMockMethodCallHandler(secureStorageChannel, null);
    TestDefaultBinaryMessengerBinding.instance.defaultBinaryMessenger
        .setMockMethodCallHandler(localAuthChannel, null);
  });

  testWidgets('app bootstraps', (tester) async {
    await tester.pumpWidget(const WuminApp());
    // 等待异步锁检查完成并渲染主界面。
    await tester.pumpAndSettle();

    // 底部导航栏应包含 '交易' 标签。
    expect(find.text('交易'), findsWidgets);
  });
}
