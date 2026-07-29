import 'dart:convert';
import 'dart:typed_data';

import 'package:flutter_test/flutter_test.dart';
import 'package:shared_preferences/shared_preferences.dart';

import 'package:citizenapp/8964/profile/services/square_session_provider.dart';
import 'package:citizenapp/8964/services/square_api_client.dart';
import 'package:citizenapp/my/myid/identity_account_cache.dart';
import 'package:citizenapp/my/myid/identity_account_resolver.dart';
import 'package:citizenapp/my/user/contact_service.dart';
import 'package:citizenapp/my/user/user_service.dart';
import 'package:citizenapp/wallet/core/wallet_manager.dart';

import '../support/isar_test_env.dart';

const _owner = 'w5BekTimvtfYZvFpkDzy7ypqUntPgTbjRFCt9weR8vMgf7o8E';
final _accountId = UserContactService.accountIdFromSs58(_owner);
const _contactA = 'w5Bc7ma8qUcECfQDJmRyQM2wGmga5XSYtz7DvEengQ86xBWrT';

class _FakeWalletManager extends WalletManager {
  @override
  Future<WalletProfile?> getDefaultWallet() async => WalletProfile(
        walletIndex: 1,
        walletName: '默认钱包',
        walletIcon: '',
        balance: 0,
        ss58Address: _owner,
        accountId: _accountId,
        alg: 'sr25519',
        ss58: 2027,
        createdAtMillis: 1,
        source: 'test',
        signMode: 'local',
      );

  @override
  Future<ContactKeyMaterial> ensureContactKeyMaterialForAccountId(
    String accountId,
  ) async =>
      ContactKeyMaterial(
        encryptionKey: Uint8List.fromList(List<int>.filled(32, 7)),
        indexKey: Uint8List.fromList(List<int>.filled(32, 9)),
      );
}

/// 身份账户缓存 fake:恒返回测试账户(身份=账户0 常态),供 contact_service 的
/// `_requireIdentityAccountId` 与 `session.accountId` 校验一致。
class _FakeIdentityCache extends IdentityAccountCache {
  @override
  Future<ResolvedIdentity?> resolve({bool allowChainRead = true}) async =>
      ResolvedIdentity(
        accountId: _accountId,
        ss58Address: _owner,
        accountIndex: 0,
        snapshot: null,
      );
  @override
  Future<String?> accountId({bool allowChainRead = true}) async => _accountId;
}

class _FakeSessionProvider extends SquareSessionProvider {
  @override
  Future<SquareSession?> ensureSession() async => SquareSession(
        sessionToken: 'token',
        cidNumber: "CN220-CTZN2-198805200-2026",
        accountId: _accountId,
        expiresAt: DateTime.now().millisecondsSinceEpoch + 60000,
      );
}

class _FakeApi extends SquareApiClient {
  _FakeApi() : super(baseUrl: 'https://contacts.test');

  final Map<String, SquareEncryptedContact> cloud = {};

  @override
  Future<({List<SquareEncryptedContact> items, String? nextCursor})>
      fetchEncryptedContacts({
    required SquareSession session,
    String? cursor,
    int limit = 100,
  }) async =>
          (items: cloud.values.toList(), nextCursor: null);

  @override
  Future<void> putEncryptedContact({
    required SquareSession session,
    required SquareEncryptedContact contact,
  }) async {
    cloud[contact.contactId] = contact;
  }

  @override
  Future<void> deleteEncryptedContact({
    required SquareSession session,
    required String contactId,
  }) async {
    cloud.remove(contactId);
  }
}

/// 可配身份账户的假缓存/会话(换绑迁移测试需要 old 与 new 两个身份账户)。
class _AccountCache extends IdentityAccountCache {
  _AccountCache(this._id);
  final String _id;
  @override
  Future<ResolvedIdentity?> resolve({bool allowChainRead = true}) async =>
      ResolvedIdentity(
        accountId: _id,
        ss58Address: _owner,
        accountIndex: 0,
        snapshot: null,
      );
  @override
  Future<String?> accountId({bool allowChainRead = true}) async => _id;
}

