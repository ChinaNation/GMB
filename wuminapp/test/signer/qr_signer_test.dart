import 'package:flutter_test/flutter_test.dart';
import 'package:wuminapp_mobile/signer/qr_signer.dart';

void main() {
  group('QrSigner V2', () {
    const account = '5GrwvaEF5zXb26Fz9rcQpDWS57CtERHpNehXCPcNoHGKutQY';
    const pubkey =
        '0xaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa';
    const payload = '0x01020304';
    const signature =
        '0xbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbb';
    final display = {
      'action': 'transfer',
      'summary': '转账 100.00 GMB',
      'fields': {'to': account, 'amount_yuan': '100.00'},
    };

    final signer = QrSigner();

    test('protocol should be V2', () {
      expect(QrSigner.protocol, 'WUMIN_SIGN_V1.0.0');
    });

    test('build + parse request should round-trip with display', () {
      final request = signer.buildRequest(
        requestId: 'req-onchain-1',
        account: account,
        pubkey: pubkey,
        payloadHex: payload,
        display: display,
      );
      final encoded = signer.encodeRequest(request);
      final parsed = signer.parseRequest(encoded);

      expect(parsed.proto, 'WUMIN_SIGN_V1.0.0');
      expect(parsed.requestId, 'req-onchain-1');
      expect(parsed.account, account);
      expect(parsed.pubkey, pubkey);
      expect(parsed.payloadHex, payload);
      expect(parsed.display['action'], 'transfer');
      expect(parsed.display['summary'], '转账 100.00 GMB');
    });

    test('parseRequest should reject missing display', () {
      // 手动构造缺少 display 的 JSON
      final json =
          '{"proto":"WUMIN_SIGN_V1.0.0","type":"sign_request",'
          '"request_id":"req-1","account":"$account","pubkey":"$pubkey",'
          '"sig_alg":"sr25519","payload_hex":"$payload",'
          '"issued_at":${DateTime.now().millisecondsSinceEpoch ~/ 1000},'
          '"expires_at":${DateTime.now().millisecondsSinceEpoch ~/ 1000 + 90}}';

      expect(
        () => signer.parseRequest(json),
        throwsA(isA<QrSignException>()),
      );
    });

    test('parseRequest should reject display without action', () {
      final request = signer.buildRequest(
        requestId: 'req-no-action',
        account: account,
        pubkey: pubkey,
        payloadHex: payload,
        display: {'summary': '摘要', 'fields': {}},
      );
      // buildRequest 不校验 display，parseRequest 才校验
      final encoded = signer.encodeRequest(request);
      expect(
        () => signer.parseRequest(encoded),
        throwsA(
          isA<QrSignException>().having(
            (e) => e.message,
            'message',
            contains('display.action'),
          ),
        ),
      );
    });

    test('parseRequest should reject expired request', () {
      final now = DateTime.now().millisecondsSinceEpoch ~/ 1000;
      final expired = signer.buildRequest(
        requestId: 'req-expired',
        account: account,
        pubkey: pubkey,
        payloadHex: payload,
        display: {'action': 'login', 'summary': '登录'},
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

    test('parseResponse should round-trip with payloadHash', () {
      final now = DateTime.now().millisecondsSinceEpoch ~/ 1000;
      final payloadHash = QrSigner.computePayloadHash(payload);
      final response = QrSignResponse(
        proto: QrSigner.protocol,
        requestId: 'req-match',
        pubkey: pubkey,
        sigAlg: 'sr25519',
        signature: signature,
        payloadHash: payloadHash,
        signedAt: now,
      );

      final encoded = signer.encodeResponse(response);
      final parsed = signer.parseResponse(
        encoded,
        expectedRequestId: 'req-match',
        expectedPayloadHash: payloadHash,
      );
      expect(parsed.requestId, 'req-match');
      expect(parsed.payloadHash, payloadHash);
    });

    test('parseResponse should reject mismatched request id', () {
      final now = DateTime.now().millisecondsSinceEpoch ~/ 1000;
      final payloadHash = QrSigner.computePayloadHash(payload);
      final response = QrSignResponse(
        proto: QrSigner.protocol,
        requestId: 'req-other',
        pubkey: pubkey,
        sigAlg: 'sr25519',
        signature: signature,
        payloadHash: payloadHash,
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

    test('parseResponse should reject mismatched payloadHash', () {
      final now = DateTime.now().millisecondsSinceEpoch ~/ 1000;
      final response = QrSignResponse(
        proto: QrSigner.protocol,
        requestId: 'req-hash',
        pubkey: pubkey,
        sigAlg: 'sr25519',
        signature: signature,
        payloadHash: QrSigner.computePayloadHash('0xdead'),
        signedAt: now,
      );

      final encoded = signer.encodeResponse(response);
      expect(
        () => signer.parseResponse(
          encoded,
          expectedRequestId: 'req-hash',
          expectedPayloadHash: QrSigner.computePayloadHash('0xbeef'),
        ),
        throwsA(
          isA<QrSignException>().having(
            (e) => e.code,
            'code',
            QrSignErrorCode.mismatchedPayloadHash,
          ),
        ),
      );
    });

    test('computePayloadHash should be deterministic', () {
      final h1 = QrSigner.computePayloadHash('0x01020304');
      final h2 = QrSigner.computePayloadHash('0x01020304');
      expect(h1, h2);
      expect(h1.length, 64); // SHA-256 hex
    });
  });
}
