import 'package:flutter_chat_core/flutter_chat_core.dart';

import 'chat_media_limits.dart';
import 'chat_models.dart';
import 'chat_payload.dart';
import 'storage/chat_store.dart';

/// 解析媒体消息在本机缓存中的绝对路径。字节尚未到达时返回 null,由 UI 占位。
typedef ChatMediaPathResolver = String? Function(ChatContent content);

/// 把 GMB Chat 本地消息转换为现成聊天 UI 的消息模型。
///
/// 消息类型来自端到端载荷([ChatPayloadCodec]),按 kind 分发为文本 / 图片 /
/// 视频 / 文件 / 贴纸。图片、视频、文件的字节走 WebRTC,渲染时用
/// [resolveLocalMediaPath] 查本机缓存路径,未到达则留空 source 由 UI 占位。
Message storedMessageToChatMessage(
  ChatStoredMessage message, {
  required String ownerAccount,
  ChatMediaPathResolver? resolveLocalMediaPath,
}) {
  final createdAt = DateTime.fromMillisecondsSinceEpoch(
    message.createdAtMillis,
  ).toUtc();
  final content = ChatPayloadCodec.decode(message.plaintext ?? '');
  final status = _messageStatus(message.deliveryState);
  final sentAt = _sentAt(message, createdAt);
  final deliveredAt = _deliveredAt(message, createdAt);
  final failedAt = message.deliveryState == ChatMessageDeliveryState.failed
      ? createdAt
      : null;
  final metadata = <String, dynamic>{
    'conversation_id': message.conversationId,
    'direction': message.direction,
    'is_mine': message.senderAccount == ownerAccount,
    'message_kind': message.messageKind.name,
  };

  // 门④:控制消息声明的大小超出该类型上限 → 渲染"已拒收"占位,永不解析/展示其
  // 字节(接收端本就在字节层拒收,此处保证 UI 一致,不诱导用户去拉取)。
  if (content.isMedia &&
      ChatMediaLimits.exceedsForKind(content.kind, content.byteSize ?? 0)) {
    return Message.text(
      id: message.envelopeId,
      authorId: message.senderAccount,
      createdAt: createdAt,
      sentAt: sentAt,
      deliveredAt: deliveredAt,
      failedAt: failedAt,
      status: status,
      text: '⚠️ 对方发送的媒体超出大小上限，已拒收',
      metadata: {...metadata, 'oversized': true},
    );
  }

  switch (content.kind) {
    case ChatMessageKind.text:
      return Message.text(
        id: message.envelopeId,
        authorId: message.senderAccount,
        createdAt: createdAt,
        sentAt: sentAt,
        deliveredAt: deliveredAt,
        failedAt: failedAt,
        status: status,
        text: content.text ?? '',
        metadata: metadata,
      );
    case ChatMessageKind.image:
      return Message.image(
        id: message.envelopeId,
        authorId: message.senderAccount,
        createdAt: createdAt,
        sentAt: sentAt,
        deliveredAt: deliveredAt,
        failedAt: failedAt,
        status: status,
        source: resolveLocalMediaPath?.call(content) ?? '',
        blurhash: content.blurhash,
        width: content.width?.toDouble(),
        height: content.height?.toDouble(),
        size: content.byteSize,
        metadata: {
          ...metadata,
          'attachment_id': content.attachmentId,
          'attachment_control_plaintext': message.plaintext ?? '',
          'file_name': content.fileName,
        },
      );
    case ChatMessageKind.video:
      return Message.video(
        id: message.envelopeId,
        authorId: message.senderAccount,
        createdAt: createdAt,
        sentAt: sentAt,
        deliveredAt: deliveredAt,
        failedAt: failedAt,
        status: status,
        source: resolveLocalMediaPath?.call(content) ?? '',
        name: content.fileName,
        width: content.width?.toDouble(),
        height: content.height?.toDouble(),
        size: content.byteSize,
        metadata: {
          ...metadata,
          'attachment_id': content.attachmentId,
          'attachment_control_plaintext': message.plaintext ?? '',
          'blurhash': content.blurhash,
          'file_name': content.fileName,
        },
      );
    case ChatMessageKind.file:
      return Message.file(
        id: message.envelopeId,
        authorId: message.senderAccount,
        createdAt: createdAt,
        sentAt: sentAt,
        deliveredAt: deliveredAt,
        failedAt: failedAt,
        status: status,
        source: resolveLocalMediaPath?.call(content) ?? '',
        name: content.fileName ?? '文件',
        size: content.byteSize,
        mimeType: content.mime,
        metadata: {
          ...metadata,
          'attachment_id': content.attachmentId,
          'attachment_control_plaintext': message.plaintext ?? '',
        },
      );
    case ChatMessageKind.sticker:
      // 步骤1 占位:贴纸美术与自定义渲染在步骤3;此处只保留 id 供后续渲染。
      return Message.text(
        id: message.envelopeId,
        authorId: message.senderAccount,
        createdAt: createdAt,
        sentAt: sentAt,
        deliveredAt: deliveredAt,
        failedAt: failedAt,
        status: status,
        text: '[贴纸]',
        metadata: {
          ...metadata,
          'pack_id': content.packId,
          'sticker_id': content.stickerId,
        },
      );
  }
}

/// 把本地消息列表转换为聊天 UI controller 的初始列表。
List<Message> storedMessagesToChatMessages(
  List<ChatStoredMessage> messages, {
  required String ownerAccount,
  ChatMediaPathResolver? resolveLocalMediaPath,
}) {
  return messages
      .map(
        (message) => storedMessageToChatMessage(
          message,
          ownerAccount: ownerAccount,
          resolveLocalMediaPath: resolveLocalMediaPath,
        ),
      )
      .toList(growable: false);
}

MessageStatus _messageStatus(ChatMessageDeliveryState state) {
  return switch (state) {
    ChatMessageDeliveryState.queued => MessageStatus.sending,
    ChatMessageDeliveryState.sending => MessageStatus.sending,
    ChatMessageDeliveryState.sent => MessageStatus.sent,
    ChatMessageDeliveryState.receivedByDevice => MessageStatus.delivered,
    ChatMessageDeliveryState.failed => MessageStatus.error,
  };
}

DateTime? _sentAt(ChatStoredMessage message, DateTime createdAt) {
  return switch (message.deliveryState) {
    ChatMessageDeliveryState.sent ||
    ChatMessageDeliveryState.receivedByDevice =>
      createdAt,
    _ => null,
  };
}

DateTime? _deliveredAt(ChatStoredMessage message, DateTime createdAt) {
  return message.deliveryState == ChatMessageDeliveryState.receivedByDevice
      ? createdAt
      : null;
}
