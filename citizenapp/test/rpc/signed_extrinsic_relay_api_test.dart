import 'dart:convert';

import 'package:flutter_test/flutter_test.dart';
import 'package:http/http.dart' as http;
import 'package:http/testing.dart';

import 'package:citizenapp/rpc/signed_extrinsic_relay_api.dart';

const _signedExtrinsicHex = '0x01020304';
const _txHash =
    '0x2222222222222222222222222222222222222222222222222222222222222222';

void main() {
  test('SignedExtrinsicRelayApi 提交已签名交易并解析 broadcast 响应', () async {
    final api = SignedExtrinsicRelayApi(
      baseUrl: 'http://127.0.0.1:8787',
      httpClient: MockClient((request) async {
        expect(request.method, 'POST');
        expect(request.url.path, SignedExtrinsicRelayApi.relayPath);
        final body = jsonDecode(request.body) as Map<String, dynamic>;
        expect(body, {'signed_extrinsic_hex': _signedExtrinsicHex});
        return http.Response(jsonEncode(_relayResponse()), 202);
      }),
    );

    final result = await api.relaySignedExtrinsic(
      signedExtrinsicHex: _signedExtrinsicHex,
    );

    expect(result.relayId, 'cer_test');
    expect(result.relayStatus, 'broadcast');
    expect(result.deduplicated, isFalse);
    expect(result.txHash, _txHash);
    expect(result.chainSuccessSource, 'finalized_runtime_storage_or_events');
  });

  test('SignedExtrinsicRelayApi 拒绝本地非法 hex', () {
    expect(
      () => SignedExtrinsicRelayApi.normalizeSignedExtrinsicHex('not-hex'),
      throwsA(isA<SignedExtrinsicRelayApiException>()),
    );
    expect(
      () => SignedExtrinsicRelayApi.normalizeSignedExtrinsicHex('0x1'),
      throwsA(isA<SignedExtrinsicRelayApiException>()),
    );
  });

  test('SignedExtrinsicRelayApi 透传 Worker 错误码但不接收错误 schema', () async {
    final api = SignedExtrinsicRelayApi(
      baseUrl: 'http://127.0.0.1:8787',
      httpClient: MockClient((_) async {
        return http.Response.bytes(
          utf8.encode(jsonEncode({
            'ok': false,
            'error_code': 'chain_extrinsic_relay_disabled',
            'message': '签名交易广播兜底未启用',
          })),
          503,
          headers: {'content-type': 'application/json; charset=utf-8'},
        );
      }),
    );

    await expectLater(
      api.relaySignedExtrinsic(signedExtrinsicHex: _signedExtrinsicHex),
      throwsA(
        isA<SignedExtrinsicRelayApiException>()
            .having((e) => e.statusCode, 'statusCode', 503)
            .having(
              (e) => e.errorCode,
              'errorCode',
              'chain_extrinsic_relay_disabled',
            ),
      ),
    );

    expect(
      () => SignedExtrinsicRelayResult.fromJson({
        ..._relayResponse(),
        'chain_success_source': 'api_broadcast',
      }),
      throwsA(isA<SignedExtrinsicRelayApiException>()),
    );
  });
}

Map<String, dynamic> _relayResponse() => {
      'ok': true,
      'schema': 'citizenapp.chain.extrinsic_relay.v1',
      'relay_id': 'cer_test',
      'relay_status': 'broadcast',
      'deduplicated': false,
      'tx_hash': _txHash,
      'accepted_at': 1800000000000,
      'chain_success_source': 'finalized_runtime_storage_or_events',
    };
