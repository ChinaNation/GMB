import 'dart:typed_data';

import 'package:citizenapp/qr/qr_protocols.dart';
import 'package:citizenapp/signer/citizen_identity_sign_service.dart';
import 'package:citizenapp/signer/qr_signer.dart';
import 'package:citizenapp/wallet/core/wallet_manager.dart';
import 'package:flutter_test/flutter_test.dart';

class _FakeWalletManager extends WalletManager {
  @override
  Future<List<WalletProfile>> getWallets() async => const [];
}

String _request({required int action, required List<int> payload}) {
  final signer = QrSigner();
  return signer.encodeRequest(signer.buildRequest(
    requestId: QrSigner.generateRequestId(prefix: 'citizen-'),
    pubkey: '0x${'11' * 32}',
    payloadHex:
        '0x${payload.map((byte) => byte.toRadixString(16).padLeft(2, '0')).join()}',
    action: action,
  ));
}

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
}
