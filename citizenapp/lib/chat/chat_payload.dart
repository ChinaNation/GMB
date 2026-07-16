import 'dart:convert';

import 'chat_models.dart';

/// 聊天消息载荷(即 OpenMLS 明文内容)的唯一编解码。
///
/// [ChatPayloadCodec] 是全仓消息类型与媒体元数据的单一真源:发送端编码、
/// 接收端与展示端解码都只经此处。它用显式 `kind` 判别消息类型,取代早期
/// "能否 jsonDecode 且带某 type 字段"的脆弱启发式(该启发式会把内容恰好
/// 是 JSON 的纯文本误判为附件)。
///
/// 本 JSON 只存在于端到端明文里:图片/视频/文件的**字节**走 WebRTC 端到端
/// 直传,本结构只承载文件名、尺寸、时长、blurhash 占位等**控制元数据**;
/// Cloudflare 只中转其密文,不接触明文,也从不接触媒体字节。
class ChatContent {
  const ChatContent._({
    required this.kind,
    this.text,
    this.attachmentId,
    this.fileName,
    this.mime,
    this.byteSize,
    this.width,
    this.height,
    this.durationMs,
    this.blurhash,
    this.relayObjectKey,
    this.contentKeyB64,
    this.chunkSize,
    this.encSize,
    this.packId,
    this.stickerId,
  });

  /// 纯文本消息。
  factory ChatContent.text(String text) =>
      ChatContent._(kind: ChatMessageKind.text, text: text);

  /// 媒体消息(image / video / file)。字节另经 WebRTC 传输,以 [attachmentId]
  /// 关联;本结构只带控制元数据。
  factory ChatContent.media({
    required ChatMessageKind kind,
    required String attachmentId,
    required String fileName,
    required String mime,
    required int byteSize,
    int? width,
    int? height,
    int? durationMs,
    String? blurhash,
    String? relayObjectKey,
    String? contentKeyB64,
    int? chunkSize,
    int? encSize,
  }) {
    assert(
      kind == ChatMessageKind.image ||
          kind == ChatMessageKind.video ||
          kind == ChatMessageKind.file,
      'ChatContent.media 只接受 image/video/file',
    );
    return ChatContent._(
      kind: kind,
      attachmentId: attachmentId,
      fileName: fileName,
      mime: mime,
      byteSize: byteSize,
      width: width,
      height: height,
      durationMs: durationMs,
      blurhash: blurhash,
      relayObjectKey: relayObjectKey,
      contentKeyB64: contentKeyB64,
      chunkSize: chunkSize,
      encSize: encSize,
    );
  }

  /// 贴纸消息:只承载内置贴纸包 id,不传任何字节。
  factory ChatContent.sticker({
    required String packId,
    required String stickerId,
  }) =>
      ChatContent._(
        kind: ChatMessageKind.sticker,
        packId: packId,
        stickerId: stickerId,
      );

  final ChatMessageKind kind;

  /// kind=text。
  final String? text;

  /// kind=image/video/file:关联 WebRTC 字节流的附件 ID。
  final String? attachmentId;
  final String? fileName;
  final String? mime;
  final int? byteSize;

  /// image/video 像素宽高;video 带 [durationMs]。
  final int? width;
  final int? height;
  final int? durationMs;

  /// image/video 的低清占位串(blurhash),字节到达前先渲染占位。
  final String? blurhash;

  /// >100MB 大媒体经 Cloudflare R2 瞬时中转时携带(仅此情形):R2 对象键、
  /// 一次性内容密钥(base64,**只随 E2E 信封传,Cloudflare 拿不到**)、分块大小、
  /// 密文总字节。字段缺失即非中转媒体(走 WebRTC)。
  final String? relayObjectKey;
  final String? contentKeyB64;
  final int? chunkSize;
  final int? encSize;

  /// kind=sticker:内置贴纸包与贴纸 id。
  final String? packId;
  final String? stickerId;

  /// 是否为带 WebRTC 字节的媒体(image/video/file)。
  bool get isMedia =>
      kind == ChatMessageKind.image ||
      kind == ChatMessageKind.video ||
      kind == ChatMessageKind.file;

  /// 是否为经 Cloudflare R2 中转的大媒体(>100MB,携 relayObjectKey)。
  bool get isRelayMedia => isMedia && (relayObjectKey ?? '').isNotEmpty;

