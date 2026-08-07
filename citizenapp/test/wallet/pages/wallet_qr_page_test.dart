import 'package:flutter/material.dart';
import 'package:flutter_test/flutter_test.dart';

import 'package:citizenapp/citizen/shared/account_derivation.dart';
import 'package:citizenapp/qr/bodies/account_id_code_body.dart';
import 'package:citizenapp/qr/envelope.dart';
import 'package:citizenapp/qr/qr_protocols.dart';
import 'package:citizenapp/wallet/pages/wallet_qr_page.dart';

/// 账户码展示页（`k=5 account_id_code`，固定码，唯一入口钱包-账户详情）。
///
/// 验证点：
/// - 载荷严格为固定 `k=5`，body 只有 `account_id`，顶层无 `i/e`
/// - 本机账户标签只在页面顶部显示，绝不进载荷（防止被扫码端当成公开身份）
/// - 无条件出账户码：不读链、不判身份，任意账户同一行为
/// - account_id 非法时不进页面
void main() {
  const accountId =
      '0x8eaf04151687736326c9fea17e25fc5287613693c912909cb226aa4794f26a48';
  const accountLabel = '日常账户';
  final ss58Address = ss58FromAccountIdText(accountId);

  Future<void> openPage(WidgetTester tester) async {
    await tester.pumpWidget(
      const MaterialApp(
        home: WalletQrPage(accountId: accountId, accountLabel: accountLabel),
      ),
    );
    await tester.pump();
    await tester.pump(const Duration(milliseconds: 100));
  }

  testWidgets('页面渲染账户标签、完整地址、复制与下载入口', (tester) async {
    await openPage(tester);

    expect(find.text('二维码'), findsOneWidget);
    expect(find.text(accountLabel), findsOneWidget);
    expect(find.text(ss58Address), findsOneWidget);
    expect(find.byIcon(Icons.copy), findsOneWidget);
    expect(find.byIcon(Icons.download), findsOneWidget);
  });

  testWidgets('底部文案覆盖转账与扫码登录，且不出现任何时效文案', (tester) async {
    await openPage(tester);

    expect(find.text('账户码：扫描可向本账户转账，或用于扫码登录'), findsOneWidget);
    expect(find.textContaining('分钟内有效'), findsNothing);
    // 账户码不是名片码：不得出现加联系人文案。
    expect(find.text('扫描此二维码可加为联系人，或向其转账'), findsNothing);
  });

  test('account_id_code 载荷为固定 k=5，body 只含 account_id', () {
    const envelope = QrEnvelope<AccountIdCodeBody>(
      kind: QrKind.accountIdCode,
      id: null,
      issuedAt: null,
      expiresAt: null,
      body: AccountIdCodeBody(accountId: accountId),
    );
    final raw = envelope.toRawJson();
    final parsed = QrEnvelope.parse(raw);
    final body = parsed.body as AccountIdCodeBody;

    expect(parsed.kind, QrKind.accountIdCode);
    expect(raw.contains('"k":${QrKind.accountIdCode.code}'), isTrue);
    expect(body.accountId, accountId);
    // 固定码：顶层不得出现 i/e。
    expect(parsed.id, isNull);
    expect(parsed.expiresAt, isNull);
    expect(raw, isNot(contains('"i"')));
    expect(raw, isNot(contains('"e"')));
    // 不得携带账户标签、昵称、CID 或 SS58。
    expect(raw, isNot(contains(accountLabel)));
    expect(raw, isNot(contains('display_name')));
    expect(raw, isNot(contains('cid_number')));
    expect(raw, isNot(contains('ss58_address')));
  });

  test('account_id_code 拒绝多余字段与非法 account_id', () {
    expect(
      () => QrEnvelope.parse(
        '{"p":"QR_V1","k":5,"b":{"account_id":"$accountId","display_name":"张三"}}',
      ),
      throwsFormatException,
    );
    expect(
      () => QrEnvelope.parse('{"p":"QR_V1","k":5,"b":{"account_id":"0xABC"}}'),
      throwsFormatException,
    );
    // 固定码带时效字段必须拒绝。
    expect(
      () => QrEnvelope.parse(
        '{"p":"QR_V1","k":5,"i":"x","e":1,"b":{"account_id":"$accountId"}}',
      ),
      throwsFormatException,
    );
  });

  test('已废止的 chat_node_pairing 旧 k=5 载荷被 body 字段集拒绝', () {
    expect(
      () => QrEnvelope.parse(
        '{"p":"QR_V1","k":5,"b":{"node_peer_id":"12D3Koo","node_multiaddr":'
        '"/ip4/1.2.3.4/tcp/30333","endpoint_kind":"ip4"}}',
      ),
      throwsFormatException,
    );
  });

  testWidgets('account_id 非法时不进入账户码页', (tester) async {
    await tester.pumpWidget(
      MaterialApp(
        // SnackBar 需要一个已注册的 Scaffold 才会渲染，不能只有裸 Builder。
        home: Scaffold(
          body: Builder(
            builder: (context) => TextButton(
              onPressed: () => openWalletQrPage(
                context,
                accountId: 'not-an-account-id',
                accountLabel: accountLabel,
              ),
              child: const Text('打开'),
            ),
          ),
        ),
      ),
    );

    await tester.tap(find.text('打开'));
    // 不用 pumpAndSettle：它会把时间推进到 SnackBar 自动消失之后。
    await tester.pump();
    await tester.pump(const Duration(milliseconds: 500));

    expect(find.byType(WalletQrPage), findsNothing);
    expect(find.text('账户标识无效，无法生成二维码'), findsOneWidget);
  });
}
