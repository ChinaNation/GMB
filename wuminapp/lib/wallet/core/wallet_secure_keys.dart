class WalletSecureKeys {
  const WalletSecureKeys._();

  /// 热钱包 seed（32 字节 hex）存储键。
  static String seedHexV1(int walletId) {
    if (walletId <= 0) {
      throw ArgumentError.value(walletId, 'walletId', 'must be positive');
    }
    return 'wallet.secret.$walletId.seed_hex.v1';
  }

  /// 热钱包助记词存储键。
  static String mnemonicV1(int walletId) {
    if (walletId <= 0) {
      throw ArgumentError.value(walletId, 'walletId', 'must be positive');
    }
    return 'wallet.secret.$walletId.mnemonic.v1';
  }

  static String sessionTokenV1(String scope) {
    final normalized = _normalizeScope(scope);
    return 'wallet.session.$normalized.token.v1';
  }

  static String sessionKeyV1(String scope) {
    final normalized = _normalizeScope(scope);
    return 'wallet.session.$normalized.key.v1';
  }

  static String _normalizeScope(String scope) {
    final normalized = scope.trim().toLowerCase();
    if (normalized.isEmpty ||
        !RegExp(r'^[a-z0-9._:-]{2,64}$').hasMatch(normalized)) {
      throw ArgumentError.value(scope, 'scope', 'invalid session scope');
    }
    return normalized;
  }
}
