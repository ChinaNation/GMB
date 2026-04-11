/// WUMIN_QR_V1 统一二维码协议常量。
///
/// 唯一事实源:`memory/05-architecture/qr-protocol-spec.md`
/// Golden fixtures:`memory/05-architecture/qr-protocol-fixtures/*.json`
///
/// 与 wuminapp/lib/qr/qr_protocols.dart 逐字节一致(两个独立 Flutter app,
/// 无代码依赖,靠 fixture 对齐)。
class QrProtocols {
  QrProtocols._();

  /// 唯一协议版本字符串。
  static const String v1 = 'WUMIN_QR_V1';
}

/// 统一 kind 枚举。snake_case 字面量用于 JSON 序列化。
enum QrKind {
  loginChallenge('login_challenge', temporary: true),
  loginReceipt('login_receipt', temporary: true),
  signRequest('sign_request', temporary: true),
  signResponse('sign_response', temporary: true),
  userContact('user_contact', temporary: false),
  userTransfer('user_transfer', temporary: true),
  userDuoqian('user_duoqian', temporary: false);

  const QrKind(this.wire, {required this.temporary});

  /// JSON 线上字面量(snake_case)。
  final String wire;

  /// `true` = 临时码(必填 id / issued_at / expires_at)
  /// `false` = 固定码(上述三字段直接不出现在 JSON 中)
  final bool temporary;

  /// 固定码 = 永久有效,JSON 不含时效字段。
  bool get fixed => !temporary;

  static QrKind fromWire(String wire) {
    for (final k in QrKind.values) {
      if (k.wire == wire) return k;
    }
    throw FormatException('未知 kind: $wire');
  }
}
