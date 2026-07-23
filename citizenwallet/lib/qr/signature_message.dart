import 'package:citizenwallet/qr/qr_protocols.dart';

/// 唯一的签名原文拼接函数。与 citizenapp/lib/qr/signature_message.dart 逐字节一致。
///
/// 格式:
/// ```
/// QR_V1|<k>|<i>|<system 或空>|<e 或 0>|<principal>
/// ```
String buildSignatureMessage({
  required QrKind kind,
  required String id,
  String? system,
  int? expiresAt,
  required String principal,
}) {
  final sys = system ?? '';
  final exp = expiresAt ?? 0;
  if (!RegExp(r'^0x[0-9a-f]{64}$').hasMatch(principal)) {
    throw const FormatException('principal 必须是小写 0x 加 64 位十六进制');
  }
  final pp = principal.substring(2);
  return '${QrProtocols.v1}|${kind.code}|$id|$sys|$exp|$pp';
}
