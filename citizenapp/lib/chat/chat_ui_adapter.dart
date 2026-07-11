import 'dart:convert';

import 'package:flutter_chat_core/flutter_chat_core.dart';

import 'chat_models.dart';
import 'storage/chat_store.dart';

/// 把 GMB Chat 本地消息转换为现成聊天 UI 的消息模型。
Message storedMessageToChatMessage(
  ChatStoredMessage message, {
  required String ownerAccount,
}) {
  final createdAt = DateTime.fromMillisecondsSinceEpoch(
    message.createdAtMillis,
  ).toUtc();
  return Message.text(
    id: message.envelopeId,
    authorId: message.senderAccount,
    createdAt: createdAt,
    sentAt: _sentAt(message, createdAt),
    deliveredAt: _deliveredAt(message, createdAt),
    failedAt: message.deliveryState == ChatMessageDeliveryState.failed
        ? createdAt
        : null,
    status: _messageStatus(message.deliveryState),
    text: _visibleText(message),
    metadata: {
      'conversation_id': message.conversationId,
      'direction': message.direction,
      'is_mine': message.senderAccount == ownerAccount,
      'message_kind': message.messageKind.name,
      if (message.messageKind == ChatMessageKind.attachment)
        'attachment_control_plaintext': message.plaintext ?? '',
    },
  );
}

String _visibleText(ChatStoredMessage message) {
  if (message.messageKind != ChatMessageKind.attachment) {
    return message.plaintext ?? '';
  }
  final plaintext = message.plaintext ?? '';
  try {
    final decoded = jsonDecode(plaintext);
    if (decoded is Map) {
      final fileName = decoded['file_name']?.toString() ?? '';
      return fileName.isEmpty ? '[附件]' : '[附件] $fileName';
    }
  } catch (_) {
    return '[附件]';
  }
  return '[附件]';
}

/// 把本地消息列表转换为聊天 UI controller 的初始列表。
List<Message> storedMessagesToChatMessages(
  List<ChatStoredMessage> messages, {
  required String ownerAccount,
}) {
  return messages
      .map(
        (message) => storedMessageToChatMessage(
          message,
          ownerAccount: ownerAccount,
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
