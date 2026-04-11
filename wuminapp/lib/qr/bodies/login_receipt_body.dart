import 'package:wuminapp_mobile/qr/envelope.dart';

/// kind = login_receipt
///
/// 由冷钱包 wumin 生成,笔记本摄像头反扫后提交给 SFID/CPMS 后端验证。
class LoginReceiptBody implements QrBody {
  const LoginReceiptBody({
    required this.system,
    required this.pubkey,
    required this.sigAlg,
    required this.signature,
    required this.payloadHash,
    required this.signedAt,
  });

  final String system;
  final String pubkey;
  final String sigAlg;
  final String signature;
  final String payloadHash;
  final int signedAt;

  @override
  Map<String, dynamic> toJson() => <String, dynamic>{
        'system': system,
        'pubkey': pubkey,
        'sig_alg': sigAlg,
        'signature': signature,
        'payload_hash': payloadHash,
        'signed_at': signedAt,
      };

  static LoginReceiptBody fromJson(Map<String, dynamic> data) {
    final system = data['system'];
    final pubkey = data['pubkey'];
    final sigAlg = data['sig_alg'];
    final signature = data['signature'];
    final payloadHash = data['payload_hash'];
    final signedAt = data['signed_at'];
    if (system is! String || (system != 'sfid' && system != 'cpms')) {
      throw const FormatException('login_receipt.system 必须为 sfid 或 cpms');
    }
    if (pubkey is! String || !pubkey.startsWith('0x')) {
      throw const FormatException('login_receipt.pubkey 必填 0x hex');
    }
    if (sigAlg != 'sr25519') {
      throw const FormatException('login_receipt.sig_alg 必须为 sr25519');
    }
    if (signature is! String || !signature.startsWith('0x')) {
      throw const FormatException('login_receipt.signature 必填 0x hex');
    }
    if (payloadHash is! String || !payloadHash.startsWith('0x')) {
      throw const FormatException('login_receipt.payload_hash 必填 0x hex');
    }
    if (signedAt is! int) {
      throw const FormatException('login_receipt.signed_at 必填整数');
    }
    return LoginReceiptBody(
      system: system,
      pubkey: pubkey,
      sigAlg: sigAlg,
      signature: signature,
      payloadHash: payloadHash,
      signedAt: signedAt,
    );
  }
}
