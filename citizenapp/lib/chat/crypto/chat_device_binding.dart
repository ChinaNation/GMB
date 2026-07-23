import 'dart:typed_data';

import 'package:citizenapp/signer/signing.dart' as signing;

/// Chat 设备绑定证明。
///
/// 绑定摘要由无生物门禁的硬件 P-256 设备子钥签名。钱包主私钥、QR 和
/// CitizenWallet 均不得参与此流程。
class ChatDeviceBinding {
  const ChatDeviceBinding({
    required this.accountId,
    required this.deviceId,
    required this.devicePublicKey,
    required this.expiresAt,
    required this.nonce,
  });

  /// 当前 Worker session 对应的钱包账户。
  final String accountId;

  /// 本机 MLS 设备 ID。
  final String deviceId;

  /// 本机 MLS 设备签名公钥。
  final String devicePublicKey;

  /// 绑定凭证过期时间；Worker 按毫秒时间戳校验。
  final DateTime expiresAt;

  /// 一次性防重放 nonce。
  final String nonce;

  /// 与 Worker `buildChatDeviceBindingMessage` 逐字节一致的 32 字节摘要。
  Uint8List signingMessage() {
    final payload = <int>[
      ...signing.scaleString(accountId),
      ...signing.scaleString(deviceId),
      ...signing.scaleString(devicePublicKey),
      ...signing.u64Le(expiresAt.toUtc().millisecondsSinceEpoch),
      ...signing.scaleString(nonce),
    ];
    return signing.signingMessage(
      opTag: signing.kOpSignChatDeviceBind,
      scalePayload: payload,
    );
  }

  /// 绑定是否已过期。
  bool get isExpired => DateTime.now().isAfter(expiresAt);
}
