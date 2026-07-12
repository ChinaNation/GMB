import 'dart:convert';
import 'dart:io';
import 'dart:math';

import 'crypto/mls_boundary.dart';
import 'chat_models.dart';
import 'proto/chat_envelope.pb.dart';
import 'storage/chat_store.dart';
import 'transport/chat_transport.dart';

typedef ChatEnvelopeDeliverer = Future<ChatDeliveryResult> Function(
  ChatEnvelope envelope,
  List<int> envelopeBytes,
);

typedef ChatAttachmentDeviceSender = Future<void> Function({
  required String recipientAccount,
  required String conversationId,
  required String attachmentId,
  required String fileName,
  required String contentType,
  required List<int> bytes,
});

typedef ChatLocalAttachmentSaver = Future<void> Function({
  required String conversationId,
  required String attachmentId,
  required String fileName,
  required String contentType,
  required List<int> bytes,
});

/// 待发送的本机明文附件。
class ChatAttachmentDraft {
  const ChatAttachmentDraft({
    required this.fileName,
    required this.contentType,
    required this.bytes,
  });

  /// 用户本机可见文件名。该字段只会进入 OpenMLS 明文，不写入 Worker 明文表。
  final String fileName;

  /// 文件 MIME 类型。
  final String contentType;

  /// 附件明文字节，只允许在手机本地进入本方法。
  final List<int> bytes;
}

/// 已在本机解密并保存的附件。
class ChatDownloadedAttachment {
  const ChatDownloadedAttachment({
    required this.attachmentId,
    required this.fileName,
    required this.contentType,
    required this.clearByteSize,
    required this.filePath,
    required this.bytes,
  });

  /// OpenMLS 附件控制消息中的附件 ID。
  final String attachmentId;

  /// 用户可见文件名。
  final String fileName;

  /// 文件 MIME 类型。
  final String contentType;

  /// 明文字节数。
  final int clearByteSize;

  /// App 私有缓存中的保存路径。
  final String filePath;

  /// 已解密的明文字节。
  final List<int> bytes;
}

/// Chat 入站处理结果。
class ChatIncomingProcessResult {
  const ChatIncomingProcessResult({
    required this.envelopeId,
    required this.accepted,
    required this.queuedPending,
    this.plaintext,
  });

  final String envelopeId;
  final bool accepted;
  final bool queuedPending;
  final String? plaintext;
}

/// 公民 Chat 消息收发状态机。
///
/// 本类是聊天收发编排层。它不实现密码学，只负责把 OpenMLS native、
/// GMB_CHAT_V1 envelope、本地 Isar 和正式 transport 串起来。
class ChatFlow {
  const ChatFlow({
    required MlsCrypto crypto,
    required ChatStore store,
    required ChatEnvelopeDeliverer deliverer,
    this.defaultTtlMillis = 30 * 24 * 60 * 60 * 1000,
  })  : _crypto = crypto,
        _store = store,
        _deliverer = deliverer;

  final MlsCrypto _crypto;
  final ChatStore _store;
  final ChatEnvelopeDeliverer _deliverer;
  final int defaultTtlMillis;

  Future<List<ChatDeliveryResult>> sendText({
    required String conversationId,
    required String senderAccount,
    required String recipientAccount,
    required String senderDeviceId,
    MlsKeyPackage? recipientKeyPackage,
    required String text,
  }) async {
    final now = DateTime.now().millisecondsSinceEpoch;
    final outbound = await _crypto.encrypt(
      conversationId: conversationId,
      recipientAccount: recipientAccount,
      recipientKeyPackage: recipientKeyPackage,
      plaintext: utf8.encode(text),
    );

    final results = <ChatDeliveryResult>[];
    var index = 0;
    for (final wireMessage in outbound.wireMessages) {
      final envelope = wireMessage.toEnvelope(
        envelopeId: _newEnvelopeId(conversationId, now, index),
        senderAccount: senderAccount,
        recipientAccount: recipientAccount,
        senderDeviceId: senderDeviceId,
        createdAtMillis: now + index,
        ttlMillis: defaultTtlMillis,
      );
      final envelopeBytes = envelope.writeToBuffer();
      final isApplication =
          wireMessage.messageKind == MlsMessageKind.application;
      if (isApplication) {
        await _store.saveOutgoingEnvelope(
          envelope: envelope,
          envelopeBytes: envelopeBytes,
          messageKind: ChatMessageKind.text,
          deliveryState: ChatMessageDeliveryState.queued,
          plaintext: text,
        );
      } else {
        await _store.queueOutgoingEnvelope(
          envelope: envelope,
          envelopeBytes: envelopeBytes,
          deliveryState: ChatMessageDeliveryState.queued,
        );
      }

      final result = await _deliverer(envelope, envelopeBytes);
      await _store.markOutgoingDelivery(
        envelopeId: envelope.envelopeId,
        state: result.state,
        errorMessage: result.errorMessage,
      );
      results.add(result);
      index += 1;
    }
    return results;
  }

