import 'dart:convert';
import 'dart:typed_data';

import 'package:flutter_test/flutter_test.dart';
import 'package:shared_preferences/shared_preferences.dart';

import 'package:citizenapp/8964/profile/services/square_session_provider.dart';
import 'package:citizenapp/8964/services/square_api_client.dart';
import 'package:citizenapp/chat/identity/peer_cid_resolver.dart';
import 'package:citizenapp/citizen/shared/account_derivation.dart';
import 'package:citizenapp/citizen/public/data/isar_public_institution_store.dart';
import 'package:citizenapp/citizen/public/data/public_institution_dto.dart';
import 'package:citizenapp/isar/app_isar.dart';
import 'package:citizenapp/my/myid/identity_account_cache.dart';
import 'package:citizenapp/my/myid/identity_account_resolver.dart';
import 'package:citizenapp/my/myid/citizen_identity_chain_reader.dart';
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
  /// 固定当前钱包用途子钥，避免测试触碰硬件金库或平台通道。
  /// 只替换密钥来源，通讯录本地 KV 仍走真实 AES-256-GCM。
  @override
  Future<Uint8List> readDataKeyForCurrentBinding(
    String accountId,
    LocalKeyPurpose purpose, {
    String? context,
  }) async =>
      Uint8List.fromList(List<int>.generate(
        32,
        (i) => (i + purpose.index + (context?.length ?? 0) + 1) % 256,
      ));

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

/// 身份缓存 fake：恒返回已注册 CID 及其当前绑定账户。
class _FakeIdentityCache extends IdentityAccountCache {
  @override
  Future<ResolvedIdentity?> resolve({bool allowChainRead = true}) async =>
      ResolvedIdentity(
        accountId: _accountId,
        ss58Address: _owner,
        accountIndex: 0,
        snapshot: CitizenIdentityChainSnapshot(
          cidNumber: _ownerCidNumber,
          accountId: Uint8List(32),
          bindingRevision: 1,
          votingIdentity: null,
        ),
      );
  @override
  Future<String?> accountId({bool allowChainRead = true}) async => _accountId;
}

class _FixedIdentityCache extends IdentityAccountCache {
  _FixedIdentityCache(this.boundAccountId, {this.bindingRevision = 2});

  final String boundAccountId;
  final int bindingRevision;

  @override
  Future<ResolvedIdentity?> resolve({bool allowChainRead = true}) async =>
      ResolvedIdentity(
        accountId: boundAccountId,
        ss58Address: _owner,
        accountIndex: 0,
        snapshot: CitizenIdentityChainSnapshot(
          cidNumber: _ownerCidNumber,
          accountId: Uint8List(32),
          bindingRevision: bindingRevision,
          votingIdentity: null,
        ),
      );
}

class _FakeSessionProvider extends SquareSessionProvider {
  @override
  Future<SquareSession?> ensureSession() async => SquareSession(
        sessionToken: 'token',
        cidNumber: _ownerCidNumber,
        bindingRevision: 1,
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
    int? bindingRevision,
    String? accountId,
  }) async {
    cloud.remove(contactId);
  }
}

class _HandoverWalletManager extends WalletManager {
  Uint8List _key(
    String accountId,
    LocalKeyPurpose purpose, {
    String? context,
  }) {
    final accountByte = int.parse(accountId.substring(2, 4), radix: 16);
    return Uint8List.fromList(List<int>.generate(
      32,
      (index) =>
          (accountByte + purpose.index * 17 + (context?.length ?? 0) + index) &
          0xff,
    ));
  }

  @override
  Future<Uint8List> readDataKeyForCurrentBinding(
    String accountId,
    LocalKeyPurpose purpose, {
    String? context,
  }) async =>
      _key(accountId, purpose, context: context);

  @override
  Future<List<Uint8List>> deriveDataKeysForBindingHandover(
    AccountDataBinding binding,
    List<({String? context, LocalKeyPurpose purpose})> requests,
  ) async =>
      requests
          .map((request) => _key(
                binding.accountId,
                request.purpose,
                context: request.context,
              ))
          .toList(growable: false);

