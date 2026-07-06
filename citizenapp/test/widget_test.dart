import 'package:flutter/services.dart';
import 'package:flutter_test/flutter_test.dart';
import 'package:shared_preferences/shared_preferences.dart';
import 'package:citizenapp/main.dart';
import 'package:citizenapp/security/app_permission_bootstrap.dart';

import 'support/smoldot_native_probe.dart';

void main() {
  TestWidgetsFlutterBinding.ensureInitialized();

  const secureStorageChannel =
      MethodChannel('plugins.it_nomads.com/flutter_secure_storage');
  const localAuthChannel = MethodChannel('plugins.flutter.io/local_auth');

  setUp(() {
    SharedPreferences.setMockInitialValues({
      AppPermissionBootstrap.guideSeenKey: true,
    });

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

  testWidgets('first run shows permission guide', (tester) async {
    SharedPreferences.setMockInitialValues({});

    await tester.pumpWidget(const CitizenApp());
    await tester.pumpAndSettle();

    expect(find.text('权限设置'), findsOneWidget);
    expect(find.text('开启通知并继续'), findsOneWidget);
    expect(find.text('稍后再说'), findsOneWidget);
  });

  testWidgets('app bootstraps', (tester) async {
    await tester.pumpWidget(const CitizenApp());
    // 等待异步锁检查完成并渲染主界面。
    await tester.pumpAndSettle();

    // 底部导航最左侧为广场，公民 tab 右移；个人多签入口迁到交易页。
    expect(find.text('广场'), findsWidgets);
    expect(find.text('暂无推荐动态'), findsOneWidget);
    expect(find.text('交易'), findsWidgets);
    expect(find.text('多签'), findsNothing);
    expect(find.text('消息'), findsNothing);
    // app 启动会初始化链 RPC(smoldot);libsmoldot 不可用(纯 Dart CI 无宿主 .so)
    // 则跳过此全量启动冒烟,真机/集成构建照跑(首启权限引导用例不依赖 native,仍跑)。
    // testWidgets 的 skip 仅接受 bool,故以「有无 skip 原因」转 bool。
  }, skip: smoldotNativeSkipReason() != null);
}
