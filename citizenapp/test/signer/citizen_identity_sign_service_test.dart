import 'dart:typed_data';

import 'package:citizenapp/qr/qr_protocols.dart';
import 'package:citizenapp/signer/citizen_identity_sign_service.dart';
import 'package:citizenapp/signer/qr_signer.dart';
import 'package:citizenapp/wallet/core/wallet_manager.dart';
import 'package:flutter_test/flutter_test.dart';

class _FakeWalletManager extends WalletManager {
  _FakeWalletManager({this.account});

  final Account? account;
  String? signedAccountId;

  @override
  Future<Account?> getAccountByAccountId(String accountId) async =>
      account?.accountId == accountId ? account : null;

  @override
  Future<Uint8List> signForAccountId(
      String accountId, Uint8List payload) async {
    signedAccountId = accountId;
    return Uint8List(64);
  }
}

String _request({required int action, required List<int> payload}) {
  final signer = QrSigner();
  return signer.encodeRequest(signer.buildRequest(
    requestId: QrSigner.generateRequestId(prefix: 'citizen-'),
    signerPublicKey: '0x${'11' * 32}',
    payloadHex:
        '0x${payload.map((byte) => byte.toRadixString(16).padLeft(2, '0')).join()}',
    action: action,
  ));
}

List<int> _u32Le(int value) => [
      value & 0xff,
      (value >> 8) & 0xff,
      (value >> 16) & 0xff,
      (value >> 24) & 0xff,
    ];

List<int> _scaleText(String value) => [
      value.codeUnits.length << 2,
      ...value.codeUnits,
    ];

Uint8List _validVotingPayload() => Uint8List.fromList([
      ..._scaleText('CN220-CTZN2-198805200-2026'),
      ...List<int>.filled(32, 0x11),
      ..._u32Le(20260728),
      ..._u32Le(20360728),
      0,
      ..._scaleText('CN22'),
      ..._scaleText('CN2201'),
      ..._scaleText('CN220101'),
    ]);

void main() {
  final service = CitizenIdentitySignService();

  test('协议登记的公民动作统一展示公民签名确认', () {
    expect(
      QrActions.actionLabelForCode(QrActions.citizenIdentity),
      '公民签名确认',
    );
  });

  test('非公民签名动作在读取钱包前即拒绝', () async {
    await expectLater(
      service.prepare(
        _request(action: QrActions.login, payload: Uint8List(1)),
        _FakeWalletManager(),
      ),
      throwsA(isA<CitizenIdentitySignException>()),
    );
  });

  test('无法完整解码的公民身份载荷禁止签名', () async {
    await expectLater(
      service.prepare(
        _request(action: QrActions.citizenIdentity, payload: Uint8List(1)),
        _FakeWalletManager(),
      ),
      throwsA(
        isA<CitizenIdentitySignException>().having(
          (error) => error.message,
          'message',
          contains('无法完整中文展示'),
        ),
      ),
    );
  });

  test('卡片指定账户与请求一致时按该 account_id 签名', () async {
    final account = Account(
      masterId: '0x${'22' * 32}',
      accountIndex: 5,
      accountId: '0x${'11' * 32}',
      ss58Address: 'w5CitizenAccount',
      accountName: '账户5',
    );
    final walletManager = _FakeWalletManager(account: account);
    final raw = _request(
      action: QrActions.citizenIdentity,
      payload: _validVotingPayload(),
    );
    final prep = await service.prepare(
      raw,
      walletManager,
      requiredAccount: account,
    );
    await service.sign(prep, walletManager);
    expect(prep.account.accountIndex, 5);
    expect(walletManager.signedAccountId, account.accountId);
  });
}