  @override
  Future<ContactKeyMaterial> ensureContactKeyMaterialForAccountId(
    String accountId,
  ) async =>
      ContactKeyMaterial(
        encryptionKey: _key(accountId, LocalKeyPurpose.contactsCloud),
        indexKey: _key(
          accountId,
          LocalKeyPurpose.contactsCloud,
          context: 'index',
        ),
      );

  @override
  Future<ContactKeyMaterial> contactKeyMaterialForBinding(
    AccountDataBinding binding,
  ) =>
      ensureContactKeyMaterialForAccountId(binding.accountId);
}

class _HandoverSessionProvider extends SquareSessionProvider {
  _HandoverSessionProvider({
    required this.sourceAccountId,
    required this.targetAccountId,
  });

  final String sourceAccountId;
  final String targetAccountId;

  SquareSession _session(String accountId, int revision) => SquareSession(
        sessionToken: 'token-$revision',
        cidNumber: _ownerCidNumber,
        bindingRevision: revision,
        accountId: accountId,
        expiresAt: DateTime.now().millisecondsSinceEpoch + 60000,
      );

  @override
  Future<SquareSession?> ensureSession() async => _session(sourceAccountId, 1);

  @override
  Future<SquareSession?> ensureSessionForAccountId(String accountId) async =>
      accountId == targetAccountId ? _session(targetAccountId, 2) : null;
}

class _HandoverApi extends SquareApiClient {
  _HandoverApi() : super(baseUrl: 'https://contacts.test');

  final Map<String, SquareEncryptedContact> cloud =
      <String, SquareEncryptedContact>{};

  String _key(SquareEncryptedContact contact) =>
      '${contact.bindingRevision}:${contact.accountId}:${contact.contactId}';

  @override
  Future<({List<SquareEncryptedContact> items, String? nextCursor})>
      fetchEncryptedContacts({
    required SquareSession session,
    String? cursor,
    int limit = 100,
  }) async =>
          (
            items: cloud.values
                .where((contact) =>
                    contact.bindingRevision == session.bindingRevision &&
                    contact.accountId == session.accountId)
                .toList(growable: false),
            nextCursor: null,
          );

  @override
  Future<void> putEncryptedContact({
    required SquareSession session,
    required SquareEncryptedContact contact,
  }) async {
    cloud[_key(contact)] = contact;
  }

