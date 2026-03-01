class LoginException implements Exception {
  const LoginException(this.code, this.message);

  final String code;
  final String message;

  @override
  String toString() => '[$code] $message';
}

class LoginErrorCode {
  static const String invalidFormat = 'L1001';
  static const String invalidProtocol = 'L1002';
  static const String invalidSystem = 'L1003';
  static const String missingField = 'L1004';
  static const String invalidField = 'L1005';
  static const String expired = 'L1101';
  static const String replay = 'L1102';
  static const String unauthorizedAud = 'L1201';
  static const String unauthorizedOrigin = 'L1202';
  static const String walletMissing = 'L1301';
  static const String walletNotFound = 'L1302';
  static const String walletMismatch = 'L1303';
  static const String biometricUnavailable = 'L1401';
  static const String biometricRejected = 'L1402';
}
