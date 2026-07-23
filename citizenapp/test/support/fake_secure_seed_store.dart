import 'package:citizenapp/wallet/core/secure_seed_store.dart';

/// 内存版 [SecureSeedStore]，供 WalletManager 单测注入。
///
/// 通过开关模拟硬件后端的三种异常路径：严档 KEK 失效、用户取消、无锁屏；
/// 并记录读写计数用于断言"每次签名都读一次 seed""自愈发生了 re-put"等行为。
class FakeSecureSeedStore implements SecureSeedStore {
  final Map<int, String> seeds = <int, String>{};
  final Map<int, String> mnemonics = <int, String>{};

  /// 这些钱包的 [readSeed] 抛 [SeedKeyInvalidated]（模拟换/加指纹后 KEK 失效）。
  final Set<int> invalidatedSeeds = <int>{};

  /// 这些钱包的 [readSeed] 抛 [AuthCancelled]（模拟用户取消/超时）。
  final Set<int> cancelSeedReads = <int>{};

  /// 设备无锁屏：所有写入 fail-closed，[authStatus] 返回 noDeviceLock。
  bool noDeviceLock = false;

  int readSeedCount = 0;
  int putSeedCount = 0;

  @override
  Future<SecureAuthStatus> authStatus() async {
    return noDeviceLock
        ? SecureAuthStatus.noDeviceLock
        : SecureAuthStatus.available;
  }

  @override
  Future<void> putSeed(int walletIndex, String seedHex) async {
    if (noDeviceLock) {
      throw const NoDeviceCredential('设备无锁屏，无法写入密钥');
    }
    putSeedCount++;
    seeds[walletIndex] = seedHex;
    // 写入即视为 KEK 已重建，清除失效标记（自愈重封装后应可正常读取）。
    invalidatedSeeds.remove(walletIndex);
  }

  @override
  Future<String?> readSeed(int walletIndex) async {
    readSeedCount++;
    if (cancelSeedReads.contains(walletIndex)) {
      throw const AuthCancelled('用户取消认证');
    }
    if (invalidatedSeeds.contains(walletIndex)) {
      throw const SeedKeyInvalidated('KEK 已失效');
    }
    return seeds[walletIndex];
  }

  /// 存在性判定：对齐真实现只探密文 blob 的语义 —— **不计入 [readSeedCount]**
  /// （它不是一次 seed 读取），也不受 KEK 失效 / 用户取消标记影响。
  @override
  Future<bool> hasSeed(int walletIndex) async => seeds.containsKey(walletIndex);

  @override
  Future<void> deleteSeed(int walletIndex) async {
    seeds.remove(walletIndex);
    invalidatedSeeds.remove(walletIndex);
  }

  @override
  Future<void> putMnemonic(int walletIndex, String mnemonic) async {
    if (noDeviceLock) {
      throw const NoDeviceCredential('设备无锁屏，无法写入密钥');
    }
    mnemonics[walletIndex] = mnemonic;
  }

  @override
  Future<String?> readMnemonic(int walletIndex) async {
    if (cancelSeedReads.contains(walletIndex)) {
      throw const AuthCancelled('用户取消认证');
    }
    return mnemonics[walletIndex];
  }

  @override
  Future<void> deleteMnemonic(int walletIndex) async {
    mnemonics.remove(walletIndex);
  }
}
