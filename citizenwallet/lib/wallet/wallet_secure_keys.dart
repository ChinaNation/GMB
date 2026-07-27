/// SecureStorage 键命名单源。
///
/// model B 无根:设备**不存** master 种子 / 助记词,只存**每账户自己的
/// child mini-secret**(32B,AES-256-GCM 密文),按 accountId 拼键。签名时读该
/// 账户密钥现场重建、用后即弃;派生新账户需临时重建种子(要该钱包助记词)。
class WalletSecureKeys {
  const WalletSecureKeys._();

  static final RegExp _accountIdPattern = RegExp(r'^0x[0-9a-f]{64}$');

  /// 账户私钥(child mini-secret,AES-256-GCM 密文)存储键。按 accountId。
  static String accountMiniSecretV1(String accountId) {
    _requireAccountId(accountId);
    return 'wallet.account.$accountId.minisecret.v1';
  }

  static void _requireAccountId(String accountId) {
    if (!_accountIdPattern.hasMatch(accountId)) {
      throw ArgumentError.value(
        accountId,
        'accountId',
        'must be 0x + 64 lowercase hex',
      );
    }
  }
}
