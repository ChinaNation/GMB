import 'dart:convert';
import 'dart:io';
import 'dart:math';
import 'dart:typed_data';

import 'package:crypto/crypto.dart';
import 'package:cryptography/cryptography.dart';

import 'crypto/mls_boundary.dart';
import 'chat_models.dart';
import 'proto/chat_envelope.pb.dart';
import 'storage/chat_store.dart';
import 'transport/chat_transport.dart';

typedef ChatEnvelopeDeliverer = Future<ChatDeliveryResult> Function(
  ChatEnvelope envelope,
  List<int> envelopeBytes,
);

typedef ChatAttachmentPrepareUploader = Future<ChatAttachmentUploadPlan>
    Function({
  required String conversationId,
  required String attachmentId,
  required int manifestByteSize,
  required List<ChatAttachmentChunkDraft> chunks,
});

typedef ChatAttachmentObjectUploader = Future<void> Function({
  required Uri uploadUrl,
  required List<int> bytes,
  required String contentType,
});

typedef ChatAttachmentCompleteUploader = Future<void> Function(
  ChatAttachmentCompleteRequest input,
);

typedef ChatAttachmentDownloadPreparer = Future<ChatAttachmentDownloadPlan>
    Function(ChatAttachmentDownloadRequest input);

typedef ChatAttachmentObjectDownloader = Future<List<int>> Function(
  Uri downloadUrl,
);

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
    required ChatAttachmentPrepareUploader prepareAttachmentUpload,
    required ChatAttachmentObjectUploader uploadAttachmentObject,
    required ChatAttachmentCompleteUploader completeAttachmentUpload,
    ChatLocalAttachmentSaver? saveLocalAttachment,
  }) async {
    final now = DateTime.now().millisecondsSinceEpoch;
    final attachmentId = _newAttachmentId(now);
    final algorithm = AesGcm.with256bits();
    final secretKey = await algorithm.newSecretKey();
    final keyBytes = await secretKey.extractBytes();
    final chunkBox = await algorithm.encrypt(
      attachment.bytes,
      secretKey: secretKey,
    );
    final chunkCipherBytes = chunkBox.cipherText;
    final uploadPlan = await prepareAttachmentUpload(
      conversationId: conversationId,
      attachmentId: attachmentId,
      // 真实 manifest 大小要等 object key 返回后才能确定；Worker 这里只做
      // 正数边界校验，完成阶段会用 R2 head 校验密文对象确实存在。
      manifestByteSize: 1,
      chunks: [
        ChatAttachmentChunkDraft(
          chunkId: 'chunk-001',
          byteSize: chunkCipherBytes.length,
        ),
      ],
    );
    if (uploadPlan.chunks.isEmpty) {
      throw StateError('Cloudflare 未返回附件分片上传目标');
    }
    final chunkTarget = uploadPlan.chunks.first;
    final chunkCipherHash = _sha256Hex(chunkCipherBytes);
    final manifestPlaintext = utf8.encode(
      jsonEncode({
        'type': 'gmb_chat_attachment_manifest_v1',
        'attachment_id': attachmentId,
        'file_name': attachment.fileName,
        'content_type': attachment.contentType,
        'clear_byte_size': attachment.bytes.length,
        'chunks': [
          {
            'chunk_id': chunkTarget.chunkId,
            'object_key': chunkTarget.objectKey,
            'cipher_sha256': chunkCipherHash,
            'cipher_byte_size': chunkCipherBytes.length,
            'nonce': _base64UrlEncode(chunkBox.nonce),
            'mac': _base64UrlEncode(chunkBox.mac.bytes),
          },
        ],
      }),
    );
    final manifestBox = await algorithm.encrypt(
      manifestPlaintext,
      secretKey: secretKey,
    );
    final manifestCipherBytes = manifestBox.cipherText;
    final manifestHash = _sha256Hex(manifestCipherBytes);

    await uploadAttachmentObject(
      uploadUrl: uploadPlan.manifestUploadUrl,
      bytes: manifestCipherBytes,
      contentType: 'application/octet-stream',
    );
    await uploadAttachmentObject(
      uploadUrl: chunkTarget.uploadUrl,
      bytes: chunkCipherBytes,
      contentType: 'application/octet-stream',
    );
    await completeAttachmentUpload(
      ChatAttachmentCompleteRequest(
        attachmentId: attachmentId,
        conversationId: conversationId,
        manifestObjectKey: uploadPlan.manifestObjectKey,
        manifestHash: manifestHash,
        chunkObjectKeys: [chunkTarget.objectKey],
      ),
    );

    final controlPlaintext = jsonEncode({
      'type': 'gmb_chat_attachment_v1',
      'attachment_id': attachmentId,
      'file_name': attachment.fileName,
      'content_type': attachment.contentType,
      'clear_byte_size': attachment.bytes.length,
      'algorithm': 'AES_GCM_256',
      'content_key': _base64UrlEncode(keyBytes),
      'manifest_object_key': uploadPlan.manifestObjectKey,
      'manifest_nonce': _base64UrlEncode(manifestBox.nonce),
      'manifest_mac': _base64UrlEncode(manifestBox.mac.bytes),
      'manifest_hash': manifestHash,
      'chunk_refs': [chunkTarget.objectKey],
    });
    await saveLocalAttachment?.call(
      conversationId: conversationId,
      attachmentId: attachmentId,
      fileName: attachment.fileName,
      contentType: attachment.contentType,
      bytes: attachment.bytes,
    );
    final outbound = await _crypto.encrypt(
      conversationId: conversationId,
      recipientAccount: recipientAccount,
      recipientKeyPackage: recipientKeyPackage,
      plaintext: utf8.encode(controlPlaintext),
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
        attachmentManifestHash:
            wireMessage.messageKind == MlsMessageKind.application
                ? manifestHash
                : '',
        chunkRefs: wireMessage.messageKind == MlsMessageKind.application
            ? [uploadPlan.manifestObjectKey, chunkTarget.objectKey]
            : const [],
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

  Future<int> fetchAndProcessPending({
    required Future<List<ChatPendingEncryptedEnvelope>> Function() fetchPending,
    required Future<void> Function(String envelopeId) ackEnvelope,
    Future<ChatDownloadedAttachment> Function(
      String conversationId,
      String controlPlaintext,
    )? cacheIncomingAttachment,
  }) async {
    final rows = await fetchPending();
    var processed = 0;
    for (final row in rows) {
      final result = await processIncomingEnvelopeBytes(row.envelopeBytes);
      if (result.accepted) {
        final plaintext = result.plaintext;
        if (plaintext != null && _isAttachmentControlPlaintext(plaintext)) {
          final cacheAttachment = cacheIncomingAttachment;
          if (cacheAttachment == null) {
            throw StateError('附件消息必须先缓存到本机后才能确认删除 Cloudflare 副本');
          }
          final envelope = ChatEnvelope.fromBuffer(row.envelopeBytes);
          await cacheAttachment(envelope.conversationId, plaintext);
        }
        await ackEnvelope(row.envelopeId);
        processed += 1;
      } else if (result.queuedPending) {
        processed += 1;
      }
    }
    return processed;
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
    required ChatAttachmentDownloadPreparer prepareAttachmentDownload,
    required ChatAttachmentObjectDownloader downloadAttachmentObject,
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
    if (cached != null) {
      return cached;
    }
    final downloadPlan = await prepareAttachmentDownload(
      ChatAttachmentDownloadRequest(
        attachmentId: control.attachmentId,
        conversationId: conversationId,
        manifestObjectKey: control.manifestObjectKey,
        manifestHash: control.manifestHash,
        chunkObjectKeys: control.chunkRefs,
      ),
    );
    final manifestCipherBytes = await downloadAttachmentObject(
      downloadPlan.manifestDownloadUrl,
    );
    final actualManifestHash = _sha256Hex(manifestCipherBytes);
    if (actualManifestHash != control.manifestHash.toLowerCase()) {
      throw StateError('附件 manifest 密文 hash 不匹配');
    }

    final algorithm = AesGcm.with256bits();
    final secretKey = SecretKey(control.contentKey);
    final manifestPlainBytes = await algorithm.decrypt(
      SecretBox(
        manifestCipherBytes,
        nonce: control.manifestNonce,
        mac: Mac(control.manifestMac),
      ),
      secretKey: secretKey,
    );
    final manifest = _AttachmentManifest.fromPlaintext(
      utf8.decode(manifestPlainBytes),
    );
    if (manifest.attachmentId != control.attachmentId) {
      throw StateError('附件 manifest 与控制消息不一致');
    }

    final downloadTargets = {
      for (final target in downloadPlan.chunks) target.objectKey: target,
    };
    final clearBytes = BytesBuilder(copy: false);
    for (final chunk in manifest.chunks) {
      final target = downloadTargets[chunk.objectKey];
      if (target == null) {
        throw StateError('附件分片下载计划缺少 ${chunk.objectKey}');
      }
      final chunkCipherBytes = await downloadAttachmentObject(
        target.downloadUrl,
      );
      if (_sha256Hex(chunkCipherBytes) != chunk.cipherSha256.toLowerCase()) {
        throw StateError('附件分片密文 hash 不匹配');
      }
      final chunkPlainBytes = await algorithm.decrypt(
        SecretBox(
          chunkCipherBytes,
          nonce: chunk.nonce,
          mac: Mac(chunk.mac),
        ),
        secretKey: secretKey,
      );
      clearBytes.add(chunkPlainBytes);
    }
    final bytes = clearBytes.takeBytes();
    if (bytes.length != manifest.clearByteSize ||
        bytes.length != control.clearByteSize) {
      throw StateError('附件明文字节数不匹配');
    }

    return saveAttachmentBytesToCache(
      conversationId: conversationId,
      attachmentId: control.attachmentId,
      fileName: control.fileName,
      contentType: control.contentType,
      bytes: bytes,
      cacheDirectory: cacheDirectory,
    );
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
    if (decoded is Map && decoded['type'] == 'gmb_chat_attachment_v1') {
      return ChatMessageKind.attachment;
    }
  } catch (_) {
    return ChatMessageKind.text;
  }
  return ChatMessageKind.text;
}

bool _isAttachmentControlPlaintext(String plaintext) {
  try {
    final decoded = jsonDecode(plaintext);
    return decoded is Map && decoded['type'] == 'gmb_chat_attachment_v1';
  } catch (_) {
    return false;
  }
}

String _sha256Hex(List<int> bytes) {
  return sha256.convert(bytes).toString();
}

String _base64UrlEncode(List<int> bytes) {
  return base64Url.encode(bytes).replaceAll('=', '');
}

List<int> _base64UrlDecode(String value) {
  final normalized = value.replaceAll('-', '+').replaceAll('_', '/');
  final padded = normalized.padRight(
    ((normalized.length + 3) ~/ 4) * 4,
    '=',
  );
  return base64.decode(padded);
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

List<String> _jsonStringList(Object? value, String fieldName) {
  if (value is! List || value.isEmpty) {
    throw FormatException('附件字段 $fieldName 必须是非空列表');
  }
  return value.map((item) => _jsonString(item, fieldName)).toList();
}

class _AttachmentControl {
  const _AttachmentControl({
    required this.attachmentId,
    required this.fileName,
    required this.contentType,
    required this.clearByteSize,
    required this.contentKey,
    required this.manifestObjectKey,
    required this.manifestNonce,
    required this.manifestMac,
    required this.manifestHash,
    required this.chunkRefs,
  });

  final String attachmentId;
  final String fileName;
  final String contentType;
  final int clearByteSize;
  final List<int> contentKey;
  final String manifestObjectKey;
  final List<int> manifestNonce;
  final List<int> manifestMac;
  final String manifestHash;
  final List<String> chunkRefs;

  factory _AttachmentControl.fromPlaintext(String plaintext) {
    final decoded = jsonDecode(plaintext);
    if (decoded is! Map || decoded['type'] != 'gmb_chat_attachment_v1') {
      throw const FormatException('不是有效的 Chat 附件控制消息');
    }
    final algorithm = _jsonString(decoded['algorithm'], 'algorithm');
    if (algorithm != 'AES_GCM_256') {
      throw FormatException('不支持的附件加密算法：$algorithm');
    }
    return _AttachmentControl(
      attachmentId: _jsonString(decoded['attachment_id'], 'attachment_id'),
      fileName: _jsonString(decoded['file_name'], 'file_name'),
      contentType: _jsonString(decoded['content_type'], 'content_type'),
      clearByteSize: _jsonInt(decoded['clear_byte_size'], 'clear_byte_size'),
      contentKey: _base64UrlDecode(_jsonString(
        decoded['content_key'],
        'content_key',
      )),
      manifestObjectKey: _jsonString(
        decoded['manifest_object_key'],
        'manifest_object_key',
      ),
      manifestNonce: _base64UrlDecode(_jsonString(
        decoded['manifest_nonce'],
        'manifest_nonce',
      )),
      manifestMac: _base64UrlDecode(_jsonString(
        decoded['manifest_mac'],
        'manifest_mac',
      )),
      manifestHash:
          _jsonString(decoded['manifest_hash'], 'manifest_hash').toLowerCase(),
      chunkRefs: _jsonStringList(decoded['chunk_refs'], 'chunk_refs'),
    );
  }
}

class _AttachmentManifest {
  const _AttachmentManifest({
    required this.attachmentId,
    required this.clearByteSize,
    required this.chunks,
  });

  final String attachmentId;
  final int clearByteSize;
  final List<_AttachmentManifestChunk> chunks;

  factory _AttachmentManifest.fromPlaintext(String plaintext) {
    final decoded = jsonDecode(plaintext);
    if (decoded is! Map ||
        decoded['type'] != 'gmb_chat_attachment_manifest_v1') {
      throw const FormatException('不是有效的 Chat 附件 manifest');
    }
    final rawChunks = decoded['chunks'];
    if (rawChunks is! List || rawChunks.isEmpty) {
      throw const FormatException('附件 manifest 缺少分片');
    }
    return _AttachmentManifest(
      attachmentId: _jsonString(decoded['attachment_id'], 'attachment_id'),
      clearByteSize: _jsonInt(decoded['clear_byte_size'], 'clear_byte_size'),
      chunks: rawChunks
          .whereType<Map>()
          .map(_AttachmentManifestChunk.fromJson)
          .toList(growable: false),
    );
  }
}

class _AttachmentManifestChunk {
  const _AttachmentManifestChunk({
    required this.objectKey,
    required this.cipherSha256,
    required this.nonce,
    required this.mac,
  });

  final String objectKey;
  final String cipherSha256;
  final List<int> nonce;
  final List<int> mac;

  factory _AttachmentManifestChunk.fromJson(Map<dynamic, dynamic> json) {
    return _AttachmentManifestChunk(
      objectKey: _jsonString(json['object_key'], 'object_key'),
      cipherSha256:
          _jsonString(json['cipher_sha256'], 'cipher_sha256').toLowerCase(),
      nonce: _base64UrlDecode(_jsonString(json['nonce'], 'nonce')),
      mac: _base64UrlDecode(_jsonString(json['mac'], 'mac')),
    );
  }
}
