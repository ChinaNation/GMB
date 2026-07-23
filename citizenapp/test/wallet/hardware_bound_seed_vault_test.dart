import 'package:flutter/services.dart';
import 'package:flutter_test/flutter_test.dart';

import 'package:citizenapp/wallet/core/fake_hardware_bound_seed_vault.dart';
import 'package:citizenapp/wallet/core/hardware_bound_seed_vault.dart';
import 'package:citizenapp/wallet/core/secure_seed_store.dart';

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

    test('seed put/read/delete round-trip', () async {
      expect(await vault.readSeed(1), isNull);
      await vault.putSeed(1, 'deadbeef');
      expect(await vault.readSeed(1), 'deadbeef');
      await vault.deleteSeed(1);
      expect(await vault.readSeed(1), isNull);
    });

    test('mnemonic put/read/delete round-trip', () async {
      await vault.putMnemonic(2, 'word1 word2 word3');
      expect(await vault.readMnemonic(2), 'word1 word2 word3');
      await vault.deleteMnemonic(2);
      expect(await vault.readMnemonic(2), isNull);
    });

    test('injected readSeed error thrown once then cleared', () async {
      await vault.putSeed(1, 'x');
      vault.nextSeedReadError = const SeedKeyInvalidated('changed');
      await expectLater(
        () => vault.readSeed(1),
        throwsA(isA<SeedKeyInvalidated>()),
      );
      expect(await vault.readSeed(1), 'x');
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

    test('putSeed uses strict tier + strict blob key', () async {
      await vault.putSeed(3, 'seedhex');
      final enc = calls.firstWhere((c) => c.method == 'encrypt');
      expect(enc.arguments['tier'], 'strict');
      expect(enc.arguments['walletIndex'], 3);
      expect(blobs.store['wallet_seed_env_v1_3'], isNotNull);
    });

    test('putMnemonic uses recovery tier + recovery blob key', () async {
      await vault.putMnemonic(5, 'w1 w2 w3');
      final enc = calls.firstWhere((c) => c.method == 'encrypt');
      expect(enc.arguments['tier'], 'recovery');
      expect(blobs.store['wallet_recovery_env_v1_5'], isNotNull);
    });

    test('seed put/read round-trip through channel + blob store', () async {
      await vault.putSeed(3, 'seedhex');
      expect(await vault.readSeed(3), 'seedhex');
    });

    test('readSeed returns null and skips decrypt when no blob', () async {
      expect(await vault.readSeed(42), isNull);
      expect(calls.where((c) => c.method == 'decrypt'), isEmpty);
    });

    test('deleteSeed removes blob and deletes strict KEK', () async {
      await vault.putSeed(3, 'x');
      await vault.deleteSeed(3);
      expect(blobs.store.containsKey('wallet_seed_env_v1_3'), isFalse);
      final del = calls.firstWhere((c) => c.method == 'deleteKey');
      expect(del.arguments['tier'], 'strict');
      expect(del.arguments['walletIndex'], 3);
    });

    test('encrypt returning null throws SecureStoreUnavailable', () async {
      encryptReturnsNull = true;
      await expectLater(
        () => vault.putSeed(1, 'x'),
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
      test('readSeed maps ${mapping.$1} error code', () async {
        await vault.putSeed(3, 'x');
        decryptErrorCode = mapping.$1;
        await expectLater(() => vault.readSeed(3), throwsA(mapping.$2));
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
