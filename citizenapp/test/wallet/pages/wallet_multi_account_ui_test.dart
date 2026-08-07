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
  // 文件级唯一 Isar 生命周期(隔离临时目录)。必须在 main() 顶部调一次,不能在多个 group
  // 内各调一次——两个 group 各开一次 IsarCore 会导致第二个 setUpAll 挂死(12 分钟超时)。
  useIsolatedIsar();

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

  group('「＋」入口三项菜单（添加下一个账户 / 添加指定账户 / 导入冷钱包）', () {
    testWidgets('有热钱包时三项齐全,导入冷钱包在最下', (tester) async {
      var next = false;
      var specify = false;
      var cold = false;
      await tester.pumpWidget(
        MaterialApp(
          home: Scaffold(
            body: WalletEntryChooserSheet(
              canAddAccount: true,
              onAddNextAccount: () => next = true,
              onAddSpecifyAccount: () => specify = true,
              onImportCold: () => cold = true,
            ),
          ),
        ),
      );
      expect(find.text('添加下一个账户'), findsOneWidget);
      expect(find.text('添加指定账户'), findsOneWidget);
      expect(find.text('导入冷钱包'), findsOneWidget);
      // 不得出现热钱包创建 / 导入入口。
      expect(find.text('创建钱包'), findsNothing);
      expect(find.text('导入热钱包'), findsNothing);

      await tester.tap(find.text('添加下一个账户'));
      await tester.tap(find.text('添加指定账户'));
      await tester.tap(find.text('导入冷钱包'));
      await tester.pump();
      expect(next && specify && cold, isTrue);
    });

    testWidgets('无热钱包时只有「导入冷钱包」', (tester) async {
      await tester.pumpWidget(
        MaterialApp(
          home: Scaffold(
            body: WalletEntryChooserSheet(
              canAddAccount: false,
              onAddNextAccount: () {},
              onAddSpecifyAccount: () {},
              onImportCold: () {},
            ),
          ),
        ),
      );
      expect(find.text('导入冷钱包'), findsOneWidget);
      expect(find.text('添加下一个账户'), findsNothing);
      expect(find.text('添加指定账户'), findsNothing);
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
            body: WalletAccountTile(
              account: _makeAccount(),
              onTap: () {},
              onScan: () {},
              onRename: () {},
              onDetail: () {},
              onDelete: () {},
            ),
          ),
        ),
      );
      expect(find.text('账户1'), findsOneWidget);
      expect(find.text('#1'), findsOneWidget);
      // 长 SS58 被截断展示（首8…末6）。
      expect(find.textContaining('…'), findsOneWidget);
    });

    testWidgets('点击账户行触发 onTap', (tester) async {
      var tapped = false;
      await tester.pumpWidget(
        MaterialApp(
          home: Scaffold(
            body: WalletAccountTile(
              account: _makeAccount(),
              onTap: () => tapped = true,
              onScan: () {},
              onRename: () {},
              onDetail: () {},
              onDelete: () {},
            ),
          ),
        ),
      );
      await tester.tap(find.text('账户1'));
      await tester.pump();
      expect(tapped, isTrue);
    });

    testWidgets('扫码在竖三点左侧且点击只触发当前账户扫码', (tester) async {
      var scanned = false;
      var cardTapped = false;
      await tester.pumpWidget(
        MaterialApp(
          home: Scaffold(
            body: WalletAccountTile(
              account: _makeAccount(),
              onTap: () => cardTapped = true,
              onScan: () => scanned = true,
              onRename: () {},
              onDetail: () {},
              onDelete: () {},
            ),
          ),
        ),
      );
      final scan = find.byTooltip('扫码签名');
      final menu = find.byTooltip('账户操作');
      expect(scan, findsOneWidget);
      expect(menu, findsOneWidget);
      expect(tester.getCenter(scan).dx, lessThan(tester.getCenter(menu).dx));
      await tester.tap(scan);
      await tester.pump();
      expect(scanned, isTrue);
      expect(cardTapped, isFalse);
    });

    testWidgets('账户0菜单为重命名/账户详情/删除钱包，非0显示删除账户', (tester) async {
      await tester.pumpWidget(
        MaterialApp(
          home: Scaffold(
            body: WalletAccountTile(
              account: _makeAccount(index: 0, name: '账户0'),
              onTap: () {},
              onScan: () {},
              onRename: () {},
              onDetail: () {},
              onDelete: () {},
            ),
          ),
        ),
      );
      await tester.tap(find.byTooltip('账户操作'));
      await tester.pumpAndSettle();
      expect(find.text('重命名'), findsOneWidget);
      expect(find.text('账户详情'), findsOneWidget);
      expect(find.text('删除钱包'), findsOneWidget);
      expect(find.text('删除账户'), findsNothing);
    });

    testWidgets('非0账户菜单显示删除账户而不是删除钱包', (tester) async {
      await tester.pumpWidget(
        MaterialApp(
          home: Scaffold(
            body: WalletAccountTile(
              account: _makeAccount(index: 5, name: '账户5'),
              onTap: () {},
              onScan: () {},
              onRename: () {},
              onDetail: () {},
              onDelete: () {},
            ),
          ),
        ),
      );
      await tester.tap(find.byTooltip('账户操作'));
      await tester.pumpAndSettle();
      expect(find.text('删除账户'), findsOneWidget);
      expect(find.text('删除钱包'), findsNothing);
    });

    testWidgets('账户行与冷钱包行可在同一列表共存', (tester) async {
      await tester.pumpWidget(
        MaterialApp(
          home: Scaffold(
            body: ListView(
              children: [
                WalletAccountTile(
                  account: _makeAccount(index: 0, name: '账户0'),
                  onTap: () {},
                  onScan: () {},
                  onRename: () {},
                  onDetail: () {},
                  onDelete: () {},
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

  group('AccountDetailPage（顶部账户资料 + AppBar 菜单 + 找回钱包功能）', () {
    // 账户详情渲染 WalletActionCard(读 ClearingBankPrefs/SharedPreferences)并加载本地
    // 交易记录(Isar,由文件级 useIsolatedIsar 提供);此处只补 SharedPreferences mock。
    setUp(() {
      SharedPreferences.setMockInitialValues(<String, Object>{});
    });

    testWidgets('顶部完整地址和卡片右上角二维码，删除/私钥/清算行不残留在正文', (tester) async {
      tester.view.physicalSize = const Size(1200, 3200);
      tester.view.devicePixelRatio = 1.0;
      addTearDown(tester.view.reset);
      final account = _makeAccount(name: '账户1');
      await tester.pumpWidget(
        MaterialApp(
          home: AccountDetailPage(
            account: account,
          ),
        ),
      );
      await tester.pump();
      expect(find.text('账户详情'), findsOneWidget);
      expect(find.text(account.ss58Address), findsOneWidget);
      expect(find.byTooltip('账户二维码'), findsOneWidget);
      expect(find.byTooltip('复制 SS58 地址'), findsOneWidget);
      final headerFinder = find.byWidgetPredicate(
        (widget) =>
            widget is Container &&
            widget.decoration is BoxDecoration &&
            (widget.decoration! as BoxDecoration).gradient != null,
        description: '账户详情渐变资料卡',
      );
      final headerRect = tester.getRect(headerFinder);
      final nameRect = tester.getRect(find.text(account.accountName));
      final qrRect = tester.getRect(find.byTooltip('账户二维码'));
      final copyRect = tester.getRect(find.byTooltip('复制 SS58 地址'));
      expect(qrRect.left, greaterThanOrEqualTo(nameRect.right),
          reason: '二维码必须位于账户名布局区之外，不能紧挨名称排版');
      expect(qrRect.top - headerRect.top, lessThanOrEqualTo(12),
          reason: '二维码触控区必须贴在账户卡顶部');
      expect(headerRect.right - qrRect.right, lessThanOrEqualTo(12),
          reason: '二维码触控区必须贴在账户卡右侧');
      expect(copyRect.top, greaterThanOrEqualTo(qrRect.bottom),
          reason: '复制按钮必须下移到独立地址行，不能继续占用二维码旁的地址宽度');
      expect(headerRect.right - copyRect.right, lessThanOrEqualTo(24),
          reason: '复制按钮必须靠齐账户卡内容右侧');
      expect(find.text('点击查看私钥'), findsNothing);
      expect(find.text('删除账户'), findsNothing);
      expect(find.text('删除钱包'), findsNothing);
      expect(find.text('绑定 / 切换清算行'), findsNothing);
    });

    testWidgets('AppBar 右侧竖三点只有「清算行 / 查看私钥」', (tester) async {
      await tester.pumpWidget(
        MaterialApp(
          home: AccountDetailPage(
            account: _makeAccount(index: 0, name: '账户0'),
          ),
        ),
      );
      await tester.pump();
      await tester.tap(find.byIcon(Icons.more_vert));
      await tester.pumpAndSettle();
      expect(find.text('清算行'), findsOneWidget);
      expect(find.text('查看私钥'), findsOneWidget);
      expect(find.text('删除钱包'), findsNothing);
      expect(find.text('重命名'), findsNothing);
    });

    testWidgets('账户右上角二维码无条件进入固定账户码页', (tester) async {
      final account = _makeAccount(index: 5, name: '日常账户');
      await tester.pumpWidget(
        MaterialApp(
          home: AccountDetailPage(account: account),
        ),
      );
      await tester.pump();
      await tester.tap(find.byTooltip('账户二维码'));
      await tester.pumpAndSettle();
      expect(find.text('二维码'), findsOneWidget);
      expect(find.text('日常账户'), findsOneWidget);
      expect(find.text('账户码：扫描可向本账户转账，或用于扫码登录'), findsOneWidget);
      // 账户详情表达账户，不表达身份：不读链、不出名片码、无任何时效文案。
      expect(find.text('扫描此二维码可加为联系人，或向其转账'), findsNothing);
      expect(find.textContaining('分钟内有效'), findsNothing);
    });
  });

  group('AddAccountSheet（重录助记词 + 多序号解析 → addAccounts）', () {
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

    testWidgets('指定序号模式:直接露出序号输入框,可录入多序号 "1 5 9",提交按钮在场', (tester) async {
      // 只验 UI 装配(指定序号模式渲染 / 多序号录入 / 提交按钮在场)。模式由入口固定,
      // 面板内不再有切换器。「1 5 9」→ [1,5,9] 解析由 parseAccountIndices 单测覆盖;
      // addAccounts([1,5,9]) 落库效果由 wallet_multi_account_test 覆盖。
      await tester.pumpWidget(
        const MaterialApp(
          home: Scaffold(
            body: AddAccountSheet(
              masterId: '0xmaster',
              mode: AddAccountMode.specify,
            ),
          ),
        ),
      );
      await tester.pumpAndSettle();

      // 指定序号模式直接露出序号输入框(面板内不再有模式切换器)。
      expect(find.text('添加指定账户'), findsOneWidget);
      await tester.enterText(find.byType(TextField).at(1), '1 5 9');
      await tester.pump();

      expect(find.text('1 5 9'), findsOneWidget);
      expect(find.text('确认添加'), findsOneWidget);
    });
  });
}

