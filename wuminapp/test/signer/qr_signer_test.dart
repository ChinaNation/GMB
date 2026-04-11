import 'dart:convert';

import 'package:flutter_test/flutter_test.dart';
import 'package:wuminapp_mobile/qr/qr_protocols.dart';
import 'package:wuminapp_mobile/qr/envelope.dart';
import 'package:wuminapp_mobile/qr/bodies/sign_request_body.dart';
import 'package:wuminapp_mobile/qr/bodies/sign_response_body.dart';
import 'package:wuminapp_mobile/signer/qr_signer.dart';

void main() {
  group('QrSigner WUMIN_QR_V1', () {
    const address = '5GrwvaEF5zXb26Fz9rcQpDWS57CtERHpNehXCPcNoHGKutQY';
    const pubkey =
        '0xaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa';
    const payload = '0x01020304';
    const signature =
        '0xbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbb';
    final display = SignDisplay(
      action: 'transfer',
      summary: '转账 100.00 GMB',
      fields: [
        const SignDisplayField(label: '收款账户', value: '5Grw...'),
        const SignDisplayField(label: '金额', value: '100.00 GMB'),
      ],
    );

    final signer = QrSigner();

    String longId(String prefix) =>
        '$prefix-${List.filled(16, 'a').join()}';

    test('build + parse request should round-trip with envelope', () {
      final requestId = longId('req-onchain');
      final request = signer.buildRequest(
        requestId: requestId,
        address: address,
        pubkey: pubkey,
        payloadHex: payload,
        specVersion: 100,
        display: display,
      );
      final encoded = signer.encodeRequest(request);

      // 验证 JSON 结构为 WUMIN_QR_V1 envelope
      final json = jsonDecode(encoded) as Map<String, dynamic>;
      expect(json['proto'], QrProtocols.v1);
      expect(json['kind'], 'sign_request');
      expect(json['id'], requestId);
      expect(json['body']['address'], address);
      expect(json['body']['pubkey'], pubkey);
      expect(json['body']['display']['action'], 'transfer');

      final parsed = signer.parseRequest(encoded);
      expect(parsed.kind, QrKind.signRequest);
      expect(parsed.id, requestId);
      expect(parsed.body.address, address);
      expect(parsed.body.pubkey, pubkey);
      expect(parsed.body.payloadHex, payload);
      expect(parsed.body.display.action, 'transfer');
    });

    test('parseRequest should reject missing display', () {
      final reqId = longId('req');
      final now = DateTime.now().millisecondsSinceEpoch ~/ 1000;
      // 手动构造缺少 display 的 envelope JSON
      final json = jsonEncode({
        'proto': QrProtocols.v1,
        'kind': 'sign_request',
        'id': reqId,
        'issued_at': now,
        'expires_at': now + 90,
        'body': {
          'address': address,
          'pubkey': pubkey,
          'sig_alg': 'sr25519',
          'payload_hex': payload,
          'spec_version': 100,
        },
      });

      expect(
        () => signer.parseRequest(json),
        throwsA(isA<QrSignException>()),
      );
    });

    test('parseRequest should reject expired request', () {
      final now = DateTime.now().millisecondsSinceEpoch ~/ 1000;
      final requestId = longId('req-expired');
      final expired = signer.buildRequest(
        requestId: requestId,
        address: address,
        pubkey: pubkey,
        payloadHex: payload,
        specVersion: 100,
        display: SignDisplay(action: 'login', summary: '登录'),
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
      final requestId = longId('req-match');
      final payloadHash = QrSigner.computePayloadHash(payload);
      final responseEnv = QrEnvelope<SignResponseBody>(
        kind: QrKind.signResponse,
        id: requestId,
        issuedAt: now,
        expiresAt: now + 90,
        body: SignResponseBody(
          pubkey: pubkey,
          sigAlg: 'sr25519',
          signature: signature,
          payloadHash: payloadHash,
          signedAt: now,
        ),
      );

      final encoded = responseEnv.toRawJson();
      final parsed = signer.parseResponse(
        encoded,
        expectedRequestId: requestId,
        expectedPayloadHash: payloadHash,
      );
      expect(parsed.id, requestId);
      expect(parsed.body.payloadHash, payloadHash);
    });

    test('parseResponse should reject mismatched request id', () {
      final now = DateTime.now().millisecondsSinceEpoch ~/ 1000;
      final requestId = longId('req-other');
      final expectedId = longId('req-expected');
      final payloadHash = QrSigner.computePayloadHash(payload);
      final responseEnv = QrEnvelope<SignResponseBody>(
        kind: QrKind.signResponse,
        id: requestId,
        issuedAt: now,
        expiresAt: now + 90,
        body: SignResponseBody(
          pubkey: pubkey,
          sigAlg: 'sr25519',
          signature: signature,
          payloadHash: payloadHash,
          signedAt: now,
        ),
      );

      final encoded = responseEnv.toRawJson();
      expect(
        () => signer.parseResponse(
          encoded,
          expectedRequestId: expectedId,
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
      final requestId = longId('req-hash');
      final responseEnv = QrEnvelope<SignResponseBody>(
        kind: QrKind.signResponse,
        id: requestId,
        issuedAt: now,
        expiresAt: now + 90,
        body: SignResponseBody(
          pubkey: pubkey,
          sigAlg: 'sr25519',
          signature: signature,
          payloadHash: QrSigner.computePayloadHash('0xdead'),
          signedAt: now,
        ),
      );

      final encoded = responseEnv.toRawJson();
      expect(
        () => signer.parseResponse(
          encoded,
          expectedRequestId: requestId,
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
      expect(h1.startsWith('0x'), true);
      expect(h1.length, 66); // 0x + 64 hex SHA-256
    });
  });
}
