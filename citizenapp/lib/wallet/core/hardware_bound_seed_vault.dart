import 'package:flutter/services.dart';
import 'package:flutter_secure_storage/flutter_secure_storage.dart';
import 'package:citizenapp/security/secure_storage.dart';
import 'package:citizenapp/wallet/core/secure_seed_store.dart';

/// 密文 blob 的持久化抽象（与硬件金库解耦，便于单测）。
///
/// blob 已由硬件 KEK 加密，落地存储只需静默读写；默认实现走
/// `flutter_secure_storage`（Keystore/Keychain 静止态再加一层，防御纵深）。
abstract interface class VaultBlobStore {
  Future<String?> read(String key);
  Future<void> write(String key, String value);
  Future<void> delete(String key);
}

/// [VaultBlobStore] 的 flutter_secure_storage 实现（静默读写）。
class SecureStorageBlobStore implements VaultBlobStore {
  SecureStorageBlobStore([FlutterSecureStorage? storage])
      : _storage = storage ?? appSecureStorage;

  final FlutterSecureStorage _storage;

  @override
  Future<String?> read(String key) => _storage.read(key: key);

  @override
  Future<void> write(String key, String value) =>
      _storage.write(key: key, value: value);

  @override
  Future<void> delete(String key) => _storage.delete(key: key);
}

/// [SecureSeedStore] 的硬件绑定后端（信封加密 + auth-bound KEK）——ROOTLESS。
///
/// 账户的 child mini-secret 经原生桥（Android RSA-2048 KEK +
/// BiometricPrompt.CryptoObject）加密成密文 blob，blob 由 [VaultBlobStore]
/// 静默持久化：
/// - 写（put）：公钥加密，**静默**，不弹生物识别（创建钱包 0 弹窗）。
/// - 读（read）：私钥解密，**触发一次强生物识别**（唯一的严档金库）。
///
/// KEK 按 walletIndex 绑定，密文 blob 按 accountId 分键——同一钱包多账户共享
/// 一把严档 KEK，各账户各存一份密文；[hasAccountKey] 只探 blob 存在性。
///
/// 原生错误按 [SecureSeedException] 子类型分类，供 [WalletManager] 决定
/// 中止（[AuthCancelled]）/ 报告设备私钥不可用（[SeedKeyInvalidated]）/ fail-closed
/// （[NoDeviceCredential]）/ 上抛（[SecureStoreUnavailable]）。
class HardwareBoundSeedVault implements SecureSeedStore {
  HardwareBoundSeedVault({
    MethodChannel? channel,
    VaultBlobStore? blobStore,
  })  : _channel = channel ?? const MethodChannel(_channelName),
        _blobStore = blobStore ?? SecureStorageBlobStore();

  static const String _channelName = 'org.citizenapp/hw_seed_vault';
  static const String _tierStrict = 'strict';

  final MethodChannel _channel;
  final VaultBlobStore _blobStore;

  static String _accountBlobKey(String accountId) =>
      'wallet_account_key_v1_$accountId';

  @override
  Future<SecureAuthStatus> authStatus() async {
    try {
      final res = await _channel.invokeMapMethod<String, dynamic>('authStatus');
      // 方案 A：创建热钱包要求已录入强生物识别（严档 child 是纯生物档）。
      final biometric = res?['strongBiometricEnrolled'] == true;
      return biometric
          ? SecureAuthStatus.available
          : SecureAuthStatus.noDeviceLock;
    } on PlatformException {
      return SecureAuthStatus.unsupported;
    } on MissingPluginException {
      return SecureAuthStatus.unsupported;
    }
  }

  @override
  Future<void> putAccountKey({
    required int walletIndex,
    required String accountId,
    required String childMiniSecretHex,
  }) =>
      _put(_accountBlobKey(accountId), walletIndex, childMiniSecretHex);

  @override
  Future<String?> readAccountKey({
    required int walletIndex,
    required String accountId,
  }) =>
      _read(_accountBlobKey(accountId), walletIndex);

  /// 只读密文 blob 判存在，**不调 `decrypt`**——因此不触发生物识别。
  @override
  Future<bool> hasAccountKey(String accountId) async {
    try {
      return await _blobStore.read(_accountBlobKey(accountId)) != null;
    } on PlatformException catch (e) {
      throw SecureStoreUnavailable(e.message ?? e.code);
    }
  }

  @override
  Future<void> deleteAccountKey({
    required int walletIndex,
    required String accountId,
  }) =>
      _deleteBlob(_accountBlobKey(accountId));

  @override
  Future<void> deleteWalletKey({required int walletIndex}) =>
      _deleteKek(walletIndex);

  Future<void> _put(
    String blobKey,
    int walletIndex,
    String plaintext,
  ) async {
    final String blob;
    try {
      final result =
          await _channel.invokeMethod<String>('encrypt', <String, dynamic>{
        'tier': _tierStrict,
        'walletIndex': walletIndex,
        'plaintext': plaintext,
      });
      if (result == null) {
        throw const SecureStoreUnavailable('加密返回空');
      }
      blob = result;
    } on PlatformException catch (e) {
      _mapAndThrow(e);
    }
    try {
      await _blobStore.write(blobKey, blob);
    } on PlatformException catch (e) {
      throw SecureStoreUnavailable(e.message ?? e.code);
    }
  }

  Future<String?> _read(String blobKey, int walletIndex) async {
    final String? blob;
    try {
      blob = await _blobStore.read(blobKey);
    } on PlatformException catch (e) {
      throw SecureStoreUnavailable(e.message ?? e.code);
    }
    if (blob == null) {
      return null;
    }
    try {
      return await _channel.invokeMethod<String>('decrypt', <String, dynamic>{
        'tier': _tierStrict,
        'walletIndex': walletIndex,
        'blob': blob,
      });
    } on PlatformException catch (e) {
      _mapAndThrow(e);
    }
  }

  /// 单账户删除只删该账户密文。KEK 按钱包共享，不能在这里连带删除。
  Future<void> _deleteBlob(String blobKey) async {
    try {
      await _blobStore.delete(blobKey);
    } on PlatformException catch (e) {
      throw SecureStoreUnavailable(e.message ?? e.code);
    }
  }

  /// 整钱包生命周期结束时才删除共享严档 KEK。
  Future<void> _deleteKek(int walletIndex) async {
    try {
      await _channel.invokeMethod<void>('deleteKey', <String, dynamic>{
        'tier': _tierStrict,
        'walletIndex': walletIndex,
      });
    } on PlatformException catch (e) {
      throw SecureStoreUnavailable(e.message ?? e.code);
    }
  }

  /// 原生错误码 → [SecureSeedException] 子类型。
  Never _mapAndThrow(PlatformException e) {
    final message = e.message ?? e.code;
    switch (e.code) {
      case 'keyPermanentlyInvalidated':
        throw SeedKeyInvalidated(message);
      case 'userCancelled':
      case 'lockout':
        throw AuthCancelled(message);
      case 'notEnrolled':
        throw NoDeviceCredential(message);
      default:
        throw SecureStoreUnavailable(message);
    }
  }
}
