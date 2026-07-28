// 离线签名页 widget 测试(既有基线修复:runtime 升级哈希签的绿 banner 与明细
// 不再自相矛盾)。normal + decoded==null 是 runtime 升级哈希签唯一产生的组合,
// 旧代码在明细里错误地走了“拒绝签名”分支。
import 'package:flutter/material.dart';
import 'package:flutter/services.dart';
import 'package:flutter_test/flutter_test.dart';
import 'package:citizenwallet/qr/qr_protocols.dart';
import 'package:citizenwallet/qr/envelope.dart';
import 'package:citizenwallet/qr/bodies/sign_request_body.dart';
import 'package:citizenwallet/signer/qr_signer.dart';
import 'package:citizenwallet/ui/offline_sign_page.dart';
import 'package:citizenwallet/wallet/wallet_manager.dart';

void main() {
  final binding = TestWidgetsFlutterBinding.ensureInitialized();

  const securityChannel = MethodChannel('org.citizenwallet/security');
  const securityEvents = MethodChannel('org.citizenwallet/security_events');

  setUp(() {
    // ScreenshotGuard 走平台通道:mock 成 no-op,避免 MissingPluginException。
    binding.defaultBinaryMessenger
        .setMockMethodCallHandler(securityChannel, (call) async => null);
    binding.defaultBinaryMessenger
        .setMockMethodCallHandler(securityEvents, (call) async => null);
  });

  tearDown(() {
    binding.defaultBinaryMessenger
        .setMockMethodCallHandler(securityChannel, null);
    binding.defaultBinaryMessenger
        .setMockMethodCallHandler(securityEvents, null);
  });

  const signerPk =
      '0xb606fc73f57f03cdb4c932d475ab426043e429cecc2ffff0d2672b0df8398c48';
  const account = Account(
    masterId:
        '0x2afba9278e30ccf6a6ceb3a8b6e336b70068f045c666f2e7f4f9cc5f47db8972',
    accountIndex: 1,
    accountId: signerPk,
    ss58Address: 'w5FhUDLW4BxsE1QXK4sNjPZ8rqSnK2QeVpUfXzqczpWdxChxV',
    accountName: '账户1',
    createdAtMillis: 0,
  );

  testWidgets('runtime 升级哈希签:绿 banner 且无“拒绝签名”矛盾行', (tester) async {
    final now = DateTime.now().millisecondsSinceEpoch ~/ 1000;
    // 32 字节升级摘要。
    final hashHex = '0x'
        '${List<int>.generate(32, (i) => i).map((b) => b.toRadixString(16).padLeft(2, '0')).join()}';
    final request = QrEnvelope<SignRequestBody>(
      kind: QrKind.signRequest,
      id: 'offline-req-hashonly-0001',
      expiresAt: now + 90,
      body: SignRequestBody.fromHex(
        action: QrActions.runtimeUpgradeHash,
        signerPublicKeyHex: signerPk,
        payloadHex: hashHex,
      ),
    );
    final raw = QrSigner().encodeRequest(request);

    await tester.pumpWidget(MaterialApp(
      home: OfflineSignPage(account: account, walletName: '钱包1', raw: raw),
    ));
    // initState 用 addPostFrameCallback 解析请求 → 需额外 pump 落地 setState。
    await tester.pump();
    await tester.pump();

    // 绿 banner(hash-only 变体)在场。
    expect(find.textContaining('可以签名'), findsWidgets);
    // 矛盾修复核心:hash-only 合法签绝不出现“拒绝签名”明细行。
    expect(find.text('拒绝签名'), findsNothing);
    // hash-only 说明行在场。
    expect(find.text('32 字节升级摘要（哈希）'), findsOneWidget);

    // 强制释放页面,取消倒计时 Timer,避免 pending timer 报错。
    await tester.pumpWidget(const SizedBox());
  });
}
