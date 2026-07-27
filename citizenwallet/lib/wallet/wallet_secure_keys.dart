/// SecureStorage 键命名单源。
///
/// 钱包升级为 HD 后，seed / 助记词**按钱包(master)存一份**，不再按账户；
/// 键以 master 指纹（= 账户0 的 accountId，公开值，非机密）拼接。账户由
/// master seed 按 accountIndex 派生，不单独持久化密钥。
class WalletSecureKeys {
  const WalletSecureKeys._();

  static final RegExp _masterIdPattern = RegExp(r'^0x[0-9a-f]{64}$');

  /// 钱包(master) mini-secret（32 字节 hex）存储键。
  static String masterSeedHexV1(String masterId) {
    _requireMasterId(masterId);
    return 'wallet.master.$masterId.seed_hex.v1';
  }

  /// 钱包(master) 助记词（AES-256-GCM 密文）存储键。
  static String masterMnemonicV1(String masterId) {
    _requireMasterId(masterId);
    return 'wallet.master.$masterId.mnemonic.v1';
  }

  static void _requireMasterId(String masterId) {
    if (!_masterIdPattern.hasMatch(masterId)) {
      throw ArgumentError.value(
        masterId,
        'masterId',
        'must be 0x + 64 lowercase hex',
      );
    }
  }
}