class _AccountSession extends SquareSessionProvider {
  _AccountSession(this._id);
  final String _id;
  @override
  Future<SquareSession?> ensureSession() async => SquareSession(
        sessionToken: 'token',
        cidNumber: "CN220-CTZN2-198805200-2026",
        accountId: _id,
        expiresAt: DateTime.now().millisecondsSinceEpoch + 60000,
      );
}

/// 记录旧账户通讯录密钥删除的假钱包(换绑迁移测试),不触碰 secure storage。
class _RecordingWallet extends WalletManager {
  final List<String> deletedContactKeys = <String>[];
  @override
  Future<WalletProfile?> getDefaultWallet() async => WalletProfile(
        walletIndex: 1,
        walletName: '默认钱包',
        walletIcon: '',
        balance: 0,
        ss58Address: _owner,
        accountId: _accountId,
        alg: 'sr25519',
        ss58: 2027,
        createdAtMillis: 1,
        source: 'test',
        signMode: 'local',
      );
  @override
  Future<ContactKeyMaterial> ensureContactKeyMaterialForAccountId(
    String accountId,
  ) async =>
      ContactKeyMaterial(
        encryptionKey: Uint8List.fromList(List<int>.filled(32, 7)),
        indexKey: Uint8List.fromList(List<int>.filled(32, 9)),
      );
  @override
  Future<void> deleteContactKeysForAccountId(String accountId) async {
    deletedContactKeys.add(accountId);
  }
}

