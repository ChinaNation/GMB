import 'dart:typed_data';

import 'package:flutter_test/flutter_test.dart';

import 'package:citizenapp/8964/profile/services/citizen_profile_cache.dart';
import 'package:citizenapp/8964/services/square_account_deletion_service.dart';
import 'package:citizenapp/8964/services/square_api_client.dart';
import 'package:citizenapp/im/storage/im_isar_store.dart';
import 'package:citizenapp/wallet/core/device_subkey.dart';

const _owner = '5GrwvaEF5zXb26Fz9rcQpDWS57CtERHpNehXCPcNoHGKutQY';

class _FakeApi extends SquareApiClient {
  _FakeApi({this.fail = false});
  final bool fail;
  bool deleteCalled = false;
  bool sessionCleared = false;
  bool signerInvoked = false;

  @override
  Future<void> deleteAccount({
    required String ownerAccount,
    required SquareActionSigner signAction,
  }) async {
    deleteCalled = true;
    await signAction(Uint8List(32)); // 触发一次签名器，模拟真实往返
    signerInvoked = true;
    if (fail) throw const SquareApiException('服务端删除失败');
  }

  @override
  void clearSession(String ownerAccount) {
    sessionCleared = true;
  }
}

class _FakeCache extends CitizenProfileCache {
  _FakeCache() : super();
  bool cleared = false;
  @override
  Future<void> clear(String ownerAccount) async {
    cleared = true;
  }
}

class _FakeSubkey extends DeviceSubkey {
  bool deleted = false;
  @override
  Future<void> delete(int walletIndex) async {
    deleted = true;
  }
}

class _FakeImStore extends ImIsarStore {
  bool cleared = false;
  @override
  Future<void> clearAllForOwner(String ownerChatAccount) async {
    cleared = true;
  }
}

void main() {
  test('注销成功：服务端删后清齐所有本地数据', () async {
    final api = _FakeApi();
    final cache = _FakeCache();
    final subkey = _FakeSubkey();
    final imStore = _FakeImStore();
    final service = SquareAccountDeletionService(
      apiClient: api,
      profileCache: cache,
      deviceSubkey: subkey,
      imStore: imStore,
    );

    await service.deleteAccount(
      ownerAccount: _owner,
      walletIndex: 3,
      signAction: (_) async => '0xSIG',
    );

    expect(api.deleteCalled, isTrue);
    expect(api.signerInvoked, isTrue);
    expect(cache.cleared, isTrue);
    expect(api.sessionCleared, isTrue);
    expect(imStore.cleared, isTrue);
    expect(subkey.deleted, isTrue);
  });

  test('服务端删除失败：本地一律不动（数据一致）', () async {
    final api = _FakeApi(fail: true);
    final cache = _FakeCache();
    final subkey = _FakeSubkey();
    final imStore = _FakeImStore();
    final service = SquareAccountDeletionService(
      apiClient: api,
      profileCache: cache,
      deviceSubkey: subkey,
      imStore: imStore,
    );

    await expectLater(
      service.deleteAccount(
        ownerAccount: _owner,
        walletIndex: 3,
        signAction: (_) async => '0xSIG',
      ),
      throwsA(isA<SquareApiException>()),
    );

    expect(cache.cleared, isFalse);
    expect(api.sessionCleared, isFalse);
    expect(imStore.cleared, isFalse);
    expect(subkey.deleted, isFalse);
  });
}
