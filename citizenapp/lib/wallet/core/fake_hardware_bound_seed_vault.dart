import 'package:citizenapp/wallet/core/secure_seed_store.dart';

/// [SecureSeedStore] 的内存 fake（ROOTLESS），供单测与非真机场景注入
/// （[WalletManager.debugSeedStore]）。
///
/// 只存账户 child mini-secret（按 accountId 分键），无母种子 / 助记词档。默认
/// 所有「认证」通过；可通过 [nextReadError] 注入一次性错误，模拟 KEK 失效
/// （[SeedKeyInvalidated]）/ 用户取消（[AuthCancelled]）等中止路径。
class FakeHardwareBoundSeedVault implements SecureSeedStore {
  FakeHardwareBoundSeedVault({
    this.authStatusValue = SecureAuthStatus.available,
  });

  /// [authStatus] 的返回值，测试可改写模拟无锁屏 / 不支持。
  SecureAuthStatus authStatusValue;

  /// accountId → child mini-secret hex。
  final Map<String, String> _accountKeys = <String, String>{};

  /// 下一次 [readAccountKey] 抛出的错误；抛出后自动清空（一次性）。
  SecureSeedException? nextReadError;

  @override
  Future<SecureAuthStatus> authStatus() async => authStatusValue;

  @override
  Future<void> putAccountKey({
    required int walletIndex,
    required String accountId,
    required String childMiniSecretHex,
  }) async {
    _accountKeys[accountId] = childMiniSecretHex;
  }

  @override
  Future<String?> readAccountKey({
    required int walletIndex,
    required String accountId,
  }) async {
    final error = nextReadError;
    if (error != null) {
      nextReadError = null;
      throw error;
    }
    return _accountKeys[accountId];
  }

  @override
  Future<bool> hasAccountKey(String accountId) async =>
      _accountKeys.containsKey(accountId);

  @override
  Future<void> deleteAccountKey({
    required int walletIndex,
    required String accountId,
  }) async {
    _accountKeys.remove(accountId);
  }
}
