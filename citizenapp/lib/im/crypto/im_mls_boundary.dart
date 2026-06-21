// 公民 IM 的 OpenMLS 边界模型。
//
// 本文件只定义 Dart 侧可测试的数据边界。真正的 OpenMLS 加解密由
// Rust OpenMLS native 边界实现；这里禁止自研密码学。

export 'im_mls_session.dart';

import 'im_mls_session.dart';

/// 本机 IM 设备身份。
///
/// `walletChatAccount` 是聊天账户/公民币收款账户，不是 IM 密钥来源。
/// IM 设备私钥必须由 OpenMLS/安全存储独立生成并保存在本机。
class ImMlsDeviceIdentity {
  const ImMlsDeviceIdentity({
    required this.walletChatAccount,
    required this.deviceId,
    required this.devicePublicKeyHex,
  });

  /// 钱包账户作为聊天账户名和收款账户。
  final String walletChatAccount;

  /// IM 设备 ID，独立于钱包地址。
  final String deviceId;

  /// IM 设备身份公钥 hex，不包含私钥。
  final String devicePublicKeyHex;

  /// 校验身份边界，避免把空账户或空公钥写入 IM 路由记录。
  String? validate() {
    if (walletChatAccount.trim().isEmpty) {
      return 'IM 钱包聊天账户不能为空';
    }
    if (deviceId.trim().isEmpty) {
      return 'IM 设备 ID 不能为空';
    }
    if (devicePublicKeyHex.trim().isEmpty) {
      return 'IM 设备公钥不能为空';
    }
    final normalized = _stripHexPrefix(devicePublicKeyHex);
    if (normalized.length.isOdd || !_isHex(normalized)) {
      return 'IM 设备公钥必须是合法 hex';
    }
    return null;
  }
}

/// OpenMLS KeyPackage。
class ImMlsKeyPackage {
  const ImMlsKeyPackage({
    required this.ownerChatAccount,
    required this.deviceId,
    required this.keyPackageId,
    required this.keyPackageBytes,
    required this.cipherSuite,
    required this.createdAtMillis,
    required this.expiresAtMillis,
    this.devicePublicKeyHex = '',
    this.consumedAtMillis,
  });

  /// KeyPackage 所属的钱包聊天账户。
  final String ownerChatAccount;

  /// 发布设备 ID。
  final String deviceId;

  /// OpenMLS 设备签名公钥 hex，用于 IM 路由记录和安全码展示。
  final String devicePublicKeyHex;

  /// KeyPackage 全局去重 ID。
  final String keyPackageId;

  /// OpenMLS 标准 KeyPackage wire bytes。
  final List<int> keyPackageBytes;

  /// MLS cipher suite。
  final String cipherSuite;

  /// 创建时间，毫秒。
  final int createdAtMillis;

  /// 过期时间，毫秒。
  final int expiresAtMillis;

  /// 被远端消费的时间。
  final int? consumedAtMillis;

  /// node Spike RPC 使用的 hex 表达。
  String get keyPackageHex => _bytesToHex(keyPackageBytes);

  /// 转为 node 端 `PublishImKeyPackageRequest` JSON。
  Map<String, Object?> toPublishJson() {
    return {
      'owner_wallet_account': ownerChatAccount,
      'device_id': deviceId,
      'device_public_key_hex': devicePublicKeyHex,
      'key_package_id': keyPackageId,
      'key_package_hex': keyPackageHex,
      'cipher_suite': cipherSuite,
      'created_at_millis': createdAtMillis,
      'expires_at_millis': expiresAtMillis,
    };
  }

  /// 从 node 端 `ImKeyPackage` JSON 还原。
  factory ImMlsKeyPackage.fromNodeJson(Map<String, dynamic> json) {
    return ImMlsKeyPackage(
      ownerChatAccount: (json['owner_wallet_account'] ?? '').toString(),
      deviceId: (json['device_id'] ?? '').toString(),
      devicePublicKeyHex: (json['device_public_key_hex'] ?? '').toString(),
      keyPackageId: (json['key_package_id'] ?? '').toString(),
      keyPackageBytes: _hexToBytes((json['key_package_hex'] ?? '').toString()),
      cipherSuite: (json['cipher_suite'] ?? '').toString(),
      createdAtMillis: (json['created_at_millis'] as num?)?.toInt() ?? 0,
      expiresAtMillis: (json['expires_at_millis'] as num?)?.toInt() ?? 0,
      consumedAtMillis: (json['consumed_at_millis'] as num?)?.toInt(),
    );
  }
}

/// OpenMLS FFI 边界接口。
///
/// 后续实现必须调用成熟 OpenMLS 库，不允许在 Dart 中自研加密协议。
abstract class ImMlsCryptoBoundary {
  Future<ImMlsKeyPackage> createKeyPackage(ImMlsDeviceIdentity identity);

  Future<ImMlsOutboundMessage> encrypt({
    required String conversationId,
    required String recipientChatAccount,
    ImMlsKeyPackage? recipientKeyPackage,
    required List<int> plaintext,
  });

  Future<List<int>> decrypt(ImMlsWireMessage message);

  Future<ImMlsInboundMessage> processIncoming(ImMlsWireMessage message);
}

/// 未注入 OpenMLS native 实现时的显式占位。
class UnsupportedImMlsCrypto implements ImMlsCryptoBoundary {
  const UnsupportedImMlsCrypto();

  @override
  Future<ImMlsKeyPackage> createKeyPackage(
    ImMlsDeviceIdentity identity,
  ) async {
    throw UnimplementedError('OpenMLS native 实现未注入');
  }

  @override
  Future<ImMlsOutboundMessage> encrypt({
    required String conversationId,
    required String recipientChatAccount,
    ImMlsKeyPackage? recipientKeyPackage,
    required List<int> plaintext,
  }) async {
    throw UnimplementedError('OpenMLS native 实现未注入');
  }

  @override
  Future<List<int>> decrypt(ImMlsWireMessage message) async {
    throw UnimplementedError('OpenMLS native 实现未注入');
  }

  @override
  Future<ImMlsInboundMessage> processIncoming(ImMlsWireMessage message) async {
    throw UnimplementedError('OpenMLS native 实现未注入');
  }
}

String _bytesToHex(List<int> bytes) {
  return bytes.map((byte) => byte.toRadixString(16).padLeft(2, '0')).join();
}

List<int> _hexToBytes(String value) {
  final normalized = _stripHexPrefix(value);
  if (normalized.length.isOdd) {
    throw const FormatException('IM MLS hex 长度必须为偶数');
  }
  if (!_isHex(normalized)) {
    throw const FormatException('IM MLS hex 必须合法');
  }
  final bytes = <int>[];
  for (var i = 0; i < normalized.length; i += 2) {
    bytes.add(int.parse(normalized.substring(i, i + 2), radix: 16));
  }
  return bytes;
}

String _stripHexPrefix(String value) {
  return value.startsWith('0x') ? value.substring(2) : value;
}

bool _isHex(String value) {
  return RegExp(r'^[0-9a-fA-F]+$').hasMatch(value);
}
