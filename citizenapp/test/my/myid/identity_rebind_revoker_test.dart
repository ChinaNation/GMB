import 'dart:typed_data';

import 'package:flutter_test/flutter_test.dart';

import 'package:citizenapp/8964/services/square_api_client.dart';
import 'package:citizenapp/my/myid/identity_rebind_revoker.dart';
import 'package:citizenapp/wallet/core/device_subkey.dart';
import 'package:citizenapp/wallet/core/wallet_manager.dart';

const _oldAccountId =
    '0x1111111111111111111111111111111111111111111111111111111111111111';

WalletProfile _wallet(int walletIndex) => WalletProfile(
      walletIndex: walletIndex,
      walletName: '默认钱包',
      walletIcon: '',
      balance: 0,
      ss58Address: 'ss58',
      accountId: _oldAccountId,
      alg: 'sr25519',
      ss58: 2027,
      createdAtMillis: 1,
      source: 'test',
      signMode: 'local',
    );

class _FakeApi extends SquareApiClient {
  _FakeApi() : super(baseUrl: 'https://revoke.test');
  final List<String> sessionsFor = <String>[];
  final List<String> revokedSessions = <String>[];

  @override
  Future<SquareSession> ensureSession({
    required String accountId,
    required SquareLoginSigner signLoginPayload,
    Future<void> Function()? onDeviceNotRegistered,
  }) async {
    sessionsFor.add(accountId);
    // 触发一次登录签名,验证走设备子钥静默签名。
    await signLoginPayload(Uint8List.fromList(const [1, 2, 3]));
    return SquareSession(
      sessionToken: 'tok-$accountId',
      cidNumber: "CN220-CTZN2-198805200-2026",
      accountId: accountId,
      expiresAt: DateTime.now().millisecondsSinceEpoch + 60000,
    );
  }

  @override
  Future<void> revokeRebindOldAccount({required SquareSession session}) async {
    revokedSessions.add(session.accountId);
  }
}

class _FakeDeviceSubkey extends DeviceSubkey {
  final List<int> signedWalletIndexes = <int>[];
  @override
  Future<String> signRawHex(int walletIndex, Uint8List payload) async {
    signedWalletIndexes.add(walletIndex);
    return 'abcd';
  }
}

class _WalletManagerWith extends WalletManager {
  _WalletManagerWith(this._profile);
  final WalletProfile? _profile;
  @override
  Future<WalletProfile?> getDefaultWallet() async => _profile;
}

void main() {
  test('用旧账户设备子钥静默建旧会话并调吊销端点', () async {
    final api = _FakeApi();
    final subkey = _FakeDeviceSubkey();
    final revoker = IdentityRebindRevoker(
      apiClient: api,
      deviceSubkey: subkey,
      walletManager: _WalletManagerWith(_wallet(7)),
    );

    await revoker.revokeOldAccount(_oldAccountId);

    expect(api.sessionsFor, [_oldAccountId]); // 为旧账户建会话
    expect(subkey.signedWalletIndexes, [7]); // 用该钱包设备子钥签名(静默)
    expect(api.revokedSessions, [_oldAccountId]); // 调吊销端点
  });

  test('无热钱包时静默跳过', () async {
    final api = _FakeApi();
    final revoker = IdentityRebindRevoker(
      apiClient: api,
      deviceSubkey: _FakeDeviceSubkey(),
      walletManager: _WalletManagerWith(null),
    );

    await revoker.revokeOldAccount(_oldAccountId);

    expect(api.sessionsFor, isEmpty);
    expect(api.revokedSessions, isEmpty);
  });
}
