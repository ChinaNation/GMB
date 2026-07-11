import 'package:isar_community/isar.dart';

import '../../isar/app_isar.dart';
import '../chat_models.dart';
import '../proto/chat_envelope.pb.dart';

/// Chat 本地消息记录。
class ChatStoredMessage {
  const ChatStoredMessage({
    required this.envelopeId,
    required this.conversationId,
    required this.direction,
    required this.senderAccount,
    required this.recipientAccount,
    required this.messageKind,
    required this.deliveryState,
    required this.createdAtMillis,
    this.plaintext,
  });

  final String envelopeId;
  final String conversationId;
  final String direction;
  final String senderAccount;
  final String recipientAccount;
  final ChatMessageKind messageKind;
  final ChatMessageDeliveryState deliveryState;
  final int createdAtMillis;
  final String? plaintext;
}

/// Chat 路由缓存记录。
class ChatRoute {
  const ChatRoute({
    required this.peerAccount,
    required this.routeDisplayName,
    required this.deviceId,
    required this.devicePublicKeyHex,
    required this.safetyNumber,
    this.cloudflareMailboxId,
    this.nearbyPeerHint,
    this.note,
    this.createdAtMillis,
    this.updatedAtMillis,
  });

  final String peerAccount;
  final String routeDisplayName;
  final String deviceId;
  final String devicePublicKeyHex;
  final String safetyNumber;
  final String? cloudflareMailboxId;
  final String? nearbyPeerHint;
  final String? note;
  final int? createdAtMillis;
  final int? updatedAtMillis;

  String get routeId => peerAccount;
}

/// 公民 Chat 的 Isar 持久化仓库。
///
/// 本仓库只保存手机本地状态。Cloudflare 和近场 transport 只拿到完整
/// Protobuf envelope bytes，不会接触 [plaintext]。
class ChatStore {
  ChatStore({
    WalletIsar? walletIsar,
  }) : _walletIsar = walletIsar ?? WalletIsar.instance;

  final WalletIsar _walletIsar;

  Future<List<ChatConversationPreview>> readConversationPreviews({
    String? ownerAccount,
  }) {
    return _walletIsar.read((isar) async {
      final rows = await isar.chatConversationEntitys
          .filter()
          .idGreaterThan(0, include: true)
          .findAll();
      final filtered = ownerAccount == null || ownerAccount.isEmpty
          ? rows
          : rows
              .where((row) => row.ownerAccount == ownerAccount)
              .toList(growable: false);
      filtered.sort(
          (a, b) => b.lastUpdatedAtMillis.compareTo(a.lastUpdatedAtMillis));
      return filtered
          .map(_conversationPreviewFromEntity)
          .toList(growable: false);
    });
  }

  Future<List<ChatRoute>> readRouteRecords() {
    return _walletIsar.read((isar) async {
      final rows = await isar.chatRouteCacheEntitys
          .filter()
          .idGreaterThan(0, include: true)
          .findAll();
      rows.sort((a, b) => a.routeDisplayName.compareTo(b.routeDisplayName));
      return rows.map(_routeFromEntity).toList(growable: false);
    });
  }

  Future<ChatRoute?> getRouteRecord(String peerAccount) {
    return _walletIsar.read((isar) async {
      final row =
          await isar.chatRouteCacheEntitys.getByPeerAccount(peerAccount);
      return row == null ? null : _routeFromEntity(row);
    });
  }

  Future<void> upsertRouteRecord(ChatRoute route) {
    return _walletIsar.writeTxn((isar) async {
      final now = DateTime.now().millisecondsSinceEpoch;
      final existing =
          await isar.chatRouteCacheEntitys.getByRouteId(route.routeId);
      final entity = existing ?? ChatRouteCacheEntity();
      entity
        ..routeId = route.routeId
        ..peerAccount = route.peerAccount
        ..routeDisplayName = route.routeDisplayName
        ..deviceId = route.deviceId
        ..devicePublicKeyHex = route.devicePublicKeyHex
        ..safetyNumber = route.safetyNumber
        ..cloudflareMailboxId = route.cloudflareMailboxId
        ..nearbyPeerHint = route.nearbyPeerHint
        ..note = route.note
        ..createdAtMillis =
            existing?.createdAtMillis ?? route.createdAtMillis ?? now
        ..updatedAtMillis = route.updatedAtMillis ?? now;
      await isar.chatRouteCacheEntitys.putByRouteId(entity);
    });
  }

