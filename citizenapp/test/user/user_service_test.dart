import 'dart:convert';
import 'dart:typed_data';

import 'package:flutter_test/flutter_test.dart';
import 'package:shared_preferences/shared_preferences.dart';

import 'package:citizenapp/8964/profile/services/square_session_provider.dart';
import 'package:citizenapp/8964/services/square_api_client.dart';
import 'package:citizenapp/chat/identity/peer_cid_resolver.dart';
import 'package:citizenapp/isar/app_isar.dart';
import 'package:citizenapp/my/myid/identity_account_cache.dart';
import 'package:citizenapp/my/myid/identity_account_resolver.dart';
import 'package:citizenapp/my/user/contact_service.dart';
import 'package:citizenapp/my/user/user_service.dart';
import 'package:citizenapp/qr/bodies/user_contact_body.dart';
import 'package:citizenapp/qr/pages/qr_scan_page.dart';
import 'package:citizenapp/security/local_cipher.dart';
import 'package:isar_community/isar.dart';
import 'package:citizenapp/security/local_data_key.dart';
import 'package:citizenapp/wallet/core/wallet_manager.dart';

import '../support/isar_test_env.dart';

const _owner = 'w5BekTimvtfYZvFpkDzy7ypqUntPgTbjRFCt9weR8vMgf7o8E';
final _accountId = UserContactService.accountIdFromSs58(_owner);
const _contactA = 'w5Bc7ma8qUcECfQDJmRyQM2wGmga5XSYtz7DvEengQ86xBWrT';
const _ownerCidNumber = 'CN220-CTZN2-198805200-2026';
const _contactCidNumber = 'CN220-CTZN2-100000001-2026';

class _FakeWalletManager extends WalletManager {
  /// 本地静止态数据密钥:固定值,避免测试触碰硬件金库/平台通道。
  /// 只换密钥来源,**不绕过加密**——通讯录本地 KV 仍走真实 AES-256-GCM。
  @override
  Future<LocalDataKey> ensureLocalDataKeyForAccountId(String accountId) async =>
      LocalDataKey(Uint8List.fromList(List<int>.generate(32, (i) => i + 1)));

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
        cidNumber: _ownerCidNumber,
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
        cidNumber: _ownerCidNumber,
        accountId: _id,
        expiresAt: DateTime.now().millisecondsSinceEpoch + 60000,
      );
}

/// 记录旧账户通讯录密钥删除的假钱包(换绑迁移测试),不触碰 secure storage。
class _RecordingWallet extends WalletManager {
  /// 本地静止态数据密钥:固定值,避免测试触碰硬件金库/平台通道。
  /// 只换密钥来源,**不绕过加密**。
  @override
  Future<LocalDataKey> ensureLocalDataKeyForAccountId(String accountId) async =>
      LocalDataKey(Uint8List.fromList(List<int>.generate(32, (i) => i + 1)));

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

class _FixedPeerCidResolver extends PeerCidResolver {
  _FixedPeerCidResolver(this.cidNumber);

  final String cidNumber;
  String? resolvedAccountId;

