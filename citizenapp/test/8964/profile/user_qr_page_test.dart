import 'dart:typed_data';

import 'package:flutter/material.dart';
import 'package:flutter_test/flutter_test.dart';

import 'package:citizenapp/8964/profile/models/citizen_profile.dart';
import 'package:citizenapp/8964/profile/services/citizen_profile_cache.dart';
import 'package:citizenapp/8964/profile/user_qr_page.dart';
import 'package:citizenapp/citizen/shared/account_derivation.dart';
import 'package:citizenapp/my/myid/citizen_identity_chain_reader.dart';
import 'package:citizenapp/my/myid/identity_account_resolver.dart';
import 'package:citizenapp/qr/bodies/user_contact_body.dart';
import 'package:citizenapp/qr/envelope.dart';
import 'package:citizenapp/qr/qr_protocols.dart';

/// 身份用户码与普通账户临时收款码共用展示页。
///
/// 验证点：
/// - 页面渲染公开昵称、完整 SS58 地址、复制图标、下载图标
/// - 复制点击不抛异常（Clipboard 在 test 环境由 services binding 静默接管）
/// - 下载点击进入保存流程不抛异常（单测环境 SaverGallery 无 native 实现，
///   走 `_saveQr` 的 catch 兜底；不用 pumpAndSettle，保存中的进度圈永不 settle）
/// - k=3 只接受 cid_number + ss58_address + display_name
/// - 普通账户展示 k=4 临时收款码语义
void main() {
  const accountId =
      '0x0000000000000000000000000000000000000000000000000000000000000000';
  const cidNumber = 'CN001-CTZN-000000001-2026';
  const displayName = '晨光寻路者';
  final ss58Address = ss58FromAccountIdText(accountId);

  Future<void> openPage(WidgetTester tester) async {
    await tester.pumpWidget(
      const MaterialApp(
        home: UserQrPage.userContact(
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

  testWidgets('普通账户二维码明确显示五分钟临时收款码语义', (tester) async {
    await tester.pumpWidget(
      const MaterialApp(
        home: UserQrPage.userTransfer(
          displayName: '账户0',
          accountId: accountId,
        ),
      ),
    );
    await tester.pump();

    expect(find.text('临时收款码，5 分钟内有效'), findsOneWidget);
    expect(find.text('扫描此二维码可加为联系人，或向其转账'), findsNothing);
  });

  testWidgets('当前账户命中链上 CID 时进入固定 k=3 身份码', (tester) async {
    await tester.pumpWidget(
      MaterialApp(
        home: Builder(
          builder: (context) => TextButton(
            onPressed: () => openAccountQrPage(
              context,
              accountId: accountId,
              paymentDisplayName: '本机钱包标签',
              identityResolver: _FixedIdentityResolver(
                _resolvedIdentity(accountId: accountId, registered: true),
              ),
              profileCache: const _EmptyProfileCache(),
            ),
            child: const Text('打开'),
          ),
        ),
      ),
    );

    await tester.tap(find.text('打开'));
    await tester.pumpAndSettle();

    final page = tester.widget<UserQrPage>(find.byType(UserQrPage));
    expect(page.userContact, isTrue);
    expect(page.cidNumber, cidNumber);
    expect(page.displayName, isNot('本机钱包标签'));
  });

  testWidgets('当前账户没有链上 CID 时进入五分钟 k=4 临时收款码', (tester) async {
    await tester.pumpWidget(
      MaterialApp(
        home: Builder(
          builder: (context) => TextButton(
            onPressed: () => openAccountQrPage(
              context,
              accountId: accountId,
              paymentDisplayName: '账户0',
              identityResolver: _FixedIdentityResolver(
                _resolvedIdentity(accountId: accountId, registered: false),
              ),
              profileCache: const _EmptyProfileCache(),
            ),
            child: const Text('打开'),
          ),
        ),
      ),
    );

    await tester.tap(find.text('打开'));
    await tester.pumpAndSettle();

    final page = tester.widget<UserQrPage>(find.byType(UserQrPage));
    expect(page.userContact, isFalse);
    expect(page.cidNumber, isNull);
    expect(page.displayName, '账户0');
  });
}

ResolvedIdentity _resolvedIdentity({
  required String accountId,
  required bool registered,
}) {
  return ResolvedIdentity(
    accountId: accountId,
    ss58Address: ss58FromAccountIdText(accountId),
    accountIndex: 0,
    snapshot: registered
        ? CitizenIdentityChainSnapshot(
            cidNumber: 'CN001-CTZN-000000001-2026',
            accountId: Uint8List(32),
            votingIdentity: null,
          )
        : null,
  );
}

class _FixedIdentityResolver extends IdentityAccountResolver {
  _FixedIdentityResolver(this.identity);

  final ResolvedIdentity identity;

  @override
  Future<ResolvedIdentity?> resolve() async => identity;
}

class _EmptyProfileCache extends CitizenProfileCache {
  const _EmptyProfileCache();

  @override
  Future<CitizenProfile?> read(String cidNumber) async => null;
}
