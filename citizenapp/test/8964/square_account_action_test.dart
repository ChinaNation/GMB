import 'dart:convert';
import 'dart:typed_data';

import 'package:flutter_test/flutter_test.dart';
import 'package:http/http.dart' as http;
import 'package:http/testing.dart';

import 'package:citizenapp/8964/services/square_api_client.dart';
import 'package:citizenapp/signer/signing.dart';
import 'package:citizenapp/wallet/core/device_subkey.dart' show bytesToHex, hexToBytes;

const _owner = '5GrwvaEF5zXb26Fz9rcQpDWS57CtERHpNehXCPcNoHGKutQY';
const _payloadHex = '73712d616374696f6e';

void main() {
  test('deleteAccount 钉死 op_tag 0x1D，走 challenge→sign→confirm', () async {
    Uint8List? signedMessage;
    Map<String, dynamic>? confirmBody;

    final client = SquareApiClient(
      baseUrl: 'https://square.test',
      httpClient: MockClient((request) async {
        if (request.url.path == '/v1/square/account/delete/challenge') {
          expect(jsonDecode(request.body)['owner_account'], _owner);
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
          confirmBody = jsonDecode(request.body) as Map<String, dynamic>;
          return http.Response(jsonEncode({'ok': true}), 200);
        }
        return http.Response('not found', 404);
      }),
    );

    await client.deleteAccount(
      ownerAccount: _owner,
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
      'owner_account': _owner,
      'challenge_id': 'sqa_1',
      'signature': '0xSIG',
    });
  });

  test('challenge 响应缺 signing_payload_hex → SquareApiException', () async {
    final client = SquareApiClient(
      baseUrl: 'https://square.test',
      httpClient: MockClient(
        (request) async =>
            http.Response(jsonEncode({'ok': true, 'challenge_id': 'x'}), 200),
      ),
    );

    await expectLater(
      client.deleteAccount(ownerAccount: _owner, signAction: (_) async => '0x'),
      throwsA(isA<SquareApiException>()),
    );
  });
}
