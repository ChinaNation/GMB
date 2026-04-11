import 'package:wumin/qr/qr_protocols.dart';

/// 唯一的签名原文拼接函数。与 wuminapp/lib/qr/signature_message.dart 逐字节一致。
///
/// 格式:
/// ```
/// WUMIN_QR_V1|<kind>|<id>|<system 或空>|<expires_at 或 0>|<principal>
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
  return '${QrProtocols.v1}|${kind.wire}|$id|$sys|$exp|$pp';
}
