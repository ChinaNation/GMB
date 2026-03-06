import 'package:flutter/services.dart';
import 'package:flutter_test/flutter_test.dart';
import 'package:shared_preferences/shared_preferences.dart';
import 'package:wuminapp_mobile/wallet/core/wallet_manager.dart';

void main() {
  TestWidgetsFlutterBinding.ensureInitialized();

  const secureStorageChannel =
      MethodChannel('plugins.it_nomads.com/flutter_secure_storage');
  final secureStorage = <String, String>{};

  setUp(() async {
    SharedPreferences.setMockInitialValues(<String, Object>{});
    secureStorage.clear();
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

  group('WalletManager', () {
    test(
        'create/import/delete wallet should keep profile and secure data synced',
        () async {
      final manager = WalletManager();

      final created = await manager.createWallet();
      expect(created.profile.walletIndex, 1);
      expect(created.profile.alg, 'sr25519');
      expect(created.profile.ss58, 2027);
      expect(created.mnemonic.trim().split(RegExp(r'\s+')).length, 12);

      var wallets = await manager.getWallets();
      expect(wallets.length, 1);
      expect(await manager.getActiveWalletIndex(), 1);
      expect(secureStorage['wallet.mnemonic.1'], isNotEmpty);

      final imported = await manager.importWallet(
        'legal winner thank year wave sausage worth useful legal winner thank yellow',
      );
      expect(imported.walletIndex, 2);
      expect(imported.alg, 'sr25519');

      wallets = await manager.getWallets();
      expect(wallets.length, 2);
      expect(await manager.getActiveWalletIndex(), 2);
      expect(secureStorage['wallet.mnemonic.2'], isNotEmpty);

      final latestSecret = await manager.getLatestWalletSecret();
      expect(latestSecret, isNotNull);
      expect(latestSecret!.profile.walletIndex, 2);

      await manager.deleteWallet(2);
      wallets = await manager.getWallets();
      expect(wallets.length, 1);
      expect(await manager.getActiveWalletIndex(), 1);
      expect(secureStorage.containsKey('wallet.mnemonic.2'), isFalse);

      await manager.deleteWallet(1);
      expect(await manager.getWallet(), isNull);
      expect(await manager.getWallets(), isEmpty);
      expect(secureStorage, isEmpty);
    });

    test('importWallet should reject invalid mnemonic', () async {
      final manager = WalletManager();
      expect(
        () => manager.importWallet('hello world'),
        throwsA(
          isA<Exception>().having(
            (e) => e.toString(),
            'message',
            contains('助记词无效'),
          ),
        ),
      );
    });
  });
}
