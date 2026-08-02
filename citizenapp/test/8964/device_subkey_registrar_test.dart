import 'dart:convert';
import 'dart:typed_data';

import 'package:flutter_test/flutter_test.dart';
import 'package:http/http.dart' as http;
import 'package:http/testing.dart';

import 'package:citizenapp/8964/services/device_subkey_registrar.dart';
import 'package:citizenapp/8964/services/square_api_client.dart';
import 'package:citizenapp/8964/profile/services/square_session_provider.dart';
import 'package:citizenapp/my/myid/identity_account_cache.dart';
import 'package:citizenapp/security/local_data_key.dart';
import 'package:citizenapp/wallet/core/device_subkey.dart';
import 'package:citizenapp/wallet/core/wallet_manager.dart';

/// 只覆写 publicKeyHex（返回**裸**公钥），其余走原生桥（本测试不触发）。
class _FakeDeviceSubkey extends DeviceSubkey {
  _FakeDeviceSubkey(this._pub);
  final String _pub;
  @override
  Future<String> publicKeyHex(int walletIndex) async => _pub;

  @override
  Future<String> signRawHex(int walletIndex, Uint8List payload) async =>
      'aa' * 64;
}

const _accountId =
    '0x1111111111111111111111111111111111111111111111111111111111111111';
const _binding = AccountDataBinding(
  genesisHash:
      '0xabababababababababababababababababababababababababababababababab',
  cidNumber: 'CN220-CTZN2-198805200-2026',
  bindingRevision: 1,
  accountId: _accountId,
);

class _SessionApi extends SquareApiClient {
  _SessionApi({required this.deviceMissing});

  final bool deviceMissing;

  @override
  Future<SquareSession> ensureSession({
    required String accountId,
    required SquareLoginSigner signLoginPayload,
    Future<void> Function()? onDeviceNotRegistered,
  }) async {
    await signLoginPayload(Uint8List(32));
    if (deviceMissing) await onDeviceNotRegistered!();
    return SquareSession(
      sessionToken: 'session',
      cidNumber: _binding.cidNumber,
      bindingRevision: _binding.bindingRevision,
      accountId: accountId,
      expiresAt: DateTime.now().millisecondsSinceEpoch + 60000,
      signRequest: signLoginPayload,
    );
  }
}

class _SessionWalletManager extends WalletManager {
  int registrationCalls = 0;

  @override
  Future<WalletProfile?> getDefaultWallet() async => const WalletProfile(
        walletIndex: 7,
        walletName: '测试钱包',
        walletIcon: '',
        balance: 0,
        accountId: _accountId,
        ss58Address: 'ss58',
        alg: 'sr25519',
        ss58: 2027,
        createdAtMillis: 1,
        source: 'test',
        signMode: 'local',
      );

  @override
  Future<int> walletIndexForAccountId(String accountId) async => 7;

  @override
  Future<AccountDataBinding> accountDataBindingForAccountId(
    String accountId,
  ) async =>
      _binding;

  @override
  Future<void> activateAccountDataBinding({
    required String genesisHash,
    required String cidNumber,
    required int bindingRevision,
    required String accountId,
  }) async {}

  @override
  Future<void> registerDeviceSubkeyForBinding(
    AccountDataBinding binding,
  ) async {
    registrationCalls++;
  }
}

class _SessionIdentityCache extends IdentityAccountCache {
  @override
  Future<String?> accountId({bool allowChainRead = true}) async => _accountId;
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
        if (request.url.path == '/square/auth/device/register') {
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

  test('广场已有子钥直接静默登录；Worker 确认缺钥时才登记一次', () async {
    final existingWallet = _SessionWalletManager();
    final existing = SquareSessionProvider(
      client: _SessionApi(deviceMissing: false),
      walletManager: existingWallet,
      deviceSubkey: _FakeDeviceSubkey('04${'a' * 128}'),
      identityAccountCache: _SessionIdentityCache(),
    );
    expect(await existing.ensureSession(), isNotNull);
    expect(existingWallet.registrationCalls, 0);

    final missingWallet = _SessionWalletManager();
    final missing = SquareSessionProvider(
      client: _SessionApi(deviceMissing: true),
      walletManager: missingWallet,
      deviceSubkey: _FakeDeviceSubkey('04${'b' * 128}'),
      identityAccountCache: _SessionIdentityCache(),
    );
    expect(await missing.ensureSession(), isNotNull);
    expect(missingWallet.registrationCalls, 1);
  });
}
