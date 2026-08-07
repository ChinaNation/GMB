// 钱包详情 widget 测试:身份卡助记词区默认隐藏 + 查看确认取消 + 揭示成功显示明文。
import 'package:flutter/material.dart';
import 'package:flutter/services.dart';
import 'package:flutter_secure_storage/flutter_secure_storage.dart';
import 'package:flutter_test/flutter_test.dart';
import 'package:qr_flutter/qr_flutter.dart';
import 'package:citizenwallet/isar/wallet_isar.dart';
import 'package:citizenwallet/ui/wallet_detail_page.dart';
import 'package:citizenwallet/wallet/wallet_manager.dart';

const String kDevPhrase =
    'bottom drive obey lake curtain smoke basket hold race lonely fit walk';

void main() {
  final binding = TestWidgetsFlutterBinding.ensureInitialized();
  const securityChannel = MethodChannel('org.citizenwallet/security');
  const securityEvents = MethodChannel('org.citizenwallet/security_events');

  setUp(() async {
    FlutterSecureStorage.setMockInitialValues({});
    WalletManager.debugAuthGate = () async {};
    await WalletIsar.instance.resetForTest();
    // 揭示助记词会启用 ScreenshotGuard(平台通道):mock 成 no-op。
    binding.defaultBinaryMessenger
        .setMockMethodCallHandler(securityChannel, (call) async => null);
    binding.defaultBinaryMessenger
        .setMockMethodCallHandler(securityEvents, (call) async => null);
  });

  tearDown(() async {
    WalletManager.debugAuthGate = null;
    await WalletIsar.instance.resetForTest();
    binding.defaultBinaryMessenger
        .setMockMethodCallHandler(securityChannel, null);
    binding.defaultBinaryMessenger
        .setMockMethodCallHandler(securityEvents, null);
  });

  const walletFixture = Wallet(
    walletIndex: 1,
    walletName: '钱包1',
    masterId:
        '0x2afba9278e30ccf6a6ceb3a8b6e336b70068f045c666f2e7f4f9cc5f47db8972',
    createdAtMillis: 0,
    source: 'created',
  );

  testWidgets('身份卡助记词区默认隐藏 + 查看确认取消', (tester) async {
    // _load 走真实 Isar I/O + 加载 spinner 会让 pumpAndSettle 在 fake-async 下卡死;
    // 用 runAsync 让真实事件循环完成 getAccounts,再 pump 退出 loading。
    await tester.runAsync(() async {
      await tester.pumpWidget(const MaterialApp(
        home: WalletDetailPage(wallet: walletFixture),
      ));
      await Future<void>.delayed(const Duration(milliseconds: 300));
    });
    await tester.pump();

    expect(find.text('点击查看助记词'), findsOneWidget);
    expect(find.textContaining('钱包唯一备份'), findsOneWidget);
    expect(find.byTooltip('扫码签名'), findsOneWidget);

    await tester.tap(find.text('点击查看助记词'));
    await tester.pumpAndSettle();
    expect(find.text('查看助记词'), findsOneWidget);

    await tester.tap(find.text('取消'));
    await tester.pumpAndSettle();
    expect(find.text('查看助记词'), findsNothing);
    expect(find.text('点击查看助记词'), findsOneWidget);
  });

  testWidgets('查看助记词→确认→验证通过→显示助记词明文', (tester) async {
    // 造一个真实钱包(种子+助记词已加密落库),供 getMasterMnemonic 解密取回。
    late Wallet wallet;
    await tester.runAsync(() async {
      final created = await WalletManager().importWallet(kDevPhrase);
      wallet = created.wallet;
    });

    await tester.runAsync(() async {
      await tester
          .pumpWidget(MaterialApp(home: WalletDetailPage(wallet: wallet)));
      await Future<void>.delayed(const Duration(milliseconds: 300));
    });
    await tester.pump();

    await tester.tap(find.text('点击查看助记词'));
    await tester.pumpAndSettle();
    await tester.tap(find.text('查看'));
    // 查看 → getMasterMnemonic(真实 Isar+SecureStorage+解密)+ 启用防截屏。
    await tester.runAsync(() async {
      await Future<void>.delayed(const Duration(milliseconds: 300));
    });
    await tester.pumpAndSettle();

    expect(find.text(kDevPhrase), findsOneWidget);
  });

  testWidgets('账户列表每行有账户码入口，点它出账户码且不进账户详情', (tester) async {
    // 真实导入一个钱包，账户0 落库后账户列表才有行可点。
    late Wallet wallet;
    await tester.runAsync(() async {
      final created = await WalletManager().importWallet(kDevPhrase);
      wallet = created.wallet;
    });

    await tester.runAsync(() async {
      await tester
          .pumpWidget(MaterialApp(home: WalletDetailPage(wallet: wallet)));
      await Future<void>.delayed(const Duration(milliseconds: 300));
    });
    await tester.pump();

    // 列表标题为「账户列表」;二维码入口在每个账户行上，与右箭头并存。
    expect(find.text('账户列表'), findsOneWidget);
    final qrEntry = find.byTooltip('显示账户码');
    expect(qrEntry, findsWidgets);
    expect(find.byIcon(Icons.chevron_right), findsWidgets);

    await tester.tap(qrEntry.first);
    await tester.pumpAndSettle();

    // 弹出的是账户码（与账户详情共用同一份弹窗），且没有跳进账户详情。
    expect(find.byType(QrImageView), findsOneWidget);
    expect(find.text('复制'), findsOneWidget);
    expect(find.text('账户详情'), findsNothing);
    expect(find.textContaining('扫码登录'), findsNothing);
    expect(find.textContaining('分钟内有效'), findsNothing);

    await tester.tap(find.text('关闭'));
    await tester.pumpAndSettle();
    expect(find.byType(QrImageView), findsNothing);
  });

  testWidgets('账户序号三位数起 # 挪到左上角、数字另起一行', (tester) async {
    // 真实导入钱包(账户0)+ 指定序号添加 //100,列表同时出现两种徽章形态。
    late Wallet wallet;
    await tester.runAsync(() async {
      final created = await WalletManager().importWallet(kDevPhrase);
      wallet = created.wallet;
      await WalletManager().addAccount(wallet.masterId, index: 100);
    });

    await tester.runAsync(() async {
      await tester
          .pumpWidget(MaterialApp(home: WalletDetailPage(wallet: wallet)));
      await Future<void>.delayed(const Duration(milliseconds: 300));
    });
    await tester.pump();

    // 账户0:两位数以内保持 `#xx` 单行整串。
    expect(find.text('#0'), findsOneWidget);
    // 账户100:整串 `#100` 不复存在,# 与数字拆分各自成行。
    expect(find.text('#100'), findsNothing);
    expect(find.text('#'), findsOneWidget);
    expect(find.text('100'), findsOneWidget);
  });
}
