import 'package:flutter_test/flutter_test.dart';
import 'package:wuminapp_mobile/signer/qr_signer.dart';

void main() {
  group('QrSigner', () {
    const account = '5GrwvaEF5zXb26Fz9rcQpDWS57CtERHpNehXCPcNoHGKutQY';
    const pubkey =
        '0xaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa';
    const payload = '0x01020304';

    final signer = QrSigner();

    test('build + parse request should round-trip', () {
      final request = signer.buildRequest(
        scope: QrSignScope.onchainTx,
        requestId: 'req-onchain-1',
        account: account,
        pubkey: pubkey,
        payloadHex: payload,
      );
      final encoded = signer.encodeRequest(request);
      final parsed = signer.parseRequest(encoded);

      expect(parsed.proto, QrSigner.protocol);
      expect(parsed.scope, QrSignScope.onchainTx);
      expect(parsed.requestId, 'req-onchain-1');
      expect(parsed.account, account);
      expect(parsed.pubkey, pubkey);
      expect(parsed.payloadHex, payload);
    });

    test('parseRequest should reject expired request', () {
      final now = DateTime.now().millisecondsSinceEpoch ~/ 1000;
      final expired = signer.buildRequest(
        scope: QrSignScope.login,
        requestId: 'req-expired',
        account: account,
        pubkey: pubkey,
        payloadHex: payload,
        nowEpochSeconds: now - 200,
      );
      final encoded = signer.encodeRequest(expired);

      expect(
        () => signer.parseRequest(encoded),
        throwsA(
          isA<QrSignException>().having(
            (e) => e.code,
            'code',
            QrSignErrorCode.expired,
          ),
        ),
      );
    });

    test('parseResponse should reject mismatched request id', () {
      final now = DateTime.now().millisecondsSinceEpoch ~/ 1000;
      final response = QrSignResponse(
        proto: QrSigner.protocol,
        requestId: 'req-other',
        pubkey: pubkey,
        sigAlg: 'sr25519',
        signature:
            '0xbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbb',
        signedAt: now,
      );

      final encoded = signer.encodeResponse(response);
      expect(
        () => signer.parseResponse(
          encoded,
          expectedRequestId: 'req-expected',
        ),
        throwsA(
          isA<QrSignException>().having(
            (e) => e.code,
            'code',
            QrSignErrorCode.mismatchedRequest,
          ),
        ),
      );
    });

    test('parseResponse should accept matching request id', () {
      final now = DateTime.now().millisecondsSinceEpoch ~/ 1000;
      final response = QrSignResponse(
        proto: QrSigner.protocol,
        requestId: 'req-match',
        pubkey: pubkey,
        sigAlg: 'sr25519',
        signature:
            '0xbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbb',
        signedAt: now,
      );

      final encoded = signer.encodeResponse(response);
      final parsed = signer.parseResponse(
        encoded,
        expectedRequestId: 'req-match',
      );
      expect(parsed.requestId, 'req-match');
      expect(parsed.pubkey, pubkey);
      // 回执码不再包含 account 字段。
    });
  });
}
