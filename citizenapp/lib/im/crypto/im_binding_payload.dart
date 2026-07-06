import 'dart:convert';
import 'dart:typed_data';

/// 公民 IM 钱包账户绑定 payload。
///
/// 钱包账户是用户可见聊天账户；IM 消息加密仍由独立设备密钥承担。
/// 此 payload 只用于让钱包签名确认“此设备属于此账户”。
class ImBindingPayload {
  const ImBindingPayload({
    required this.walletAccount,
    required this.imDeviceId,
    required this.imDevicePubkey,
    required this.expiresAtMillis,
    required this.nonce,
  });

  /// 用户可见聊天账户。
  final String walletAccount;

  /// 手机本地生成的 IM 设备 ID。
  final String imDeviceId;

  /// IM 设备公钥；真实 OpenMLS 接入后由密码模块提供。
  final String imDevicePubkey;

  /// 绑定凭证过期时间，毫秒时间戳。
  final int expiresAtMillis;

  /// 防重放 nonce。
  final String nonce;

  /// 构造与 node 端一致的 SCALE 签名载荷。
  Uint8List signingPayloadBytes() {
    final builder = BytesBuilder(copy: false)
      ..add(_scaleString(walletAccount))
      ..add(_scaleString(imDeviceId))
      ..add(_scaleString(imDevicePubkey))
      ..add(_u64Le(expiresAtMillis))
      ..add(_scaleString(nonce));
    return builder.toBytes();
  }

  /// SCALE 签名载荷 hex,供 QR_V1 sign_request 的 `b.d` 使用。
  String signingPayloadHex() => _hex(signingPayloadBytes());

  /// 转为提交给 Cloudflare mailbox 设备绑定接口的 JSON map。
  Map<String, Object?> toUnsignedJson() {
    return {
      'wallet_account': walletAccount,
      'im_device_id': imDeviceId,
      'im_device_pubkey': imDevicePubkey,
      'expires_at_millis': expiresAtMillis,
      'nonce': nonce,
    };
  }
}

Uint8List _scaleString(String value) {
  final bytes = utf8.encode(value);
  return Uint8List.fromList([..._scaleCompact(bytes.length), ...bytes]);
}

List<int> _scaleCompact(int value) {
  if (value < 0) {
    throw ArgumentError.value(value, 'value', 'SCALE compact 不允许负数');
  }
  if (value < 1 << 6) {
    return [value << 2];
  }
  if (value < 1 << 14) {
    final v = (value << 2) | 0x01;
    return [v & 0xff, (v >> 8) & 0xff];
  }
  if (value < 1 << 30) {
    final v = (value << 2) | 0x02;
    return [
      v & 0xff,
      (v >> 8) & 0xff,
      (v >> 16) & 0xff,
      (v >> 24) & 0xff,
    ];
  }
  throw ArgumentError.value(value, 'value', 'SCALE compact 超出本地支持范围');
}

List<int> _u64Le(int value) {
  if (value < 0) {
    throw ArgumentError.value(value, 'value', 'u64 不允许负数');
  }
  final out = List<int>.filled(8, 0);
  var current = value;
  for (var i = 0; i < out.length; i++) {
    out[i] = current & 0xff;
    current >>= 8;
  }
  return out;
}

String _hex(List<int> bytes) {
  final buffer = StringBuffer('0x');
  for (final byte in bytes) {
    buffer.write(byte.toRadixString(16).padLeft(2, '0'));
  }
  return buffer.toString();
}
