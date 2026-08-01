import 'dart:convert';
import 'dart:typed_data';

import 'package:crypto/crypto.dart' as hashes;
import 'package:cryptography/cryptography.dart';
import 'package:flutter_test/flutter_test.dart';
import 'package:http/http.dart' as http;
import 'package:http/testing.dart';

import 'package:citizenapp/8964/services/square_api_client.dart';
import 'package:citizenapp/signer/signing.dart';
import 'package:citizenapp/wallet/core/device_subkey.dart'
    show bytesToHex, hexToBytes;

const _accountId =
    '0x625e25364c7b68e0a83065ccb40afed43f8fe933e669b24f3d69a57eddb3b715';
const _payloadHex = '73712d616374696f6e';
const _genesisHash =
    '0x1212121212121212121212121212121212121212121212121212121212121212';

/// Worker 登录响应按链上绑定下发身份主键 CID 号；缺失即会话不完整。
const _cidNumber = 'CN220-CTZN2-198805200-2026';

Future<Map<String, Object>> _encryptedGrant(
  Map<String, dynamic> confirmBody,
  Uint8List dataRoot,
) async {
  final recipientPublicKeyHex = confirmBody['recovery_public_key'] as String;
  final challengeId = confirmBody['challenge_id'] as String;
  final x25519 = X25519();
  final senderKeyPair = await x25519.newKeyPair();
  try {
    final senderPublicKey = await senderKeyPair.extractPublicKey();
    final senderPublicKeyHex = '0x${bytesToHex(senderPublicKey.bytes)}';
    final sharedSecret = await x25519.sharedSecretKey(
      keyPair: senderKeyPair,
      remotePublicKey: SimplePublicKey(
        hexToBytes(recipientPublicKeyHex),
        type: KeyPairType.x25519,
      ),
    );
    try {
      final envelopeKey =
          await Hkdf(hmac: Hmac.sha256(), outputLength: 32).deriveKey(
        secretKey: sharedSecret,
        nonce: utf8.encode(
          'citizenapp.cid-data-root/recovery-grant/salt|$_genesisHash|'
          '$_cidNumber|7|$_accountId|$challengeId',
        ),
        info: utf8.encode('citizenapp.cid-data-root/recovery-grant'),
      );
      try {
        final dataRootHash = hashes.sha256.convert(dataRoot).toString();
        final nonce = Uint8List.fromList(List<int>.generate(12, (i) => i + 1));
        final box = await AesGcm.with256bits().encrypt(
          dataRoot,
          secretKey: envelopeKey,
          nonce: nonce,
          aad: utf8.encode(
            'citizenapp.cid-data-root/recovery-grant|$_genesisHash|'
            '$_cidNumber|7|$_accountId|$challengeId|$recipientPublicKeyHex|'
            '$senderPublicKeyHex|$dataRootHash',
          ),
        );
        return <String, Object>{
          'ok': true,
          'chain_genesis_hash': _genesisHash,
          'cid_number': _cidNumber,
          'binding_revision': 7,
          'account_id': _accountId,
          'recovery_recipient_public_key': recipientPublicKeyHex,
          'recovery_sender_public_key': senderPublicKeyHex,
          'recovery_nonce_base64': base64Encode(nonce),
          'encrypted_cid_data_root_base64': base64Encode(<int>[
            ...box.cipherText,
            ...box.mac.bytes,
          ]),
          'data_root_hash': dataRootHash,
        };
      } finally {
        envelopeKey.destroy();
      }
    } finally {
      sharedSecret.destroy();
    }
  } finally {
    senderKeyPair.destroy();
  }
}

