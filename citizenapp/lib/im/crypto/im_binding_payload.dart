import 'dart:typed_data';

import 'package:citizenapp/signer/signing.dart';

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
      ..add(scaleString(walletAccount))
      ..add(scaleString(imDeviceId))
      ..add(scaleString(imDevicePubkey))
      ..add(u64Le(expiresAtMillis))
      ..add(scaleString(nonce));
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

String _hex(List<int> bytes) {
  final buffer = StringBuffer('0x');
  for (final byte in bytes) {
    buffer.write(byte.toRadixString(16).padLeft(2, '0'));
  }
  return buffer.toString();
}
