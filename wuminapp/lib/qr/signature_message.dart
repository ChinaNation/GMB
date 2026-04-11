import 'package:wuminapp_mobile/qr/qr_protocols.dart';

/// 唯一的签名原文拼接函数。
///
/// 格式(逐字节与 wumin / Rust 后端 / TS 前端一致):
/// ```
/// WUMIN_QR_V1|<kind>|<id>|<system 或空>|<expires_at 或 0>|<principal>
/// ```
///
/// - `system` 为 null 时写空串
/// - `expiresAt` 为 null 时写 `0`
/// - `principal` 去掉 `0x` 前缀,小写 hex
///
/// 用于:
/// - `login_challenge.body.sys_sig`(principal = sys_pubkey)
/// - `login_receipt.body.signature`(principal = pubkey)
/// - `sign_response` 本身不用此函数签名 envelope,而是对 `payload_hex` 原字节签名
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