void main() {
  TestWidgetsFlutterBinding.ensureInitialized();
  useIsolatedIsar();

  setUp(() {
    SharedPreferences.setMockInitialValues(<String, Object>{});
    IdentityAccountCache.debugInstance = _FakeIdentityCache();
  });

  tearDown(IdentityAccountCache.resetDebugInstance);

  group('UserProfileService', () {
    test('returns empty profile when nothing stored', () async {
      final service = UserProfileService();
      final state = await service.getState();
      expect(state.avatarPath, isNull);
      expect(state.backgroundPath, isNull);
    });

    test('persists avatar and background paths across reads', () async {
      final service = UserProfileService();
      await service.updateAvatarPath('/tmp/avatar.png');
      await service.updateBackgroundPath('/tmp/bg.png');
      final state = await UserProfileService().getState();
      expect(state.avatarPath, '/tmp/avatar.png');
      expect(state.backgroundPath, '/tmp/bg.png');
    });
  });

  group('UserContactService', () {
    UserContactService createService() => UserContactService(
          walletManager: _FakeWalletManager(),
          sessionProvider: _FakeSessionProvider(),
          apiClient: _FakeApi(),
          autoSync: false,
        );

    test('字段收口后支持添加与修改名称', () async {
      final service = createService();
      final created = await service.addContact(
        ss58Address: _contactA,
        contactName: '轻节点A',
      );
      expect(created.created, isTrue);
      expect(created.contact.contactName, '轻节点A');

      final renamed =
          await service.renameContact(created.contact.accountId, '张三');
      expect(renamed.single.contactName, '张三');
      expect(renamed.single.toJson().keys.toSet(), <String>{
        'account_id',
        'ss58_address',
        'contact_name',
        'created_at',
        'updated_at',
      });
    });

    test('拒绝把默认钱包自己加入通讯录', () async {
      final service = createService();
      await expectLater(
        service.addContact(
          ss58Address: _owner,
          contactName: '自己',
        ),
        throwsA(isA<FormatException>()),
      );
    });

    test('AES-GCM 可跨设备解密且篡改 MAC 后失败', () async {
      final keys = await _FakeWalletManager()
          .ensureContactKeyMaterialForAccountId(_accountId);
      final deviceA = ContactCryptor(accountId: _accountId, keys: keys);
      final deviceB = ContactCryptor(accountId: _accountId, keys: keys);
      final contact = UserContact(
        accountId: UserContactService.accountIdFromSs58(_contactA),
        ss58Address: _contactA,
        contactName: '张三',
        createdAt: 1,
        updatedAt: 2,
      );

      final encrypted = await deviceA.encrypt(contact);
      expect((await deviceB.decrypt(encrypted)).contactName, '张三');
      final broken = SquareEncryptedContact(
        contactId: encrypted.contactId,
        ciphertext: encrypted.ciphertext,
        nonce: encrypted.nonce,
        mac: base64UrlEncode(List<int>.filled(16, 0)).replaceAll('=', ''),
        updatedAt: encrypted.updatedAt,
      );
      await expectLater(deviceB.decrypt(broken), throwsFormatException);
    });

    test('同步到云端的记录不含联系人明文', () async {
      final api = _FakeApi();
      final service = UserContactService(
        walletManager: _FakeWalletManager(),
        sessionProvider: _FakeSessionProvider(),
        apiClient: api,
        autoSync: false,
      );
      await service.addContact(ss58Address: _contactA, contactName: '张三');
      await service.sync();

      final envelope = api.cloud.values.single;
      final base64Url = RegExp(r'^[A-Za-z0-9_-]+$');
      expect(envelope.ciphertext, matches(base64Url));
      expect(envelope.nonce, matches(base64Url));
      expect(envelope.mac, matches(base64Url));
      final serialized = jsonEncode({
        'contact_id': envelope.contactId,
        'ciphertext': envelope.ciphertext,
        'nonce': envelope.nonce,
        'mac': envelope.mac,
      });
      expect(serialized, isNot(contains(_contactA)));
      expect(serialized, isNot(contains('张三')));
    });
  });

  group('通讯录换绑迁移', () {
    final newId = '0x${'a1' * 32}';

    test('migrateContactsToNewIdentity 搬本地明文+清旧账户密钥+新账户重加密上云',
        () async {
      final wallet = _RecordingWallet();
      final api = _FakeApi();

      // 1. 旧身份下加一个联系人(autoSync=false,只落本地明文缓存)。
      IdentityAccountCache.debugInstance = _AccountCache(_accountId);
      final oldService = UserContactService(
        walletManager: wallet,
        sessionProvider: _AccountSession(_accountId),
        apiClient: api,
        autoSync: false,
      );
      await oldService.addContact(ss58Address: _contactA, contactName: '轻节点A');

      // 2. 换绑迁移 old→new(新身份 + 新会话)。
      IdentityAccountCache.debugInstance = _AccountCache(newId);
      final newService = UserContactService(
        walletManager: wallet,
        sessionProvider: _AccountSession(newId),
        apiClient: api,
        autoSync: false,
      );
      await newService.migrateContactsToNewIdentity(_accountId, newId);

      // 3a. 新身份读到迁来的联系人(本地明文搬迁成功)。
      final migrated = await newService.getContacts();
      expect(migrated.single.contactName, '轻节点A');
      // 3b. 旧账户通讯录密钥已删(杜绝残留)。
      expect(wallet.deletedContactKeys, contains(_accountId));
      // 3c. 用新账户会话重加密上云(云端非空且密文不含明文)。
      expect(api.cloud, isNotEmpty);
      final serialized = jsonEncode(
        api.cloud.values.map((e) => e.ciphertext).toList(),
      );
      expect(serialized, isNot(contains('轻节点A')));
      // 3d. 旧身份本地缓存已清空。
      IdentityAccountCache.debugInstance = _AccountCache(_accountId);
      expect(await oldService.getContacts(), isEmpty);
    });

    test('迁移到相同账户是空操作', () async {
      final wallet = _RecordingWallet();
      IdentityAccountCache.debugInstance = _AccountCache(_accountId);
      final service = UserContactService(
        walletManager: wallet,
        sessionProvider: _AccountSession(_accountId),
        apiClient: _FakeApi(),
        autoSync: false,
      );
      await service.migrateContactsToNewIdentity(_accountId, _accountId);
      expect(wallet.deletedContactKeys, isEmpty);
    });
  });
}
