import 'dart:convert';

import 'package:flutter_chat_core/flutter_chat_core.dart';

import 'im_session_models.dart';
import 'storage/im_isar_store.dart';

/// 把 GMB IM 本地消息转换为现成聊天 UI 的消息模型。
Message imStoredMessageToChatMessage(
  ImStoredMessage message, {
  required String currentUserId,
}) {
  final createdAt = DateTime.fromMillisecondsSinceEpoch(
    message.createdAtMillis,
  ).toUtc();
  return Message.text(
    id: message.envelopeId,
    authorId: message.senderChatAccount,
    createdAt: createdAt,
    sentAt: _sentAt(message, createdAt),
    deliveredAt: _deliveredAt(message, createdAt),
    failedAt: message.deliveryState == ImMessageDeliveryState.failed
        ? createdAt
        : null,
    status: _messageStatus(message.deliveryState),
    text: _visibleText(message),
    metadata: {
      'conversation_id': message.conversationId,
      'direction': message.direction,
      'is_mine': message.senderChatAccount == currentUserId,
      'message_kind': message.messageKind.name,
      if (message.messageKind == ImMessageKind.attachment)
        'attachment_control_plaintext': message.plaintext ?? '',
    },
  );
}

String _visibleText(ImStoredMessage message) {
  if (message.messageKind != ImMessageKind.attachment) {
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
List<Message> imStoredMessagesToChatMessages(
  List<ImStoredMessage> messages, {
  required String currentUserId,
}) {
  return messages
      .map(
        (message) => imStoredMessageToChatMessage(
          message,
          currentUserId: currentUserId,
        ),
      )
      .toList(growable: false);
}

MessageStatus _messageStatus(ImMessageDeliveryState state) {
  return switch (state) {
    ImMessageDeliveryState.queued => MessageStatus.sending,
    ImMessageDeliveryState.sending => MessageStatus.sending,
    ImMessageDeliveryState.sent => MessageStatus.sent,
    ImMessageDeliveryState.receivedByDevice => MessageStatus.delivered,
    ImMessageDeliveryState.failed => MessageStatus.error,
  };
}

DateTime? _sentAt(ImStoredMessage message, DateTime createdAt) {
  return switch (message.deliveryState) {
    ImMessageDeliveryState.sent ||
    ImMessageDeliveryState.receivedByDevice =>
      createdAt,
    _ => null,
  };
}

DateTime? _deliveredAt(ImStoredMessage message, DateTime createdAt) {
  return message.deliveryState == ImMessageDeliveryState.receivedByDevice
      ? createdAt
      : null;
}
