import 'package:wumin/qr/envelope.dart';

class SignResponseBody implements QrBody {
  const SignResponseBody({
    required this.pubkey,
    required this.sigAlg,
    required this.signature,
    required this.payloadHash,
    required this.signedAt,
  });

  final String pubkey;
  final String sigAlg;
  final String signature;
  final String payloadHash;
  final int signedAt;

  @override
  Map<String, dynamic> toJson() => <String, dynamic>{
        'pubkey': pubkey,
        'sig_alg': sigAlg,
        'signature': signature,
        'payload_hash': payloadHash,
        'signed_at': signedAt,
      };

  static SignResponseBody fromJson(Map<String, dynamic> data) {
    final pubkey = data['pubkey'];
    final sigAlg = data['sig_alg'];
    final signature = data['signature'];
    final payloadHash = data['payload_hash'];
    final signedAt = data['signed_at'];
    if (pubkey is! String || !pubkey.startsWith('0x')) {
      throw const FormatException('sign_response.pubkey 必填 0x hex');
    }
    if (sigAlg != 'sr25519') {
      throw const FormatException('sign_response.sig_alg 必须为 sr25519');
    }
    if (signature is! String || !signature.startsWith('0x')) {
      throw const FormatException('sign_response.signature 必填 0x hex');
    }
    if (payloadHash is! String || !payloadHash.startsWith('0x')) {
      throw const FormatException('sign_response.payload_hash 必填 0x hex');
    }
    if (signedAt is! int) {
      throw const FormatException('sign_response.signed_at 必填整数');
    }
    return SignResponseBody(
      pubkey: pubkey,
      sigAlg: sigAlg,
      signature: signature,
      payloadHash: payloadHash,
      signedAt: signedAt,
    );
  }
}
