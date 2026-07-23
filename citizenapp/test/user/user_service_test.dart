import 'dart:convert';
import 'dart:typed_data';

import 'package:flutter_test/flutter_test.dart';
import 'package:shared_preferences/shared_preferences.dart';

import 'package:citizenapp/8964/profile/services/square_session_provider.dart';
import 'package:citizenapp/8964/services/square_api_client.dart';
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
  Future<ContactKeyMaterial> ensureContactKeyMaterial({
    required int walletIndex,
    required String accountId,
  }) async =>
      ContactKeyMaterial(
        encryptionKey: Uint8List.fromList(List<int>.filled(32, 7)),
        indexKey: Uint8List.fromList(List<int>.filled(32, 9)),
      );
}

class _FakeSessionProvider extends SquareSessionProvider {
  @override
  Future<SquareSession?> ensureSession() async => SquareSession(
        sessionToken: 'token',
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

void main() {
  TestWidgetsFlutterBinding.ensureInitialized();
  useIsolatedIsar();

  setUp(() {
    SharedPreferences.setMockInitialValues(<String, Object>{});
  });

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
      final keys = await _FakeWalletManager().ensureContactKeyMaterial(
        walletIndex: 1,
        accountId: _accountId,
      );
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
}
