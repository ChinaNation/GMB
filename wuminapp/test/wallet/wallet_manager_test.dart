import 'package:flutter/services.dart';
import 'package:flutter_test/flutter_test.dart';
import 'package:shared_preferences/shared_preferences.dart';
import 'package:wuminapp_mobile/wallet/core/wallet_manager.dart';
import 'package:wuminapp_mobile/Isar/wallet_isar.dart';
import 'package:wuminapp_mobile/wallet/core/wallet_secure_keys.dart';

void main() {
  TestWidgetsFlutterBinding.ensureInitialized();

  const secureStorageChannel =
      MethodChannel('plugins.it_nomads.com/flutter_secure_storage');
  const localAuthChannel = MethodChannel('plugins.flutter.io/local_auth');
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
    TestDefaultBinaryMessengerBinding.instance.defaultBinaryMessenger
        .setMockMethodCallHandler(localAuthChannel, (call) async {
      switch (call.method) {
        case 'isDeviceSupported':
        case 'deviceSupportsBiometrics':
          return false;
        case 'getAvailableBiometrics':
          return const <String>[];
        case 'authenticate':
          return true;
        default:
          return null;
      }
    });
  });

  tearDown(() {
    TestDefaultBinaryMessengerBinding.instance.defaultBinaryMessenger
        .setMockMethodCallHandler(secureStorageChannel, null);
    TestDefaultBinaryMessengerBinding.instance.defaultBinaryMessenger
        .setMockMethodCallHandler(localAuthChannel, null);
  });

  group('WalletManager — 热钱包', () {
    test(
        'create/import/delete wallet should keep profile and secure data synced',
        () async {
      final manager = WalletManager();

      final created = await manager.createWallet();
      expect(created.profile.walletIndex, 1);
      expect(created.profile.alg, 'sr25519');
      expect(created.profile.ss58, 2027);
      expect(created.profile.signMode, 'local');
      expect(created.mnemonic.trim().split(RegExp(r'\s+')).length, 12);

      var wallets = await manager.getWallets();
      expect(wallets.length, 1);
      expect(await manager.getActiveWalletIndex(), 1);
      // 热钱包应存储 seedHex（非助记词）。
      expect(secureStorage[WalletSecureKeys.seedHexV1(1)], isNotEmpty);

      final imported = await manager.importWallet(
        'legal winner thank year wave sausage worth useful legal winner thank yellow',
      );
      expect(imported.walletIndex, 2);
      expect(imported.alg, 'sr25519');
      expect(imported.signMode, 'local');

      wallets = await manager.getWallets();
      expect(wallets.length, 2);
      expect(await manager.getActiveWalletIndex(), 2);
      expect(secureStorage[WalletSecureKeys.seedHexV1(2)], isNotEmpty);

      final latestSecret = await manager.getLatestWalletSecret();
      expect(latestSecret, isNotNull);
      expect(latestSecret!.profile.walletIndex, 2);
      // seedHex 应为 64 个 hex 字符（32 字节）。
      expect(latestSecret.seedHex.length, 64);

      await manager.deleteWallet(2);
      wallets = await manager.getWallets();
      expect(wallets.length, 1);
      expect(await manager.getActiveWalletIndex(), 1);
      expect(secureStorage.containsKey(WalletSecureKeys.seedHexV1(2)), isFalse);

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

    test('should not read removed seed key', () async {
      final manager = WalletManager();
      final imported = await manager.importWallet(
        'legal winner thank year wave sausage worth useful legal winner thank yellow',
      );
      final walletIndex = imported.walletIndex;
      final seedKey = WalletSecureKeys.seedHexV1(walletIndex);

      secureStorage.remove(seedKey);

      final secret = await manager.getWalletSecretByIndex(walletIndex);
      expect(secret, isNull);
    });
  });

  group('WalletManager — 冷钱包', () {
    test('importColdWallet should store only public key, no seed', () async {
      final manager = WalletManager();

      // 先创建一个热钱包获取有效地址。
      final hot = await manager.createWallet();
      final address = hot.profile.address;

      final cold = await manager.importColdWallet(address: address);
      expect(cold.signMode, 'external');
      expect(cold.address, address);

      // 冷钱包不存 seed。
      expect(
        secureStorage.containsKey(WalletSecureKeys.seedHexV1(cold.walletIndex)),
        isFalse,
      );

      // getLatestWalletSecret 返回 null（冷钱包无密钥）。
      await manager.setActiveWallet(cold.walletIndex);
      final secret = await manager.getLatestWalletSecret();
      expect(secret, isNull);
    });

    test('createColdWallet should return mnemonic but not store seed',
        () async {
      final manager = WalletManager();

      final result = await manager.createColdWallet();
      expect(result.profile.signMode, 'external');
      expect(result.mnemonic.trim().split(RegExp(r'\s+')).length, 12);

      // 冷钱包不在 secure storage 存任何东西。
      expect(
        secureStorage.containsKey(
            WalletSecureKeys.seedHexV1(result.profile.walletIndex)),
        isFalse,
      );
    });

    test('deleteColdWallet should not touch secure storage', () async {
      final manager = WalletManager();
      final result = await manager.createColdWallet();
      final walletIndex = result.profile.walletIndex;

      await manager.deleteWallet(walletIndex);
      expect(await manager.getWallets(), isEmpty);
    });
  });
}
