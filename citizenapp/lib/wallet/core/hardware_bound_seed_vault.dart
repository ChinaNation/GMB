import 'package:flutter/services.dart';
import 'package:flutter_secure_storage/flutter_secure_storage.dart';
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
      : _storage = storage ?? const FlutterSecureStorage();

  final FlutterSecureStorage _storage;

  @override
  Future<String?> read(String key) => _storage.read(key: key);

  @override
  Future<void> write(String key, String value) =>
      _storage.write(key: key, value: value);

  @override
  Future<void> delete(String key) => _storage.delete(key: key);
}

/// [SecureSeedStore] 的硬件绑定后端（信封加密 + auth-bound KEK）。
///
/// seed / 助记词经原生桥（Android RSA-2048 KEK + BiometricPrompt.CryptoObject）
/// 加密成密文 blob，blob 由 [VaultBlobStore] 静默持久化：
/// - 写（put）：公钥加密，**静默**，不弹生物识别（创建钱包 0 弹窗）。
/// - 读（read）：私钥解密，触发一次系统认证 —— 严档 seed 仅强生物识别、宽档
///   助记词允许生物识别或设备凭证。
///
/// 原生错误按 [SecureSeedException] 子类型分类，供 [WalletManager] 决定
/// 自愈（[SeedKeyInvalidated]）/ 中止（[AuthCancelled]）/ fail-closed
/// （[NoDeviceCredential]）/ 上抛（[SecureStoreUnavailable]）。
class HardwareBoundSeedVault implements SecureSeedStore {
  HardwareBoundSeedVault({
    MethodChannel? channel,
    VaultBlobStore? blobStore,
  })  : _channel = channel ?? const MethodChannel(_channelName),
        _blobStore = blobStore ?? SecureStorageBlobStore();

  static const String _channelName = 'org.citizenapp/hw_seed_vault';
  static const String _tierStrict = 'strict';
  static const String _tierRecovery = 'recovery';

  final MethodChannel _channel;
  final VaultBlobStore _blobStore;

  static String _seedBlobKey(int walletIndex) =>
      'wallet_seed_env_v1_$walletIndex';
  static String _recoveryBlobKey(int walletIndex) =>
      'wallet_recovery_env_v1_$walletIndex';

  @override
  Future<SecureAuthStatus> authStatus() async {
    try {
      final res = await _channel.invokeMapMethod<String, dynamic>('authStatus');
      // 方案 A：创建热钱包要求已录入强生物识别（严档 seed 是纯生物档）。
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
  Future<void> putSeed(int walletIndex, String seedHex) =>
      _put(_tierStrict, _seedBlobKey(walletIndex), walletIndex, seedHex);

  @override
  Future<String?> readSeed(int walletIndex) =>
      _read(_tierStrict, _seedBlobKey(walletIndex), walletIndex);

  /// 只读密文 blob 判存在，**不调 `decrypt`**——因此不触发生物识别。
  @override
  Future<bool> hasSeed(int walletIndex) async {
    try {
      return await _blobStore.read(_seedBlobKey(walletIndex)) != null;
    } on PlatformException catch (e) {
      throw SecureStoreUnavailable(e.message ?? e.code);
    }
  }

  @override
  Future<void> deleteSeed(int walletIndex) =>
      _delete(_tierStrict, _seedBlobKey(walletIndex), walletIndex);

  @override
  Future<void> putMnemonic(int walletIndex, String mnemonic) =>
      _put(_tierRecovery, _recoveryBlobKey(walletIndex), walletIndex, mnemonic);

  @override
  Future<String?> readMnemonic(int walletIndex) =>
      _read(_tierRecovery, _recoveryBlobKey(walletIndex), walletIndex);

  @override
  Future<void> deleteMnemonic(int walletIndex) =>
      _delete(_tierRecovery, _recoveryBlobKey(walletIndex), walletIndex);

  Future<void> _put(
    String tier,
    String blobKey,
    int walletIndex,
    String plaintext,
  ) async {
    final String blob;
    try {
      final result =
          await _channel.invokeMethod<String>('encrypt', <String, dynamic>{
        'tier': tier,
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

  Future<String?> _read(String tier, String blobKey, int walletIndex) async {
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
        'tier': tier,
        'walletIndex': walletIndex,
        'blob': blob,
      });
    } on PlatformException catch (e) {
      _mapAndThrow(e);
    }
  }

  Future<void> _delete(String tier, String blobKey, int walletIndex) async {
    try {
      await _blobStore.delete(blobKey);
    } on PlatformException catch (e) {
      throw SecureStoreUnavailable(e.message ?? e.code);
    }
    try {
      await _channel.invokeMethod<void>('deleteKey', <String, dynamic>{
        'tier': tier,
        'walletIndex': walletIndex,
      });
    } on PlatformException catch (e) {
      // blob 已删，KEK 删除失败不致命（无 blob 也解不出），但上抛便于上层记录。
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
