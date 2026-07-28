import 'package:flutter/services.dart';
import 'package:flutter_test/flutter_test.dart';

import 'package:citizenapp/wallet/core/fake_hardware_bound_seed_vault.dart';
import 'package:citizenapp/wallet/core/hardware_bound_seed_vault.dart';
import 'package:citizenapp/wallet/core/secure_seed_store.dart';

/// 样例 accountId（规范 0x + 64 hex），仅用于键控密文 blob。
const String _accountA =
    '0x2afba9278e30ccf6a6ceb3a8b6e336b70068f045c666f2e7f4f9cc5f47db8972';
const String _accountB =
    '0xb606fc73f57f03cdb4c932d475ab426043e429cecc2ffff0d2672b0df8398c48';

/// 内存 blob store，避免单测耦合 flutter_secure_storage v10 的通道内部。
class _MemBlobStore implements VaultBlobStore {
  final Map<String, String> store = <String, String>{};

  @override
  Future<String?> read(String key) async => store[key];

  @override
  Future<void> write(String key, String value) async {
    store[key] = value;
  }

  @override
  Future<void> delete(String key) async {
    store.remove(key);
  }
}

void main() {
  TestWidgetsFlutterBinding.ensureInitialized();

  group('FakeHardwareBoundSeedVault', () {
    late FakeHardwareBoundSeedVault vault;

    setUp(() => vault = FakeHardwareBoundSeedVault());

    test('account key put/read/delete round-trip', () async {
      expect(
        await vault.readAccountKey(walletIndex: 1, accountId: _accountA),
        isNull,
      );
      await vault.putAccountKey(
        walletIndex: 1,
        accountId: _accountA,
        childMiniSecretHex: 'deadbeef',
      );
      expect(
        await vault.readAccountKey(walletIndex: 1, accountId: _accountA),
        'deadbeef',
      );
      expect(await vault.hasAccountKey(_accountA), isTrue);
      await vault.deleteAccountKey(walletIndex: 1, accountId: _accountA);
      expect(
        await vault.readAccountKey(walletIndex: 1, accountId: _accountA),
        isNull,
      );
      expect(await vault.hasAccountKey(_accountA), isFalse);
    });

    test('injected readAccountKey error thrown once then cleared', () async {
      await vault.putAccountKey(
        walletIndex: 1,
        accountId: _accountA,
        childMiniSecretHex: 'x',
      );
      vault.nextReadError = const SeedKeyInvalidated('changed');
      await expectLater(
        () => vault.readAccountKey(walletIndex: 1, accountId: _accountA),
        throwsA(isA<SeedKeyInvalidated>()),
      );
      expect(
        await vault.readAccountKey(walletIndex: 1, accountId: _accountA),
        'x',
      );
    });

    test('authStatusValue is returned', () async {
      vault.authStatusValue = SecureAuthStatus.noDeviceLock;
      expect(await vault.authStatus(), SecureAuthStatus.noDeviceLock);
    });
  });

  group('HardwareBoundSeedVault', () {
    const channel = MethodChannel('org.citizenapp/hw_seed_vault');
    late _MemBlobStore blobs;
    late HardwareBoundSeedVault vault;
    late Map<String, String> blobToPlain;
    late List<MethodCall> calls;
    String? decryptErrorCode;
    bool encryptReturnsNull = false;
    bool biometricEnrolled = true;
    int counter = 0;

    setUp(() {
      blobs = _MemBlobStore();
      blobToPlain = <String, String>{};
      calls = <MethodCall>[];
      decryptErrorCode = null;
      encryptReturnsNull = false;
      biometricEnrolled = true;
      counter = 0;
      TestDefaultBinaryMessengerBinding.instance.defaultBinaryMessenger
          .setMockMethodCallHandler(channel, (MethodCall call) async {
        calls.add(call);
        final args = (call.arguments as Map?)?.cast<String, dynamic>() ??
            <String, dynamic>{};
        switch (call.method) {
          case 'authStatus':
            return <String, dynamic>{
              'sdk': 36,
              'strongBiometricEnrolled': biometricEnrolled,
              'deviceSecure': true,
            };
          case 'encrypt':
            if (encryptReturnsNull) return null;
            final blob = 'blob${counter++}';
            blobToPlain[blob] = args['plaintext'] as String;
            return blob;
          case 'decrypt':
            if (decryptErrorCode != null) {
              throw PlatformException(code: decryptErrorCode!);
            }
            return blobToPlain[args['blob'] as String];
          case 'deleteKey':
            return null;
        }
        return null;
      });
      vault = HardwareBoundSeedVault(channel: channel, blobStore: blobs);
    });

    tearDown(() {
      TestDefaultBinaryMessengerBinding.instance.defaultBinaryMessenger
          .setMockMethodCallHandler(channel, null);
    });

    test('putAccountKey uses strict tier + per-account blob key', () async {
      await vault.putAccountKey(
        walletIndex: 3,
        accountId: _accountA,
        childMiniSecretHex: 'childhex',
      );
      final enc = calls.firstWhere((c) => c.method == 'encrypt');
      expect(enc.arguments['tier'], 'strict');
      expect(enc.arguments['walletIndex'], 3);
      expect(blobs.store['wallet_account_key_v1_$_accountA'], isNotNull);
    });

    test('two accounts share wallet KEK but keep distinct blobs', () async {
      await vault.putAccountKey(
        walletIndex: 3,
        accountId: _accountA,
        childMiniSecretHex: 'a',
      );
      await vault.putAccountKey(
        walletIndex: 3,
        accountId: _accountB,
        childMiniSecretHex: 'b',
      );
      // 两个账户共享 walletIndex=3 的严档 KEK（tier=strict），各存独立 blob。
      expect(blobs.store['wallet_account_key_v1_$_accountA'], isNotNull);
      expect(blobs.store['wallet_account_key_v1_$_accountB'], isNotNull);
      expect(
        blobs.store['wallet_account_key_v1_$_accountA'],
        isNot(blobs.store['wallet_account_key_v1_$_accountB']),
      );
    });

    test('account key put/read round-trip through channel + blob store',
        () async {
      await vault.putAccountKey(
        walletIndex: 3,
        accountId: _accountA,
        childMiniSecretHex: 'childhex',
      );
      expect(
        await vault.readAccountKey(walletIndex: 3, accountId: _accountA),
        'childhex',
      );
    });

    test('hasAccountKey probes blob without decrypt', () async {
      expect(await vault.hasAccountKey(_accountA), isFalse);
      await vault.putAccountKey(
        walletIndex: 3,
        accountId: _accountA,
        childMiniSecretHex: 'x',
      );
      expect(await vault.hasAccountKey(_accountA), isTrue);
      expect(calls.where((c) => c.method == 'decrypt'), isEmpty);
    });

    test('readAccountKey returns null and skips decrypt when no blob',
        () async {
      expect(
        await vault.readAccountKey(walletIndex: 42, accountId: _accountA),
        isNull,
      );
      expect(calls.where((c) => c.method == 'decrypt'), isEmpty);
    });

    test('deleteAccountKey removes blob and deletes strict KEK', () async {
      await vault.putAccountKey(
        walletIndex: 3,
        accountId: _accountA,
        childMiniSecretHex: 'x',
      );
      await vault.deleteAccountKey(walletIndex: 3, accountId: _accountA);
      expect(blobs.store.containsKey('wallet_account_key_v1_$_accountA'),
          isFalse);
      final del = calls.firstWhere((c) => c.method == 'deleteKey');
      expect(del.arguments['tier'], 'strict');
      expect(del.arguments['walletIndex'], 3);
    });

    test('encrypt returning null throws SecureStoreUnavailable', () async {
      encryptReturnsNull = true;
      await expectLater(
        () => vault.putAccountKey(
          walletIndex: 1,
          accountId: _accountA,
          childMiniSecretHex: 'x',
        ),
        throwsA(isA<SecureStoreUnavailable>()),
      );
    });

    final mappings = <(String, Matcher)>[
      ('keyPermanentlyInvalidated', isA<SeedKeyInvalidated>()),
      ('userCancelled', isA<AuthCancelled>()),
      ('lockout', isA<AuthCancelled>()),
      ('notEnrolled', isA<NoDeviceCredential>()),
      ('somethingElse', isA<SecureStoreUnavailable>()),
    ];
    for (final mapping in mappings) {
      test('readAccountKey maps ${mapping.$1} error code', () async {
        await vault.putAccountKey(
          walletIndex: 3,
          accountId: _accountA,
          childMiniSecretHex: 'x',
        );
        decryptErrorCode = mapping.$1;
        await expectLater(
          () => vault.readAccountKey(walletIndex: 3, accountId: _accountA),
          throwsA(mapping.$2),
        );
      });
    }

    test('authStatus available when biometric enrolled', () async {
      expect(await vault.authStatus(), SecureAuthStatus.available);
    });

    test('authStatus noDeviceLock when no biometric', () async {
      biometricEnrolled = false;
      expect(await vault.authStatus(), SecureAuthStatus.noDeviceLock);
    });
  });
}
