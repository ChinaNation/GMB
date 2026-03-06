class WalletSecureKeys {
  const WalletSecureKeys._();

  // Secret layer: mnemonic/private material must stay in secure storage.
  static String mnemonicV1(int walletId) {
    if (walletId <= 0) {
      throw ArgumentError.value(walletId, 'walletId', 'must be positive');
    }
    return 'wallet.secret.$walletId.mnemonic.v1';
  }

  static String sr25519PrivateKeyV1(int walletId) {
    if (walletId <= 0) {
      throw ArgumentError.value(walletId, 'walletId', 'must be positive');
    }
    return 'wallet.secret.$walletId.sr25519.v1';
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
