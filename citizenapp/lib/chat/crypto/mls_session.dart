import 'package:fixnum/fixnum.dart';

import '../proto/chat_envelope.pb.dart';

/// OpenMLS wire message 类型。
enum MlsMessageKind {
  welcome('welcome'),
  application('application');

  const MlsMessageKind(this.wireName);

  final String wireName;

  static MlsMessageKind fromWireName(String value) {
    for (final kind in values) {
      if (kind.wireName == value) {
        return kind;
      }
    }
    throw ArgumentError('未知 MLS 消息类型: $value');
  }
}

/// OpenMLS wire message bytes。
class MlsWireMessage {
  const MlsWireMessage({
    required this.wireBytes,
    required this.cipherSuite,
    required this.conversationId,
    required this.messageKind,
    this.ratchetTreeBytes,
  });

  /// OpenMLS 标准 wire bytes。
  final List<int> wireBytes;

  /// 生成该消息的 cipher suite。
  final String cipherSuite;

  /// 本地会话 ID，对应 MLS group id。
  final String conversationId;

  /// OpenMLS wire message 类型。
  final MlsMessageKind messageKind;

  /// 首次 Welcome 所需的 ratchet tree bytes。
  final List<int>? ratchetTreeBytes;

  /// Spike 阶段提交给节点的 hex 表达。
  String get wireHex => _bytesToHex(wireBytes);

  /// ratchet tree 的 hex 表达；非 Welcome 消息通常为空。
  String? get ratchetTreeHex {
    final bytes = ratchetTreeBytes;
    return bytes == null ? null : _bytesToHex(bytes);
  }

  /// 转为 GMB_CHAT_V1 外层 envelope。
  ///
  /// OpenMLS wire bytes 和 ratchet tree 都是密文/协议字节，
  /// Cloudflare 只在当前请求中转发，不解析也不保存其中内容。
  ChatEnvelope toEnvelope({
    required String envelopeId,
    required String senderAccount,
    required String recipientAccount,
    required String senderDeviceId,
    required int createdAtMillis,
    required int ttlMillis,
    List<int> encryptedMetadata = const [],
  }) {
    return ChatEnvelope(
      protocolVersion: 1,
      envelopeId: envelopeId,
      conversationId: conversationId,
      senderAccount: senderAccount,
      recipientAccount: recipientAccount,
      senderDeviceId: senderDeviceId,
      mlsWireMessage: wireBytes,
      encryptedMetadata: encryptedMetadata,
      createdAtMillis: Int64(createdAtMillis),
      ttlMillis: Int64(ttlMillis),
      mlsMessageKind: _toProtoMessageKind(messageKind),
      ratchetTree: ratchetTreeBytes ?? const [],
    );
  }
}

/// 一次发送产生的 MLS 输出。
///
/// 首次会话会同时返回 Welcome 和 application；已有会话只返回 application。
class MlsOutboundMessage {
  const MlsOutboundMessage({
    required this.conversationId,
    required this.applicationMessage,
    this.welcomeMessage,
  });

  final String conversationId;
  final MlsWireMessage? welcomeMessage;
  final MlsWireMessage applicationMessage;

  bool get createdNewSession => welcomeMessage != null;

  Iterable<MlsWireMessage> get wireMessages sync* {
    final welcome = welcomeMessage;
    if (welcome != null) {
      yield welcome;
    }
    yield applicationMessage;
  }
}

/// 收到 MLS wire message 后的处理结果。
class MlsInboundMessage {
  const MlsInboundMessage({
    required this.conversationId,
    required this.messageKind,
    this.plaintext,
  });

  final String conversationId;
  final MlsMessageKind messageKind;
  final List<int>? plaintext;

  bool get hasPlaintext => plaintext != null;
}

String _bytesToHex(List<int> bytes) {
  return bytes.map((byte) => byte.toRadixString(16).padLeft(2, '0')).join();
}

/// 从 GMB_CHAT_V1 外层 envelope 还原 OpenMLS wire message。
MlsWireMessage imMlsWireMessageFromEnvelope(ChatEnvelope envelope) {
  return MlsWireMessage(
    wireBytes: List<int>.from(envelope.mlsWireMessage),
    cipherSuite: '',
    conversationId: envelope.conversationId,
    messageKind: _fromProtoMessageKind(envelope.mlsMessageKind),
    ratchetTreeBytes: envelope.ratchetTree.isEmpty
        ? null
        : List<int>.from(envelope.ratchetTree),
  );
}

MlsWireMessageKind _toProtoMessageKind(MlsMessageKind kind) {
  return switch (kind) {
    MlsMessageKind.welcome => MlsWireMessageKind.MLS_WIRE_MESSAGE_KIND_WELCOME,
    MlsMessageKind.application =>
      MlsWireMessageKind.MLS_WIRE_MESSAGE_KIND_APPLICATION,
  };
}

MlsMessageKind _fromProtoMessageKind(MlsWireMessageKind kind) {
  if (kind == MlsWireMessageKind.MLS_WIRE_MESSAGE_KIND_WELCOME) {
    return MlsMessageKind.welcome;
  }
  if (kind == MlsWireMessageKind.MLS_WIRE_MESSAGE_KIND_APPLICATION) {
    return MlsMessageKind.application;
  }
  throw ArgumentError('未知 MLS envelope 消息类型: ${kind.name}');
}
