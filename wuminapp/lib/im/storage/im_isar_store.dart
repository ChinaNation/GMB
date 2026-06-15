import 'package:isar_community/isar.dart';

import '../../isar/wallet_isar.dart';
import '../im_session_models.dart';
import '../proto/im_envelope.pb.dart';

/// IM 本地消息记录。
class ImStoredMessage {
  const ImStoredMessage({
    required this.envelopeId,
    required this.conversationId,
    required this.direction,
    required this.senderChatAccount,
    required this.recipientChatAccount,
    required this.messageKind,
    required this.deliveryState,
    required this.createdAtMillis,
    this.plaintext,
  });

  final String envelopeId;
  final String conversationId;
  final String direction;
  final String senderChatAccount;
  final String recipientChatAccount;
  final ImMessageKind messageKind;
  final ImMessageDeliveryState deliveryState;
  final int createdAtMillis;
  final String? plaintext;
}

/// IM 路由缓存记录。
class ImRouteRecord {
  const ImRouteRecord({
    required this.walletChatAccount,
    required this.displayName,
    required this.deviceId,
    required this.devicePublicKeyHex,
    required this.safetyNumber,
    required this.nodePeerId,
    required this.nodeMultiaddr,
    this.note,
    this.createdAtMillis,
    this.updatedAtMillis,
  });

  final String walletChatAccount;
  final String displayName;
  final String deviceId;
  final String devicePublicKeyHex;
  final String safetyNumber;
  final String nodePeerId;
  final String nodeMultiaddr;
  final String? note;
  final int? createdAtMillis;
  final int? updatedAtMillis;

  String get routeId => walletChatAccount;
}

/// 公民 IM 的 Isar 持久化仓库。
///
/// 中文注释：本仓库只保存手机本地状态。节点投递层只拿到完整 Protobuf
/// envelope bytes，不会接触 [plaintext]。
class ImIsarStore {
  ImIsarStore({
    WalletIsar? walletIsar,
  }) : _walletIsar = walletIsar ?? WalletIsar.instance;

  final WalletIsar _walletIsar;

  Future<List<ImConversationPreview>> readConversationPreviews({
    String? ownerChatAccount,
  }) {
    return _walletIsar.read((isar) async {
      final rows = await isar.imConversationEntitys
          .filter()
          .idGreaterThan(0, include: true)
          .findAll();
      final filtered = ownerChatAccount == null || ownerChatAccount.isEmpty
          ? rows
          : rows
              .where((row) => row.ownerChatAccount == ownerChatAccount)
              .toList(growable: false);
      filtered.sort(
          (a, b) => b.lastUpdatedAtMillis.compareTo(a.lastUpdatedAtMillis));
      return filtered
          .map(_conversationPreviewFromEntity)
          .toList(growable: false);
    });
  }

  Future<List<ImRouteRecord>> readRouteRecords() {
    return _walletIsar.read((isar) async {
      final rows = await isar.imRouteCacheEntitys
          .filter()
          .idGreaterThan(0, include: true)
          .findAll();
      rows.sort((a, b) => a.displayName.compareTo(b.displayName));
      return rows.map(_routeFromEntity).toList(growable: false);
    });
  }

  Future<ImRouteRecord?> getRouteRecord(String walletChatAccount) {
    return _walletIsar.read((isar) async {
      final row = await isar.imRouteCacheEntitys
          .getByWalletChatAccount(walletChatAccount);
      return row == null ? null : _routeFromEntity(row);
    });
  }

  Future<void> upsertRouteRecord(ImRouteRecord route) {
    return _walletIsar.writeTxn((isar) async {
      final now = DateTime.now().millisecondsSinceEpoch;
      final existing =
          await isar.imRouteCacheEntitys.getByRouteId(route.routeId);
      final entity = existing ?? ImRouteCacheEntity();
      entity
        ..routeId = route.routeId
        ..walletChatAccount = route.walletChatAccount
        ..displayName = route.displayName
        ..deviceId = route.deviceId
        ..devicePublicKeyHex = route.devicePublicKeyHex
        ..safetyNumber = route.safetyNumber
        ..nodePeerId = route.nodePeerId
        ..nodeMultiaddr = route.nodeMultiaddr
        ..note = route.note
        ..createdAtMillis =
            existing?.createdAtMillis ?? route.createdAtMillis ?? now
        ..updatedAtMillis = route.updatedAtMillis ?? now;
      await isar.imRouteCacheEntitys.putByRouteId(entity);
    });
  }

