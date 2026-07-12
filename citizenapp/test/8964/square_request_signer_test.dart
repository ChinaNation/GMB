import 'dart:typed_data';

import 'package:flutter_test/flutter_test.dart';

import 'package:citizenapp/8964/services/square_request_signer.dart';

void main() {
  test('请求证明剥离同域 API 前缀并生成固定头', () async {
    Uint8List? signedMessage;
    final signature = '0x${List.filled(64, '11').join()}';
    final headers = await squareRequestHeaders(
      method: 'POST',
      uri: Uri.parse('https://www.crcfrcn.com/api/v1/chat/signals'),
      body: '{}',
      sessionToken: 'session-token',
      requestTime: 1700000000000,
      nonce: '00112233445566778899aabbccddeeff',
      sign: (message) async {
        signedMessage = message;
        return signature;
      },
    );

    expect(signedMessage, isNotNull);
    expect(signedMessage, hasLength(32));
    expect(headers, {
      'x-device-time': '1700000000000',
      'x-device-nonce': '00112233445566778899aabbccddeeff',
      'x-device-signature': signature,
    });
  });
}
