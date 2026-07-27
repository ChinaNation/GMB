/// SecureStorage 键命名单源。
///
/// 冷钱包按钱包(master)存一份 master 种子 + 助记词(AES-256-GCM 密文)。键以
/// master 指纹（= 账户0 `//0` 的 accountId，公开值，非机密）拼接。账户不单独
/// 持久化密钥,签名/私钥导出时按 accountIndex 从种子现场派生。
class WalletSecureKeys {
  const WalletSecureKeys._();

  static final RegExp _masterIdPattern = RegExp(r'^0x[0-9a-f]{64}$');

  /// 钱包(master) mini-secret 种子（AES-256-GCM 密文）存储键。
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
