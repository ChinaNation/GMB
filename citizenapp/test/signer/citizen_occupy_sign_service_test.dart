import 'dart:convert';

import 'package:citizenapp/qr/bodies/sign_request_body.dart';
import 'package:citizenapp/qr/envelope.dart';
import 'package:citizenapp/qr/generated/qr_action_registry.g.dart';
import 'package:citizenapp/qr/qr_protocols.dart';
import 'package:citizenapp/signer/citizen_occupy_sign_service.dart';
import 'package:citizenapp/signer/qr_signer.dart';
import 'package:citizenapp/wallet/core/wallet_manager.dart';
import 'package:flutter_test/flutter_test.dart';

const _cid = 'CN220-CTZN2-198805200-2026';

WalletProfile _wallet({String signMode = 'local'}) => WalletProfile(
      walletIndex: 0,
      walletName: '账户0',
      walletIcon: '',
      balance: 0,
      accountId: '0x${'ab' * 32}',
      ss58Address: 'w5FhTestAddress',
      alg: 'sr25519',
      ss58: 42,
      createdAtMillis: 0,
      source: 'created',
      signMode: signMode,
    );

/// 造占号/换绑域签名 QR(b.u 留空、d=append_bounded(cid))。
String _domainRaw({int? action, List<int>? payload}) {
  final boundedCid = payload ?? <int>[_cid.length << 2, ..._cid.codeUnits];
  final payloadB64 = base64Url.encode(boundedCid).replaceAll('=', '');
  return QrSigner().encodeRequest(QrEnvelope<SignRequestBody>(
    kind: QrKind.signRequest,
    id: 'citizen-occupy-req-000001',
    issuedAt: 1800000000,
    expiresAt: 1900000000,
    body: SignRequestBody(
      action: action ?? QrActions.citizenOccupy,
      signerPublicKey: '', // 占号/换绑 b.u 留空
      payload: payloadB64,
    ),
  ));
}

void main() {
  final service = CitizenOccupySignService();

  test('citizenOccupy/citizenRebind 硬编码常量与 registry 一致(防漂移)', () {
    expect(QrActions.citizenOccupy,
        GeneratedQrActionRegistry.actionCodeByKey['citizen_occupy']);
    expect(QrActions.citizenRebind,
        GeneratedQrActionRegistry.actionCodeByKey['citizen_rebind']);
  });

  test('prepare 从 bounded d 解出 CID(占号)', () async {
    final prep = await service.prepare(_domainRaw(), _wallet());
    expect(prep.cidNumber, _cid);
    expect(prep.isOccupy, isTrue);
    expect(prep.wallet.accountId, '0x${'ab' * 32}');
  });

  test('prepare 换绑动作 isOccupy=false', () async {
    final prep =
        await service.prepare(_domainRaw(action: QrActions.citizenRebind), _wallet());
    expect(prep.isOccupy, isFalse);
    expect(prep.cidNumber, _cid);
  });

  test('非占号/换绑动作即拒', () async {
    final raw = QrSigner().encodeRequest(QrSigner().buildRequest(
      requestId: 'citizen-identity-req-0001',
      signerPublicKey: '0x${'11' * 32}',
      payloadHex: '0x01020304',
      action: QrActions.citizenIdentity,
    ));
    await expectLater(
      service.prepare(raw, _wallet()),
      throwsA(isA<CitizenOccupySignException>()),
    );
  });

  test('冷钱包不能代签', () async {
    await expectLater(
      service.prepare(_domainRaw(), _wallet(signMode: 'external')),
      throwsA(isA<CitizenOccupySignException>()),
    );
  });

  test('d 非合法 bounded cid 即拒(不签寂寞)', () async {
    await expectLater(
      service.prepare(_domainRaw(payload: [0xff, 0xff, 0xff]), _wallet()),
      throwsA(isA<CitizenOccupySignException>()),
    );
  });
}
