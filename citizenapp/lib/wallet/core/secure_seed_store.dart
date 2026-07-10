/// 钱包密钥的硬件级安全存储抽象。
///
/// 公民 App 的热钱包 seed 与助记词必须绑定系统级用户认证（生物识别/设备
/// 密码），解密动作本身在 Keystore/Keychain 触发验证，密钥永不出硬件。本
/// 接口把这层能力从具体插件后端解耦，[WalletManager] 只依赖它。
///
/// 分两档金库（access control 语义不同，见 [HardwareBoundSeedVault]）：
/// - seedVault（严）：seed hex，增/删任一指纹即永久失效，保护高频签名路径。
/// - recoveryVault（宽）：助记词，跟随生物变更不失效、设备密码可兜底，
///   供 seed 失效后静默自愈（自愈编排在 [WalletManager] 层，本 store 不做）。
///
/// 本 store 只负责「存储 + 错误分类」，抛出的 [SecureSeedException] 子类型
/// 让上层区分「该自愈」「该中止」「无锁屏」。
abstract interface class SecureSeedStore {
  /// 设备认证能力，仅供 UI 文案参考；D3 硬门禁以实际读写抛出的
  /// [NoDeviceCredential] 为准，不依赖本方法。
  Future<SecureAuthStatus> authStatus();

  /// 写入指定钱包的 seed 到严档金库；触发一次系统认证。
  Future<void> putSeed(int walletIndex, String seedHex);

  /// 从严档金库读取 seed；触发系统认证。
  ///
  /// - 用户取消/超时 → 抛 [AuthCancelled]（中止，绝不自愈）。
  /// - KEK 失效（换/加指纹等）→ 抛 [SeedKeyInvalidated]（上层从助记词自愈）。
  /// - 条目不存在 → 返回 `null`。
  Future<String?> readSeed(int walletIndex);

  /// 删除指定钱包的 seed 条目，连带释放其 keystore key。
  ///
  /// 自愈重派生前必须先删失效条目，`putSeed` 才会生成全新的有效 key。
  Future<void> deleteSeed(int walletIndex);

  /// 写入指定钱包的助记词到宽档金库；触发一次系统认证。
  Future<void> putMnemonic(int walletIndex, String mnemonic);

  /// 从宽档金库读取助记词；触发系统认证。
  ///
  /// - 用户取消/超时 → 抛 [AuthCancelled]。
  /// - 条目不存在 → 返回 `null`。
  Future<String?> readMnemonic(int walletIndex);

  /// 删除指定钱包的助记词条目。
  Future<void> deleteMnemonic(int walletIndex);
}

/// 设备认证能力（咨询用）。
enum SecureAuthStatus {
  /// 可用生物识别或设备密码认证。
  available,

  /// 设备未设置任何锁屏；创建/读取钱包应 fail-closed。
  noDeviceLock,

  /// 无相关硬件或平台不支持。
  unsupported,
}

/// 安全存储层的错误分类根。上层据具体子类型决定自愈 / 中止 / 提示。
sealed class SecureSeedException implements Exception {
  const SecureSeedException(this.message);

  final String message;

  @override
  String toString() => '$runtimeType: $message';
}

/// 严档 KEK 已失效（换/加指纹、锁屏变更等）——上层应从宽档助记词自愈。
final class SeedKeyInvalidated extends SecureSeedException {
  const SeedKeyInvalidated(super.message);
}

/// 用户取消或认证超时——中止当前操作，绝不触发自愈。
final class AuthCancelled extends SecureSeedException {
  const AuthCancelled(super.message);
}

/// 设备无锁屏，无法安全存取密钥——D3 fail-closed。
final class NoDeviceCredential extends SecureSeedException {
  const NoDeviceCredential(super.message);
}

/// 后端不可用或非上述三类的未知底层错误——上抛，不静默、不误判成自愈。
final class SecureStoreUnavailable extends SecureSeedException {
  const SecureStoreUnavailable(super.message);
}
