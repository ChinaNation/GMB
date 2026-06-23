import 'dart:convert';

import 'package:flutter_test/flutter_test.dart';
import 'package:citizenapp/qr/bodies/sign_request_body.dart';
import 'package:citizenapp/qr/bodies/sign_response_body.dart';
import 'package:citizenapp/qr/envelope.dart';
import 'package:citizenapp/qr/qr_protocols.dart';
import 'package:citizenapp/signer/qr_signer.dart';

void main() {
  group('QrSigner QR_V1', () {
    const pubkey =
        '0xaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa';
    const payload = '0x01020304';
    final signature = '0x${'bb' * 64}';

    final signer = QrSigner();

    String longId(String prefix) => '$prefix-${List.filled(16, 'a').join()}';

    test('build + parse request should round-trip with compact envelope', () {
      final requestId = longId('req-onchain');
      final request = signer.buildRequest(
        requestId: requestId,
        pubkey: pubkey,
        payloadHex: payload,
        action: QrActions.balancesTransfer,
      );
      final encoded = signer.encodeRequest(request);

      final json = jsonDecode(encoded) as Map<String, dynamic>;
      expect(json['p'], QrProtocols.v1);
      expect(json['k'], QrKind.signRequest.code);
      expect(json['i'], requestId);
      expect(json['e'], isA<int>());
      expect(json['b']['a'], QrActions.balancesTransfer);
      expect(json['b']['g'], 1);
      expect(json['b']['u'], isA<String>());
      expect(json['b']['d'], isA<String>());
      expect(json['body'], isNull);

      final parsed = signer.parseRequest(encoded);
      expect(parsed.kind, QrKind.signRequest);
      expect(parsed.id, requestId);
      expect(parsed.body.action, QrActions.balancesTransfer);
      expect(parsed.body.pubkeyHex, pubkey);
      expect(parsed.body.payloadHex, payload);
    });

    test('parseRequest should reject missing action', () {
      final reqId = longId('req');
      final now = DateTime.now().millisecondsSinceEpoch ~/ 1000;
      final body = SignRequestBody.fromHex(
        action: QrActions.login,
        pubkeyHex: pubkey,
        payloadHex: payload,
      ).toJson()
        ..remove('a');
      final raw = jsonEncode({
        'p': QrProtocols.v1,
        'k': QrKind.signRequest.code,
        'i': reqId,
        'e': now + 90,
        'b': body,
      });

      expect(
        () => signer.parseRequest(raw),
        throwsA(isA<QrSignException>()),
      );
    });

    test('parseRequest should reject expired request', () {
      final now = DateTime.now().millisecondsSinceEpoch ~/ 1000;
      final requestId = longId('req-expired');
      final expired = signer.buildRequest(
        requestId: requestId,
        pubkey: pubkey,
        payloadHex: payload,
        action: QrActions.login,
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

    test('parseResponse should round-trip without payload hash in QR', () {
      final now = DateTime.now().millisecondsSinceEpoch ~/ 1000;
      final requestId = longId('req-match');
      final responseEnv = QrEnvelope<SignResponseBody>(
        kind: QrKind.signResponse,
        id: requestId,
        issuedAt: null,
        expiresAt: now + 90,
        body: SignResponseBody.fromHex(
          pubkeyHex: pubkey,
          signatureHex: signature,
        ),
      );

      final encoded = responseEnv.toRawJson();
      final parsed = signer.parseResponse(
        encoded,
        expectedRequestId: requestId,
        expectedPubkey: pubkey,
      );
      expect(parsed.id, requestId);
      expect(parsed.body.pubkeyHex, pubkey);
      expect(parsed.body.signatureHex, signature);
      final json = jsonDecode(encoded) as Map<String, dynamic>;
      expect(json['b'].containsKey('payload_hash'), isFalse);
    });

    test('parseResponse should reject mismatched request id', () {
      final now = DateTime.now().millisecondsSinceEpoch ~/ 1000;
      final requestId = longId('req-other');
      final expectedId = longId('req-expected');
      final responseEnv = QrEnvelope<SignResponseBody>(
        kind: QrKind.signResponse,
        id: requestId,
        issuedAt: null,
        expiresAt: now + 90,
        body: SignResponseBody.fromHex(
          pubkeyHex: pubkey,
          signatureHex: signature,
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

    test('parseResponse should reject mismatched local payload hash', () {
      final now = DateTime.now().millisecondsSinceEpoch ~/ 1000;
      final requestId = longId('req-hash');
      final responseEnv = QrEnvelope<SignResponseBody>(
        kind: QrKind.signResponse,
        id: requestId,
        issuedAt: null,
        expiresAt: now + 90,
        body: SignResponseBody.fromHex(
          pubkeyHex: pubkey,
          signatureHex: signature,
        ),
      );

      final encoded = responseEnv.toRawJson();
      expect(
        () => signer.parseResponse(
          encoded,
          expectedRequestId: requestId,
          expectedPayloadHash: QrSigner.computePayloadHash('0xbeef'),
          expectedPayloadHex: payload,
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
      expect(h1.length, 66);
    });
  });
}