  Future<List<ChatDeliveryResult>> sendAttachment({
    required String conversationId,
    required String senderAccount,
    required String recipientAccount,
    required String senderDeviceId,
    MlsKeyPackage? recipientKeyPackage,
    required ChatAttachmentDraft attachment,
    required ChatAttachmentDeviceSender sendDeviceAttachment,
    ChatLocalAttachmentSaver? saveLocalAttachment,
  }) async {
    final now = DateTime.now().millisecondsSinceEpoch;
    final attachmentId = _newAttachmentId(now);
    final controlPlaintext = jsonEncode({
      'type': 'gmb_chat_attachment_v2',
      'attachment_id': attachmentId,
      'file_name': attachment.fileName,
      'content_type': attachment.contentType,
      'clear_byte_size': attachment.bytes.length,
    });
    // 先建立或恢复 MLS 会话，避免首次会话缺少 KeyPackage 时重复发送附件字节。
    final outbound = await _crypto.encrypt(
      conversationId: conversationId,
      recipientAccount: recipientAccount,
      recipientKeyPackage: recipientKeyPackage,
      plaintext: utf8.encode(controlPlaintext),
    );
    // 附件由WebRTC DTLS端到端传输并直接保存到接收设备；Cloudflare只转发
    // SDP/ICE信令，不能收到附件字节、文件名或持久化引用。
    await sendDeviceAttachment(
      recipientAccount: recipientAccount,
      conversationId: conversationId,
      attachmentId: attachmentId,
      fileName: attachment.fileName,
      contentType: attachment.contentType,
      bytes: attachment.bytes,
    );

    await saveLocalAttachment?.call(
      conversationId: conversationId,
      attachmentId: attachmentId,
      fileName: attachment.fileName,
      contentType: attachment.contentType,
      bytes: attachment.bytes,
    );
    final results = <ChatDeliveryResult>[];
    var index = 0;
    for (final wireMessage in outbound.wireMessages) {
      final envelope = wireMessage.toEnvelope(
        envelopeId: _newEnvelopeId(conversationId, now, index),
        senderAccount: senderAccount,
        recipientAccount: recipientAccount,
        senderDeviceId: senderDeviceId,
        createdAtMillis: now + index,
        ttlMillis: defaultTtlMillis,
      );
      final envelopeBytes = envelope.writeToBuffer();
      final isApplication =
          wireMessage.messageKind == MlsMessageKind.application;
      if (isApplication) {
        await _store.saveOutgoingEnvelope(
          envelope: envelope,
          envelopeBytes: envelopeBytes,
          messageKind: ChatMessageKind.attachment,
          deliveryState: ChatMessageDeliveryState.queued,
          plaintext: controlPlaintext,
        );
      } else {
        await _store.queueOutgoingEnvelope(
          envelope: envelope,
          envelopeBytes: envelopeBytes,
          deliveryState: ChatMessageDeliveryState.queued,
        );
      }

      final result = await _deliverer(envelope, envelopeBytes);
      await _store.markOutgoingDelivery(
        envelopeId: envelope.envelopeId,
        state: result.state,
        errorMessage: result.errorMessage,
      );
      results.add(result);
      index += 1;
    }
    return results;
  }

  Future<ChatIncomingProcessResult> processIncomingEnvelopeBytes(
    List<int> envelopeBytes,
  ) async {
    final envelope = ChatEnvelope.fromBuffer(envelopeBytes);
    final wireMessage = imMlsWireMessageFromEnvelope(envelope);
    try {
      final inbound = await _crypto.processIncoming(wireMessage);
      if (inbound.messageKind == MlsMessageKind.welcome) {
        final pending =
            await _store.takePendingInbound(envelope.conversationId);
        for (final item in pending) {
          await processIncomingEnvelopeBytes(item.writeToBuffer());
        }
        return ChatIncomingProcessResult(
          envelopeId: envelope.envelopeId,
          accepted: true,
          queuedPending: false,
        );
      }

      final plaintext = utf8.decode(inbound.plaintext ?? const []);
      await _store.saveIncomingEnvelope(
        envelope: envelope,
        envelopeBytes: envelopeBytes,
        messageKind: _messageKindFromPlaintext(plaintext),
        plaintext: plaintext,
      );
      return ChatIncomingProcessResult(
        envelopeId: envelope.envelopeId,
        accepted: true,
        queuedPending: false,
        plaintext: plaintext,
      );
    } catch (error) {
      if (wireMessage.messageKind == MlsMessageKind.application) {
        await _store.savePendingInbound(
          envelope: envelope,
          envelopeBytes: envelopeBytes,
          reason: error.toString(),
        );
        return ChatIncomingProcessResult(
          envelopeId: envelope.envelopeId,
          accepted: false,
          queuedPending: true,
        );
      }
      rethrow;
    }
  }

  static Future<ChatDeliveryResult> deliverWithTransport({
    required ChatTransport transport,
    required ChatEnvelope envelope,
  }) {
    return transport.sendEncryptedEnvelope(
      envelopeId: envelope.envelopeId,
      envelopeBytes: envelope.writeToBuffer(),
    );
  }