  @override
  Future<void> deleteEncryptedContact({
    required SquareSession session,
    required String contactId,
    int? bindingRevision,
    String? accountId,
  }) async {
    cloud.remove('${bindingRevision ?? session.bindingRevision}:'
        '${accountId ?? session.accountId}:$contactId');
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

class _FakeBindingReader extends CitizenIdentityChainReader {
  _FakeBindingReader(this.bindings);

  final Map<String, CitizenBindingChainSnapshot> bindings;
  int batchReads = 0;
  int singleReads = 0;

  @override
  Future<Map<String, CitizenBindingChainSnapshot>> readBindingsByCidNumbers(
    Iterable<String> cidNumbers,
  ) async {
    batchReads++;
    return <String, CitizenBindingChainSnapshot>{
      for (final cidNumber in cidNumbers)
        if (bindings[cidNumber] case final binding?) cidNumber: binding,
    };
  }

  @override
  Future<CitizenBindingChainSnapshot?> readBindingByCidNumber(
    String cidNumber,
  ) async {
    singleReads++;
    return bindings[cidNumber];
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

    test('按 CID 批量刷新联系人最新绑定账户并保留关系与私人备注', () async {
      const newAccountId =
          '0xaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa';
      final reader = _FakeBindingReader(<String, CitizenBindingChainSnapshot>{
        _contactCidNumber: CitizenBindingChainSnapshot(
          cidNumber: _contactCidNumber,
          accountId: Uint8List.fromList(List<int>.filled(32, 0xaa)),
          bindingRevision: 2,
        ),
      });
      final service = UserContactService(
        walletManager: _FakeWalletManager(),
        sessionProvider: _FakeSessionProvider(),
        apiClient: _FakeApi(),
        chainReader: reader,
        autoSync: false,
      );
      await service.addContact(
        cidNumber: _contactCidNumber,
        ss58Address: _contactA,
        contactRemark: '换绑后保留',
      );

      final refreshed = await service.refreshContactBindings();

      expect(reader.batchReads, 1);
      expect(refreshed.single.cidNumber, _contactCidNumber);
      expect(refreshed.single.accountId, newAccountId);
      expect(
        refreshed.single.ss58Address,
        ss58FromAccountIdText(newAccountId),
      );
      expect(refreshed.single.contactRemark, '换绑后保留');
    });

    test('转账前按 CID 严格读取当前绑定，失效时禁止回退旧地址', () async {
      final reader = _FakeBindingReader(
        const <String, CitizenBindingChainSnapshot>{},
      );
      final service = UserContactService(
        walletManager: _FakeWalletManager(),
        sessionProvider: _FakeSessionProvider(),
        apiClient: _FakeApi(),
        chainReader: reader,
        autoSync: false,
      );
      await service.addContact(
        cidNumber: _contactCidNumber,
        ss58Address: _contactA,
        contactRemark: '',
      );

      await expectLater(
        service.resolveCurrentContact(_contactCidNumber),
        throwsA(isA<StateError>()),
      );
      expect(reader.singleReads, 1);
    });

    test('AES-GCM 可跨设备解密且篡改 MAC 后失败', () async {
      final keys = await _FakeWalletManager()
          .ensureContactKeyMaterialForAccountId(_accountId);
      final deviceA = ContactCryptor(
        ownerCidNumber: _ownerCidNumber,
        bindingRevision: 1,
        accountId: _accountId,
        keys: keys,
      );
      final deviceB = ContactCryptor(
        ownerCidNumber: _ownerCidNumber,
        bindingRevision: 1,
        accountId: _accountId,
        keys: keys,
      );
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
        bindingRevision: encrypted.bindingRevision,
        accountId: encrypted.accountId,
        contactId: encrypted.contactId,
        ciphertext: encrypted.ciphertext,
        nonce: encrypted.nonce,
        mac: base64UrlEncode(List<int>.filled(16, 0)).replaceAll('=', ''),
        updatedAt: encrypted.updatedAt,
      );
      await expectLater(deviceB.decrypt(broken), throwsFormatException);
    });

    test('同一钱包绑定内 contact_id 由联系人 CID 和索引钥稳定确定', () async {
      final keys = await _FakeWalletManager()
          .ensureContactKeyMaterialForAccountId(_accountId);
      final cryptor = ContactCryptor(
        ownerCidNumber: _ownerCidNumber,
        bindingRevision: 1,
        accountId: _accountId,
        keys: keys,
      );

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
      final contactRows =
          rows.where((r) => r.key.startsWith('contact_book_by_cid:')).toList();
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

    test('当前钱包签名换绑先本地暂存新密文，finalized 后新钱包上传并清理此前版本', () async {
      const newAccountId =
          '0xaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa';
      const genesisHash =
          '0x1111111111111111111111111111111111111111111111111111111111111111';
      final wallet = _HandoverWalletManager();
      final api = _HandoverApi();
      final sessions = _HandoverSessionProvider(
        sourceAccountId: _accountId,
        targetAccountId: newAccountId,
      );
      final currentBinding = UserContactService(
        walletManager: wallet,
        identityAccountCache:
            _FixedIdentityCache(_accountId, bindingRevision: 1),
        sessionProvider: sessions,
        apiClient: api,
        autoSync: false,
      );
      await currentBinding.addContact(
        cidNumber: _contactCidNumber,
        ss58Address: _contactA,
        contactRemark: '换绑后仍保留',
      );
      await currentBinding.sync();
      expect(api.cloud.values.single.bindingRevision, 1);

      final source = AccountDataBinding(
        genesisHash: genesisHash,
        cidNumber: _ownerCidNumber,
        bindingRevision: 1,
        accountId: _accountId,
      );
      const target = AccountDataBinding(
        genesisHash: genesisHash,
        cidNumber: _ownerCidNumber,
        bindingRevision: 2,
        accountId: newAccountId,
      );
      await currentBinding.stageAccountHandover(source: source, target: target);
      expect(api.cloud.values.map((record) => record.bindingRevision).toSet(),
          <int>{1});
      expect(
        (await currentBinding.getContacts()).single.contactRemark,
        '换绑后仍保留',
        reason: 'finalized 前正式本地密文仍由当前钱包读取，目标密文不得预写 Worker',
      );

      await currentBinding.commitAccountHandover(
          source: source, target: target);
      expect(api.cloud.values.single.bindingRevision, 2);
      expect(api.cloud.values.single.accountId, newAccountId);

      final newBinding = UserContactService(
        walletManager: wallet,
        identityAccountCache: _FixedIdentityCache(newAccountId),
        sessionProvider: sessions,
        apiClient: api,
        autoSync: false,
      );
      final contacts = await newBinding.getContacts();

      expect(contacts.single.cidNumber, _contactCidNumber);
      expect(contacts.single.contactRemark, '换绑后仍保留');
    });

    test('无当前账户签名换绑只隔离此前通讯录密文，新绑定从空通讯录开始', () async {
      const newAccountId =
          '0xaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa';
      const genesisHash =
          '0x1111111111111111111111111111111111111111111111111111111111111111';
      final wallet = _HandoverWalletManager();
      final sourceService = UserContactService(
        walletManager: wallet,
        identityAccountCache:
            _FixedIdentityCache(_accountId, bindingRevision: 1),
        sessionProvider: _HandoverSessionProvider(
          sourceAccountId: _accountId,
          targetAccountId: newAccountId,
        ),
        apiClient: _HandoverApi(),
        autoSync: false,
      );
      await sourceService.addContact(
        cidNumber: _contactCidNumber,
        ss58Address: _contactA,
        contactRemark: '只属于此前密文',
      );
      final source = AccountDataBinding(
        genesisHash: genesisHash,
        cidNumber: _ownerCidNumber,
        bindingRevision: 1,
        accountId: _accountId,
      );
      final canonicalBefore = await WalletIsar.instance.read((isar) async {
        return isar.appKvEntitys
            .filter()
            .keyStartsWith('contact_book_by_cid:')
            .findAll();
      });
      expect(canonicalBefore, hasLength(1));

      await sourceService.isolateInaccessibleBinding(source);

      final rows = await WalletIsar.instance.read((isar) async =>
          isar.appKvEntitys.filter().idGreaterThan(0, include: true).findAll());
      expect(
        rows.where((row) => row.key.startsWith('contact_book_by_cid:')),
        isEmpty,
      );
      final archived = rows
          .where(
              (row) => row.key.startsWith('contact_inaccessible_by_binding:'))
          .toList(growable: false);
      expect(archived, hasLength(3));
      expect(
        archived.map((row) => row.stringValue),
        contains(canonicalBefore.single.stringValue),
      );

      final targetService = UserContactService(
        walletManager: wallet,
        identityAccountCache:
            _FixedIdentityCache(newAccountId, bindingRevision: 2),
        sessionProvider: _HandoverSessionProvider(
          sourceAccountId: _accountId,
          targetAccountId: newAccountId,
        ),
        apiClient: _HandoverApi(),
        autoSync: false,
      );
      expect(await targetService.getContacts(), isEmpty);
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
          if (row.key.startsWith('contact_book_by_cid:')) {
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

  test('公权机构关注按订阅者 CID 持久化，钱包换绑不产生新分区', () async {
    const institutionCidNumber = 'GD001-CGOV0-000000001-2026';
    final isar = await WalletIsar.instance.db();
    await isar.writeTxn(() async {
      await isar.publicInstitutionEntitys.clear();
      await isar.publicInstitutionSubscriptionEntitys.clear();
    });
    final store = IsarPublicInstitutionStore(isar: isar);
    await store.upsertInstitutions(
      [
        PublicInstitutionDto.fromJson(<String, dynamic>{
          'cid_number': institutionCidNumber,
          'cid_full_name': '广东省人民政府',
          'province_code': 'GD',
          'city_code': '001',
          'institution_code': 'CGOV',
          'account_count': 2,
        }),
      ],
      catalogVersion: 'test',
    );

    await store.subscribe(_ownerCidNumber, institutionCidNumber);

    expect(
      (await store.listSubscribed(_ownerCidNumber)).map((row) => row.cidNumber),
      [institutionCidNumber],
      reason: '关注归属永久 CID，不接收或存储当前钱包账户作为分区键',
    );
    expect(
      await store.listSubscribed('CN220-CTZN2-OTHER-2026'),
      isEmpty,
    );
  });
}