  Future<List<ImStoredMessage>> readMessages(String conversationId) {
    return _walletIsar.read((isar) async {
      final rows = await isar.imMessageEntitys
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

  Future<void> saveOutgoingEnvelope({
    required ImEnvelope envelope,
    required List<int> envelopeBytes,
    required ImMessageKind messageKind,
    required ImMessageDeliveryState deliveryState,
    String? plaintext,
  }) {
    return _walletIsar.writeTxn((isar) async {
      await _putConversationInTxn(
        isar: isar,
        conversationId: envelope.conversationId,
        ownerChatAccount: envelope.senderChatAccount,
        peerChatAccount: envelope.recipientChatAccount,
        title: envelope.recipientChatAccount,
        lastMessage: _messageSummary(messageKind, plaintext),
        lastUpdatedAtMillis: envelope.createdAtMillis.toInt(),
        unreadDelta: 0,
        deliveryState: deliveryState,
      );
      await isar.imMessageEntitys.putByEnvelopeId(
        _messageEntity(
          envelope: envelope,
          envelopeBytes: envelopeBytes,
          ownerChatAccount: envelope.senderChatAccount,
          direction: 'outgoing',
          messageKind: messageKind,
          deliveryState: deliveryState,
          plaintext: plaintext,
        ),
      );
      await isar.imOutboundQueueEntitys.putByEnvelopeId(
        ImOutboundQueueEntity()
          ..envelopeId = envelope.envelopeId
          ..conversationId = envelope.conversationId
          ..recipientChatAccount = envelope.recipientChatAccount
          ..envelopeBytesHex = _bytesToHex(envelopeBytes)
          ..deliveryState = deliveryState.name
          ..attemptCount = 0
          ..lastError = null
          ..updatedAtMillis = DateTime.now().millisecondsSinceEpoch,
      );
    });
  }

  Future<void> queueOutgoingEnvelope({
    required ImEnvelope envelope,
    required List<int> envelopeBytes,
    required ImMessageDeliveryState deliveryState,
  }) {
    return _walletIsar.writeTxn((isar) async {
      await isar.imOutboundQueueEntitys.putByEnvelopeId(
        ImOutboundQueueEntity()
          ..envelopeId = envelope.envelopeId
          ..conversationId = envelope.conversationId
          ..recipientChatAccount = envelope.recipientChatAccount
          ..envelopeBytesHex = _bytesToHex(envelopeBytes)
          ..deliveryState = deliveryState.name
          ..attemptCount = 0
          ..lastError = null
          ..updatedAtMillis = DateTime.now().millisecondsSinceEpoch,
      );
    });
  }

  Future<void> saveIncomingEnvelope({
    required ImEnvelope envelope,
    required List<int> envelopeBytes,
    required ImMessageKind messageKind,
    required String plaintext,
  }) {
    return _walletIsar.writeTxn((isar) async {
      await _putConversationInTxn(
        isar: isar,
        conversationId: envelope.conversationId,
        ownerChatAccount: envelope.recipientChatAccount,
        peerChatAccount: envelope.senderChatAccount,
        title: envelope.senderChatAccount,
        lastMessage: _messageSummary(messageKind, plaintext),
        lastUpdatedAtMillis: envelope.createdAtMillis.toInt(),
        unreadDelta: 1,
        deliveryState: ImMessageDeliveryState.receivedByDevice,
      );
      await isar.imMessageEntitys.putByEnvelopeId(
        _messageEntity(
          envelope: envelope,
          envelopeBytes: envelopeBytes,
          ownerChatAccount: envelope.recipientChatAccount,
          direction: 'incoming',
          messageKind: messageKind,
          deliveryState: ImMessageDeliveryState.receivedByDevice,
          plaintext: plaintext,
        ),
      );
    });
  }

  Future<void> markOutgoingDelivery({
    required String envelopeId,
    required ImMessageDeliveryState state,
    String? errorMessage,
  }) {
    return _walletIsar.writeTxn((isar) async {
      final queue =
          await isar.imOutboundQueueEntitys.getByEnvelopeId(envelopeId);
      if (queue != null) {
        queue
          ..deliveryState = state.name
          ..attemptCount = queue.attemptCount + 1
          ..lastError = errorMessage
          ..updatedAtMillis = DateTime.now().millisecondsSinceEpoch;
        await isar.imOutboundQueueEntitys.putByEnvelopeId(queue);
      }
      final message = await isar.imMessageEntitys.getByEnvelopeId(envelopeId);
      if (message != null) {
        message.deliveryState = state.name;
        await isar.imMessageEntitys.putByEnvelopeId(message);
        final conversation = await isar.imConversationEntitys
            .getByConversationId(message.conversationId);
        if (conversation != null) {
          conversation.lastDeliveryState = state.name;
          await isar.imConversationEntitys.putByConversationId(conversation);
        }
      }
    });
  }

  Future<void> savePendingInbound({
    required ImEnvelope envelope,
    required List<int> envelopeBytes,
    required String reason,
  }) {
    return _walletIsar.writeTxn((isar) async {
      await isar.imPendingInboundEntitys.putByEnvelopeId(
        ImPendingInboundEntity()
          ..envelopeId = envelope.envelopeId
          ..conversationId = envelope.conversationId
          ..envelopeBytesHex = _bytesToHex(envelopeBytes)
          ..reason = reason
          ..createdAtMillis = DateTime.now().millisecondsSinceEpoch,
      );
    });
  }

  Future<List<ImEnvelope>> takePendingInbound(String conversationId) {
    return _walletIsar.writeTxn((isar) async {
      final rows = await isar.imPendingInboundEntitys
          .filter()
          .idGreaterThan(0, include: true)
          .findAll();
      final matched = rows
          .where((row) => row.conversationId == conversationId)
          .toList(growable: false)
        ..sort((a, b) => a.createdAtMillis.compareTo(b.createdAtMillis));
      for (final row in matched) {
        await isar.imPendingInboundEntitys.delete(row.id);
      }
      return matched
          .map(
              (row) => ImEnvelope.fromBuffer(_hexToBytes(row.envelopeBytesHex)))
          .toList(growable: false);
    });
  }

  Future<int> pendingInboundCount() {
    return _walletIsar.read((isar) async {
      final rows = await isar.imPendingInboundEntitys
          .filter()
          .idGreaterThan(0, include: true)
          .findAll();
      return rows.length;
    });
  }

  Future<int> outboundQueueCount() {
    return _walletIsar.read((isar) async {
      final rows = await isar.imOutboundQueueEntitys
          .filter()
          .idGreaterThan(0, include: true)
          .findAll();
      return rows.length;
    });
  }

  Future<void> _putConversationInTxn({
    required Isar isar,
    required String conversationId,
    required String ownerChatAccount,
    required String peerChatAccount,
    required String title,
    required String lastMessage,
    required int lastUpdatedAtMillis,
    required int unreadDelta,
    required ImMessageDeliveryState deliveryState,
  }) async {
    final existing =
        await isar.imConversationEntitys.getByConversationId(conversationId);
    final entity = existing ?? ImConversationEntity();
    entity
      ..conversationId = conversationId
      ..ownerChatAccount = ownerChatAccount
      ..peerChatAccount = peerChatAccount
      ..title = title
      ..lastMessage = lastMessage
      ..lastUpdatedAtMillis = lastUpdatedAtMillis
      ..unreadCount = (existing?.unreadCount ?? 0) + unreadDelta
      ..lastDeliveryState = deliveryState.name;
    await isar.imConversationEntitys.putByConversationId(entity);
  }
}

ImConversationPreview _conversationPreviewFromEntity(ImConversationEntity row) {
  return ImConversationPreview(
    conversationId: row.conversationId,
    title: row.title,
    walletAddress: row.peerChatAccount,
    lastMessage: row.lastMessage,
    lastUpdatedAt: DateTime.fromMillisecondsSinceEpoch(row.lastUpdatedAtMillis),
    unreadCount: row.unreadCount,
    deliveryState: _deliveryStateFromName(row.lastDeliveryState),
  );
}

ImStoredMessage _messageFromEntity(ImMessageEntity row) {
  return ImStoredMessage(
    envelopeId: row.envelopeId,
    conversationId: row.conversationId,
    direction: row.direction,
    senderChatAccount: row.senderChatAccount,
    recipientChatAccount: row.recipientChatAccount,
    messageKind: _messageKindFromName(row.messageKind),
    deliveryState: _deliveryStateFromName(row.deliveryState),
    createdAtMillis: row.createdAtMillis,
    plaintext: row.plaintext,
  );
}

ImRouteRecord _routeFromEntity(ImRouteCacheEntity row) {
  return ImRouteRecord(
    walletChatAccount: row.walletChatAccount,
    displayName: row.displayName,
    deviceId: row.deviceId,
    devicePublicKeyHex: row.devicePublicKeyHex,
    safetyNumber: row.safetyNumber,
    nodePeerId: row.nodePeerId,
    nodeMultiaddr: row.nodeMultiaddr,
    note: row.note,
    createdAtMillis: row.createdAtMillis,
    updatedAtMillis: row.updatedAtMillis,
  );
}

ImMessageEntity _messageEntity({
  required ImEnvelope envelope,
  required List<int> envelopeBytes,
  required String ownerChatAccount,
  required String direction,
  required ImMessageKind messageKind,
  required ImMessageDeliveryState deliveryState,
  String? plaintext,
}) {
  return ImMessageEntity()
    ..envelopeId = envelope.envelopeId
    ..conversationId = envelope.conversationId
    ..ownerChatAccount = ownerChatAccount
    ..direction = direction
    ..senderChatAccount = envelope.senderChatAccount
    ..recipientChatAccount = envelope.recipientChatAccount
    ..senderDeviceId = envelope.senderDeviceId
    ..messageKind = messageKind.name
    ..mlsMessageKind = envelope.mlsMessageKind.name
    ..deliveryState = deliveryState.name
    ..plaintext = plaintext
    ..envelopeBytesHex = _bytesToHex(envelopeBytes)
    ..createdAtMillis = envelope.createdAtMillis.toInt();
}

String _messageSummary(ImMessageKind kind, String? plaintext) {
  return switch (kind) {
    ImMessageKind.text => plaintext ?? '',
    ImMessageKind.attachment => '[附件]',
  };
}

ImMessageDeliveryState _deliveryStateFromName(String value) {
  return ImMessageDeliveryState.values.firstWhere(
    (item) => item.name == value,
    orElse: () => ImMessageDeliveryState.failed,
  );
}

ImMessageKind _messageKindFromName(String value) {
  return ImMessageKind.values.firstWhere(
    (item) => item.name == value,
    orElse: () => ImMessageKind.text,
  );
}

String _bytesToHex(List<int> bytes) {
  return bytes.map((item) => item.toRadixString(16).padLeft(2, '0')).join();
}

List<int> _hexToBytes(String value) {
  final normalized = value.startsWith('0x') ? value.substring(2) : value;
  if (normalized.length.isOdd) {
    throw const FormatException('IM envelope hex 长度必须为偶数');
  }
  final bytes = <int>[];
  for (var i = 0; i < normalized.length; i += 2) {
    bytes.add(int.parse(normalized.substring(i, i + 2), radix: 16));
  }
  return bytes;
}