  Future<List<ChatStoredMessage>> readMessages(String conversationId) {
    return _walletIsar.read((isar) async {
      final rows = await isar.chatMessageEntitys
          .filter()
          .idGreaterThan(0, include: true)
          .findAll();
      final filtered = rows
          .where((row) => row.conversationId == conversationId)
          .toList(growable: false)
        ..sort((a, b) => a.createdAtMillis.compareTo(b.createdAtMillis));
      return filtered.map(_messageFromEntity).toList(growable: false);
    });
  }

  /// 彻底删除本机会话记录。
  ///
  /// Cloudflare 只做临时投递队列；用户删除聊天记录时，本地 Isar 是唯一
  /// 需要清理的聊天历史真源。附件缓存目录由运行态在同一操作中删除。
  Future<void> deleteConversation(String conversationId) {
    return _walletIsar.writeTxn((isar) async {
      final messages = await isar.chatMessageEntitys
          .filter()
          .idGreaterThan(0, include: true)
          .findAll();
      for (final message in messages.where(
        (message) => message.conversationId == conversationId,
      )) {
        await isar.chatMessageEntitys.delete(message.id);
      }

      final conversations = await isar.chatConversationEntitys
          .filter()
          .idGreaterThan(0, include: true)
          .findAll();
      for (final conversation in conversations.where(
        (conversation) => conversation.conversationId == conversationId,
      )) {
        await isar.chatConversationEntitys.delete(conversation.id);
      }

      final outboundRows = await isar.chatOutboundQueueEntitys
          .filter()
          .idGreaterThan(0, include: true)
          .findAll();
      for (final row in outboundRows.where(
        (row) => row.conversationId == conversationId,
      )) {
        await isar.chatOutboundQueueEntitys.delete(row.id);
      }

      final pendingRows = await isar.chatPendingInboundEntitys
          .filter()
          .idGreaterThan(0, include: true)
          .findAll();
      for (final row in pendingRows.where(
        (row) => row.conversationId == conversationId,
      )) {
        await isar.chatPendingInboundEntitys.delete(row.id);
      }
    });
  }

  /// 注销用户：清除该 owner 在本机的全部 Chat 历史（会话/消息/出入站队列）。
  ///
  /// Cloudflare 端 A 的 Chat 数据由 Worker purge 删除；本地 Isar 是 A 私信**明文**
  /// 的唯一残留处，须一并清空以做到零残留。路由缓存（imRouteCacheEntity）是设备级
  /// 对端路由、非 owner 归属，不在此清除。
  Future<void> clearAllForOwner(String ownerAccount) {
    return _walletIsar.writeTxn((isar) async {
      final conversations = await isar.chatConversationEntitys
          .filter()
          .idGreaterThan(0, include: true)
          .findAll();
      final owned = conversations
          .where((c) => c.ownerAccount == ownerAccount)
          .toList(growable: false);
      final ownedIds = owned.map((c) => c.conversationId).toSet();
      for (final c in owned) {
        await isar.chatConversationEntitys.delete(c.id);
      }

      final messages = await isar.chatMessageEntitys
          .filter()
          .idGreaterThan(0, include: true)
          .findAll();
      for (final m in messages.where(
        (m) =>
            m.ownerAccount == ownerAccount ||
            ownedIds.contains(m.conversationId),
      )) {
        await isar.chatMessageEntitys.delete(m.id);
      }

      final outbound = await isar.chatOutboundQueueEntitys
          .filter()
          .idGreaterThan(0, include: true)
          .findAll();
      for (final o in outbound.where(
        (o) => ownedIds.contains(o.conversationId),
      )) {
        await isar.chatOutboundQueueEntitys.delete(o.id);
      }

      final pending = await isar.chatPendingInboundEntitys
          .filter()
          .idGreaterThan(0, include: true)
          .findAll();
      for (final p in pending.where(
        (p) => ownedIds.contains(p.conversationId),
      )) {
        await isar.chatPendingInboundEntitys.delete(p.id);
      }
    });
  }