void main() {
  test('deleteAccount 钉死 op_tag 0x1D，走 challenge→sign→confirm', () async {
    Uint8List? signedMessage;
    Map<String, dynamic>? confirmBody;

    final client = SquareApiClient(
      baseUrl: 'https://square.test',
      httpClient: MockClient((request) async {
        if (request.url.path == '/v1/square/auth/challenge') {
          expect(jsonDecode(request.body)['account_id'], _accountId);
          return http.Response(
            jsonEncode({
              'ok': true,
              'challenge_id': 'sql_1',
              'cid_number': _cidNumber,
              'binding_revision': 1,
              'signing_payload_hex': _payloadHex,
            }),
            200,
          );
        }
        if (request.url.path == '/v1/square/auth/session') {
          expect(jsonDecode(request.body)['account_id'], _accountId);
          return http.Response(
            jsonEncode({
              'ok': true,
              'session_token': 'sqs_test',
              'cid_number': _cidNumber,
              'binding_revision': 1,
              'expires_at': 4102444800000,
            }),
            200,
          );
        }
        if (request.url.path == '/v1/square/account/delete/challenge') {
          expect(request.headers['authorization'], 'Bearer sqs_test');
          expect(jsonDecode(request.body)['account_id'], _accountId);
          return http.Response(
            jsonEncode({
              'ok': true,
              'challenge_id': 'sqa_1',
              'op_tag': 0x99, // 服务端乱下发 op_tag，客户端必须无视
              'signing_payload_hex': _payloadHex,
              'expires_at': 1800000000000,
            }),
            200,
          );
        }
        if (request.url.path == '/v1/square/account/delete') {
          expect(request.headers['authorization'], 'Bearer sqs_test');
          confirmBody = jsonDecode(request.body) as Map<String, dynamic>;
          return http.Response(jsonEncode({'ok': true}), 200);
        }
        return http.Response('not found', 404);
      }),
    );

    await client.ensureSession(
      accountId: _accountId,
      signLoginPayload: (_) async => '0xLOGIN',
    );
    await client.deleteAccount(
      accountId: _accountId,
      signAction: (message) async {
        signedMessage = message;
        return '0xSIG';
      },
    );

    // 客户端钉死 kOpSignSquareAction(0x1D)，绝不采用服务端下发的 0x99。
    expect(
      bytesToHex(signedMessage!),
      bytesToHex(
        signingMessage(
          opTag: kOpSignSquareAction,
          scalePayload: hexToBytes(_payloadHex),
        ),
      ),
    );
    expect(confirmBody, {
      'account_id': _accountId,
      'challenge_id': 'sqa_1',
      'signature': '0xSIG',
    });
  });

  test('challenge 响应缺 signing_payload_hex → SquareApiException', () async {
    final client = SquareApiClient(
      baseUrl: 'https://square.test',
      httpClient: MockClient(
        (request) async {
          if (request.url.path == '/v1/square/auth/challenge') {
            return http.Response(
              jsonEncode({
                'ok': true,
                'challenge_id': 'sql_2',
                'cid_number': _cidNumber,
                'binding_revision': 1,
                'signing_payload_hex': _payloadHex,
              }),
              200,
            );
          }
          if (request.url.path == '/v1/square/auth/session') {
            return http.Response(
              jsonEncode({
                'ok': true,
                'session_token': 'sqs_test',
                'cid_number': _cidNumber,
                'binding_revision': 1,
                'expires_at': 4102444800000,
              }),
              200,
            );
          }
          return http.Response(
            jsonEncode({'ok': true, 'challenge_id': 'x'}),
            200,
          );
        },
      ),
    );

    await client.ensureSession(
      accountId: _accountId,
      signLoginPayload: (_) async => '0xLOGIN',
    );
    await expectLater(
      client.deleteAccount(
        accountId: _accountId,
        signAction: (_) async => '0x',
      ),
      throwsA(isA<SquareApiException>()),
    );
  });

  test('CID 数据根接管钉死动作域，临时公钥绑定签名并解密 X25519 信封', () async {
    Uint8List? signedMessage;
    Map<String, dynamic>? challengeBody;
    Map<String, dynamic>? confirmBody;
    final expectedRoot =
        Uint8List.fromList(List<int>.generate(32, (index) => 200 - index));
    final client = SquareApiClient(
      baseUrl: 'https://square.test',
      httpClient: MockClient((request) async {
        final body = jsonDecode(request.body) as Map<String, dynamic>;
        if (request.url.path == '/v1/square/identity/takeover/challenge') {
          challengeBody = body;
          return http.Response(
            jsonEncode({
              'ok': true,
              'chain_genesis_hash': _genesisHash,
              'cid_number': _cidNumber,
              'binding_revision': 7,
              'account_id': _accountId,
              'recovery_public_key': body['recovery_public_key'],
              'challenge_id': 'cidt_test',
              'op_tag': 0x99,
              'signing_payload_hex': _payloadHex,
            }),
            200,
          );
        }
        if (request.url.path == '/v1/square/identity/takeover') {
          confirmBody = body;
          return http.Response(
            jsonEncode(await _encryptedGrant(body, expectedRoot)),
            200,
          );
        }
        return http.Response('not found', 404);
      }),
    );

    final grant = await client.takeoverCidDataRoot(
      cidNumber: _cidNumber,
      bindingRevision: 7,
      accountId: _accountId,
      expectedGenesisHash: _genesisHash,
      signAction: (message) async {
        signedMessage = message;
        return '0xNEW';
      },
    );

    expect(
      bytesToHex(signedMessage!),
      bytesToHex(signingMessage(
        opTag: kOpSignSquareAction,
        scalePayload: hexToBytes(_payloadHex),
      )),
    );
    final recoveryPublicKey = challengeBody!['recovery_public_key'];
    expect(recoveryPublicKey, matches(RegExp(r'^0x[0-9a-f]{64}$')));
    expect(confirmBody, {
      'cid_number': _cidNumber,
      'binding_revision': 7,
      'account_id': _accountId,
      'recovery_public_key': recoveryPublicKey,
      'challenge_id': 'cidt_test',
      'signature': '0xNEW',
    });
    expect(grant.dataRoot, expectedRoot);
  });

  test('Worker 创世与本地链不一致时在钱包签名前拒绝', () async {
    var signed = false;
    final client = SquareApiClient(
      baseUrl: 'https://square.test',
      httpClient: MockClient((request) async {
        final body = jsonDecode(request.body) as Map<String, dynamic>;
        return http.Response(
          jsonEncode({
            'ok': true,
            'chain_genesis_hash': '0x${'99' * 32}',
            'cid_number': _cidNumber,
            'binding_revision': 7,
            'account_id': _accountId,
            'recovery_public_key': body['recovery_public_key'],
            'challenge_id': 'cidt_cross_chain',
            'signing_payload_hex': _payloadHex,
          }),
          200,
        );
      }),
    );
    await expectLater(
      client.takeoverCidDataRoot(
        cidNumber: _cidNumber,
        bindingRevision: 7,
        accountId: _accountId,
        expectedGenesisHash: _genesisHash,
        signAction: (_) async {
          signed = true;
          return '0x';
        },
      ),
      throwsA(isA<SquareApiException>()),
    );
    expect(signed, isFalse);
  });
}
