import 'package:isar_community/isar.dart';

import '../../isar/app_isar.dart';
import '../chat_models.dart';
import '../chat_payload.dart';
import '../group/group_model.dart';
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

/// 仅保存在发送设备上的待重试密文。
class ChatQueuedEnvelope {
  const ChatQueuedEnvelope({
    required this.envelopeId,
    required this.recipientAccount,
    required this.envelopeBytes,
  });

  final String envelopeId;
  final String recipientAccount;
  final List<int> envelopeBytes;
}

/// 待设备投递的媒体(离线补发)。缓存路径在补发时由 conversationId/attachmentId/
/// fileName 用当前 Documents 目录重算,不持久化绝对路径。
class ChatPendingMedia {
  const ChatPendingMedia({
    required this.attachmentId,
    required this.recipientAccount,
    required this.conversationId,
    required this.fileName,
    required this.contentType,
    required this.byteSize,
  });

  final String attachmentId;
  final String recipientAccount;
  final String conversationId;
  final String fileName;
  final String contentType;
  final int byteSize;
}

/// Chat 路由缓存记录。
class ChatRoute {
  const ChatRoute({
    required this.peerAccount,
    required this.routeDisplayName,
    required this.deviceId,
    required this.devicePublicKeyHex,
    required this.safetyNumber,
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
  final String? nearbyPeerHint;
  final String? note;
  final int? createdAtMillis;
  final int? updatedAtMillis;

  String get routeId => peerAccount;
}

/// 公民 Chat 的 Isar 持久化仓库。
///
/// 本仓库只保存手机本地状态。Cloudflare 瞬时转发和近场 transport 只拿到完整
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
  /// Cloudflare 不保存聊天内容；用户删除聊天记录时，本地 Isar 是唯一
  /// 需要清理的聊天历史真源，附件缓存目录由运行态在同一操作中删除。
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

      final outgoingMediaRows = await isar.chatOutgoingMediaEntitys
          .filter()
          .idGreaterThan(0, include: true)
          .findAll();
      for (final row in outgoingMediaRows.where(
        (row) => row.conversationId == conversationId,
      )) {
        await isar.chatOutgoingMediaEntitys.delete(row.id);
      }
    });
  }

  /// 注销用户：清除该 owner 在本机的全部 Chat 历史（会话/消息/出入站队列）。
  ///
  /// Cloudflare 端 A 的设备登记由 Worker purge 删除；本地 Isar 是 A 私信**明文**
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

      final outgoingMedia = await isar.chatOutgoingMediaEntitys
          .filter()
          .idGreaterThan(0, include: true)
          .findAll();
      for (final row in outgoingMedia.where(
        (row) => ownedIds.contains(row.conversationId),
      )) {
        await isar.chatOutgoingMediaEntitys.delete(row.id);
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
        lastMessage: _messageSummary(plaintext),
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
        lastMessage: _messageSummary(plaintext),
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
        if (state == ChatMessageDeliveryState.sent ||
            state == ChatMessageDeliveryState.receivedByDevice) {
          await isar.chatOutboundQueueEntitys.delete(queue.id);
        } else {
          queue
            ..deliveryState = state.name
            ..attemptCount = queue.attemptCount + 1
            ..lastError = errorMessage
            ..updatedAtMillis = DateTime.now().millisecondsSinceEpoch;
          await isar.chatOutboundQueueEntitys.putByEnvelopeId(queue);
        }
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

  /// 读取发送设备上的待重试密文；Cloudflare 不提供远程补拉。
  Future<List<ChatQueuedEnvelope>> readQueuedEnvelopes({
    String? recipientAccount,
  }) {
    return _walletIsar.read((isar) async {
      final rows = await isar.chatOutboundQueueEntitys
          .filter()
          .idGreaterThan(0, include: true)
          .findAll();
      final matched = recipientAccount == null
          ? rows
          : rows
              .where((row) => row.recipientAccount == recipientAccount)
              .toList(growable: false);
      matched.sort((a, b) => a.updatedAtMillis.compareTo(b.updatedAtMillis));
      return matched
          .map(
            (row) => ChatQueuedEnvelope(
              envelopeId: row.envelopeId,
              recipientAccount: row.recipientAccount,
              envelopeBytes: _hexToBytes(row.envelopeBytesHex),
            ),
          )
          .toList(growable: false);
    });
  }

  /// 登记一条待设备投递的媒体(字节未送达对方设备,留待上线补发)。
  Future<void> recordOutgoingMedia({
    required String attachmentId,
    required String recipientAccount,
    required String conversationId,
    required String fileName,
    required String contentType,
    required int byteSize,
  }) {
    return _walletIsar.writeTxn((isar) async {
      await isar.chatOutgoingMediaEntitys.putByPendingKey(
        ChatOutgoingMediaEntity()
          ..pendingKey = '$attachmentId|$recipientAccount'
          ..attachmentId = attachmentId
          ..recipientAccount = recipientAccount
          ..conversationId = conversationId
          ..fileName = fileName
          ..contentType = contentType
          ..byteSize = byteSize
          ..createdAtMillis = DateTime.now().millisecondsSinceEpoch,
      );
    });
  }

  /// 字节已送达某成员设备(收到 WebRTC ack)后删除该 (媒体, 成员) 待投递行。
  Future<void> deleteOutgoingMedia(String attachmentId, String recipientAccount) {
    return _walletIsar.writeTxn((isar) async {
      await isar.chatOutgoingMediaEntitys
          .deleteByPendingKey('$attachmentId|$recipientAccount');
    });
  }

  /// 读取待设备投递的媒体(可按对端过滤),供上线补发。
  Future<List<ChatPendingMedia>> readPendingOutgoingMedia({
    String? recipientAccount,
  }) {
    return _walletIsar.read((isar) async {
      final rows = await isar.chatOutgoingMediaEntitys
          .filter()
          .idGreaterThan(0, include: true)
          .findAll();
      final matched = recipientAccount == null
          ? rows
          : rows
              .where((row) => row.recipientAccount == recipientAccount)
              .toList(growable: false);
      matched.sort((a, b) => a.createdAtMillis.compareTo(b.createdAtMillis));
      return matched
          .map(
            (row) => ChatPendingMedia(
              attachmentId: row.attachmentId,
              recipientAccount: row.recipientAccount,
              conversationId: row.conversationId,
              fileName: row.fileName,
              contentType: row.contentType,
              byteSize: row.byteSize,
            ),
          )
          .toList(growable: false);
    });
  }

  Future<int> outgoingMediaCount() {
    return _walletIsar.read((isar) async {
      final rows = await isar.chatOutgoingMediaEntitys
          .filter()
          .idGreaterThan(0, include: true)
          .findAll();
      return rows.length;
    });
  }

  // ==== 私密小群 ====

  /// 建群/入群时落群会话壳 + 群会话记录(conversationKind=group,title=群名)。
  Future<void> upsertGroupShell({
    required String groupId,
    required String groupName,
    required String creatorAccount,
    required String ownerAccount,
    required int epoch,
  }) {
    return _walletIsar.writeTxn((isar) async {
      final now = DateTime.now().millisecondsSinceEpoch;
      final existing =
          await isar.chatGroupEntitys.getByGroupId(groupId);
      final entity = existing ?? ChatGroupEntity();
      entity
        ..groupId = groupId
        ..groupName = groupName
        ..creatorAccount = creatorAccount
        ..ownerAccount = ownerAccount
        ..epoch = epoch
        ..memberCount = existing?.memberCount ?? 1
        ..leftLocally = existing?.leftLocally ?? false
        ..createdAtMillis = existing?.createdAtMillis ?? now
        ..updatedAtMillis = now;
      await isar.chatGroupEntitys.putByGroupId(entity);

      final conversation =
          await isar.chatConversationEntitys.getByConversationId(groupId);
      final shell = conversation ?? ChatConversationEntity();
      shell
        ..conversationId = groupId
        ..ownerAccount = ownerAccount
        ..peerAccount = creatorAccount
        ..title = groupName
        ..conversationKind = 'group'
        ..lastMessage = conversation?.lastMessage ?? ''
        ..lastUpdatedAtMillis = conversation?.lastUpdatedAtMillis ?? now
        ..unreadCount = conversation?.unreadCount ?? 0
        ..lastDeliveryState = conversation?.lastDeliveryState ??
            ChatMessageDeliveryState.queued.name;
      await isar.chatConversationEntitys.putByConversationId(shell);
    });
  }

  /// 按 MLS 名册(account→role)覆盖群成员镜像 + 更新 epoch/人数。
  Future<void> reconcileGroupRoster({
    required String groupId,
    required Map<String, GroupMemberRole> members,
    required int epoch,
  }) {
    return _walletIsar.writeTxn((isar) async {
      final now = DateTime.now().millisecondsSinceEpoch;
      final existing = await isar.chatGroupMemberEntitys
          .filter()
          .groupIdEqualTo(groupId)
          .findAll();
      final joinedAt = <String, int>{
        for (final row in existing) row.memberAccount: row.joinedAtMillis,
      };
      for (final row in existing) {
        await isar.chatGroupMemberEntitys.delete(row.id);
      }
      for (final entry in members.entries) {
        await isar.chatGroupMemberEntitys.putByMemberKey(
          ChatGroupMemberEntity()
            ..memberKey = '$groupId|${entry.key}'
            ..groupId = groupId
            ..memberAccount = entry.key
            ..role = entry.value.wireName
            ..joinedAtMillis = joinedAt[entry.key] ?? now,
        );
      }
      final group = await isar.chatGroupEntitys.getByGroupId(groupId);
      if (group != null) {
        group
          ..epoch = epoch
          ..memberCount = members.length
          ..updatedAtMillis = now;
        await isar.chatGroupEntitys.putByGroupId(group);
      }
    });
  }

  Future<ChatGroup?> readGroup(String groupId) {
    return _walletIsar.read((isar) async {
      final group = await isar.chatGroupEntitys.getByGroupId(groupId);
      if (group == null) return null;
      final members = await isar.chatGroupMemberEntitys
          .filter()
          .groupIdEqualTo(groupId)
          .findAll();
      return _groupFromEntities(group, members);
    });
  }

  Future<List<ChatGroup>> readGroups({String? ownerAccount}) {
    return _walletIsar.read((isar) async {
      final groups = await isar.chatGroupEntitys
          .filter()
          .idGreaterThan(0, include: true)
          .findAll();
      final filtered = ownerAccount == null || ownerAccount.isEmpty
          ? groups
          : groups
              .where((row) => row.ownerAccount == ownerAccount)
              .toList(growable: false);
      final result = <ChatGroup>[];
      for (final group in filtered) {
        final members = await isar.chatGroupMemberEntitys
            .filter()
            .groupIdEqualTo(group.groupId)
            .findAll();
        result.add(_groupFromEntities(group, members));
      }
      return result;
    });
  }

  /// 退群/被移除:本机标记已退,停止参与。
  Future<void> markGroupLeft(String groupId) {
    return _walletIsar.writeTxn((isar) async {
      final group = await isar.chatGroupEntitys.getByGroupId(groupId);
      if (group != null) {
        group
          ..leftLocally = true
          ..updatedAtMillis = DateTime.now().millisecondsSinceEpoch;
        await isar.chatGroupEntitys.putByGroupId(group);
      }
    });
  }

  /// 改群名(群记录 + 群会话 title 同步)。空名忽略。
  Future<void> renameGroup(String groupId, String name) {
    final trimmed = name.trim();
    if (trimmed.isEmpty) {
      return Future<void>.value();
    }
    return _walletIsar.writeTxn((isar) async {
      final now = DateTime.now().millisecondsSinceEpoch;
      final group = await isar.chatGroupEntitys.getByGroupId(groupId);
      if (group != null) {
        group
          ..groupName = trimmed
          ..updatedAtMillis = now;
        await isar.chatGroupEntitys.putByGroupId(group);
      }
      final conversation =
          await isar.chatConversationEntitys.getByConversationId(groupId);
      if (conversation != null) {
        conversation.title = trimmed;
        await isar.chatConversationEntitys.putByConversationId(conversation);
      }
    });
  }

  /// 缓冲一条乱序群 Commit(键 groupId+messageEpoch)。
  Future<void> bufferGroupCommit({
    required String groupId,
    required int messageEpoch,
    required ChatEnvelope envelope,
    required List<int> envelopeBytes,
  }) {
    return _walletIsar.writeTxn((isar) async {
      await isar.chatGroupPendingCommitEntitys.putByEnvelopeId(
        ChatGroupPendingCommitEntity()
          ..envelopeId = envelope.envelopeId
          ..groupId = groupId
          ..messageEpoch = messageEpoch
          ..envelopeBytesHex = _bytesToHex(envelopeBytes)
          ..createdAtMillis = DateTime.now().millisecondsSinceEpoch,
      );
    });
  }

  /// 取出并删除某 (groupId, messageEpoch) 下最早的一条缓冲;无则 null。
  Future<ChatEnvelope?> takeGroupPendingCommit(
    String groupId,
    int messageEpoch,
  ) {
    return _walletIsar.writeTxn((isar) async {
      final rows = await isar.chatGroupPendingCommitEntitys
          .filter()
          .groupIdEqualTo(groupId)
          .messageEpochEqualTo(messageEpoch)
          .findAll();
      if (rows.isEmpty) return null;
      rows.sort((a, b) => a.createdAtMillis.compareTo(b.createdAtMillis));
      final row = rows.first;
      await isar.chatGroupPendingCommitEntitys.delete(row.id);
      return ChatEnvelope.fromBuffer(_hexToBytes(row.envelopeBytesHex));
    });
  }

  /// 群发出:一条逻辑消息 + N 条按收件人的出站队列(投递/重试复用 1:1 路径)。
  Future<void> saveOutgoingGroupMessage({
    required String groupId,
    required String senderAccount,
    required String senderDeviceId,
    required String logicalEnvelopeId,
    required ChatMessageKind messageKind,
    required String payload,
    required int createdAtMillis,
    required List<ChatEnvelope> envelopes,
  }) {
    return _walletIsar.writeTxn((isar) async {
      await _touchGroupConversationInTxn(
        isar: isar,
        groupId: groupId,
        ownerAccount: senderAccount,
        lastMessage: _messageSummary(payload),
        lastUpdatedAtMillis: createdAtMillis,
        unreadDelta: 0,
        deliveryState: ChatMessageDeliveryState.queued,
      );
      await isar.chatMessageEntitys.putByEnvelopeId(
        ChatMessageEntity()
          ..envelopeId = logicalEnvelopeId
          ..conversationId = groupId
          ..ownerAccount = senderAccount
          ..direction = 'outgoing'
          ..senderAccount = senderAccount
          ..recipientAccount = groupId
          ..senderDeviceId = senderDeviceId
          ..messageKind = messageKind.name
          ..mlsMessageKind = MlsWireMessageKind.MLS_WIRE_MESSAGE_KIND_APPLICATION.name
          ..deliveryState = ChatMessageDeliveryState.queued.name
          ..plaintext = payload
          ..envelopeBytesHex = ''
          ..createdAtMillis = createdAtMillis,
      );
      for (final envelope in envelopes) {
        await isar.chatOutboundQueueEntitys.putByEnvelopeId(
          ChatOutboundQueueEntity()
            ..envelopeId = envelope.envelopeId
            ..conversationId = groupId
            ..recipientAccount = envelope.recipientAccount
            ..envelopeBytesHex = _bytesToHex(envelope.writeToBuffer())
            ..deliveryState = ChatMessageDeliveryState.queued.name
            ..attemptCount = 0
            ..lastError = null
            ..updatedAtMillis = DateTime.now().millisecondsSinceEpoch,
        );
      }
    });
  }

  /// 群收到:一条入站逻辑消息(该成员就收到一封)。会话保持群名,不被发送方覆盖。
  Future<void> saveIncomingGroupMessage({
    required ChatEnvelope envelope,
    required List<int> envelopeBytes,
    required ChatMessageKind messageKind,
    required String plaintext,
  }) {
    return _walletIsar.writeTxn((isar) async {
      await _touchGroupConversationInTxn(
        isar: isar,
        groupId: envelope.conversationId,
        ownerAccount: envelope.recipientAccount,
        lastMessage: _messageSummary(plaintext),
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

  /// 更新群会话的 lastMessage/未读/投递态,但保留群名 title 与 conversationKind。
  Future<void> _touchGroupConversationInTxn({
    required Isar isar,
    required String groupId,
    required String ownerAccount,
    required String lastMessage,
    required int lastUpdatedAtMillis,
    required int unreadDelta,
    required ChatMessageDeliveryState deliveryState,
  }) async {
    final existing =
        await isar.chatConversationEntitys.getByConversationId(groupId);
    final group = await isar.chatGroupEntitys.getByGroupId(groupId);
    final entity = existing ?? ChatConversationEntity();
    entity
      ..conversationId = groupId
      ..ownerAccount =
          existing?.ownerAccount.isNotEmpty == true ? existing!.ownerAccount : ownerAccount
      ..peerAccount = existing?.peerAccount ?? (group?.creatorAccount ?? '')
      ..title = group?.groupName ?? existing?.title ?? groupId
      ..conversationKind = 'group'
      ..lastMessage = lastMessage
      ..lastUpdatedAtMillis = lastUpdatedAtMillis
      ..unreadCount = (existing?.unreadCount ?? 0) + unreadDelta
      ..lastDeliveryState = deliveryState.name;
    await isar.chatConversationEntitys.putByConversationId(entity);
  }

  ChatGroup _groupFromEntities(
    ChatGroupEntity group,
    List<ChatGroupMemberEntity> members,
  ) {
    return ChatGroup(
      groupId: group.groupId,
      name: group.groupName,
      creatorAccount: group.creatorAccount,
      epoch: group.epoch,
      leftLocally: group.leftLocally,
      roster: members
          .map((row) => GroupMember(
                account: row.memberAccount,
                role: GroupMemberRole.fromName(row.role),
              ))
          .toList(growable: false),
    );
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
    conversationKind: row.conversationKind ?? 'dm',
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

String _messageSummary(String? plaintext) {
  // 摘要一律从载荷解码:文本取正文,媒体/贴纸取类型化占位([图片]/[视频]/
  // [文件] 名/[贴纸])。解码对裸文本或历史数据都退化为纯文本,故安全。
  return ChatPayloadCodec.decode(plaintext ?? '').summary;
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
