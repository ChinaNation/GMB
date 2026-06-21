class WalletSecureKeys {
  const WalletSecureKeys._();

  /// 热钱包 seed（32 字节 hex）存储键。
  static String seedHexV1(int walletId) {
    if (walletId <= 0) {
      throw ArgumentError.value(walletId, 'walletId', 'must be positive');
    }
    return 'wallet.secret.$walletId.seed_hex.v1';
  }

  /// 助记词存储键。
  static String mnemonicV1(int walletId) {
    if (walletId <= 0) {
      throw ArgumentError.value(walletId, 'walletId', 'must be positive');
    }
    return 'wallet.secret.$walletId.mnemonic.v1';
  }
}