  Future<void> saveOutgoingEnvelope({
    required ChatEnvelope envelope,
    required List<int> envelopeBytes,
    required ChatMessageKind messageKind,
    required ChatMessageDeliveryState deliveryState,
    String? plaintext,
  }) {
    return _walletIsar.writeTxn((isar) async {
      await _putConversationInTxn(
        isar: isar,
        conversationId: envelope.conversationId,
        ownerAccount: envelope.senderAccount,
        peerAccount: envelope.recipientAccount,
        title: envelope.recipientAccount,
        lastMessage: _messageSummary(messageKind, plaintext),
        lastUpdatedAtMillis: envelope.createdAtMillis.toInt(),
        unreadDelta: 0,
        deliveryState: deliveryState,
      );
      await isar.chatMessageEntitys.putByEnvelopeId(
        _messageEntity(
          envelope: envelope,
          envelopeBytes: envelopeBytes,
          ownerAccount: envelope.senderAccount,
          direction: 'outgoing',
          messageKind: messageKind,
          deliveryState: deliveryState,
          plaintext: plaintext,
        ),
      );
      await isar.chatOutboundQueueEntitys.putByEnvelopeId(
        ChatOutboundQueueEntity()
          ..envelopeId = envelope.envelopeId
          ..conversationId = envelope.conversationId
          ..recipientAccount = envelope.recipientAccount
          ..envelopeBytesHex = _bytesToHex(envelopeBytes)
          ..deliveryState = deliveryState.name
          ..attemptCount = 0
          ..lastError = null
          ..updatedAtMillis = DateTime.now().millisecondsSinceEpoch,
      );
    });
  }

  Future<void> queueOutgoingEnvelope({
    required ChatEnvelope envelope,
    required List<int> envelopeBytes,
    required ChatMessageDeliveryState deliveryState,
  }) {
    return _walletIsar.writeTxn((isar) async {
      await isar.chatOutboundQueueEntitys.putByEnvelopeId(
        ChatOutboundQueueEntity()
          ..envelopeId = envelope.envelopeId
          ..conversationId = envelope.conversationId
          ..recipientAccount = envelope.recipientAccount
          ..envelopeBytesHex = _bytesToHex(envelopeBytes)
          ..deliveryState = deliveryState.name
          ..attemptCount = 0
          ..lastError = null
          ..updatedAtMillis = DateTime.now().millisecondsSinceEpoch,
      );
    });
  }

  Future<void> saveIncomingEnvelope({
    required ChatEnvelope envelope,
    required List<int> envelopeBytes,
    required ChatMessageKind messageKind,
    required String plaintext,
  }) {
    return _walletIsar.writeTxn((isar) async {
      await _putConversationInTxn(
        isar: isar,
        conversationId: envelope.conversationId,
        ownerAccount: envelope.recipientAccount,
        peerAccount: envelope.senderAccount,
        title: envelope.senderAccount,
        lastMessage: _messageSummary(messageKind, plaintext),
        lastUpdatedAtMillis: envelope.createdAtMillis.toInt(),
        unreadDelta: 1,
        deliveryState: ChatMessageDeliveryState.receivedByDevice,
      );
      await isar.chatMessageEntitys.putByEnvelopeId(
        _messageEntity(
          envelope: envelope,
          envelopeBytes: envelopeBytes,
          ownerAccount: envelope.recipientAccount,
          direction: 'incoming',
          messageKind: messageKind,
          deliveryState: ChatMessageDeliveryState.receivedByDevice,
          plaintext: plaintext,
        ),
      );
    });
  }

