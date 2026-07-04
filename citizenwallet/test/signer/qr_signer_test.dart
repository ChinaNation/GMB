import 'dart:convert';
import 'dart:typed_data';

import 'package:flutter_test/flutter_test.dart';
import 'package:pointycastle/digests/blake2b.dart';
import 'package:polkadart_keyring/polkadart_keyring.dart';
import 'package:citizenwallet/qr/bodies/sign_request_body.dart';
import 'package:citizenwallet/qr/envelope.dart';
import 'package:citizenwallet/qr/qr_protocols.dart';
import 'package:citizenwallet/signer/qr_signer.dart';

void main() {
  late QrSigner signer;
  late String testPubkeyHex;

  setUp(() {
    signer = QrSigner();
    final pair = Keyring.sr25519.fromSeed(Uint8List(32));
    pair.ss58Format = 2027;
    testPubkeyHex =
        '0x${pair.bytes().map((b) => b.toRadixString(16).padLeft(2, '0')).join()}';
  });

  group('QrSigner.generateRequestId', () {
    test('生成 base64url 短 ID', () {
      final id = QrSigner.generateRequestId();
      expect(id.length, 22);
      expect(RegExp(r'^[A-Za-z0-9_-]{22}$').hasMatch(id), isTrue);
    });

    test('带前缀的 ID', () {
      final id = QrSigner.generateRequestId(prefix: 'tx-');
      expect(id.startsWith('tx-'), isTrue);
      expect(id.length, greaterThan(22));
    });

    test('每次生成不同 ID', () {
      final ids = List.generate(10, (_) => QrSigner.generateRequestId());
      expect(ids.toSet().length, 10);
    });
  });

  group('QrSigner.parseRequest', () {
    Map<String, dynamic> validEnvelope() {
      final now = DateTime.now().millisecondsSinceEpoch ~/ 1000;
      return {
        'p': QrProtocols.v1,
        'k': QrKind.signRequest.code,
        'i': 'test-valid-req-id-0001',
        'e': now + 90,
        'b': SignRequestBody.fromHex(
          action: QrActions.transferWithRemark,
          pubkeyHex: testPubkeyHex,
          payloadHex: '0x0102',
        ).toJson(),
      };
    }

    test('序列化和反序列化一致', () {
      final envelope = validEnvelope();
      final encoded = jsonEncode(envelope);
      final parsed = signer.parseRequest(encoded);

      expect(parsed.kind, QrKind.signRequest);
      expect(parsed.id, 'test-valid-req-id-0001');
      expect(parsed.body.action, QrActions.transferWithRemark);
      expect(parsed.body.pubkeyHex, testPubkeyHex);
      expect(parsed.body.payloadHex, '0x0102');
    });

    test('拒绝非 JSON', () {
      expect(
        () => signer.parseRequest('not json'),
        throwsA(isA<QrSignException>().having(
          (e) => e.code,
          'code',
          QrSignErrorCode.invalidFormat,
        )),
      );
    });

    test('拒绝错误协议版本', () {
      final json = validEnvelope()..['p'] = 'WRONG_PROTO';
      expect(
        () => signer.parseRequest(jsonEncode(json)),
        throwsA(isA<QrSignException>()),
      );
    });

    test('拒绝非签名请求 kind', () {
      final json = validEnvelope()..['k'] = QrKind.signResponse.code;
      expect(
        () => signer.parseRequest(jsonEncode(json)),
        throwsA(isA<QrSignException>().having(
          (e) => e.code,
          'code',
          QrSignErrorCode.invalidField,
        )),
      );
    });

    test('拒绝缺少 action', () {
      final json = validEnvelope();
      (json['b'] as Map<String, dynamic>).remove('a');
      expect(
        () => signer.parseRequest(jsonEncode(json)),
        throwsA(isA<QrSignException>()),
      );
    });

    test('拒绝已过期请求', () {
      final now = DateTime.now().millisecondsSinceEpoch ~/ 1000;
      final json = validEnvelope()..['e'] = now - 100;
      expect(
        () => signer.parseRequest(jsonEncode(json)),
        throwsA(isA<QrSignException>().having(
          (e) => e.code,
          'code',
          QrSignErrorCode.expired,
        )),
      );
    });

    test('拒绝空 payload', () {
      final json = validEnvelope();
      (json['b'] as Map<String, dynamic>)['d'] = '';
      expect(
        () => signer.parseRequest(jsonEncode(json)),
        throwsA(isA<QrSignException>()),
      );
    });
  });

  group('QrSigner.buildResponse', () {
    test('构建 compact sign_response envelope', () {
      final now = DateTime.now().millisecondsSinceEpoch ~/ 1000;
      final request = QrEnvelope<SignRequestBody>(
        kind: QrKind.signRequest,
        id: 'resp-test-req-id-0001',
        issuedAt: null,
        expiresAt: now + 90,
        body: SignRequestBody.fromHex(
          action: QrActions.login,
          pubkeyHex: testPubkeyHex,
          payloadHex: '0x01020304',
        ),
      );

      final response = signer.buildResponse(
        request: request,
        signatureHex: '0x${'aa' * 64}',
      );

      expect(response.kind, QrKind.signResponse);
      expect(response.id, request.id);
      expect(response.body.pubkeyHex, testPubkeyHex);
      expect(response.body.signatureHex, '0x${'aa' * 64}');

      final json = jsonDecode(response.toRawJson()) as Map<String, dynamic>;
      expect(json['p'], QrProtocols.v1);
      expect(json['k'], QrKind.signResponse.code);
      expect(json['b']['u'], isA<String>());
      expect(json['b']['s'], isA<String>());
      expect(json['b'].containsKey('payload_hash'), isFalse);
    });
  });

  group('QrSigner.computePayloadHash', () {
    test('相同输入产生相同哈希', () {
      final h1 = QrSigner.computePayloadHash('0x01020304');
      final h2 = QrSigner.computePayloadHash('0x01020304');
      expect(h1, h2);
    });

    test('不同输入产生不同哈希', () {
      final h1 = QrSigner.computePayloadHash('0x01020304');
      final h2 = QrSigner.computePayloadHash('0x05060708');
      expect(h1, isNot(h2));
    });

    test('哈希长度为 0x + 64 字符 hex', () {
      final h = QrSigner.computePayloadHash('0x0102');
      expect(h.startsWith('0x'), isTrue);
      expect(h.length, 66);
    });
  });

  group('QrSigner.signingBytesFor', () {
    test('公民身份确认使用 GMB OP_SIGN_CITIZEN_IDENTITY 哈希域', () {
      final body = SignRequestBody.fromHex(
        action: QrActions.citizenIdentity,
        pubkeyHex: testPubkeyHex,
        payloadHex: '0x01020304',
      );
      final input = Uint8List.fromList([0x47, 0x4d, 0x42, 0x10, 1, 2, 3, 4]);
      final digest = Blake2bDigest(digestSize: 32)
        ..update(input, 0, input.length);
      final expected = Uint8List(32);
      digest.doFinal(expected, 0);

      final actual = QrSigner.signingBytesFor(body);

      expect(actual.toList(), expected.toList());
      expect(actual.toList(), isNot(body.payloadBytes.toList()));
    });

    test('IM 钱包绑定使用 GMB OP_SIGN_IM_WALLET_BINDING 哈希域', () {
      final body = SignRequestBody.fromHex(
        action: QrActions.imWalletBinding,
        pubkeyHex: testPubkeyHex,
        payloadHex: '0x01020304',
      );
      final input = Uint8List.fromList([0x47, 0x4d, 0x42, 0x1a, 1, 2, 3, 4]);
      final digest = Blake2bDigest(digestSize: 32)
        ..update(input, 0, input.length);
      final expected = Uint8List(32);
      digest.doFinal(expected, 0);

      final actual = QrSigner.signingBytesFor(body);

      expect(actual.toList(), expected.toList());
      expect(actual.toList(), isNot(body.payloadBytes.toList()));
    });
  });

  group('QrSigner.verifySr25519Signature', () {
    test('有效签名验证通过', () {
      final pair = Keyring.sr25519.fromSeed(Uint8List(32));
      final message = Uint8List.fromList([1, 2, 3, 4]);
      final signature = pair.sign(message);
      final pubHex =
          '0x${pair.bytes().map((b) => b.toRadixString(16).padLeft(2, '0')).join()}';
      final sigHex =
          '0x${signature.map((b) => b.toRadixString(16).padLeft(2, '0')).join()}';

      expect(
        QrSigner.verifySr25519Signature(
          pubkeyHex: pubHex,
          signatureHex: sigHex,
          message: message,
        ),
        isTrue,
      );
    });

    test('无效签名验证失败', () {
      expect(
        QrSigner.verifySr25519Signature(
          pubkeyHex: '0x${'00' * 32}',
          signatureHex: '0x${'ff' * 64}',
          message: Uint8List.fromList([1, 2, 3, 4]),
        ),
        isFalse,
      );
    });
  });
}
