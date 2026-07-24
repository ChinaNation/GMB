import 'package:flutter_test/flutter_test.dart';
import 'package:isar_community/isar.dart';

import 'package:citizenapp/8964/profile/models/citizen_profile.dart';
import 'package:citizenapp/8964/profile/services/citizen_profile_api.dart';
import 'package:citizenapp/8964/profile/services/nickname_publisher.dart';
import 'package:citizenapp/8964/profile/services/square_session_provider.dart';
import 'package:citizenapp/8964/services/square_api_client.dart';
import 'package:citizenapp/isar/app_isar.dart';
import 'package:citizenapp/wallet/core/wallet_manager.dart';

import '../support/isar_test_env.dart';

/// 钱包名（昵称）云端同步 —— 模型：**云端为真源，本机为缓存**。
///
/// 覆盖：云端更新回写本机、云端更旧不覆盖、推送失败入队后重放、
/// 待推送期间不得被云端旧值覆盖、冷钱包跳过。
const _accountId =
    '0xaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa';
const _ss58 = 'w5TestAddressForNicknameSync';

CitizenProfile _profile({required String displayName, required int updatedAt}) {
  return CitizenProfile(
    accountId: _accountId,
    displayName: displayName,
    bio: '',
    avatarObjectKey: null,
    bannerObjectKey: null,
    cidNumber: null,
    isCertified: false,
    identityLevel: 'visitor',
    membershipLevel: null,
    membershipActive: false,
    following: 0,
    followers: 0,
    posts: 0,
    isFollowing: false,
    isNotifying: false,
    updatedAt: updatedAt,
  );
}

WalletProfile _wallet({
  String name = '钱包1',
  String signMode = 'local',
}) {
  return WalletProfile(
    walletIndex: 1,
    walletName: name,
    walletIcon: 'wallet',
    balance: 0,
    accountId: _accountId,
    ss58Address: _ss58,
    alg: 'sr25519',
    ss58: 2027,
    createdAtMillis: 0,
    source: 'imported',
    signMode: signMode,
  );
}

class _FakeApi extends CitizenProfileApi {
  _FakeApi({this.remote});

  CitizenProfile? remote;

  /// 非空时 [updateProfile] 抛该错误，用于模拟推送失败。
  Object? pushError;

  final List<String> pushed = <String>[];

  @override
  Future<CitizenProfile> fetchProfile(
    String accountId, {
    SquareSession? session,
  }) async {
    final value = remote;
    if (value == null) throw const SquareApiException('没有资料');
    return value;
  }

  @override
  Future<CitizenProfile> updateProfile({
    required SquareSession session,
    String? displayName,
    String? bio,
    String? avatarObjectKey,
    String? avatarContentHash,
    String? bannerObjectKey,
    String? bannerContentHash,
  }) async {
    final error = pushError;
    if (error != null) throw error;
    pushed.add(displayName ?? '');
    final next = _profile(displayName: displayName ?? '', updatedAt: 999);
    remote = next;
    return next;
  }
}

class _FakeSession extends SquareSessionProvider {
  @override
  Future<SquareSession?> ensureSessionFor(WalletProfile wallet) async {
    if (!wallet.isHotWallet) return null;
    return SquareSession(
      sessionToken: 'token',
      accountId: wallet.accountId,
      expiresAt: DateTime.now().millisecondsSinceEpoch + 600000,
    );
  }
}

Future<void> _seedWalletRow(String name) async {
  await WalletIsar.instance.writeTxn((isar) async {
    final row = WalletProfileEntity()
      ..walletIndex = 1
      ..walletName = name
      ..walletIcon = 'wallet'
      ..balance = 0
      ..accountId = _accountId
      ..ss58Address = _ss58
      ..alg = 'sr25519'
      ..ss58 = 2027
      ..createdAtMillis = 0
      ..source = 'imported'
      ..signMode = 'local'
      ..sortOrder = 0;
    await isar.walletProfileEntitys.put(row);
  });
}

Future<String?> _localName() async {
  final row = await WalletIsar.instance.read(
    (isar) =>
        isar.walletProfileEntitys.filter().walletIndexEqualTo(1).findFirst(),
  );
  return row?.walletName;
}

void main() {
  useIsolatedIsar();

  test('云端有昵称且更新过 → 回写本机（换设备导入后拿回原名）', () async {
    await _seedWalletRow('钱包1');
    final api = _FakeApi(remote: _profile(displayName: '旅行者', updatedAt: 100));
    final publisher =
        NicknamePublisher(api: api, sessionProvider: _FakeSession());

    await publisher.syncWalletName(_wallet());

    expect(await _localName(), '旅行者');
  });

  test('云端版本不比已同步的新 → 不覆盖本机', () async {
    await _seedWalletRow('钱包1');
    final api = _FakeApi(remote: _profile(displayName: '旅行者', updatedAt: 100));
    final publisher =
        NicknamePublisher(api: api, sessionProvider: _FakeSession());
    await publisher.syncWalletName(_wallet());
    expect(await _localName(), '旅行者');

    // 云端回到更旧的版本号：不得把本机改回去。
    api.remote = _profile(displayName: '旧名', updatedAt: 50);
    await publisher.syncWalletName(_wallet(name: '旅行者'));

    expect(await _localName(), '旅行者');
  });

  test('推送失败入待同步队列，下次同步重放成功', () async {
    await _seedWalletRow('旅行者');
    final api = _FakeApi(remote: _profile(displayName: '旧名', updatedAt: 10))
      ..pushError = const SquareApiException('断网');
    final publisher =
        NicknamePublisher(api: api, sessionProvider: _FakeSession());

    await publisher.onLocalRename(_wallet(name: '旅行者'), '旅行者');
    expect(api.pushed, isEmpty, reason: '断网时不应推送成功');

    // 仍有待推送项时，绝不能被云端旧值覆盖本机。
    await publisher.syncWalletName(_wallet(name: '旅行者'));
    expect(await _localName(), '旅行者');

    // 恢复网络后重放。
    api.pushError = null;
    await publisher.syncWalletName(_wallet(name: '旅行者'));
    expect(api.pushed, ['旅行者']);
  });

  test('冷钱包不参与云端同步（无设备子钥、云端无资料）', () async {
    await _seedWalletRow('冷钱包');
    final api = _FakeApi(remote: _profile(displayName: '云端名', updatedAt: 100));
    final publisher =
        NicknamePublisher(api: api, sessionProvider: _FakeSession());

    await publisher.syncWalletName(_wallet(name: '冷钱包', signMode: 'external'));

    expect(await _localName(), '冷钱包');
    expect(api.pushed, isEmpty);
  });
}
