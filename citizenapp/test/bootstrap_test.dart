import 'dart:io';

import 'package:flutter/services.dart';
import 'package:flutter_test/flutter_test.dart';
import 'package:shared_preferences/shared_preferences.dart';
import 'package:citizenapp/isar/app_isar.dart';
import 'package:citizenapp/main.dart';
import 'package:citizenapp/security/app_permission_bootstrap.dart';

import 'support/isar_test_env.dart';
import 'support/smoldot_native_probe.dart';

/// 直插一条热钱包记录（绕开 createWallet 的设备锁屏前置），让账户门禁放行。
Future<void> seedHotWallet() async {
  await WalletIsar.instance.writeTxn((isar) async {
    final entity = WalletProfileEntity()
      ..walletIndex = 1
      ..walletName = '钱包1'
      ..walletIcon = 'wallet'
      ..balance = 0
      ..ss58Address = 'bootstrap-test-address'
      ..accountId = 'ab' * 32
      ..alg = 'sr25519'
      ..ss58 = 2027
      ..createdAtMillis = 0
      ..source = 'created'
      ..signMode = 'local';
    await isar.walletProfileEntitys.put(entity);
  });
}

/// 门禁的 Isar 查询走真实事件循环，FakeAsync 的 pumpAndSettle 等不到；
/// 用 runAsync 让真异步推进，直到目标文案出现或超时。
Future<void> pumpUntilFound(
  WidgetTester tester,
  Finder finder, {
  int maxRounds = 80,
}) async {
  for (var i = 0; i < maxRounds && !tester.any(finder); i++) {
    await tester
        .runAsync(() => Future<void>.delayed(const Duration(milliseconds: 25)));
    await tester.pump();
  }
}

void main() {
  TestWidgetsFlutterBinding.ensureInitialized();
  useIsolatedIsar();

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
    // 账户门禁要求至少 1 个热钱包才放行主界面。
    await tester.runAsync(() async {
      await WalletIsar.instance.resetForTest();
      await seedHotWallet();
    });

    await tester.pumpWidget(const CitizenApp());
    // 等待异步锁检查 + 账户门禁的 Isar 查询完成并渲染主界面。
    await pumpUntilFound(tester, find.text('广场'));
    await tester.pumpAndSettle();

    // 底部导航最左侧为广场，公民 tab 右移；个人多签入口迁到交易页。
    expect(find.text('广场'), findsWidgets);
    expect(find.text('暂无推荐动态'), findsOneWidget);
    expect(find.text('交易'), findsWidgets);
    expect(find.text('多签'), findsNothing);
    expect(find.text('消息'), findsNothing);
    // app 启动会初始化链 RPC(smoldot);libsmoldot 不可用(纯 Dart CI 无宿主 .so)
    // 则跳过此全量启动冒烟,真机/集成构建照跑(首启权限引导用例不依赖 native,仍跑)。
    // testWidgets 的 skip 仅接受 bool。连活链的全量启动冒烟默认跳过(离线会 hang 到超时);
    // 本地设 RUN_BOOTSTRAP_CHAIN_SMOKE=1 且 libsmoldot native 可用时才跑,由集成 / APK 测试覆盖。
  },
      skip: Platform.environment['RUN_BOOTSTRAP_CHAIN_SMOKE'] == null ||
          smoldotNativeSkipReason() != null);

  testWidgets('no wallet: bootstraps into forced create-wallet page',
      (tester) async {
    // 无任何钱包时，账户门禁应拦在强制创建页，不进广场。
    // 强制创建页不建 AppShell(不触发 smoldot),纯 Dart CI 也照跑。
    // 页面内容较长,高视口避免 ListView 懒加载导致底部按钮未构建。
    tester.view.physicalSize = const Size(800, 1600);
    tester.view.devicePixelRatio = 1.0;
    addTearDown(tester.view.reset);
    await tester.runAsync(() => WalletIsar.instance.resetForTest());

    await tester.pumpWidget(const CitizenApp());
    await pumpUntilFound(tester, find.text('创建钱包'));
    await tester.pump();

    // 标题与创建按钮同为"创建钱包"（两处 Text），故 findsWidgets（≥1）。
    expect(find.text('创建钱包'), findsWidgets);
    expect(find.text('广场'), findsNothing);
  });
}
