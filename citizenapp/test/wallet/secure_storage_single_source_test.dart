import 'package:flutter_secure_storage/flutter_secure_storage.dart';
import 'package:flutter_test/flutter_test.dart';

import 'package:citizenapp/security/secure_storage.dart';
import 'package:citizenapp/wallet/core/hardware_bound_seed_vault.dart';

void main() {
  TestWidgetsFlutterBinding.ensureInitialized();

  test('全局安全存储使用 Android 可恢复迁移与 iOS 本机首次解锁策略', () {
    final android = appSecureStorage.aOptions.toMap();
    final ios = appSecureStorage.iOptions.toMap();

    expect(android['migrateOnAlgorithmChange'], 'true');
    expect(android['migrateWithBackup'], 'true');
    expect(
      ios['accessibility'],
      KeychainAccessibility.first_unlock_this_device.name,
    );
    expect(ios['synchronizable'], 'false');
    expect(ios['useSecureEnclave'], 'false');
  });

  test('硬件金库密文存储保留测试注入并静默读写', () async {
    final fake = _FakeSecureStorage();
    final store = SecureStorageBlobStore(fake);
    await store.write('vault-test', 'ciphertext');
    expect(await store.read('vault-test'), 'ciphertext');
    await store.delete('vault-test');
    expect(await store.read('vault-test'), isNull);
  });
}

class _FakeSecureStorage extends FlutterSecureStorage {
  _FakeSecureStorage();

  final Map<String, String> values = <String, String>{};

  @override
  Future<String?> read({
    required String key,
    AppleOptions? iOptions,
    AndroidOptions? aOptions,
    LinuxOptions? lOptions,
    WebOptions? webOptions,
    AppleOptions? mOptions,
    WindowsOptions? wOptions,
  }) async =>
      values[key];

  @override
  Future<void> write({
    required String key,
    required String? value,
    AppleOptions? iOptions,
    AndroidOptions? aOptions,
    LinuxOptions? lOptions,
    WebOptions? webOptions,
    AppleOptions? mOptions,
    WindowsOptions? wOptions,
  }) async {
    if (value == null) {
      values.remove(key);
    } else {
      values[key] = value;
    }
  }

  @override
  Future<void> delete({
    required String key,
    AppleOptions? iOptions,
    AndroidOptions? aOptions,
    LinuxOptions? lOptions,
    WebOptions? webOptions,
    AppleOptions? mOptions,
    WindowsOptions? wOptions,
  }) async {
    values.remove(key);
  }
}