  /// 会话列表 / 通知用的简短摘要。
  String get summary => switch (kind) {
        ChatMessageKind.text => text ?? '',
        ChatMessageKind.image => '[图片]',
        ChatMessageKind.video => '[视频]',
        ChatMessageKind.file =>
          (fileName ?? '').isEmpty ? '[文件]' : '[文件] ${fileName!}',
        ChatMessageKind.sticker => '[贴纸]',
      };
}

/// 载荷 JSON 的编解码器。
class ChatPayloadCodec {
  ChatPayloadCodec._();

  /// 载荷类型标记与版本;解码时用于区分本协议载荷与任意文本。
  static const String type = 'gmb.chat.msg';
  static const int version = 1;

  static String encode(ChatContent content) {
    final map = <String, Object?>{
      't': type,
      'v': version,
      'kind': content.kind.name,
    };
    switch (content.kind) {
      case ChatMessageKind.text:
        map['text'] = content.text ?? '';
      case ChatMessageKind.image:
      case ChatMessageKind.video:
      case ChatMessageKind.file:
        map['attachment_id'] = content.attachmentId;
        map['file_name'] = content.fileName;
        map['mime'] = content.mime;
        map['byte_size'] = content.byteSize;
        if (content.width != null) map['width'] = content.width;
        if (content.height != null) map['height'] = content.height;
        if (content.durationMs != null) map['duration_ms'] = content.durationMs;
        if (content.blurhash != null) map['blurhash'] = content.blurhash;
        if (content.relayObjectKey != null) {
          map['relay_object_key'] = content.relayObjectKey;
          map['content_key'] = content.contentKeyB64;
          map['chunk_size'] = content.chunkSize;
          map['enc_size'] = content.encSize;
        }
      case ChatMessageKind.sticker:
        map['pack_id'] = content.packId;
        map['sticker_id'] = content.stickerId;
    }
    return jsonEncode(map);
  }

  /// 解码明文载荷。任何不合法输入(非 JSON、非本协议、缺字段)都**退化为纯
  /// 文本且绝不抛出**——收发端据此稳定判定类型,并使"文本内容恰好是 JSON"
  /// 也被如实当作文本。
  static ChatContent decode(String raw) {
    Object? decoded;
    try {
      decoded = jsonDecode(raw);
    } catch (_) {
      return ChatContent.text(raw);
    }
    if (decoded is! Map || decoded['t'] != type) {
      return ChatContent.text(raw);
    }
    final kind = _kindFromName(decoded['kind']);
    return switch (kind) {
      ChatMessageKind.text =>
        ChatContent.text(_asString(decoded['text']) ?? ''),
      ChatMessageKind.image ||
      ChatMessageKind.video ||
      ChatMessageKind.file =>
        ChatContent.media(
          kind: kind,
          attachmentId: _asString(decoded['attachment_id']) ?? '',
          fileName: _asString(decoded['file_name']) ?? '',
          mime: _asString(decoded['mime']) ?? 'application/octet-stream',
          byteSize: _asInt(decoded['byte_size']) ?? 0,
          width: _asInt(decoded['width']),
          height: _asInt(decoded['height']),
          durationMs: _asInt(decoded['duration_ms']),
          blurhash: _asString(decoded['blurhash']),
          relayObjectKey: _asString(decoded['relay_object_key']),
          contentKeyB64: _asString(decoded['content_key']),
          chunkSize: _asInt(decoded['chunk_size']),
          encSize: _asInt(decoded['enc_size']),
        ),
      ChatMessageKind.sticker => ChatContent.sticker(
          packId: _asString(decoded['pack_id']) ?? '',
          stickerId: _asString(decoded['sticker_id']) ?? '',
        ),
    };
  }

  static ChatMessageKind _kindFromName(Object? value) {
    final name = value is String ? value : '';
    return ChatMessageKind.values.firstWhere(
      (k) => k.name == name,
      orElse: () => ChatMessageKind.text,
    );
  }

  static String? _asString(Object? value) =>
      value is String && value.isNotEmpty ? value : null;

  static int? _asInt(Object? value) {
    if (value is int) return value;
    if (value is num && value.isFinite) return value.toInt();
    return null;
  }
}