  Future<void> markOutgoingDelivery({
    required String envelopeId,
    required ChatMessageDeliveryState state,
    String? errorMessage,
  }) {
    return _walletIsar.writeTxn((isar) async {
      final queue =
          await isar.chatOutboundQueueEntitys.getByEnvelopeId(envelopeId);
      if (queue != null) {
        queue
          ..deliveryState = state.name
          ..attemptCount = queue.attemptCount + 1
          ..lastError = errorMessage
          ..updatedAtMillis = DateTime.now().millisecondsSinceEpoch;
        await isar.chatOutboundQueueEntitys.putByEnvelopeId(queue);
      }
      final message = await isar.chatMessageEntitys.getByEnvelopeId(envelopeId);
      if (message != null) {
        message.deliveryState = state.name;
        await isar.chatMessageEntitys.putByEnvelopeId(message);
        final conversation = await isar.chatConversationEntitys
            .getByConversationId(message.conversationId);
        if (conversation != null) {
          conversation.lastDeliveryState = state.name;
          await isar.chatConversationEntitys.putByConversationId(conversation);
        }
      }
    });
  }

  Future<void> savePendingInbound({
    required ChatEnvelope envelope,
    required List<int> envelopeBytes,
    required String reason,
  }) {
    return _walletIsar.writeTxn((isar) async {
      await isar.chatPendingInboundEntitys.putByEnvelopeId(
        ChatPendingInboundEntity()
          ..envelopeId = envelope.envelopeId
          ..conversationId = envelope.conversationId
          ..envelopeBytesHex = _bytesToHex(envelopeBytes)
          ..reason = reason
          ..createdAtMillis = DateTime.now().millisecondsSinceEpoch,
      );
    });
  }

  Future<List<ChatEnvelope>> takePendingInbound(String conversationId) {
    return _walletIsar.writeTxn((isar) async {
      final rows = await isar.chatPendingInboundEntitys
          .filter()
          .idGreaterThan(0, include: true)
          .findAll();
      final matched = rows
          .where((row) => row.conversationId == conversationId)
          .toList(growable: false)
        ..sort((a, b) => a.createdAtMillis.compareTo(b.createdAtMillis));
      for (final row in matched) {
        await isar.chatPendingInboundEntitys.delete(row.id);
      }
      return matched
          .map((row) =>
              ChatEnvelope.fromBuffer(_hexToBytes(row.envelopeBytesHex)))
          .toList(growable: false);
    });
  }

  Future<int> pendingInboundCount() {
    return _walletIsar.read((isar) async {
      final rows = await isar.chatPendingInboundEntitys
          .filter()
          .idGreaterThan(0, include: true)
          .findAll();
      return rows.length;
    });
  }

  Future<int> outboundQueueCount() {
    return _walletIsar.read((isar) async {
      final rows = await isar.chatOutboundQueueEntitys
          .filter()
          .idGreaterThan(0, include: true)
          .findAll();
      return rows.length;
    });
  }

  Future<void> _putConversationInTxn({
    required Isar isar,
    required String conversationId,
    required String ownerAccount,
    required String peerAccount,
    required String title,
    required String lastMessage,
    required int lastUpdatedAtMillis,
    required int unreadDelta,
    required ChatMessageDeliveryState deliveryState,
  }) async {
    final existing =
        await isar.chatConversationEntitys.getByConversationId(conversationId);
    final entity = existing ?? ChatConversationEntity();
    entity
      ..conversationId = conversationId
      ..ownerAccount = ownerAccount
      ..peerAccount = peerAccount
      ..title = title
      ..lastMessage = lastMessage
      ..lastUpdatedAtMillis = lastUpdatedAtMillis
      ..unreadCount = (existing?.unreadCount ?? 0) + unreadDelta
      ..lastDeliveryState = deliveryState.name;
    await isar.chatConversationEntitys.putByConversationId(entity);
  }
}

