// 公民 Chat 的 OpenMLS 边界模型。
//
// 本文件只定义 Dart 侧可测试的数据边界。真正的 OpenMLS 加解密由
// Rust OpenMLS native 边界实现；这里禁止自研密码学。

export 'mls_session.dart';

import 'mls_session.dart';

/// 本机 Chat 设备身份。
///
/// `ownerAccount` 是聊天账户/公民币收款账户，不是 Chat 密钥来源。
/// Chat 设备私钥必须由 OpenMLS/安全存储独立生成并保存在本机。
class ChatDevice {
  const ChatDevice({
    required this.ownerAccount,
    required this.deviceId,
    required this.devicePublicKeyHex,
  });

  /// 钱包账户作为聊天账户名和收款账户。
  final String ownerAccount;

  /// Chat 设备 ID，独立于钱包地址。
  final String deviceId;

  /// Chat 设备身份公钥 hex，不包含私钥。
  final String devicePublicKeyHex;

  /// 校验身份边界，避免把空账户或空公钥写入 Chat 路由记录。
  String? validate() {
    if (ownerAccount.trim().isEmpty) {
      return 'Chat 钱包聊天账户不能为空';
    }
    if (deviceId.trim().isEmpty) {
      return 'Chat 设备 ID 不能为空';
    }
    if (devicePublicKeyHex.trim().isEmpty) {
      return 'Chat 设备公钥不能为空';
    }
    final normalized = _stripHexPrefix(devicePublicKeyHex);
    if (normalized.length.isOdd || !_isHex(normalized)) {
      return 'Chat 设备公钥必须是合法 hex';
    }
    return null;
  }
}

/// OpenMLS KeyPackage。
class MlsKeyPackage {
  const MlsKeyPackage({
    required this.ownerAccount,
    required this.deviceId,
    required this.keyPackageId,
    required this.keyPackageBytes,
    required this.cipherSuite,
    required this.createdAtMillis,
    required this.expiresAtMillis,
    this.devicePublicKeyHex = '',
  });

  /// KeyPackage 所属的钱包聊天账户。
  final String ownerAccount;

  /// 发布设备 ID。
  final String deviceId;

  /// OpenMLS 设备签名公钥 hex，用于 Chat 路由记录和安全码展示。
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

  /// OpenMLS FFI 使用的 KeyPackage 十六进制编码。
  String get keyPackageHex => _bytesToHex(keyPackageBytes);
}

/// OpenMLS FFI 边界接口。
///
/// 后续实现必须调用成熟 OpenMLS 库，不允许在 Dart 中自研加密协议。
abstract class MlsCrypto {
  Future<MlsKeyPackage> createKeyPackage(ChatDevice identity);

  Future<MlsOutboundMessage> encrypt({
    required String conversationId,
    required String recipientAccount,
    MlsKeyPackage? recipientKeyPackage,
    required List<int> plaintext,
  });

  Future<List<int>> decrypt(MlsWireMessage message);

  Future<MlsInboundMessage> processIncoming(MlsWireMessage message);
}

/// 未注入 OpenMLS native 实现时的显式占位。
class UnsupportedMlsCrypto implements MlsCrypto {
  const UnsupportedMlsCrypto();

  @override
  Future<MlsKeyPackage> createKeyPackage(
    ChatDevice identity,
  ) async {
    throw UnimplementedError('OpenMLS native 实现未注入');
  }

  @override
  Future<MlsOutboundMessage> encrypt({
    required String conversationId,
    required String recipientAccount,
    MlsKeyPackage? recipientKeyPackage,
    required List<int> plaintext,
  }) async {
    throw UnimplementedError('OpenMLS native 实现未注入');
  }

  @override
  Future<List<int>> decrypt(MlsWireMessage message) async {
    throw UnimplementedError('OpenMLS native 实现未注入');
  }

  @override
  Future<MlsInboundMessage> processIncoming(MlsWireMessage message) async {
    throw UnimplementedError('OpenMLS native 实现未注入');
  }
}

String _bytesToHex(List<int> bytes) {
  return bytes.map((byte) => byte.toRadixString(16).padLeft(2, '0')).join();
}

String _stripHexPrefix(String value) {
  return value.startsWith('0x') ? value.substring(2) : value;
}

bool _isHex(String value) {
  return RegExp(r'^[0-9a-fA-F]+$').hasMatch(value);
}
