import 'dart:convert';
import 'dart:io';
import 'dart:math';

import 'crypto/mls_boundary.dart';
import 'chat_media_limits.dart';
import 'chat_models.dart';
import 'chat_payload.dart';
import 'proto/chat_envelope.pb.dart';
import 'storage/chat_store.dart';
import 'transport/chat_transport.dart';

typedef ChatEnvelopeDeliverer = Future<ChatDeliveryResult> Function(
  ChatEnvelope envelope,
  List<int> envelopeBytes,
);

/// 把本机源文件字节经 WebRTC 流式发给对端设备。传路径而非整块字节:大文件
/// (最大 5GB)绝不整块进内存,由发送端 openRead 分片 + 背压推送。
typedef ChatAttachmentDeviceSender = Future<void> Function({
  required String recipientAccount,
  required String conversationId,
  required String attachmentId,
  required String fileName,
  required String contentType,
  required String sourcePath,
  required int byteSize,
});

/// 发送方把自己发出的媒体自存一份到本机缓存,以便在会话里看到并支持上线补发。
typedef ChatLocalAttachmentSaver = Future<void> Function({
  required String conversationId,
  required String attachmentId,
  required String fileName,
  required String contentType,
  required String sourcePath,
  required int byteSize,
});

/// 登记一条待设备投递的媒体(字节未送达对方设备,留待上线补发)。
typedef ChatMediaPendingRecorder = Future<void> Function(String attachmentId);

/// 字节已送达对方设备(收到 WebRTC ack)后清除待投递登记。
typedef ChatMediaDeliveredMarker = Future<void> Function(String attachmentId);

/// 大媒体(>100MB)经 Cloudflare R2 瞬时中转的上传结果。
///
/// 客户端流式加密后上传密文到 R2,把 R2 对象键 + 一次性内容密钥 + 分块参数
/// 放进 E2E 控制消息;Cloudflare 只经手密文,拿不到内容密钥。
class ChatRelayDescriptor {
  const ChatRelayDescriptor({
    required this.relayObjectKey,
    required this.contentKeyB64,
    required this.chunkSize,
    required this.encSize,
  });

  final String relayObjectKey;
  final String contentKeyB64;
  final int chunkSize;
  final int encSize;
}

/// 把源文件流式加密上传到 Cloudflare R2 瞬时中转,返回描述子。仅 >100MB 走此路。
typedef ChatRelayUploader = Future<ChatRelayDescriptor> Function({
  required String conversationId,
  required String attachmentId,
  required ChatMediaDraft media,
  int recipientCount,
});

/// 待发送的本机明文媒体(图片 / 视频 / 文件)。
///
/// 承载**源文件路径**而非整块字节:发送走流式读盘,支持最大 5GB 且不 OOM。
class ChatMediaDraft {
  const ChatMediaDraft({
    required this.kind,
    required this.fileName,
    required this.contentType,
    required this.sourcePath,
    required this.byteSize,
    this.width,
    this.height,
    this.durationMs,
    this.blurhash,
  });

  /// 媒体类型:image / video / file。
  final ChatMessageKind kind;

  /// 用户本机可见文件名。该字段只会进入 OpenMLS 明文，不写入 Worker 明文表。
  final String fileName;

  /// 文件 MIME 类型。
  final String contentType;

  /// 本机源文件路径。字节从此路径流式读取,不整块载入内存。
  final String sourcePath;

  /// 源文件字节数(= File(sourcePath).length())。
  final int byteSize;

  /// image/video 像素宽高(可空;步骤2 采集时补齐)。
  final int? width;
  final int? height;

  /// video 时长毫秒(可空)。
  final int? durationMs;

  /// image/video 低清占位串(blurhash，可空;步骤2 生成)。
  final String? blurhash;
}

