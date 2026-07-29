/// 钱包账户密钥的硬件级安全存储抽象（ROOTLESS 单账户模型，S7.1）。
///
/// 公民 App 无根：**只存账户的 child mini-secret**（`//index` 叶子私钥，
/// 账户0 = `//0`），**绝不存母种子、绝不存助记词**。助记词仅在创建时一次性
/// 展示供用户手抄 / 存入 citizenwallet，之后即丢弃。
///
/// child mini-secret 是 App 唯一的认证凭据，落入唯一的严档金库（纯生物识别
/// 保护）：读取（签名用）触发生物识别，取消 / 失败 → fail-closed 拒绝。写入
/// 静默。本接口把这层能力从具体插件后端解耦，[WalletManager] 只依赖它。
///
/// 本 store 只负责「存储 + 错误分类」，抛出的 [SecureSeedException] 子类型
/// 让上层区分「该中止」「无锁屏」「金库不可用」「密钥已失效」。
abstract interface class SecureSeedStore {
  /// 设备认证能力，仅供 UI 文案参考；D3 硬门禁以实际读写抛出的
  /// [NoDeviceCredential] 为准，不依赖本方法。
  Future<SecureAuthStatus> authStatus();

  /// 写入指定账户的 child mini-secret 到严档金库；**静默**，不触发生物识别。
  ///
  /// KEK 按 [walletIndex] 绑定（同钱包多账户共享一把 KEK），blob 按
  /// [accountId] 分键，故每个账户各有独立密文。
  Future<void> putAccountKey({
    required int walletIndex,
    required String accountId,
    required String childMiniSecretHex,
  });

  /// 从严档金库读取指定账户的 child mini-secret；**触发生物识别**。
  ///
  /// - 用户取消 / 超时 → 抛 [AuthCancelled]（中止，绝不吞没）。
  /// - KEK 失效或不存在 → 抛 [SeedKeyInvalidated]；查看私钥流程只报告设备
  ///   安全存储异常，不得索要助记词或绕过生物识别。
  /// - 条目不存在 → 返回 `null`。
  Future<String?> readAccountKey({
    required int walletIndex,
    required String accountId,
  });

  /// 指定账户的 child 条目**是否存在**——只探密文 blob，**不解密、不触发认证**。
  ///
  /// child 是纯生物档，真解密会弹生物识别；门控只需排除「有壳无钥」
  /// （钱包行在、密钥没了）的钱包，故用静默存在性判定，避免每次冷启动弹指纹。
  /// 后端不可用时抛 [SecureStoreUnavailable]，由上层走错误态而非判死。
  Future<bool> hasAccountKey(String accountId);

  /// 删除指定账户的 child 密文条目，不删除同钱包其它账户共享的 KEK。
  Future<void> deleteAccountKey({
    required int walletIndex,
    required String accountId,
  });

  /// 删除整只钱包共享的硬件 KEK。
  ///
  /// 多账户共享同一 [walletIndex] 的 KEK，因此删除单个账户时绝不能调用本方法；
  /// 只有整钱包删除或创建回滚时才能调用。
  Future<void> deleteWalletKey({required int walletIndex});
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

/// 安全存储层的错误分类根。上层据具体子类型决定中止 / 提示 / fail-closed。
sealed class SecureSeedException implements Exception {
  const SecureSeedException(this.message);

  final String message;

  @override
  String toString() => '$runtimeType: $message';
}

/// 严档 KEK 已失效（换/加指纹、锁屏变更等）。
///
/// 无根模型没有母种子 / 助记词可自愈——App 只存账户 child 私钥，密钥失效即不可
/// 从本机密文再生。上层必须 fail-closed，查看私钥流程不得索要助记词。
final class SeedKeyInvalidated extends SecureSeedException {
  const SeedKeyInvalidated(super.message);
}

/// 用户取消或认证超时——中止当前操作，绝不吞没。
final class AuthCancelled extends SecureSeedException {
  const AuthCancelled(super.message);
}

/// 设备无锁屏，无法安全存取密钥——D3 fail-closed。
final class NoDeviceCredential extends SecureSeedException {
  const NoDeviceCredential(super.message);
}

/// 后端不可用或非上述三类的未知底层错误——上抛，不静默、不误判。
final class SecureStoreUnavailable extends SecureSeedException {
  const SecureStoreUnavailable(super.message);
}
