class WuminLoginChallenge {
  const WuminLoginChallenge({
    required this.proto,
    required this.system,
    required this.requestId,
    required this.challenge,
    required this.nonce,
    required this.issuedAt,
    required this.expiresAt,
    required this.aud,
    required this.origin,
    required this.raw,
  });

  final String proto;
  final String system;
  final String requestId;
  final String challenge;
  final String nonce;
  final int issuedAt;
  final int expiresAt;
  final String aud;
  final String origin;
  final String raw;

  bool get isExpired => _nowEpochSeconds() > expiresAt;
  int get ttlSeconds => expiresAt - _nowEpochSeconds();

  static int _nowEpochSeconds() => DateTime.now().millisecondsSinceEpoch ~/ 1000;
}

class WuminLoginReceipt {
  const WuminLoginReceipt({
    required this.proto,
    required this.requestId,
    required this.account,
    required this.pubkey,
    required this.sigAlg,
    required this.signature,
    required this.signedAt,
  });

  final String proto;
  final String requestId;
  final String account;
  final String pubkey;
  final String sigAlg;
  final String signature;
  final int signedAt;

  Map<String, dynamic> toJson() {
    return {
      'proto': proto,
      'request_id': requestId,
      'account': account,
      'pubkey': pubkey,
      'sig_alg': sigAlg,
      'signature': signature,
      'signed_at': signedAt,
    };
  }
}
