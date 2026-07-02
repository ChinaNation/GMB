import 'package:fixnum/fixnum.dart';

import '../proto/im_envelope.pb.dart';

/// OpenMLS wire message 类型。
enum ImMlsMessageKind {
  welcome('welcome'),
  application('application');

  const ImMlsMessageKind(this.wireName);

  final String wireName;

  static ImMlsMessageKind fromWireName(String value) {
    for (final kind in values) {
      if (kind.wireName == value) {
        return kind;
      }
    }
    throw ArgumentError('未知 MLS 消息类型: $value');
  }
}

/// OpenMLS wire message bytes。
class ImMlsWireMessage {
  const ImMlsWireMessage({
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
  final ImMlsMessageKind messageKind;

  /// 首次 Welcome 所需的 ratchet tree bytes。
  final List<int>? ratchetTreeBytes;

  /// Spike 阶段提交给节点的 hex 表达。
  String get wireHex => _bytesToHex(wireBytes);

  /// ratchet tree 的 hex 表达；非 Welcome 消息通常为空。
  String? get ratchetTreeHex {
    final bytes = ratchetTreeBytes;
    return bytes == null ? null : _bytesToHex(bytes);
  }

  /// 转为 GMB_IM_V1 外层 envelope。
  ///
  /// OpenMLS wire bytes 和 ratchet tree 都是密文/协议字节，
  /// 节点只负责保存与转发，不解析其中内容。
  ImEnvelope toEnvelope({
    required String envelopeId,
    required String senderChatAccount,
    required String recipientChatAccount,
    required String senderDeviceId,
    required int createdAtMillis,
    required int ttlMillis,
    String ackPolicy = 'account_ack',
    List<int> encryptedMetadata = const [],
    String attachmentManifestHash = '',
    List<String> chunkRefs = const [],
  }) {
    return ImEnvelope(
      protocolVersion: 1,
      envelopeId: envelopeId,
      conversationId: conversationId,
      senderChatAccount: senderChatAccount,
      recipientChatAccount: recipientChatAccount,
      senderDeviceId: senderDeviceId,
      mlsWireMessage: wireBytes,
      encryptedMetadata: encryptedMetadata,
      attachmentManifestHash: attachmentManifestHash,
      chunkRefs: chunkRefs,
      createdAtMillis: Int64(createdAtMillis),
      ttlMillis: Int64(ttlMillis),
      ackPolicy: ackPolicy,
      mlsMessageKind: _toProtoMessageKind(messageKind),
      ratchetTree: ratchetTreeBytes ?? const [],
    );
  }
}

/// 一次发送产生的 MLS 输出。
///
/// 首次会话会同时返回 Welcome 和 application；已有会话只返回 application。
class ImMlsOutboundMessage {
  const ImMlsOutboundMessage({
    required this.conversationId,
    required this.applicationMessage,
    this.welcomeMessage,
  });

  final String conversationId;
  final ImMlsWireMessage? welcomeMessage;
  final ImMlsWireMessage applicationMessage;

  bool get createdNewSession => welcomeMessage != null;

  Iterable<ImMlsWireMessage> get wireMessages sync* {
    final welcome = welcomeMessage;
    if (welcome != null) {
      yield welcome;
    }
    yield applicationMessage;
  }
}

/// 收到 MLS wire message 后的处理结果。
class ImMlsInboundMessage {
  const ImMlsInboundMessage({
    required this.conversationId,
    required this.messageKind,
    this.plaintext,
  });

  final String conversationId;
  final ImMlsMessageKind messageKind;
  final List<int>? plaintext;

  bool get hasPlaintext => plaintext != null;
}

String _bytesToHex(List<int> bytes) {
  return bytes.map((byte) => byte.toRadixString(16).padLeft(2, '0')).join();
}

/// 从 GMB_IM_V1 外层 envelope 还原 OpenMLS wire message。
ImMlsWireMessage imMlsWireMessageFromEnvelope(ImEnvelope envelope) {
  return ImMlsWireMessage(
    wireBytes: List<int>.from(envelope.mlsWireMessage),
    cipherSuite: '',
    conversationId: envelope.conversationId,
    messageKind: _fromProtoMessageKind(envelope.mlsMessageKind),
    ratchetTreeBytes: envelope.ratchetTree.isEmpty
        ? null
        : List<int>.from(envelope.ratchetTree),
  );
}

ImMlsWireMessageKind _toProtoMessageKind(ImMlsMessageKind kind) {
  return switch (kind) {
    ImMlsMessageKind.welcome =>
      ImMlsWireMessageKind.IM_MLS_WIRE_MESSAGE_KIND_WELCOME,
    ImMlsMessageKind.application =>
      ImMlsWireMessageKind.IM_MLS_WIRE_MESSAGE_KIND_APPLICATION,
  };
}

ImMlsMessageKind _fromProtoMessageKind(ImMlsWireMessageKind kind) {
  if (kind == ImMlsWireMessageKind.IM_MLS_WIRE_MESSAGE_KIND_WELCOME) {
    return ImMlsMessageKind.welcome;
  }
  if (kind == ImMlsWireMessageKind.IM_MLS_WIRE_MESSAGE_KIND_APPLICATION) {
    return ImMlsMessageKind.application;
  }
  throw ArgumentError('未知 MLS envelope 消息类型: ${kind.name}');
}
