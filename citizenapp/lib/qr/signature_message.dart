import 'package:citizenapp/qr/qr_protocols.dart';

/// 唯一的签名原文拼接函数。
///
/// 格式(逐字节与 CitizenWallet / Rust 后端 / TS 前端一致):
/// ```
/// QR_V1|<k>|<i>|<system 或空>|<e 或 0>|<principal>
/// ```
///
/// - `system` 为 null 时写空串
/// - `expiresAt` 为 null 时写 `0`
/// - `principal` 去掉 `0x` 前缀,小写 hex
///
/// 用于:
/// - `k=1,a=1` 登录请求的系统签名(principal = sys_pubkey)
/// - `k=2` 登录签名响应的 envelope 消息(principal = pubkey)
/// - 普通交易 `k=2` 响应本身不用此函数签 envelope,而是对请求载荷签名
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
