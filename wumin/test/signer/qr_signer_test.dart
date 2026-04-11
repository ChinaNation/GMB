import 'dart:convert';
import 'dart:typed_data';

import 'package:flutter_test/flutter_test.dart';
import 'package:polkadart_keyring/polkadart_keyring.dart';
import 'package:wumin/qr/qr_protocols.dart';
import 'package:wumin/qr/bodies/sign_request_body.dart';
import 'package:wumin/qr/envelope.dart';
import 'package:wumin/signer/qr_signer.dart';

void main() {
  late QrSigner signer;
  late String testAddress;
  late String testPubkeyHex;

  setUp(() {
    signer = QrSigner();
    final pair = Keyring.sr25519.fromSeed(Uint8List(32));
    pair.ss58Format = 2027;
    testAddress = pair.address;
    testPubkeyHex = '0x${pair.bytes().map((b) => b.toRadixString(16).padLeft(2, '0')).join()}';
  });

  group('QrSigner.generateRequestId', () {
    test('生成 32 字符 hex', () {
      final id = QrSigner.generateRequestId();
      expect(id.length, 32);
      expect(RegExp(r'^[0-9a-f]{32}$').hasMatch(id), isTrue);
    });

    test('带前缀的 ID', () {
      final id = QrSigner.generateRequestId(prefix: 'tx-');
      expect(id.startsWith('tx-'), isTrue);
      expect(id.length, greaterThan(32));
    });

    test('每次生成不同 ID', () {
      final ids = List.generate(10, (_) => QrSigner.generateRequestId());
      expect(ids.toSet().length, 10);
    });
  });

  group('QrSigner.parseRequest (envelope)', () {
    Map<String, dynamic> validEnvelope() {
      final now = DateTime.now().millisecondsSinceEpoch ~/ 1000;
      return {
        'proto': QrProtocols.v1,
        'kind': 'sign_request',
        'id': 'test-valid-req-id-0001',
        'issued_at': now,
        'expires_at': now + 90,
        'body': {
          'address': testAddress,
          'pubkey': testPubkeyHex,
          'sig_alg': 'sr25519',
          'payload_hex': '0x0102',
          'spec_version': 100,
          'display': {'action': 'test', 'summary': '测试', 'fields': []},
        },
      };
    }

    test('序列化和反序列化一致', () {
      final envelope = validEnvelope();
      final encoded = jsonEncode(envelope);
      final parsed = signer.parseRequest(encoded);

      expect(parsed.kind, QrKind.signRequest);
      expect(parsed.id, 'test-valid-req-id-0001');
      expect(parsed.body.address, testAddress);
      expect(parsed.body.pubkey, testPubkeyHex);
      expect(parsed.body.sigAlg, 'sr25519');
      expect(parsed.body.payloadHex, '0x0102');
      expect(parsed.body.specVersion, 100);
      expect(parsed.body.display.action, 'test');
      expect(parsed.body.display.summary, '测试');
    });

    test('拒绝非 JSON', () {
      expect(
        () => signer.parseRequest('not json'),
        throwsA(isA<QrSignException>().having(
          (e) => e.code, 'code', QrSignErrorCode.invalidFormat,
        )),
      );
    });

    test('拒绝错误协议版本', () {
      final json = validEnvelope()..['proto'] = 'WRONG_PROTO';
      expect(
        () => signer.parseRequest(jsonEncode(json)),
        throwsA(isA<QrSignException>()),
      );
    });

    test('拒绝非签名请求 kind', () {
      final json = validEnvelope()..['kind'] = 'sign_response';
      expect(
        () => signer.parseRequest(jsonEncode(json)),
        throwsA(isA<QrSignException>().having(
          (e) => e.code, 'code', QrSignErrorCode.invalidField,
        )),
      );
    });

    test('拒绝缺少 display.action', () {
      final json = validEnvelope();
      (json['body'] as Map<String, dynamic>)['display'] = {'summary': '测试', 'fields': []};
      expect(
        () => signer.parseRequest(jsonEncode(json)),
        throwsA(isA<QrSignException>()),
      );
    });

    test('拒绝缺少 display.summary', () {
      final json = validEnvelope();
      (json['body'] as Map<String, dynamic>)['display'] = {'action': 'test', 'fields': []};
      expect(
        () => signer.parseRequest(jsonEncode(json)),
        throwsA(isA<QrSignException>()),
      );
    });

    test('拒绝已过期请求', () {
      final now = DateTime.now().millisecondsSinceEpoch ~/ 1000;
      final json = validEnvelope()
        ..['issued_at'] = now - 200
        ..['expires_at'] = now - 100;
      expect(
        () => signer.parseRequest(jsonEncode(json)),
        throwsA(isA<QrSignException>().having(
          (e) => e.code, 'code', QrSignErrorCode.expired,
        )),
      );
    });

    test('拒绝空 payload_hex', () {
      final json = validEnvelope();
      (json['body'] as Map<String, dynamic>)['payload_hex'] = '';
      expect(
        () => signer.parseRequest(jsonEncode(json)),
        throwsA(isA<QrSignException>()),
      );
    });

    test('拒绝 TTL 超过 300 秒', () {
      final now = DateTime.now().millisecondsSinceEpoch ~/ 1000;
      final json = validEnvelope()
        ..['issued_at'] = now
        ..['expires_at'] = now + 500;
      expect(
        () => signer.parseRequest(jsonEncode(json)),
        throwsA(isA<QrSignException>()),
      );
    });
  });

  group('QrSigner.buildResponse', () {
    test('构建 sign_response envelope', () {
      final now = DateTime.now().millisecondsSinceEpoch ~/ 1000;
      final request = QrEnvelope<SignRequestBody>(
        kind: QrKind.signRequest,
        id: 'resp-test-req-id-0001',
        issuedAt: now,
        expiresAt: now + 90,
        body: SignRequestBody(
          address: testAddress,
          pubkey: testPubkeyHex,
          sigAlg: 'sr25519',
          payloadHex: '0x01020304',
          specVersion: 100,
          display: const SignDisplay(action: 'test', summary: '测试'),
        ),
      );

      final response = signer.buildResponse(
        request: request,
        signatureHex: '0x${'aa' * 64}',
      );

      expect(response.kind, QrKind.signResponse);
      expect(response.id, request.id);
      expect(response.body.pubkey, testPubkeyHex);
      expect(response.body.sigAlg, 'sr25519');
      expect(response.body.signature, '0x${'aa' * 64}');
      expect(response.body.payloadHash, isNotEmpty);

      // envelope JSON 结构正确
      final json = jsonDecode(response.toRawJson()) as Map<String, dynamic>;
      expect(json['proto'], QrProtocols.v1);
      expect(json['kind'], 'sign_response');
      expect(json['body']['pubkey'], testPubkeyHex);
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

  group('QrSigner.verifySr25519Signature', () {
    test('有效签名验证通过', () {
      final pair = Keyring.sr25519.fromSeed(Uint8List(32));
      final message = Uint8List.fromList([1, 2, 3, 4]);
      final signature = pair.sign(message);
      final pubHex = '0x${pair.bytes().map((b) => b.toRadixString(16).padLeft(2, '0')).join()}';
      final sigHex = '0x${signature.map((b) => b.toRadixString(16).padLeft(2, '0')).join()}';

      expect(
        QrSigner.verifySr25519Signature(
          pubkeyHex: pubHex,
          signatureHex: sigHex,
          payloadHex: '0x01020304',
        ),
        isTrue,
      );
    });

    test('无效签名验证失败', () {
      expect(
        QrSigner.verifySr25519Signature(
          pubkeyHex: '0x${'00' * 32}',
          signatureHex: '0x${'ff' * 64}',
          payloadHex: '0x01020304',
        ),
        isFalse,
      );
    });
  });
}