/// 已在本机缓存就绪的媒体句柄。
///
/// 只返回路径与大小,**不返回整块字节**:5GB 媒体不允许载入内存,读取由调用方
/// 按需流式进行。
class ChatDownloadedAttachment {
  const ChatDownloadedAttachment({
    required this.attachmentId,
    required this.fileName,
    required this.contentType,
    required this.clearByteSize,
    required this.filePath,
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
    final payload = ChatPayloadCodec.encode(ChatContent.text(text));
    final outbound = await _crypto.encrypt(
      conversationId: conversationId,
      recipientAccount: recipientAccount,
      recipientKeyPackage: recipientKeyPackage,
      plaintext: utf8.encode(payload),
    );
    return _deliverOutbound(
      outbound: outbound,
      conversationId: conversationId,
      senderAccount: senderAccount,
      recipientAccount: recipientAccount,
      senderDeviceId: senderDeviceId,
      nowMillis: now,
      messageKind: ChatMessageKind.text,
      payload: payload,
    );
  }

  /// 发送内置贴纸：只走控制信封(几十字节)，不经 WebRTC、不落缓存。
  Future<List<ChatDeliveryResult>> sendSticker({
    required String conversationId,
    required String senderAccount,
    required String recipientAccount,
    required String senderDeviceId,
    MlsKeyPackage? recipientKeyPackage,
    required String packId,
    required String stickerId,
  }) async {
    final now = DateTime.now().millisecondsSinceEpoch;
    final payload = ChatPayloadCodec.encode(
      ChatContent.sticker(packId: packId, stickerId: stickerId),
    );
    final outbound = await _crypto.encrypt(
      conversationId: conversationId,
      recipientAccount: recipientAccount,
      recipientKeyPackage: recipientKeyPackage,
      plaintext: utf8.encode(payload),
    );
    return _deliverOutbound(
      outbound: outbound,
      conversationId: conversationId,
      senderAccount: senderAccount,
      recipientAccount: recipientAccount,
      senderDeviceId: senderDeviceId,
      nowMillis: now,
      messageKind: ChatMessageKind.sticker,
      payload: payload,
    );
  }

  /// 发送图片 / 视频 / 文件：控制消息(含尺寸、时长、blurhash)走 MLS 信封,
  /// 媒体字节走 WebRTC 端到端直传。
  ///
  /// 顺序:加密 → **控制消息先离线安全入队/投递**(和文字一样,不依赖 WebRTC 成功)
  /// → 自存缓存 + 登记待设备投递 → 尝试 WebRTC 字节。字节发送失败(对方离线)**不
  /// 抛错**,留 pending 由对方上线时补发。加密仍在发字节之前,保持零泄漏顺序。
  Future<List<ChatDeliveryResult>> sendMedia({
    required String conversationId,
    required String senderAccount,
    required String recipientAccount,
    required String senderDeviceId,
    MlsKeyPackage? recipientKeyPackage,
    required ChatMediaDraft media,
    required ChatAttachmentDeviceSender sendDeviceAttachment,
    ChatLocalAttachmentSaver? saveLocalAttachment,
    ChatMediaPendingRecorder? recordPendingMedia,
    ChatMediaDeliveredMarker? onDeviceDelivered,
    ChatRelayUploader? uploadRelayMedia,
  }) async {
    // 门①:发送端大小硬门控。即使 UI 被绕过也在此拦下,此刻字节尚未进入任何
    // 通道。与接收端字节层门控(门②)配合,构成收发双端强制。
    if (ChatMediaLimits.exceedsForKind(media.kind, media.byteSize)) {
      throw ChatMediaTooLargeException(
        byteSize: media.byteSize,
        limitBytes: ChatMediaLimits.forKind(media.kind),
        kind: media.kind,
      );
    }
    final now = DateTime.now().millisecondsSinceEpoch;
    final attachmentId = _newAttachmentId(now);

    // 路由:>100MB **必须**经 Cloudflare R2 瞬时中转,绝不走 WebRTC(硬约束:
    // 只有 >100MB 走 Cloudflare、其余一律不走)。未配置中转即拒,而非降级 WebRTC。
    ChatRelayDescriptor? relay;
    if (ChatMediaLimits.needsRelay(media.byteSize)) {
      if (uploadRelayMedia == null) {
        throw StateError('>100MB 媒体必须经 Cloudflare 中转,但中转未配置');
      }
      relay = await uploadRelayMedia(
        conversationId: conversationId,
        attachmentId: attachmentId,
        media: media,
      );
    }

    final payload = ChatPayloadCodec.encode(
      ChatContent.media(
        kind: media.kind,
        attachmentId: attachmentId,
        fileName: media.fileName,
        mime: media.contentType,
        byteSize: media.byteSize,
        width: media.width,
        height: media.height,
        durationMs: media.durationMs,
        blurhash: media.blurhash,
        relayObjectKey: relay?.relayObjectKey,
        contentKeyB64: relay?.contentKeyB64,
        chunkSize: relay?.chunkSize,
        encSize: relay?.encSize,
      ),
    );
    final outbound = await _crypto.encrypt(
      conversationId: conversationId,
      recipientAccount: recipientAccount,
      recipientKeyPackage: recipientKeyPackage,
      plaintext: utf8.encode(payload),
    );
    // 控制消息先离线安全落库/投递:即便对方离线、WebRTC 发不出,消息仍成立。
    final results = await _deliverOutbound(
      outbound: outbound,
      conversationId: conversationId,
      senderAccount: senderAccount,
      recipientAccount: recipientAccount,
      senderDeviceId: senderDeviceId,
      nowMillis: now,
      messageKind: media.kind,
      payload: payload,
    );
    // 自存一份到缓存(会话里可见;WebRTC 路径还依赖它做离线补发)。
    await saveLocalAttachment?.call(
      conversationId: conversationId,
      attachmentId: attachmentId,
      fileName: media.fileName,
      contentType: media.contentType,
      sourcePath: media.sourcePath,
      byteSize: media.byteSize,
    );
    // 中转路径:密文已在 R2,收方按需拉取解密;不走 WebRTC、不登记待设备投递。
    if (relay != null) {
      return results;
    }
    await recordPendingMedia?.call(attachmentId);
    // 尝试 WebRTC 字节;对方离线/直连失败**不抛错**,留 pending 待上线补发。
    // 媒体字节由 WebRTC DTLS 端到端传输;Cloudflare 只转发 SDP/ICE,不收字节。
    try {
      await sendDeviceAttachment(
        recipientAccount: recipientAccount,
        conversationId: conversationId,
        attachmentId: attachmentId,
        fileName: media.fileName,
        contentType: media.contentType,
        sourcePath: media.sourcePath,
        byteSize: media.byteSize,
      );
      await onDeviceDelivered?.call(attachmentId);
    } on Exception {
      // 留 pending 行,对方上线(peer_ready)时由 retryOutgoing 补发。
    }
    return results;
  }

  /// 把加密结果逐条落库并投递。应用消息进消息表 + 出站队列，握手消息只进出站
  /// 队列；投递结果回写投递状态。sendText / sendMedia / sendSticker 共用。
  Future<List<ChatDeliveryResult>> _deliverOutbound({
    required MlsOutboundMessage outbound,
    required String conversationId,
    required String senderAccount,
    required String recipientAccount,
    required String senderDeviceId,
    required int nowMillis,
    required ChatMessageKind messageKind,
    required String payload,
  }) async {
    final results = <ChatDeliveryResult>[];
    var index = 0;
    for (final wireMessage in outbound.wireMessages) {
      final envelope = wireMessage.toEnvelope(
        envelopeId: _newEnvelopeId(conversationId, nowMillis, index),
        senderAccount: senderAccount,
        recipientAccount: recipientAccount,
        senderDeviceId: senderDeviceId,
        createdAtMillis: nowMillis + index,
        ttlMillis: defaultTtlMillis,
      );
      final envelopeBytes = envelope.writeToBuffer();
      final isApplication =
          wireMessage.messageKind == MlsMessageKind.application;
      if (isApplication) {
        await _store.saveOutgoingEnvelope(
          envelope: envelope,
          envelopeBytes: envelopeBytes,
          messageKind: messageKind,
          deliveryState: ChatMessageDeliveryState.queued,
          plaintext: payload,
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
        messageKind: ChatPayloadCodec.decode(plaintext).kind,
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
    final content = ChatPayloadCodec.decode(controlPlaintext);
    final attachmentId = content.attachmentId ?? '';
    final fileName = content.fileName ?? '';
    if (!content.isMedia || attachmentId.isEmpty || fileName.isEmpty) {
      throw const FormatException('不是有效的 Chat 媒体控制消息');
    }
    final cached = await readCachedAttachment(
      conversationId: conversationId,
      attachmentId: attachmentId,
      fileName: fileName,
      contentType: content.mime ?? 'application/octet-stream',
      clearByteSize: content.byteSize ?? 0,
      cacheDirectory: cacheDirectory,
    );
    if (cached != null) return cached;
    throw StateError('附件尚未完成设备间传输');
  }

  /// 把一份本机文件导入 App 私有缓存(流式,零整块内存)。
  ///
  /// [moveSource]=true 用于接收端把 WebRTC 落盘的临时文件**移动**进缓存(同卷
  /// rename 零拷贝,跨卷回退流式复制后删源);=false 用于发送端把源文件**复制**
  /// 进缓存(保留源)。两者落到同一按 conversationId/attachmentId/fileName 派生
  /// 的缓存路径。
  static Future<ChatDownloadedAttachment> importAttachmentFileToCache({
    required String conversationId,
    required String attachmentId,
    required String fileName,
    required String contentType,
    required String sourcePath,
    required int byteSize,
    required bool moveSource,
    required Directory cacheDirectory,
  }) async {
    final file = _attachmentCacheFile(
      cacheDirectory: cacheDirectory,
      conversationId: conversationId,
      attachmentId: attachmentId,
      fileName: fileName,
    );
    await file.parent.create(recursive: true);
    final source = File(sourcePath);
    if (moveSource) {
      try {
        await source.rename(file.path);
      } on FileSystemException {
        await _streamCopy(source, file);
        if (await source.exists()) {
          await source.delete();
        }
      }
    } else {
      await _streamCopy(source, file);
    }
    return ChatDownloadedAttachment(
      attachmentId: attachmentId,
      fileName: fileName,
      contentType: contentType,
      clearByteSize: byteSize,
      filePath: file.path,
    );
  }

  /// 门③:接收端把落盘的临时文件收入缓存前的**落盘二次门控**。
  ///
  /// 大小超出该 mime 上限 → 删临时、返回 null(不入缓存,纵深防御,即便传输层门②
  /// 被绕过);否则把临时文件移入缓存并返回句柄。cacheDirectory 注入以便单测。
  static Future<ChatDownloadedAttachment?> acceptReceivedMediaToCache({
    required String conversationId,
    required String attachmentId,
    required String fileName,
    required String contentType,
    required String tempFilePath,
    required int byteSize,
    required Directory cacheDirectory,
  }) async {
    if (byteSize > ChatMediaLimits.forMime(contentType)) {
      final temp = File(tempFilePath);
      if (await temp.exists()) {
        await temp.delete();
      }
      return null;
    }
    return importAttachmentFileToCache(
      conversationId: conversationId,
      attachmentId: attachmentId,
      fileName: fileName,
      contentType: contentType,
      sourcePath: tempFilePath,
      byteSize: byteSize,
      moveSource: true,
      cacheDirectory: cacheDirectory,
    );
  }

  /// 媒体在本机缓存中的确定路径(离线补发时按当前 Documents 目录重算)。
  static String attachmentCachePath({
    required Directory cacheDirectory,
    required String conversationId,
    required String attachmentId,
    required String fileName,
  }) {
    return _attachmentCacheFile(
      cacheDirectory: cacheDirectory,
      conversationId: conversationId,
      attachmentId: attachmentId,
      fileName: fileName,
    ).path;
  }

  /// 只按文件存在性 + 大小(stat,不读整块字节)判定缓存是否就绪。
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
    final length = await file.length();
    if (length != clearByteSize) {
      return null;
    }
    return ChatDownloadedAttachment(
      attachmentId: attachmentId,
      fileName: fileName,
      contentType: contentType,
      clearByteSize: length,
      filePath: file.path,
    );
  }
}

Future<void> _streamCopy(File source, File destination) async {
  final sink = destination.openWrite();
  try {
    await sink.addStream(source.openRead());
  } finally {
    await sink.close();
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
