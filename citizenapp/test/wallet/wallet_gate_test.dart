import 'package:flutter/material.dart';
import 'package:flutter_test/flutter_test.dart';
import 'package:citizenapp/citizen/shared/account_derivation.dart';
import 'package:citizenapp/wallet/core/fake_hardware_bound_seed_vault.dart';
import 'package:citizenapp/wallet/core/wallet_manager.dart';
import 'package:citizenapp/wallet/pages/create_wallet_onboarding_page.dart';
import 'package:citizenapp/wallet/pages/import_wallet_page.dart';
import 'package:citizenapp/wallet/wallet_gate.dart';

WalletProfile _hotProfile() {
  return const WalletProfile(
    walletIndex: 1,
    walletName: '钱包1',
    walletIcon: 'wallet',
    balance: 0,
    ss58Address: 'gate-test-address',
    accountId:
        '0xabababababababababababababababababababababababababababababababab',
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
    testWidgets('无钱包时进入门禁页，含创建与导入入口', (tester) async {
      useTallViewport(tester);
      await tester.pumpWidget(_gate(loader: () async => null));
      await tester.pumpAndSettle();

      // 标题与创建按钮同为"创建钱包"，故 findsWidgets（≥1）。
      expect(find.text('创建钱包'), findsWidgets);
      expect(find.text('main-shell'), findsNothing);
      // 门禁页提供创建与导入两条入口。
      expect(find.widgetWithText(FilledButton, '创建钱包'), findsOneWidget);
      expect(find.text('已有钱包？导入助记词'), findsOneWidget);
      // 门禁页自身无 AppBar 返回键（PopScope 禁止退出门禁）。
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

    testWidgets('运行期钱包被删光时立即踢回门禁页', (tester) async {
      useTallViewport(tester);
      WalletProfile? current = _hotProfile();
      await tester.pumpWidget(_gate(loader: () async => current));
      await tester.pumpAndSettle();
      expect(find.text('main-shell'), findsOneWidget);

      // 模拟「我的 → 钱包列表」里删光钱包：数据没了 + 版本号自增。
      current = null;
      WalletManager.walletsRevision.value++;
      await tester.pumpAndSettle();

      expect(find.text('main-shell'), findsNothing);
      expect(find.byType(CreateWalletOnboardingPage), findsOneWidget);
    });
  });

  group('有效热钱包谓词', () {
    const accountId =
        '0xabababababababababababababababababababababababababababababababab';

    WalletProfile profile({
      String id = accountId,
      String? ss58,
      String signMode = 'local',
    }) {
      return WalletProfile(
        walletIndex: 1,
        walletName: '钱包1',
        walletIcon: 'wallet',
        balance: 0,
        accountId: id,
        ss58Address: ss58 ?? ss58FromAccountIdText(accountId),
        alg: 'sr25519',
        ss58: 2027,
        createdAtMillis: 0,
        source: 'created',
        signMode: signMode,
      );
    }

    late FakeHardwareBoundSeedVault vault;

    setUp(() {
      vault = FakeHardwareBoundSeedVault();
      WalletManager.debugSeedStore = vault;
    });

    test('热钱包 + accountId 规范 + ss58 一致 + 有种子 → 有效', () async {
      await vault.putSeed(1, '00' * 32);
      expect(await WalletManager().isUsableHotWallet(profile()), isTrue);
    });

    test('冷钱包不作为门控依据', () async {
      await vault.putSeed(1, '00' * 32);
      expect(
        await WalletManager().isUsableHotWallet(profile(signMode: 'external')),
        isFalse,
      );
    });

    test('accountId 为空的半残钱包不作为门控依据', () async {
      await vault.putSeed(1, '00' * 32);
      expect(
        await WalletManager().isUsableHotWallet(profile(id: '', ss58: 'x')),
        isFalse,
      );
    });

    test('ss58 与 accountId 对不上不作为门控依据', () async {
      await vault.putSeed(1, '00' * 32);
      expect(
        await WalletManager().isUsableHotWallet(profile(ss58: '对不上的地址')),
        isFalse,
      );
    });

    test('有壳无钥（严档种子条目缺失）不作为门控依据', () async {
      expect(await WalletManager().isUsableHotWallet(profile()), isFalse);
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
        find.widgetWithText(FilledButton, '创建钱包'),
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
        find.widgetWithText(FilledButton, '创建钱包'),
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

    testWidgets('点导入入口进入 ImportWalletPage', (tester) async {
      useTallViewport(tester);
      await tester.pumpWidget(_onboarding(probe: () async => true));
      await tester.pumpAndSettle();

      await tester.tap(find.text('已有钱包？导入助记词'));
      await tester.pumpAndSettle();

      expect(find.byType(ImportWalletPage), findsOneWidget);
      expect(find.text('导入热钱包'), findsOneWidget);
    });
  });
}
