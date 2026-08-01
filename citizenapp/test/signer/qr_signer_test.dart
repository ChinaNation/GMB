import 'dart:convert';
import 'dart:typed_data';

import 'package:flutter_test/flutter_test.dart';
import 'package:polkadart/polkadart.dart' show Hasher;
import 'package:citizenapp/qr/bodies/sign_request_body.dart';
import 'package:citizenapp/qr/bodies/sign_response_body.dart';
import 'package:citizenapp/qr/envelope.dart';
import 'package:citizenapp/qr/qr_protocols.dart';
import 'package:citizenapp/signer/qr_signer.dart';
import 'package:citizenapp/signer/signing.dart';

String _hexBytes(List<int> bytes) =>
    bytes.map((byte) => byte.toRadixString(16).padLeft(2, '0')).join();

List<int> _u64Le(int value) =>
    List<int>.generate(8, (index) => (value >> (index * 8)) & 0xff);

List<int> _occupyAuthorizationTemplate(String cid, int expiresAt) => [
      ...List<int>.filled(32, 0x44),
      cid.length << 2,
      ...cid.codeUnits,
      ...List<int>.filled(32, 0),
      ..._u64Le(0),
      ..._u64Le(expiresAt),
    ];

List<int> _rebindAuthorizationTemplate(String cid, int expiresAt) => [
      ...List<int>.filled(32, 0x44),
      cid.length << 2,
      ...cid.codeUnits,
      ...List<int>.filled(32, 0x55),
      ...List<int>.filled(32, 0),
      ..._u64Le(7),
      ..._u64Le(expiresAt),
    ];

