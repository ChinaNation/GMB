import 'dart:convert';

import 'package:flutter_test/flutter_test.dart';
import 'package:http/http.dart' as http;
import 'package:http/testing.dart';

import 'package:citizenapp/8964/services/device_subkey_registrar.dart';
import 'package:citizenapp/8964/services/square_api_client.dart';
import 'package:citizenapp/wallet/core/device_subkey.dart';

/// 只覆写 publicKeyHex（返回**裸**公钥），其余走原生桥（本测试不触发）。
class _FakeDeviceSubkey extends DeviceSubkey {
  _FakeDeviceSubkey(this._pub);
  final String _pub;
  @override
  Future<String> publicKeyHex(int walletIndex) async => _pub;
}

void main() {
  test('注册 wire 的 p256_public_key 带 0x 前缀（ADR-041），公钥本身裸', () async {
    // 65 字节未压缩点裸 hex（04 || 128 hex）。
    final barePub = '04${'a' * 128}';
    const accountId =
        '0x1111111111111111111111111111111111111111111111111111111111111111';
    Map<String, dynamic>? registerBody;

    final api = SquareApiClient(
      baseUrl: 'https://square.test',
      httpClient: MockClient((request) async {
        if (request.url.path == '/v1/square/auth/device/register') {
          registerBody = jsonDecode(request.body) as Map<String, dynamic>;
          return http.Response(jsonEncode({'ok': true}), 200);
        }
        return http.Response('not found', 404);
      }),
    );

    final registrar = DeviceSubkeyRegistrar(
      deviceSubkey: _FakeDeviceSubkey(barePub),
      apiClient: api,
      turnstileToken: () async => null,
    );

    await registrar.register(
      walletIndex: 0,
      cidNumber: 'CN220-CTZN2-198805200-2026',
      bindingRevision: 1,
      accountId: accountId,
      signBinding: (_) async => '0xBINDINGSIG',
      issuedAtMillis: 1700000000000,
    );

    // wire 文本统一带 0x（拒裸）；公钥值本体保持裸，仅前缀。
    expect(registerBody, isNotNull);
    expect(registerBody!['p256_public_key'], '0x$barePub');
    expect(registerBody!['account_id'], accountId);
    expect(registerBody!['binding_signature'], '0xBINDINGSIG');
  });
}
