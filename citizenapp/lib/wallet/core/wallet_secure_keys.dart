class WalletSecureKeys {
  const WalletSecureKeys._();

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
