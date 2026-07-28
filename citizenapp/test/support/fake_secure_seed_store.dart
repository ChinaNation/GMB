import 'package:citizenapp/wallet/core/secure_seed_store.dart';

/// 内存版 [SecureSeedStore]（ROOTLESS），供 WalletManager 单测注入。
///
/// 只存账户 child mini-secret（按 accountId 分键），无母种子 / 助记词档。通过
/// 开关模拟硬件后端的三种异常路径：严档 KEK 失效、用户取消、无锁屏；并记录读
/// 写计数用于断言「每次签名都读一次密钥」「密钥失效 fail-closed」等行为。
class FakeSecureSeedStore implements SecureSeedStore {
  /// accountId → child mini-secret hex。
  final Map<String, String> accountKeys = <String, String>{};

  /// 这些账户的 [readAccountKey] 抛 [SeedKeyInvalidated]（模拟换/加指纹后 KEK 失效）。
  final Set<String> invalidatedAccountIds = <String>{};

  /// 这些账户的 [readAccountKey] 抛 [AuthCancelled]（模拟用户取消/超时）。
  final Set<String> cancelReads = <String>{};

  /// 设备无锁屏：所有写入 fail-closed，[authStatus] 返回 noDeviceLock。
  bool noDeviceLock = false;

  int readCount = 0;
  int putCount = 0;

  @override
  Future<SecureAuthStatus> authStatus() async {
    return noDeviceLock
        ? SecureAuthStatus.noDeviceLock
        : SecureAuthStatus.available;
  }

  @override
  Future<void> putAccountKey({
    required int walletIndex,
    required String accountId,
    required String childMiniSecretHex,
  }) async {
    if (noDeviceLock) {
      throw const NoDeviceCredential('设备无锁屏，无法写入密钥');
    }
    putCount++;
    accountKeys[accountId] = childMiniSecretHex;
    invalidatedAccountIds.remove(accountId);
  }

  @override
  Future<String?> readAccountKey({
    required int walletIndex,
    required String accountId,
  }) async {
    readCount++;
    if (cancelReads.contains(accountId)) {
      throw const AuthCancelled('用户取消认证');
    }
    if (invalidatedAccountIds.contains(accountId)) {
      throw const SeedKeyInvalidated('KEK 已失效');
    }
    return accountKeys[accountId];
  }

  /// 存在性判定：对齐真实现只探密文 blob 的语义 —— **不计入 [readCount]**
  /// （它不是一次密钥读取），也不受 KEK 失效 / 用户取消标记影响。
  @override
  Future<bool> hasAccountKey(String accountId) async =>
      accountKeys.containsKey(accountId);

  @override
  Future<void> deleteAccountKey({
    required int walletIndex,
    required String accountId,
  }) async {
    accountKeys.remove(accountId);
    invalidatedAccountIds.remove(accountId);
  }
}