ChatConversationPreview _conversationPreviewFromEntity(
    ChatConversationEntity row) {
  return ChatConversationPreview(
    conversationId: row.conversationId,
    title: row.title,
    peerAccount: row.peerAccount,
    lastMessage: row.lastMessage,
    lastUpdatedAt: DateTime.fromMillisecondsSinceEpoch(row.lastUpdatedAtMillis),
    unreadCount: row.unreadCount,
    deliveryState: _deliveryStateFromName(row.lastDeliveryState),
  );
}

ChatStoredMessage _messageFromEntity(ChatMessageEntity row) {
  return ChatStoredMessage(
    envelopeId: row.envelopeId,
    conversationId: row.conversationId,
    direction: row.direction,
    senderAccount: row.senderAccount,
    recipientAccount: row.recipientAccount,
    messageKind: _messageKindFromName(row.messageKind),
    deliveryState: _deliveryStateFromName(row.deliveryState),
    createdAtMillis: row.createdAtMillis,
    plaintext: row.plaintext,
  );
}

ChatRoute _routeFromEntity(ChatRouteCacheEntity row) {
  return ChatRoute(
    peerAccount: row.peerAccount,
    routeDisplayName: row.routeDisplayName,
    deviceId: row.deviceId,
    devicePublicKeyHex: row.devicePublicKeyHex,
    safetyNumber: row.safetyNumber,
    cloudflareMailboxId: row.cloudflareMailboxId,
    nearbyPeerHint: row.nearbyPeerHint,
    note: row.note,
    createdAtMillis: row.createdAtMillis,
    updatedAtMillis: row.updatedAtMillis,
  );
}

ChatMessageEntity _messageEntity({
  required ChatEnvelope envelope,
  required List<int> envelopeBytes,
  required String ownerAccount,
  required String direction,
  required ChatMessageKind messageKind,
  required ChatMessageDeliveryState deliveryState,
  String? plaintext,
}) {
  return ChatMessageEntity()
    ..envelopeId = envelope.envelopeId
    ..conversationId = envelope.conversationId
    ..ownerAccount = ownerAccount
    ..direction = direction
    ..senderAccount = envelope.senderAccount
    ..recipientAccount = envelope.recipientAccount
    ..senderDeviceId = envelope.senderDeviceId
    ..messageKind = messageKind.name
    ..mlsMessageKind = envelope.mlsMessageKind.name
    ..deliveryState = deliveryState.name
    ..plaintext = plaintext
    ..envelopeBytesHex = _bytesToHex(envelopeBytes)
    ..createdAtMillis = envelope.createdAtMillis.toInt();
}

String _messageSummary(ChatMessageKind kind, String? plaintext) {
  return switch (kind) {
    ChatMessageKind.text => plaintext ?? '',
    ChatMessageKind.attachment => '[附件]',
  };
}

ChatMessageDeliveryState _deliveryStateFromName(String value) {
  return ChatMessageDeliveryState.values.firstWhere(
    (item) => item.name == value,
    orElse: () => ChatMessageDeliveryState.failed,
  );
}

ChatMessageKind _messageKindFromName(String value) {
  return ChatMessageKind.values.firstWhere(
    (item) => item.name == value,
    orElse: () => ChatMessageKind.text,
  );
}

String _bytesToHex(List<int> bytes) {
  return bytes.map((item) => item.toRadixString(16).padLeft(2, '0')).join();
}

List<int> _hexToBytes(String value) {
  final normalized = value.startsWith('0x') ? value.substring(2) : value;
  if (normalized.length.isOdd) {
    throw const FormatException('Chat envelope hex 长度必须为偶数');
  }
  final bytes = <int>[];
  for (var i = 0; i < normalized.length; i += 2) {
    bytes.add(int.parse(normalized.substring(i, i + 2), radix: 16));
  }
  return bytes;
}
