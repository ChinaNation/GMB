import 'dart:typed_data';

import 'package:flutter_test/flutter_test.dart';
import 'package:shared_preferences/shared_preferences.dart';

import 'package:citizenapp/8964/services/square_api_client.dart';
import 'package:citizenapp/my/myid/identity_rebind_revoker.dart';
import 'package:citizenapp/wallet/core/device_subkey.dart';
import 'package:citizenapp/wallet/core/wallet_manager.dart';

const _oldAccountId =
    '0x1111111111111111111111111111111111111111111111111111111111111111';
const _newAccountId =
    '0x2222222222222222222222222222222222222222222222222222222222222222';
const _otherAccountId =
    '0x3333333333333333333333333333333333333333333333333333333333333333';
const _cidNumber = 'CN220-CTZN2-198805200-2026';
const _oldAccountSignature =
    '0xabababababababababababababababababababababababababababababababab'
    'abababababababababababababababababababababababababababababababab';

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
  final List<
      ({
        String sessionAccountId,
        String oldAccountId,
        String oldAccountSignature
      })> revoked = [];
  bool failRevoke = false;

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
      cidNumber: _cidNumber,
      accountId: accountId,
      expiresAt: DateTime.now().millisecondsSinceEpoch + 60000,
    );
  }

  @override
  Future<void> revokeRebindOldAccount({
    required SquareSession session,
    required String oldAccountId,
    required String oldAccountSignature,
  }) async {
    if (failRevoke) throw StateError('模拟网络失败');
    revoked.add((
      sessionAccountId: session.accountId,
      oldAccountId: oldAccountId,
      oldAccountSignature: oldAccountSignature,
    ));
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
  TestWidgetsFlutterBinding.ensureInitialized();

  setUp(() {
    SharedPreferences.setMockInitialValues(<String, Object>{});
  });

  test('换绑后严格使用新账户会话代吊销旧账户并在成功后清除 outbox', () async {
    final api = _FakeApi();
    final subkey = _FakeDeviceSubkey();
    final revoker = IdentityRebindRevoker(
      apiClient: api,
      deviceSubkey: subkey,
      walletManager: _WalletManagerWith(_wallet(7)),
    );
    await revoker.stagePendingCleanup(
      cidNumber: _cidNumber,
      oldAccountId: _oldAccountId,
      newAccountId: _newAccountId,
      oldAccountSignature: _oldAccountSignature,
    );

    // 链上仍是旧账户时只保留待办，绝不尝试用旧账户建会话。
    expect(
      await revoker.retryPendingCleanup(
        cidNumber: _cidNumber,
        currentAccountId: _oldAccountId,
      ),
      isFalse,
    );
    expect(api.sessionsFor, isEmpty);

    expect(
      await revoker.retryPendingCleanup(
        cidNumber: _cidNumber,
        currentAccountId: _newAccountId,
      ),
      isTrue,
    );

    expect(api.sessionsFor, [_newAccountId]); // 只为新账户建会话
    expect(subkey.signedWalletIndexes, [7]); // 用该钱包设备子钥签名(静默)
    expect(api.revoked.single.sessionAccountId, _newAccountId);
    expect(api.revoked.single.oldAccountId, _oldAccountId);
    expect(api.revoked.single.oldAccountSignature, _oldAccountSignature);
    expect(await revoker.readPendingCleanup(), isNull);
  });

  test('网络失败或无热钱包时不丢待清理授权', () async {
    final api = _FakeApi();
    api.failRevoke = true;
    final revoker = IdentityRebindRevoker(
      apiClient: api,
      deviceSubkey: _FakeDeviceSubkey(),
      walletManager: _WalletManagerWith(_wallet(7)),
    );
    await revoker.stagePendingCleanup(
      cidNumber: _cidNumber,
      oldAccountId: _oldAccountId,
      newAccountId: _newAccountId,
      oldAccountSignature: _oldAccountSignature,
    );

    await expectLater(
      revoker.retryPendingCleanup(
        cidNumber: _cidNumber,
        currentAccountId: _newAccountId,
      ),
      throwsA(isA<StateError>()),
    );
    expect((await revoker.readPendingCleanup())?.oldAccountId, _oldAccountId);

    final noWalletRevoker = IdentityRebindRevoker(
      apiClient: _FakeApi(),
      deviceSubkey: _FakeDeviceSubkey(),
      walletManager: _WalletManagerWith(null),
    );
    await expectLater(
      noWalletRevoker.retryPendingCleanup(
        cidNumber: _cidNumber,
        currentAccountId: _newAccountId,
      ),
      throwsA(isA<StateError>()),
    );
    expect((await noWalletRevoker.readPendingCleanup())?.oldAccountId,
        _oldAccountId);
  });

  test('未完成清理时禁止覆盖成另一笔换绑授权', () async {
    final revoker = IdentityRebindRevoker(
      apiClient: _FakeApi(),
      deviceSubkey: _FakeDeviceSubkey(),
      walletManager: _WalletManagerWith(_wallet(7)),
    );
    await revoker.stagePendingCleanup(
      cidNumber: _cidNumber,
      oldAccountId: _oldAccountId,
      newAccountId: _newAccountId,
      oldAccountSignature: _oldAccountSignature,
    );

    await expectLater(
      revoker.ensureCanStartRebind(
        cidNumber: _cidNumber,
        oldAccountId: _newAccountId,
        newAccountId: _otherAccountId,
      ),
      throwsA(isA<StateError>()),
    );
  });

  test('损坏的 outbox 必须 fail-closed，不能按无待办继续换绑', () async {
    SharedPreferences.setMockInitialValues(<String, Object>{
      'identity_rebind_cleanup_pending': '{"old_account_id":"broken"}',
    });
    final revoker = IdentityRebindRevoker(
      apiClient: _FakeApi(),
      deviceSubkey: _FakeDeviceSubkey(),
      walletManager: _WalletManagerWith(_wallet(7)),
    );

    await expectLater(
      revoker.ensureCanStartRebind(
        cidNumber: _cidNumber,
        oldAccountId: _oldAccountId,
        newAccountId: _newAccountId,
      ),
      throwsA(isA<StateError>()),
    );
  });
}
