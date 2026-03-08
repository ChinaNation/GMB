import 'package:flutter/services.dart';
import 'package:flutter_test/flutter_test.dart';
import 'package:isar/isar.dart';
import 'package:shared_preferences/shared_preferences.dart';
import 'package:wuminapp_mobile/wallet/capabilities/attestation_service.dart';
import 'package:wuminapp_mobile/Isar/wallet_isar.dart';
import 'package:wuminapp_mobile/wallet/core/wallet_manager.dart';
import 'package:wuminapp_mobile/wallet/core/wallet_secure_keys.dart';

void main() {
  TestWidgetsFlutterBinding.ensureInitialized();

  const secureStorageChannel =
      MethodChannel('plugins.it_nomads.com/flutter_secure_storage');
  final secureStorage = <String, String>{};

  setUpAll(() async {
    await WalletIsar.instance.ensureTestCoreInitialized();
  });

  setUp(() async {
    SharedPreferences.setMockInitialValues(<String, Object>{});
    secureStorage.clear();
    await WalletIsar.instance.resetForTest();
    TestDefaultBinaryMessengerBinding.instance.defaultBinaryMessenger
        .setMockMethodCallHandler(secureStorageChannel, (call) async {
      final args = (call.arguments as Map?)?.cast<String, dynamic>() ??
          <String, dynamic>{};
      final key = args['key']?.toString();
      switch (call.method) {
        case 'read':
          return key == null ? null : secureStorage[key];
        case 'write':
          if (key != null) {
            secureStorage[key] = args['value']?.toString() ?? '';
          }
          return null;
        case 'delete':
          if (key != null) {
            secureStorage.remove(key);
          }
          return null;
        case 'deleteAll':
          secureStorage.clear();
          return null;
        case 'containsKey':
          return key != null && secureStorage.containsKey(key);
        case 'readAll':
          return Map<String, String>.from(secureStorage);
        default:
          return null;
      }
    });
  });

  tearDown(() {
    TestDefaultBinaryMessengerBinding.instance.defaultBinaryMessenger
        .setMockMethodCallHandler(secureStorageChannel, null);
  });

  group('AttestationService', () {
    test('should store token in secure storage and metadata in Isar', () async {
      final service = AttestationService();
      final wallet = _walletFixture(walletIndex: 1);

      final state = await service.applyOfficialProof(wallet);
      expect(state.hasToken, isTrue);
      expect(state.token, isNotNull);
      expect(state.expiresAtMillis, isNotNull);

      final tokenKey = WalletSecureKeys.sessionTokenV1('attest');
      expect(secureStorage[tokenKey], state.token);

      final prefs = await SharedPreferences.getInstance();
      expect(prefs.getString('attest.token'), isNull);
      expect(prefs.getInt('attest.expires_at_millis'), isNull);
      expect(prefs.getString('attest.policy'), isNull);
      expect(prefs.getString('attest.last_payload'), isNull);

      final isar = await WalletIsar.instance.db();
      final expires = await isar.appKvEntitys
          .filter()
          .keyEqualTo('wallet.session.attest.expires_at_millis.v1')
          .findFirst();
      final policy = await isar.appKvEntitys
          .filter()
          .keyEqualTo('wallet.session.attest.policy.v1')
          .findFirst();
      final payload = await isar.appKvEntitys
          .filter()
          .keyEqualTo('wallet.session.attest.last_payload.v1')
          .findFirst();

      expect(expires?.intValue, state.expiresAtMillis);
      expect(policy?.stringValue, contains('alg=sr25519'));
      expect(payload?.stringValue, contains('"pubkey":"${wallet.pubkeyHex}"'));
    });
  });
}

WalletProfile _walletFixture({required int walletIndex}) {
  return WalletProfile(
    walletIndex: walletIndex,
    walletName: '钱包$walletIndex',
    walletIcon: 'wallet',
    balance: 0,
    address: 'w5FixtureAddress$walletIndex',
    pubkeyHex:
        '1234567890abcdef1234567890abcdef1234567890abcdef1234567890abcdef',
    alg: 'sr25519',
    ss58: 2027,
    createdAtMillis: DateTime.now().millisecondsSinceEpoch,
    source: 'created',
  );
}