  static Future<ChatDownloadedAttachment> downloadAttachment({
    required String conversationId,
    required String controlPlaintext,
    required Directory cacheDirectory,
  }) async {
    final control = _AttachmentControl.fromPlaintext(controlPlaintext);
    final cached = await readCachedAttachment(
      conversationId: conversationId,
      attachmentId: control.attachmentId,
      fileName: control.fileName,
      contentType: control.contentType,
      clearByteSize: control.clearByteSize,
      cacheDirectory: cacheDirectory,
    );
    if (cached != null) return cached;
    throw StateError('附件尚未完成设备间传输');
  }

  static Future<ChatDownloadedAttachment> saveAttachmentBytesToCache({
    required String conversationId,
    required String attachmentId,
    required String fileName,
    required String contentType,
    required List<int> bytes,
    required Directory cacheDirectory,
  }) async {
    final file = _attachmentCacheFile(
      cacheDirectory: cacheDirectory,
      conversationId: conversationId,
      attachmentId: attachmentId,
      fileName: fileName,
    );
    await file.parent.create(recursive: true);
    await file.writeAsBytes(bytes, flush: true);
    return ChatDownloadedAttachment(
      attachmentId: attachmentId,
      fileName: fileName,
      contentType: contentType,
      clearByteSize: bytes.length,
      filePath: file.path,
      bytes: bytes,
    );
  }

  static Future<ChatDownloadedAttachment?> readCachedAttachment({
    required String conversationId,
    required String attachmentId,
    required String fileName,
    required String contentType,
    required int clearByteSize,
    required Directory cacheDirectory,
  }) async {
    final file = _attachmentCacheFile(
      cacheDirectory: cacheDirectory,
      conversationId: conversationId,
      attachmentId: attachmentId,
      fileName: fileName,
    );
    if (!await file.exists()) {
      return null;
    }
    final bytes = await file.readAsBytes();
    if (bytes.length != clearByteSize) {
      return null;
    }
    return ChatDownloadedAttachment(
      attachmentId: attachmentId,
      fileName: fileName,
      contentType: contentType,
      clearByteSize: bytes.length,
      filePath: file.path,
      bytes: bytes,
    );
  }
}

String _newEnvelopeId(String conversationId, int millis, int index) {
  final normalized = conversationId.replaceAll(RegExp(r'[^a-zA-Z0-9_.-]'), '_');
  return '$normalized-$millis-$index';
}

String _newAttachmentId(int millis) {
  final random = Random.secure();
  final suffix = List<int>.generate(8, (_) => random.nextInt(256))
      .map((byte) => byte.toRadixString(16).padLeft(2, '0'))
      .join();
  return 'att-$millis-$suffix';
}

ChatMessageKind _messageKindFromPlaintext(String plaintext) {
  try {
    final decoded = jsonDecode(plaintext);
    if (decoded is Map && decoded['type'] == 'gmb_chat_attachment_v2') {
      return ChatMessageKind.attachment;
    }
  } catch (_) {
    return ChatMessageKind.text;
  }
  return ChatMessageKind.text;
}

String _safePath(String value) {
  return value.replaceAll(RegExp(r'[^a-zA-Z0-9_.-]'), '_');
}

String _safeFileName(String value) {
  final cleaned = value
      .split(RegExp(r'[/\\]'))
      .last
      .replaceAll(RegExp(r'[^a-zA-Z0-9_.() -]'), '_')
      .trim();
  return cleaned.isEmpty ? 'attachment.bin' : cleaned;
}

File _attachmentCacheFile({
  required Directory cacheDirectory,
  required String conversationId,
  required String attachmentId,
  required String fileName,
}) {
  final targetDirectory = Directory(
    '${cacheDirectory.path}/${_safePath(conversationId)}/${_safePath(attachmentId)}',
  );
  return File('${targetDirectory.path}/${_safeFileName(fileName)}');
}

int _jsonInt(Object? value, String fieldName) {
  if (value is int) {
    return value;
  }
  if (value is num && value.isFinite) {
    return value.toInt();
  }
  throw FormatException('附件字段 $fieldName 必须是整数');
}

String _jsonString(Object? value, String fieldName) {
  if (value is String && value.isNotEmpty) {
    return value;
  }
  throw FormatException('附件字段 $fieldName 必须是非空字符串');
}

class _AttachmentControl {
  const _AttachmentControl({
    required this.attachmentId,
    required this.fileName,
    required this.contentType,
    required this.clearByteSize,
  });

  final String attachmentId;
  final String fileName;
  final String contentType;
  final int clearByteSize;
  factory _AttachmentControl.fromPlaintext(String plaintext) {
    final decoded = jsonDecode(plaintext);
    if (decoded is! Map || decoded['type'] != 'gmb_chat_attachment_v2') {
      throw const FormatException('不是有效的 Chat 附件控制消息');
    }
    return _AttachmentControl(
      attachmentId: _jsonString(decoded['attachment_id'], 'attachment_id'),
      fileName: _jsonString(decoded['file_name'], 'file_name'),
      contentType: _jsonString(decoded['content_type'], 'content_type'),
      clearByteSize: _jsonInt(decoded['clear_byte_size'], 'clear_byte_size'),
    );
  }
}