void main() {
  group('QrSigner QR_V1', () {
    const signerPublicKey =
        '0xaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa';
    const payload = '0x01020304';
    final signature = '0x${'bb' * 64}';

    final signer = QrSigner();

    String longId(String prefix) => '$prefix-${List.filled(16, 'a').join()}';

    test('build + parse request should round-trip with compact envelope', () {
      final requestId = longId('req-onchain');
      final request = signer.buildRequest(
        requestId: requestId,
        signerPublicKey: signerPublicKey,
        payloadHex: payload,
        action: QrActions.transferWithRemark,
      );
      final encoded = signer.encodeRequest(request);

      final json = jsonDecode(encoded) as Map<String, dynamic>;
      expect(json['p'], QrProtocol.v1);
      expect(json['k'], QrKind.signRequest.code);
      expect(json['i'], requestId);
      expect(json['e'], isA<int>());
      expect(json['b']['a'], QrActions.transferWithRemark);
      expect(json['b']['g'], 1);
      expect(json['b']['u'], isA<String>());
      expect(json['b']['d'], isA<String>());
      expect(json['body'], isNull);

      final parsed = signer.parseRequest(encoded);
      expect(parsed.kind, QrKind.signRequest);
      expect(parsed.id, requestId);
      expect(parsed.body.action, QrActions.transferWithRemark);
      expect(parsed.body.signerPublicKeyHex, signerPublicKey);
      expect(parsed.body.payloadHex, payload);
    });

    test('parseRequest should reject missing action', () {
      final reqId = longId('req');
      final now = DateTime.now().millisecondsSinceEpoch ~/ 1000;
      final body = SignRequestBody.fromHex(
        action: QrActions.login,
        signerPublicKeyHex: signerPublicKey,
        payloadHex: payload,
      ).toJson()
        ..remove('a');
      final raw = jsonEncode({
        'p': QrProtocol.v1,
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
        signerPublicKey: signerPublicKey,
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
          signerPublicKeyHex: signerPublicKey,
          signatureHex: signature,
        ),
      );

      final encoded = responseEnv.toRawJson();
      final parsed = signer.parseResponse(
        encoded,
        expectedRequestId: requestId,
        expectedSignerPublicKey: signerPublicKey,
      );
      expect(parsed.id, requestId);
      expect(parsed.body.signerPublicKeyHex, signerPublicKey);
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
          signerPublicKeyHex: signerPublicKey,
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
          signerPublicKeyHex: signerPublicKey,
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

    test('citizen identity uses GMB 0x10 hash domain, not raw payload', () {
      final actual = QrSigner.signingBytesForHex(
        payloadHex: payload,
        action: QrActions.citizenIdentity,
      );
      final expected = Hasher.blake2b256.hash(
        Uint8List.fromList([0x47, 0x4d, 0x42, 0x10, 1, 2, 3, 4]),
      );
      final viaPrimitive = signingMessage(
        opTag: kOpSignCitizenIdentity,
        scalePayload: const [1, 2, 3, 4],
      );

      expect(actual, expected);
      expect(actual, viaPrimitive);
      expect(actual.toList(), isNot([1, 2, 3, 4]));
    });

    test('square account action uses GMB 0x1D hash domain, not raw payload',
        () {
      final actual = QrSigner.signingBytesForHex(
        payloadHex: payload,
        action: QrActions.squareAccountAction,
      );
      final viaPrimitive = signingMessage(
        opTag: kOpSignSquareAction,
        scalePayload: const [1, 2, 3, 4],
      );

      expect(actual, viaPrimitive);
      expect(actual.toList(), isNot([1, 2, 3, 4]));
    });

    // 冷热逐字节一致：与 CitizenWallet/Runtime 同一完整授权模板。
    test('occupy 在完整 CidOccupyAuthorization 的零槽原位填账户', () {
      const cid = 'CN220-CTZN2-198805200-2026';
      const expiresAt = 1800000000;
      final template = _occupyAuthorizationTemplate(cid, expiresAt);
      final account = List<int>.filled(32, 0xab);
      const accountOffset = 32 + 1 + cid.length;
      final exactAuthorization = List<int>.from(template)
        ..setRange(accountOffset, accountOffset + 32, account);

      final actual = QrSigner.signingBytesForHex(
        payloadHex: '0x${_hexBytes(template)}',
        action: QrActions.citizenOccupy,
        selfAccountId: Uint8List.fromList(account),
      );
      final expected = Hasher.blake2b256.hash(
        Uint8List.fromList([0x47, 0x4d, 0x42, 0x12, ...exactAuthorization]),
      );
      final viaPrimitive = signingMessage(
        opTag: kOpSignCidOccupy,
        scalePayload: exactAuthorization,
      );

      expect(actual, expected);
      expect(actual, viaPrimitive);
    });

    test('admin rebind 在完整 CidRebindAuthorization 新账户零槽原位填入', () {
      const cid = 'CN220-CTZN2-198805200-2026';
      const expiresAt = 1800000000;
      final template = _rebindAuthorizationTemplate(cid, expiresAt);
      final account = List<int>.filled(32, 0xcd);
      const accountOffset = 32 + 1 + cid.length + 32;
      final exactAuthorization = List<int>.from(template)
        ..setRange(accountOffset, accountOffset + 32, account);

      final actual = QrSigner.signingBytesForHex(
        payloadHex: '0x${_hexBytes(template)}',
        action: QrActions.citizenRebind,
        selfAccountId: Uint8List.fromList(account),
      );
      final viaPrimitive = signingMessage(
        opTag: kOpSignCidAdminRebind,
        scalePayload: exactAuthorization,
      );

      expect(actual, viaPrimitive);
    });

    test('完整模板解码展示创世、当前账户、revision 和 expires', () {
      const cid = 'CN220-CTZN2-198805200-2026';
      final occupy = QrSigner.decodeCidAccountAuthorizationTemplate(
        action: QrActions.citizenOccupy,
        payload:
            Uint8List.fromList(_occupyAuthorizationTemplate(cid, 1800000000)),
      );
      expect(occupy?.genesisHash, '0x${'44' * 32}');
      expect(occupy?.cidNumber, cid);
      expect(occupy?.currentAccountId, isNull);
      expect(occupy?.expectedBindingRevision, BigInt.zero);
      expect(occupy?.expiresAt, BigInt.from(1800000000));

      final rebind = QrSigner.decodeCidAccountAuthorizationTemplate(
        action: QrActions.citizenRebind,
        payload:
            Uint8List.fromList(_rebindAuthorizationTemplate(cid, 1800000000)),
      );
      expect(rebind?.genesisHash, '0x${'44' * 32}');
      expect(rebind?.cidNumber, cid);
      expect(rebind?.currentAccountId, '0x${'55' * 32}');
      expect(rebind?.expectedBindingRevision, BigInt.from(7));
      expect(rebind?.expiresAt, BigInt.from(1800000000));
    });

    test('占号缺少本账户时返回空，不盲签', () {
      const cid = 'CN220-CTZN2-198805200-2026';
      final actual = QrSigner.signingBytesForHex(
        payloadHex:
            '0x${_hexBytes(_occupyAuthorizationTemplate(cid, 1800000000))}',
        action: QrActions.citizenOccupy,
      );
      expect(actual.isEmpty, isTrue);
    });

    test('拒绝非零槽、尾字节、错误 revision、旧载荷与换绑同账户', () {
      const cid = 'CN220-CTZN2-198805200-2026';
      final account = Uint8List.fromList(List<int>.filled(32, 0xab));
      final nonZeroSlot = _occupyAuthorizationTemplate(cid, 1800000000);
      nonZeroSlot[32 + 1 + cid.length] = 1;
      final nonZeroRevision = _occupyAuthorizationTemplate(cid, 1800000000);
      const revisionOffset = 32 + 1 + cid.length + 32;
      nonZeroRevision[revisionOffset] = 1;
      final zeroRevisionRebind = _rebindAuthorizationTemplate(cid, 1800000000);
      const rebindRevisionOffset = 32 + 1 + cid.length + 32 + 32;
      zeroRevisionRebind.setRange(
        rebindRevisionOffset,
        rebindRevisionOffset + 8,
        List<int>.filled(8, 0),
      );
      final canonicalOccupy = _occupyAuthorizationTemplate(cid, 1800000000);
      final nonCanonicalCidLength = <int>[
        ...canonicalOccupy.sublist(0, 32),
        0x69,
        0x00,
        ...canonicalOccupy.sublist(33),
      ];

      for (final malformed in <({int action, List<int> payload})>[
        (action: QrActions.citizenOccupy, payload: nonZeroSlot),
        (
          action: QrActions.citizenOccupy,
          payload: [..._occupyAuthorizationTemplate(cid, 1800000000), 0xff],
        ),
        (action: QrActions.citizenOccupy, payload: nonZeroRevision),
        (action: QrActions.citizenOccupy, payload: nonCanonicalCidLength),
        (
          action: QrActions.citizenOccupy,
          payload: canonicalOccupy.sublist(0, canonicalOccupy.length - 1),
        ),
        (action: QrActions.citizenRebind, payload: zeroRevisionRebind),
        (
          action: QrActions.citizenOccupy,
          payload: [cid.length << 2, ...cid.codeUnits],
        ),
      ]) {
        expect(
          QrSigner.signingBytesForHex(
            payloadHex: '0x${_hexBytes(malformed.payload)}',
            action: malformed.action,
            selfAccountId: account,
          ),
          isEmpty,
        );
      }

      expect(
        QrSigner.signingBytesForHex(
          payloadHex:
              '0x${_hexBytes(_rebindAuthorizationTemplate(cid, 1800000000))}',
          action: QrActions.citizenRebind,
          selfAccountId: Uint8List.fromList(List<int>.filled(32, 0x55)),
        ),
        isEmpty,
      );
    });

    test('action registry mirror returns Chinese label or null', () {
      expect(
        QrActions.actionLabelForCode(QrActions.squareAccountAction),
        '广场账户动作签名',
      );
      expect(QrActions.actionKeyForCode(QrActions.login), 'login');
      expect(QrActions.actionLabelForCode(0x7fff), isNull);
      for (final entry in QrActions.actionKeyByCode.entries) {
        expect(
          QrActions.actionLabelForKey(entry.value),
          isNotNull,
          reason: '0x${entry.key.toRadixString(16)} 缺少中文动作名',
        );
      }
    });
  });
}
