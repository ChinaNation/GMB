import 'package:flutter/material.dart';
import 'package:flutter/services.dart';
import 'package:flutter_test/flutter_test.dart';
import 'package:shared_preferences/shared_preferences.dart';
import 'package:citizenapp/wallet/core/hardware_bound_seed_vault.dart';
import 'package:citizenapp/wallet/core/wallet_manager.dart';
import 'package:citizenapp/wallet/pages/account_detail_page.dart';
import 'package:citizenapp/wallet/pages/wallet_page.dart';
import 'package:citizenapp/wallet/widgets/add_account_sheet.dart';

import '../../support/fake_secure_seed_store.dart';
import '../../support/isar_test_env.dart';

class _MemoryBlobStore implements VaultBlobStore {
  final Map<String, String> values = <String, String>{};
  @override
  Future<String?> read(String key) async => values[key];
  @override
  Future<void> write(String key, String value) async => values[key] = value;
  @override
  Future<void> delete(String key) async => values.remove(key);
}

Account _makeAccount({
  int index = 1,
  String name = '账户1',
  String ss58 = 'w5Bc7ma8qUcECfQDJmRyQM2wGmga5XSYtz7DvEengQ86xBWrT',
}) {
  return Account(
    masterId:
        '0x0000000000000000000000000000000000000000000000000000000000000001',
    accountIndex: index,
    accountId: '0x${index.toRadixString(16).padLeft(64, '0')}',
    ss58Address: ss58,
    accountName: name,
  );
}

WalletProfile _makeColdWallet({int walletIndex = 2, String name = '冷钱包'}) {
  return WalletProfile(
    walletIndex: walletIndex,
    walletName: name,
    walletIcon: 'wallet',
    balance: 0,
    ss58Address: 'addr_$walletIndex',
    accountId: '0x${walletIndex.toRadixString(16).padLeft(64, '0')}',
    alg: 'sr25519',
    ss58: 2027,
    createdAtMillis: 0,
    source: 'test',
    signMode: 'external',
  );
}