  @override
  Future<String> resolve(String accountId) async {
    resolvedAccountId = accountId;
    return cidNumber;
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

    test('CID 真源字段支持添加与修改空值合法的私人备注', () async {
      final service = createService();
      final created = await service.addContact(
        cidNumber: _contactCidNumber,
        ss58Address: _contactA,
        contactRemark: '',
      );
      expect(created.created, isTrue);
      expect(created.contact.cidNumber, _contactCidNumber);
      expect(created.contact.contactRemark, isEmpty);

      final renamed =
          await service.renameContact(created.contact.cidNumber, '张三');
      expect(renamed.single.contactRemark, '张三');
      expect(renamed.single.toJson().keys.toSet(), <String>{
        'cid_number',
        'account_id',
        'ss58_address',
        'contact_remark',
        'created_at',
        'updated_at',
      });

      final cleared =
          await service.renameContact(created.contact.cidNumber, '');
      expect(cleared.single.contactRemark, isEmpty);
    });

    test('拒绝把默认钱包自己加入通讯录', () async {
      final service = createService();
      await expectLater(
        service.addContact(
          cidNumber: _ownerCidNumber,
          ss58Address: _owner,
          contactRemark: '',
        ),
        throwsA(isA<FormatException>()),
      );
    });

    test('AES-GCM 可跨设备解密且篡改 MAC 后失败', () async {
      final keys = await _FakeWalletManager()
          .ensureContactKeyMaterialForAccountId(_accountId);
      final deviceA =
          ContactCryptor(ownerCidNumber: _ownerCidNumber, keys: keys);
      final deviceB =
          ContactCryptor(ownerCidNumber: _ownerCidNumber, keys: keys);
      final contact = UserContact(
        cidNumber: _contactCidNumber,
        accountId: UserContactService.accountIdFromSs58(_contactA),
        ss58Address: _contactA,
        contactRemark: '张三',
        createdAt: 1,
        updatedAt: 2,
      );

      final encrypted = await deviceA.encrypt(contact);
      final decrypted = await deviceB.decrypt(encrypted);
      expect(decrypted.cidNumber, _contactCidNumber);
      expect(decrypted.contactRemark, '张三');
      final broken = SquareEncryptedContact(
        contactId: encrypted.contactId,
        ciphertext: encrypted.ciphertext,
        nonce: encrypted.nonce,
        mac: base64UrlEncode(List<int>.filled(16, 0)).replaceAll('=', ''),
        updatedAt: encrypted.updatedAt,
      );
      await expectLater(deviceB.decrypt(broken), throwsFormatException);
    });

    test('contact_id 只由永久 CID 和索引钥决定，不随账户换绑改变', () async {
      final keys = await _FakeWalletManager()
          .ensureContactKeyMaterialForAccountId(_accountId);
      final cryptor =
          ContactCryptor(ownerCidNumber: _ownerCidNumber, keys: keys);

      final before = await cryptor.contactId(_contactCidNumber);
      final after = await cryptor.contactId(_contactCidNumber);

      expect(before, after);
      expect(before, matches(RegExp(r'^[a-f0-9]{64}$')));
    });

    test('旧 contact_name JSON 缺少 CID 与 contact_remark 时被拒绝', () {
      expect(
        () => UserContact.fromJson(<String, dynamic>{
          'account_id': UserContactService.accountIdFromSs58(_contactA),
          'ss58_address': _contactA,
          'contact_name': '旧名称',
          'created_at': 1,
          'updated_at': 2,
        }),
        throwsFormatException,
      );
    });

    test('同步到云端的记录不含联系人明文', () async {
      final api = _FakeApi();
      final service = UserContactService(
        walletManager: _FakeWalletManager(),
        sessionProvider: _FakeSessionProvider(),
        apiClient: api,
        autoSync: false,
      );
      await service.addContact(
        cidNumber: _contactCidNumber,
        ss58Address: _contactA,
        contactRemark: '张三',
      );
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

    test('用户名片码校验声明 CID，且公开昵称不写入私人备注', () async {
      final resolver = _FixedPeerCidResolver(_contactCidNumber);
      final service = createService();

      final result = await addUserQrContact(
        body: const UserContactBody(
          cidNumber: _contactCidNumber,
          ss58Address: _contactA,
          displayName: '对方公开昵称',
        ),
        cidResolver: resolver,
        contactService: service,
      );

      expect(
        resolver.resolvedAccountId,
        UserContactService.accountIdFromSs58(_contactA),
      );
      expect(result.contact.cidNumber, _contactCidNumber);
      expect(result.contact.contactRemark, isEmpty);
    });

    test('用户名片码声明 CID 与 SS58 链上解析不一致时拒绝', () async {
      final resolver = _FixedPeerCidResolver('CN001-CTZN-999999999-2026');
      final service = createService();

      await expectLater(
        addUserQrContact(
          body: const UserContactBody(
            cidNumber: _contactCidNumber,
            ss58Address: _contactA,
            displayName: '对方公开昵称',
          ),
          cidResolver: resolver,
          contactService: service,
        ),
        throwsA(isA<FormatException>()),
      );
      expect(await service.getContacts(), isEmpty);
    });
  });

  group('通讯录换绑迁移', () {
    final newId = '0x${'a1' * 32}';

    test('migrateContactsToNewIdentity 搬本地明文+清旧账户密钥+新账户重加密上云', () async {
      final wallet = _RecordingWallet();
      final api = _FakeApi();

      // 1. 旧身份下加一个联系人(autoSync=false,只落本地密文缓存)。
      IdentityAccountCache.debugInstance = _AccountCache(_accountId);
      final oldService = UserContactService(
        walletManager: wallet,
        sessionProvider: _AccountSession(_accountId),
        apiClient: api,
        autoSync: false,
      );
      await oldService.addContact(
        cidNumber: _contactCidNumber,
        ss58Address: _contactA,
        contactRemark: '轻节点A',
      );
      const legacyContactId =
          'ffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffff';
      api.cloud[legacyContactId] = const SquareEncryptedContact(
        contactId: legacyContactId,
        ciphertext: 'b2xk',
        nonce: 'AAAAAAAAAAAAAAAA',
        mac: 'AAAAAAAAAAAAAAAAAAAAAA',
        updatedAt: 1,
      );

      // 2. 换绑迁移 old→new(新身份 + 新会话)。
      IdentityAccountCache.debugInstance = _AccountCache(newId);
      final newService = UserContactService(
        walletManager: wallet,
        sessionProvider: _AccountSession(newId),
        apiClient: api,
        autoSync: false,
      );
      await newService.migrateContactsToNewIdentity(_accountId, newId);

      // 3a. 新身份读到迁来的联系人(本地密文搬迁并可解密)。
      final migrated = await newService.getContacts();
      expect(migrated.single.contactRemark, '轻节点A');
      // 3b. 旧账户通讯录密钥已删(杜绝残留)。
      expect(wallet.deletedContactKeys, contains(_accountId));
      // 3c. 用新账户会话重加密上云(云端非空且密文不含明文)。
      expect(api.cloud, isNotEmpty);
      expect(api.cloud, isNot(contains(legacyContactId)));
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

    test('旧账户本地为空时也保留新账户已有 CID 联系人并重建云端', () async {
      final wallet = _RecordingWallet();
      final api = _FakeApi();
      IdentityAccountCache.debugInstance = _AccountCache(newId);
      final service = UserContactService(
        walletManager: wallet,
        sessionProvider: _AccountSession(newId),
        apiClient: api,
        autoSync: false,
      );
      await service.addContact(
        cidNumber: _contactCidNumber,
        ss58Address: _contactA,
        contactRemark: '新账户本地备注',
      );

      await service.migrateContactsToNewIdentity(_accountId, newId);

      expect((await service.getContacts()).single.contactRemark, '新账户本地备注');
      expect(api.cloud, hasLength(1));
    });
  });

  test('数据库重开时彻底清除旧通讯录缓存、待办与同步态', () async {
    final isar = await WalletIsar.instance.db();
    await isar.writeTxn(() async {
      for (final key in <String>[
        'contacts:$_accountId',
        'contact_pending_ops:$_accountId',
        'contact_sync_state:$_accountId',
      ]) {
        await isar.appKvEntitys.put(
          AppKvEntity()
            ..key = key
            ..stringValue = '旧数据',
        );
      }
    });
    await isar.close();

    final reopened = await WalletIsar.instance.db();
    for (final key in <String>[
      'contacts:$_accountId',
      'contact_pending_ops:$_accountId',
      'contact_sync_state:$_accountId',
    ]) {
      expect(await reopened.appKvEntitys.getByKey(key), isNull);
    }
  });

  group('通讯录本地静止态加密', () {
    UserContactService createService() => UserContactService(
          walletManager: _FakeWalletManager(),
          sessionProvider: _FakeSessionProvider(),
          apiClient: _FakeApi(),
          autoSync: false,
        );

    test('本地 KV 落盘为密文,Isar 原始值不含备注/CID/地址明文', () async {
      final service = createService();
      const remark = '张三备注不该出现在磁盘上';
      await service.addContact(
        cidNumber: _contactCidNumber,
        ss58Address: _contactA,
        contactRemark: remark,
      );

      // 绕过服务直接读原始 KV 行
      final rows = await WalletIsar.instance.read((isar) async {
        return isar.appKvEntitys
            .filter()
            .idGreaterThan(0, include: true)
            .findAll();
      });
      final contactRows = rows
          .where((r) => r.key.startsWith('contact_book_by_account:'))
          .toList();
      expect(contactRows, isNotEmpty);
      for (final row in contactRows) {
        final raw = row.stringValue ?? '';
        expect(raw, isNot(contains(remark)), reason: '备注不得明文落盘');
        expect(raw, isNot(contains(_contactCidNumber)), reason: 'CID 不得明文落盘');
        expect(raw, isNot(contains(_contactA)), reason: 'SS58 不得明文落盘');
        expect(raw, isNot(contains('contact_remark')), reason: '连字段名都不该露');
      }
    });

    test('加密后读回仍是完整明文对象', () async {
      final service = createService();
      await service.addContact(
        cidNumber: _contactCidNumber,
        ss58Address: _contactA,
        contactRemark: '李四',
      );
      final reopened = createService();
      final contacts = await reopened.getContacts();
      expect(contacts, hasLength(1));
      expect(contacts.single.cidNumber, _contactCidNumber);
      expect(contacts.single.contactRemark, '李四');
    });

    test('本地密文被篡改必须抛错,不得静默当成"无本地缓存"', () async {
      final service = createService();
      await service.addContact(
        cidNumber: _contactCidNumber,
        ss58Address: _contactA,
        contactRemark: '王五',
      );
      await WalletIsar.instance.writeTxn((isar) async {
        final rows = await isar.appKvEntitys
            .filter()
            .idGreaterThan(0, include: true)
            .findAll();
        for (final row in rows) {
          if (row.key.startsWith('contact_book_by_account:')) {
            row.stringValue = 'AAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAA';
            await isar.appKvEntitys.put(row);
          }
        }
      });
      await expectLater(
        createService().getContacts(),
        throwsA(isA<LocalCipherException>()),
      );
    });
  });
}
