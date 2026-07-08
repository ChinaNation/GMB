import 'package:flutter/material.dart';
import 'package:flutter_test/flutter_test.dart';
import 'package:citizenapp/wallet/core/wallet_manager.dart';
import 'package:citizenapp/wallet/pages/create_wallet_onboarding_page.dart';
import 'package:citizenapp/wallet/wallet_gate.dart';

WalletProfile _hotProfile() {
  return WalletProfile(
    walletIndex: 1,
    walletName: '钱包1',
    walletIcon: 'wallet',
    balance: 0,
    address: 'gate-test-address',
    pubkeyHex: 'ab' * 32,
    alg: 'sr25519',
    ss58: 2027,
    createdAtMillis: 0,
    source: 'created',
    signMode: 'local',
  );
}

Widget _gate({required Future<WalletProfile?> Function() loader}) {
  return MaterialApp(
    home: WalletGate(
      defaultWalletLoader: loader,
      child: const Scaffold(body: Text('main-shell')),
    ),
  );
}

Widget _onboarding({
  required Future<bool> Function() probe,
  VoidCallback? onCreated,
}) {
  return MaterialApp(
    home: CreateWalletOnboardingPage(
      onCreated: onCreated ?? () {},
      deviceSecureProbe: probe,
    ),
  );
}

void main() {
  // 强制创建页内容较长，用高视口避免 ListView 懒加载导致断言目标未构建。
  void useTallViewport(WidgetTester tester) {
    tester.view.physicalSize = const Size(800, 1600);
    tester.view.devicePixelRatio = 1.0;
    addTearDown(tester.view.reset);
  }

  group('WalletGate', () {
    testWidgets('无钱包时进入强制创建页，且无导入入口', (tester) async {
      useTallViewport(tester);
      await tester.pumpWidget(_gate(loader: () async => null));
      await tester.pumpAndSettle();

      expect(find.text('创建你的公民钱包'), findsOneWidget);
      expect(find.text('main-shell'), findsNothing);
      // 只创建不导入：页面上不允许出现任何导入入口。
      expect(find.textContaining('导入'), findsNothing);
      // 无 AppBar 返回键。
      expect(find.byType(BackButton), findsNothing);
    });

    testWidgets('有热钱包直接放行主界面', (tester) async {
      await tester.pumpWidget(_gate(loader: () async => _hotProfile()));
      await tester.pumpAndSettle();

      expect(find.text('main-shell'), findsOneWidget);
      expect(find.byType(CreateWalletOnboardingPage), findsNothing);
    });

    testWidgets('创建成功后翻转到主界面', (tester) async {
      useTallViewport(tester);
      await tester.pumpWidget(_gate(loader: () async => null));
      await tester.pumpAndSettle();

      final page = tester.widget<CreateWalletOnboardingPage>(
        find.byType(CreateWalletOnboardingPage),
      );
      page.onCreated();
      await tester.pumpAndSettle();

      expect(find.text('main-shell'), findsOneWidget);
      expect(find.byType(CreateWalletOnboardingPage), findsNothing);
    });

    testWidgets('本地库读取失败停在错误态，重试后恢复', (tester) async {
      var calls = 0;
      await tester.pumpWidget(_gate(loader: () async {
        calls++;
        if (calls == 1) {
          throw Exception('isar busy');
        }
        return _hotProfile();
      }));
      await tester.pumpAndSettle();

      // 读取失败既不误判成「无钱包」，也不放行。
      expect(find.text('本地钱包数据库繁忙，请稍后重试'), findsOneWidget);
      expect(find.text('main-shell'), findsNothing);
      expect(find.byType(CreateWalletOnboardingPage), findsNothing);

      await tester.tap(find.text('重试'));
      await tester.pumpAndSettle();
      expect(find.text('main-shell'), findsOneWidget);
    });
  });

  group('CreateWalletOnboardingPage', () {
    testWidgets('未开启系统锁屏：警示卡展示且创建按钮禁用', (tester) async {
      useTallViewport(tester);
      await tester.pumpWidget(_onboarding(probe: () async => false));
      await tester.pumpAndSettle();

      expect(find.text('未检测到系统锁屏'), findsOneWidget);
      expect(find.text('开启系统锁屏后可创建'), findsOneWidget);
      final button = tester.widget<FilledButton>(
        find.widgetWithText(FilledButton, '创建热钱包'),
      );
      expect(button.onPressed, isNull);
    });

    testWidgets('重新检测通过后创建按钮启用', (tester) async {
      useTallViewport(tester);
      var secure = false;
      await tester.pumpWidget(_onboarding(probe: () async => secure));
      await tester.pumpAndSettle();
      expect(find.text('未检测到系统锁屏'), findsOneWidget);

      secure = true;
      await tester.tap(find.text('重新检测'));
      await tester.pumpAndSettle();

      expect(find.text('未检测到系统锁屏'), findsNothing);
      expect(find.text('创建完成后进入公民广场'), findsOneWidget);
      final button = tester.widget<FilledButton>(
        find.widgetWithText(FilledButton, '创建热钱包'),
      );
      expect(button.onPressed, isNotNull);
    });

    testWidgets('默认选中 12 词（推荐），可切换 24 词', (tester) async {
      useTallViewport(tester);
      await tester.pumpWidget(_onboarding(probe: () async => true));
      await tester.pumpAndSettle();

      expect(find.text('12 个助记词'), findsOneWidget);
      expect(find.text('24 个助记词'), findsOneWidget);
      expect(find.text('推荐'), findsOneWidget);

      Finder selectedIconIn(String cardTitle) => find.descendant(
            of: find.ancestor(
              of: find.text(cardTitle),
              matching: find.byType(InkWell),
            ),
            matching: find.byIcon(Icons.check_circle),
          );

      expect(selectedIconIn('12 个助记词'), findsOneWidget);
      expect(selectedIconIn('24 个助记词'), findsNothing);

      await tester.tap(find.text('24 个助记词'));
      await tester.pump();

      expect(selectedIconIn('24 个助记词'), findsOneWidget);
      expect(selectedIconIn('12 个助记词'), findsNothing);
    });
  });
}
