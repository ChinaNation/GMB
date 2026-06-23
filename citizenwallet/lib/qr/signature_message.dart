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
  final pp = principal.startsWith('0x') || principal.startsWith('0X')
      ? principal.substring(2).toLowerCase()
      : principal.toLowerCase();
  return '${QrProtocols.v1}|${kind.code}|$id|$sys|$exp|$pp';
}
