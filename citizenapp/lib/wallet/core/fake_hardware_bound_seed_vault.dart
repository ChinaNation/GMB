import 'package:citizenapp/wallet/core/secure_seed_store.dart';

/// [SecureSeedStore] 的内存 fake，供单测与非真机场景注入
/// （[WalletManager.debugSeedStore]）。
///
/// 默认所有「认证」通过；可通过 [nextSeedReadError] / [nextMnemonicReadError]
/// 注入一次性错误，模拟 KEK 失效（[SeedKeyInvalidated]）/ 用户取消
/// （[AuthCancelled]）等自愈与中止路径。
class FakeHardwareBoundSeedVault implements SecureSeedStore {
  FakeHardwareBoundSeedVault({
    this.authStatusValue = SecureAuthStatus.available,
  });

  /// [authStatus] 的返回值，测试可改写模拟无锁屏 / 不支持。
  SecureAuthStatus authStatusValue;

  final Map<int, String> _seeds = <int, String>{};
  final Map<int, String> _mnemonics = <int, String>{};

  /// 下一次 [readSeed] 抛出的错误；抛出后自动清空（一次性）。
  SecureSeedException? nextSeedReadError;

  /// 下一次 [readMnemonic] 抛出的错误；抛出后自动清空（一次性）。
  SecureSeedException? nextMnemonicReadError;

  @override
  Future<SecureAuthStatus> authStatus() async => authStatusValue;

  @override
  Future<void> putSeed(int walletIndex, String seedHex) async {
    _seeds[walletIndex] = seedHex;
  }

  @override
  Future<String?> readSeed(int walletIndex) async {
    final error = nextSeedReadError;
    if (error != null) {
      nextSeedReadError = null;
      throw error;
    }
    return _seeds[walletIndex];
  }

  @override
  Future<void> deleteSeed(int walletIndex) async {
    _seeds.remove(walletIndex);
  }

  @override
  Future<void> putMnemonic(int walletIndex, String mnemonic) async {
    _mnemonics[walletIndex] = mnemonic;
  }

  @override
  Future<String?> readMnemonic(int walletIndex) async {
    final error = nextMnemonicReadError;
    if (error != null) {
      nextMnemonicReadError = null;
      throw error;
    }
    return _mnemonics[walletIndex];
  }

  @override
  Future<void> deleteMnemonic(int walletIndex) async {
    _mnemonics.remove(walletIndex);
  }
}
