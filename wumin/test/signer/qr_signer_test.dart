import 'dart:convert';
import 'dart:typed_data';

import 'package:flutter_test/flutter_test.dart';
import 'package:polkadart_keyring/polkadart_keyring.dart';
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

  group('QrSigner.buildRequest', () {
    test('构建包含所有必需字段的请求', () {
      final request = signer.buildRequest(
        requestId: 'test-request-id-00001',
        account: testAddress,
        pubkey: testPubkeyHex,
        payloadHex: '0x0102',
        display: const {'action': 'test', 'summary': '测试'},
      );

      expect(request.proto, QrSigner.protocol);
      expect(request.requestId, 'test-request-id-00001');
      expect(request.account, testAddress);
      expect(request.pubkey, testPubkeyHex);
      expect(request.sigAlg, 'sr25519');
      expect(request.payloadHex, '0x0102');
      expect(request.expiresAt, greaterThan(request.issuedAt));
      expect(request.specVersion, isNull);
    });

    test('自定义 TTL', () {
      final request = signer.buildRequest(
        requestId: 'test-request-id-00002',
        account: testAddress,
        pubkey: testPubkeyHex,
        payloadHex: '0x0102',
        display: const {'action': 'test', 'summary': '测试'},
        ttlSeconds: 300,
      );
      expect(request.expiresAt - request.issuedAt, 300);
    });

    test('带 specVersion', () {
      final request = signer.buildRequest(
        requestId: 'test-request-id-00003',
        account: testAddress,
        pubkey: testPubkeyHex,
        payloadHex: '0x0102',
        display: const {'action': 'test', 'summary': '测试'},
        specVersion: 100,
      );
      expect(request.specVersion, 100);
    });
  });

  group('QrSigner.encodeRequest / parseRequest', () {
    test('序列化和反序列化一致', () {
      final original = signer.buildRequest(
        requestId: 'roundtrip-test-id-001',
        account: testAddress,
        pubkey: testPubkeyHex,
        payloadHex: '0x01020304',
        display: const {'action': 'transfer', 'summary': '转账测试'},
        specVersion: 100,
      );

      final encoded = signer.encodeRequest(original);
      final parsed = signer.parseRequest(encoded);

      expect(parsed.proto, original.proto);
      expect(parsed.requestId, original.requestId);
      expect(parsed.account, original.account);
      expect(parsed.pubkey, original.pubkey);
      expect(parsed.sigAlg, original.sigAlg);
      expect(parsed.payloadHex, original.payloadHex);
      expect(parsed.issuedAt, original.issuedAt);
      expect(parsed.expiresAt, original.expiresAt);
      expect(parsed.specVersion, original.specVersion);
      expect(parsed.display['action'], 'transfer');
    });
  });

  group('QrSigner.parseRequest 校验', () {
    /// 构造一个合法 JSON 字符串，方便修改特定字段。
    Map<String, dynamic> _validJson() {
      final now = DateTime.now().millisecondsSinceEpoch ~/ 1000;
      return {
        'proto': QrSigner.protocol,
        'type': 'sign_request',
        'request_id': 'test-valid-req-id-0001',
        'account': testAddress,
        'pubkey': testPubkeyHex,
        'sig_alg': 'sr25519',
        'payload_hex': '0x0102',
        'issued_at': now,
        'expires_at': now + 90,
        'display': {'action': 'test', 'summary': '测试'},
      };
    }

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
      final json = _validJson()..['proto'] = 'WRONG_PROTO';
      expect(
        () => signer.parseRequest(jsonEncode(json)),
        throwsA(isA<QrSignException>().having(
          (e) => e.code,
          'code',
          QrSignErrorCode.invalidProtocol,
        )),
      );
    });

    test('拒绝非签名请求类型', () {
      final json = _validJson()..['type'] = 'sign_response';
      expect(
        () => signer.parseRequest(jsonEncode(json)),
        throwsA(isA<QrSignException>().having(
          (e) => e.code,
          'code',
          QrSignErrorCode.invalidField,
        )),
      );
    });

    test('拒绝缺少 display.action', () {
      final json = _validJson()..['display'] = {'summary': '测试'};
      expect(
        () => signer.parseRequest(jsonEncode(json)),
        throwsA(isA<QrSignException>()),
      );
    });

    test('拒绝缺少 display.summary', () {
      final json = _validJson()..['display'] = {'action': 'test'};
      expect(
        () => signer.parseRequest(jsonEncode(json)),
        throwsA(isA<QrSignException>()),
      );
    });

    test('拒绝已过期请求', () {
      final now = DateTime.now().millisecondsSinceEpoch ~/ 1000;
      final json = _validJson()
        ..['issued_at'] = now - 200
        ..['expires_at'] = now - 100;
      expect(
        () => signer.parseRequest(jsonEncode(json)),
        throwsA(isA<QrSignException>().having(
          (e) => e.code,
          'code',
          QrSignErrorCode.expired,
        )),
      );
    });

    test('拒绝空 payload_hex', () {
      final json = _validJson()..['payload_hex'] = '';
      expect(
        () => signer.parseRequest(jsonEncode(json)),
        throwsA(isA<QrSignException>()),
      );
    });

    test('拒绝非 sr25519 签名算法', () {
      final json = _validJson()..['sig_alg'] = 'ed25519';
      expect(
        () => signer.parseRequest(jsonEncode(json)),
        throwsA(isA<QrSignException>()),
      );
    });
  });

  group('QrSigner.parseResponse', () {
    test('正常解析回执', () {
      final request = signer.buildRequest(
        requestId: 'resp-test-req-id-0001',
        account: testAddress,
        pubkey: testPubkeyHex,
        payloadHex: '0x01020304',
        display: const {'action': 'test', 'summary': '测试'},
      );

      final payloadHash = QrSigner.computePayloadHash(request.payloadHex);

      final responseJson = {
        'proto': QrSigner.protocol,
        'type': 'sign_response',
        'request_id': request.requestId,
        'pubkey': testPubkeyHex,
        'sig_alg': 'sr25519',
        'signature': '0x${'aa' * 64}',
        'payload_hash': payloadHash,
        'signed_at': DateTime.now().millisecondsSinceEpoch ~/ 1000,
      };

      final response = signer.parseResponse(
        jsonEncode(responseJson),
        expectedRequestId: request.requestId,
        expectedPubkey: testPubkeyHex,
        expectedPayloadHash: payloadHash,
      );

      expect(response.requestId, request.requestId);
      expect(response.pubkey, testPubkeyHex);
      expect(response.payloadHash, payloadHash);
    });

    test('拒绝 request_id 不匹配', () {
      final responseJson = {
        'proto': QrSigner.protocol,
        'type': 'sign_response',
        'request_id': 'wrong-request-id-00001',
        'pubkey': testPubkeyHex,
        'sig_alg': 'sr25519',
        'signature': '0x${'aa' * 64}',
        'payload_hash': '0x${'bb' * 32}',
        'signed_at': DateTime.now().millisecondsSinceEpoch ~/ 1000,
      };

      expect(
        () => signer.parseResponse(
          jsonEncode(responseJson),
          expectedRequestId: 'expected-request-id-01',
        ),
        throwsA(isA<QrSignException>().having(
          (e) => e.code,
          'code',
          QrSignErrorCode.mismatchedRequest,
        )),
      );
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

    test('哈希长度为 64 字符 hex', () {
      final h = QrSigner.computePayloadHash('0x0102');
      expect(h.length, 64);
      expect(RegExp(r'^[0-9a-f]{64}$').hasMatch(h), isTrue);
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

    test('空输入返回 false', () {
      expect(
        QrSigner.verifySr25519Signature(
          pubkeyHex: '0x',
          signatureHex: '0x',
          payloadHex: '0x',
        ),
        isFalse,
      );
    });
  });

  group('QrSignRequest.toJson', () {
    test('无 specVersion 时 JSON 不包含该字段', () {
      final request = QrSigner().buildRequest(
        requestId: 'json-test-req-id-00001',
        account: testAddress,
        pubkey: testPubkeyHex,
        payloadHex: '0x0102',
        display: const {'action': 'test', 'summary': '测试'},
      );
      final json = request.toJson();
      expect(json.containsKey('spec_version'), isFalse);
    });

    test('有 specVersion 时 JSON 包含该字段', () {
      final request = QrSigner().buildRequest(
        requestId: 'json-test-req-id-00002',
        account: testAddress,
        pubkey: testPubkeyHex,
        payloadHex: '0x0102',
        display: const {'action': 'test', 'summary': '测试'},
        specVersion: 100,
      );
      final json = request.toJson();
      expect(json['spec_version'], 100);
    });
  });
}
