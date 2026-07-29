import 'package:flutter/material.dart';
import 'package:flutter_test/flutter_test.dart';

import 'package:citizenapp/8964/profile/user_qr_page.dart';
import 'package:citizenapp/citizen/shared/account_derivation.dart';
import 'package:citizenapp/qr/bodies/user_contact_body.dart';
import 'package:citizenapp/qr/envelope.dart';
import 'package:citizenapp/qr/qr_protocols.dart';

/// 用户码展示页（`k=3 user_contact`，固定码，唯一入口用户主页）。
///
/// 验证点：
/// - 页面渲染公开昵称、完整 SS58 地址、复制图标、下载图标
/// - 复制点击不抛异常（Clipboard 在 test 环境由 services binding 静默接管）
/// - 下载点击进入保存流程不抛异常（单测环境 SaverGallery 无 native 实现，
///   走 `_saveQr` 的 catch 兜底；不用 pumpAndSettle，保存中的进度圈永不 settle）
/// - k=3 只接受 cid_number + ss58_address + display_name
/// - 本页只出用户码：不存在任何「该出哪种码」的运行时分流（账户维度走钱包码页）
void main() {
  const accountId =
      '0x0000000000000000000000000000000000000000000000000000000000000000';
  const cidNumber = 'CN001-CTZN-000000001-2026';
  const displayName = '晨光寻路者';
  final ss58Address = ss58FromAccountIdText(accountId);

  Future<void> openPage(WidgetTester tester) async {
    await tester.pumpWidget(
      const MaterialApp(
        home: UserQrPage(
          cidNumber: cidNumber,
          displayName: displayName,
          accountId: accountId,
        ),
      ),
    );
    await tester.pump();
    await tester.pump(const Duration(milliseconds: 100));
  }

  testWidgets('页面渲染昵称、完整地址、复制与下载入口', (tester) async {
    await openPage(tester);

    expect(find.text('二维码'), findsOneWidget);
    expect(find.text(displayName), findsWidgets);
    expect(find.text(ss58Address), findsOneWidget);
    expect(find.byIcon(Icons.copy), findsOneWidget);
    expect(find.byIcon(Icons.download), findsOneWidget);
  });

  testWidgets('底部文案如实覆盖加联系人与转账两种扫码场景', (tester) async {
    await openPage(tester);

    expect(find.text('扫描此二维码可加为联系人，或向其转账'), findsOneWidget);
  });

  testWidgets('用户码是固定码，不出现任何时效文案', (tester) async {
    await openPage(tester);

    expect(find.textContaining('分钟内有效'), findsNothing);
  });

  testWidgets('点击复制地址不抛异常', (tester) async {
    await openPage(tester);

    await tester.tap(find.byIcon(Icons.copy));
    await tester.pump();

    expect(tester.takeException(), isNull);
    expect(find.text(ss58Address), findsOneWidget);
  });

  testWidgets('点击下载进入保存流程不抛异常', (tester) async {
    await openPage(tester);

    await tester.tap(find.byIcon(Icons.download));
    await tester.pump();
    for (var i = 0; i < 10; i++) {
      await tester.pump(const Duration(milliseconds: 100));
    }

    expect(tester.takeException(), isNull);
  });

  test('user_contact 载荷为 QR_V1 k=3 且只含身份三字段', () {
    final raw = QrEnvelope<UserContactBody>(
      kind: QrKind.userContact,
      id: null,
      issuedAt: null,
      expiresAt: null,
      body: UserContactBody(
        cidNumber: cidNumber,
        ss58Address: ss58Address,
        displayName: displayName,
      ),
    ).toRawJson();
    final parsed = QrEnvelope.parse(raw);
    final body = parsed.body as UserContactBody;

    expect(raw.contains(QrProtocol.v1), isTrue,
        reason: 'payload should include QR_V1 protocol');
    expect(raw.contains('"k":${QrKind.userContact.code}'), isTrue,
        reason: 'payload should include numeric k=3');
    expect(body.cidNumber, cidNumber);
    expect(body.ss58Address, ss58Address);
    expect(body.displayName, displayName);
    expect(raw, isNot(contains('contact_name')));
  });
}