void main() {
  TestWidgetsFlutterBinding.ensureInitialized();

  group('parseAccountIndices（空格分隔多序号解析）', () {
    test('连续 "1 2 3" → [1,2,3]', () {
      final parsed = parseAccountIndices('1 2 3');
      expect(parsed.isSuccess, isTrue);
      expect(parsed.indices, [1, 2, 3]);
    });

    test('断续 "1 5 9" → [1,5,9]', () {
      final parsed = parseAccountIndices('1 5 9');
      expect(parsed.isSuccess, isTrue);
      expect(parsed.indices, [1, 5, 9]);
    });

    test('多余空白 "  1   5  9 " 仍归一化成 [1,5,9]', () {
      final parsed = parseAccountIndices('  1   5  9 ');
      expect(parsed.indices, [1, 5, 9]);
    });

    test('空串 → 失败并给出提示', () {
      final parsed = parseAccountIndices('   ');
      expect(parsed.isSuccess, isFalse);
      expect(parsed.error, isNotNull);
    });

    test('含非数字 "1 a 3" → 失败', () {
      final parsed = parseAccountIndices('1 a 3');
      expect(parsed.isSuccess, isFalse);
      expect(parsed.error, contains('a'));
    });
  });

  group('入口只余导入冷钱包（热钱包创建/导入入口已删）', () {
    testWidgets('WalletEntryChooserSheet 只有「导入冷钱包」', (tester) async {
      var tapped = false;
      await tester.pumpWidget(
        MaterialApp(
          home: Scaffold(
            body: WalletEntryChooserSheet(onImportCold: () => tapped = true),
          ),
        ),
      );
      expect(find.text('导入冷钱包'), findsOneWidget);
      expect(find.text('创建钱包'), findsNothing);
      expect(find.text('导入热钱包'), findsNothing);

      await tester.tap(find.text('导入冷钱包'));
      await tester.pump();
      expect(tapped, isTrue);
    });

    testWidgets('WalletEmptyChoices 空态也只有「导入冷钱包」', (tester) async {
      await tester.pumpWidget(
        MaterialApp(
          home: Scaffold(
            body: WalletEmptyChoices(onImportCold: () {}),
          ),
        ),
      );
      expect(find.text('导入冷钱包'), findsOneWidget);
      expect(find.text('创建钱包'), findsNothing);
      expect(find.text('导入热钱包'), findsNothing);
    });
  });

  group('WalletAccountTile（账户行渲染 + 冷钱包共存）', () {
    testWidgets('渲染账户名与短地址', (tester) async {
      await tester.pumpWidget(
        MaterialApp(
          home: Scaffold(
            body: WalletAccountTile(account: _makeAccount(), onTap: () {}),
          ),
        ),
      );
      expect(find.text('账户1'), findsOneWidget);
      expect(find.text('#1'), findsOneWidget);
      // 长 SS58 被截断展示（首8…末6）。
      expect(find.textContaining('…'), findsOneWidget);
    });

    testWidgets('账户0 渲染「默认用户」徽标', (tester) async {
      await tester.pumpWidget(
        MaterialApp(
          home: Scaffold(
            body: WalletAccountTile(
              account: _makeAccount(index: 0, name: '账户0'),
              isDefault: true,
              onTap: () {},
            ),
          ),
        ),
      );
      expect(find.text('默认用户'), findsOneWidget);
    });

    testWidgets('点击账户行触发 onTap', (tester) async {
      var tapped = false;
      await tester.pumpWidget(
        MaterialApp(
          home: Scaffold(
            body: WalletAccountTile(
              account: _makeAccount(),
              onTap: () => tapped = true,
            ),
          ),
        ),
      );
      await tester.tap(find.text('账户1'));
      await tester.pump();
      expect(tapped, isTrue);
    });

    testWidgets('账户行与冷钱包行可在同一列表共存', (tester) async {
      await tester.pumpWidget(
        MaterialApp(
          home: Scaffold(
            body: ListView(
              children: [
                WalletAccountTile(
                  account: _makeAccount(index: 0, name: '账户0'),
                  isDefault: true,
                  onTap: () {},
                ),
                WalletListTile(
                  wallet: _makeColdWallet(name: '我的冷钱包'),
                  showActions: true,
                  onTap: () {},
                  onRename: () {},
                  onDelete: () {},
                ),
              ],
            ),
          ),
        ),
      );
      // 热钱包账户行与冷钱包行同列出现。
      expect(find.text('账户0'), findsOneWidget);
      expect(find.text('我的冷钱包'), findsOneWidget);
    });
  });

  group('AccountDetailPage（私钥默认隐藏）', () {
    testWidgets('默认隐藏私钥，只显示「点击查看私钥」', (tester) async {
      await tester.pumpWidget(
        MaterialApp(
          home: AccountDetailPage(
            account: _makeAccount(name: '账户1'),
            walletName: '钱包1',
          ),
        ),
      );
      await tester.pump();
      expect(find.text('账户详情'), findsOneWidget);
      expect(find.text('点击查看私钥'), findsOneWidget);
      // 未确认前不得展示已揭示态的任何文案。
      expect(find.text('请手抄备份，不支持复制；导出即等于该账户控制权'), findsNothing);
      expect(find.widgetWithText(TextButton, '隐藏'), findsNothing);
      // 非账户0 → 删除该账户；账户0 → 删除钱包。
      expect(find.text('删除该账户'), findsOneWidget);
    });

    testWidgets('账户0 底部为「删除钱包」', (tester) async {
      await tester.pumpWidget(
        MaterialApp(
          home: AccountDetailPage(
            account: _makeAccount(index: 0, name: '账户0'),
            walletName: '钱包1',
          ),
        ),
      );
      await tester.pump();
      expect(find.text('删除钱包'), findsOneWidget);
      expect(find.text('删除该账户'), findsNothing);
    });
  });

  group('AddAccountSheet（重录助记词 + 多序号解析 → addAccounts）', () {
    useIsolatedIsar();
    late FakeSecureSeedStore fakeStore;
    const localAuthChannel = MethodChannel('plugins.flutter.io/local_auth');

    setUp(() async {
      SharedPreferences.setMockInitialValues(<String, Object>{});
      fakeStore = FakeSecureSeedStore();
      WalletManager.debugSeedStore = fakeStore;
      WalletManager.debugContactKeyStore = _MemoryBlobStore();
      TestDefaultBinaryMessengerBinding.instance.defaultBinaryMessenger
          .setMockMethodCallHandler(localAuthChannel, (call) async {
        switch (call.method) {
          case 'authenticate':
            return true;
          case 'isDeviceSupported':
          case 'deviceSupportsBiometrics':
          case 'canCheckBiometrics':
            return true;
          case 'getAvailableBiometrics':
            return <String>['fingerprint'];
          default:
            return null;
        }
      });
    });

    tearDown(() {
      TestDefaultBinaryMessengerBinding.instance.defaultBinaryMessenger
          .setMockMethodCallHandler(localAuthChannel, null);
    });

    testWidgets('指定序号模式:露出序号输入框,可录入多序号 "1 5 9",提交按钮在场',
        (tester) async {
      // 只验 UI 装配(渲染 / 模式切换 / 多序号录入 / 提交按钮在场)。
      // 「1 5 9」→ [1,5,9] 解析由 parseAccountIndices 单测覆盖;addAccounts([1,5,9])
      // 落库效果由 wallet_multi_account_test 覆盖 —— 不在 widget 层重复触发真实 isar
      // 往返(testWidgets fake-async 下经 UI 触发的 addAccounts+getAccounts 不可靠)。
      await tester.pumpWidget(
        const MaterialApp(
          home: Scaffold(body: AddAccountSheet(masterId: '0xmaster')),
        ),
      );
      await tester.pumpAndSettle();

      // 切到「指定序号」模式,露出序号输入框。
      await tester.tap(find.text('指定序号'));
      await tester.pumpAndSettle();

      await tester.enterText(find.byType(TextField).at(1), '1 5 9');
      await tester.pump();

      expect(find.text('1 5 9'), findsOneWidget);
      expect(find.text('确认添加'), findsOneWidget);
    });
  });
}
