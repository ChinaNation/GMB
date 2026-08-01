import 'dart:convert';

import 'package:isar_community/isar.dart';

import '../../isar/app_isar.dart';
import '../../security/local_data_key.dart';
import '../chat_models.dart';
import '../chat_payload.dart';
import '../group/group_model.dart';
import '../proto/chat_envelope.pb.dart';
import 'chat_crypto.dart';

/// Chat 本地消息记录。
class ChatStoredMessage {
  const ChatStoredMessage({
    required this.envelopeId,
    required this.conversationId,
    required this.direction,
    required this.senderCidNumber,
    required this.recipientCidNumber,
    required this.messageKind,
    required this.deliveryState,
    required this.createdAtMillis,
    this.plaintext,
  });

  final String envelopeId;
  final String conversationId;
  final String direction;
  final String senderCidNumber;
  final String recipientCidNumber;
  final ChatMessageKind messageKind;
  final ChatMessageDeliveryState deliveryState;
  final int createdAtMillis;
  final String? plaintext;
}

/// 仅保存在发送设备上的待重试密文。
class ChatQueuedEnvelope {
  const ChatQueuedEnvelope({
    required this.envelopeId,
    required this.recipientCidNumber,
    required this.envelopeBytes,
  });

  final String envelopeId;

  /// 收件人身份主键 CID，也是信封与 Worker 的唯一投递键。
  final String recipientCidNumber;
  final List<int> envelopeBytes;
}

/// 待设备投递的媒体(离线补发)。缓存路径在补发时由 conversationId/attachmentId/
/// fileName 用当前 Documents 目录重算,不持久化绝对路径。
class ChatPendingMedia {
  const ChatPendingMedia({
    required this.attachmentId,
    required this.recipientCidNumber,
    required this.conversationId,
    required this.fileName,
    required this.contentType,
    required this.byteSize,
  });

  final String attachmentId;

  /// 收件人身份主键 CID 号（WebRTC 补发按 CID 路由信令）。
  final String recipientCidNumber;
  final String conversationId;
  final String fileName;
  final String contentType;
  final int byteSize;
}

/// Chat 路由缓存记录。
class ChatRoute {
  const ChatRoute({
    required this.peerCidNumber,
    required this.routeDisplayName,
    required this.deviceId,
    required this.devicePublicKey,
    required this.safetyNumber,
    this.nearbyPeerHint,
    this.note,
    this.createdAtMillis,
    this.updatedAtMillis,
  });

  final String peerCidNumber;
  final String routeDisplayName;
  final String deviceId;
  final String devicePublicKey;
  final String safetyNumber;
  final String? nearbyPeerHint;
  final String? note;
  final int? createdAtMillis;
  final int? updatedAtMillis;
}

/// 公民 Chat 的 Isar 持久化仓库。
///
/// 本仓库只保存手机本地状态。Cloudflare 瞬时转发和近场 transport 只拿到完整
/// Protobuf envelope bytes，不会接触 [plaintext]。
class ChatStore {
  ChatStore({
    WalletIsar? walletIsar,
    ChatCrypto? crypto,
  })  : _walletIsar = walletIsar ?? WalletIsar.instance,
        _crypto = crypto ?? ChatCrypto();

  final WalletIsar _walletIsar;

  /// 聊天本地密文的唯一加解密边界。加解密一律在 Isar 事务**之外**完成,
  /// 不让密码学运算占住写事务。
  final ChatCrypto _crypto;

  static const String _handoverPrefix = 'chat_handover_by_cid:';

  /// 把正文加密成密文 + 搜索索引;正文为空时返回空密文与空索引。
  Future<_SealedMessage> _sealMessage({
    required String ownerCidNumber,
    required String currentAccountId,
    required String envelopeId,
    required String? plaintext,
  }) async {
    if (plaintext == null || plaintext.isEmpty) {
      return const _SealedMessage(cipher: null, tokens: <String>[]);
    }
    return _SealedMessage(
      cipher: await _crypto.encryptText(
        ownerCidNumber: ownerCidNumber,
        currentAccountId: currentAccountId,
        recordId: envelopeId,
        plaintext: plaintext,
      ),
      // 索引建在**摘要**上,与搜索时的匹配口径一致(媒体/贴纸取类型化占位)。
      tokens: await _crypto.buildSearchTokens(
        ownerCidNumber: ownerCidNumber,
        currentAccountId: currentAccountId,
        text: _messageSummary(plaintext),
      ),
    );
  }

  Future<String> _sealSummary({
    required String ownerCidNumber,
    required String currentAccountId,
    required String conversationId,
    required String? plaintext,
  }) =>
      _crypto.encryptText(
        ownerCidNumber: ownerCidNumber,
        currentAccountId: currentAccountId,
        recordId: conversationId,
        plaintext: _messageSummary(plaintext),
      );

  /// 换绑交易提交前，把全部聊天正文、会话摘要和搜索索引预演成目标账户密文。
  ///
  /// 正式聊天行不改动；暂存清单只含目标密文、记录 ID 和 HMAC token，不含明文。
  /// 任一此前密文认证失败都会整体中止，禁止带着半套历史记录继续换绑。
  Future<void> stageAccountHandover({
    required AccountDataBinding source,
    required AccountDataBinding target,
  }) async {
    _validateHandover(source, target);
    final sourceKeys = await _crypto.handoverKeys(source);
    final targetKeys = await _crypto.handoverKeys(target);
    try {
      final snapshot = await _walletIsar.read((isar) async {
        final conversations = (await isar.chatConversationEntitys
                .filter()
                .idGreaterThan(0, include: true)
                .findAll())
            .where((row) =>
                row.ownerCidNumber == source.cidNumber &&
                row.bindingRevision == source.bindingRevision &&
                row.accountId == source.accountId)
            .toList(growable: false);
        final messages = (await isar.chatMessageEntitys
                .filter()
                .idGreaterThan(0, include: true)
                .findAll())
            .where((row) =>
                row.ownerCidNumber == source.cidNumber &&
                row.bindingRevision == source.bindingRevision &&
                row.accountId == source.accountId)
            .toList(growable: false);
        return (conversations: conversations, messages: messages);
      });
      final conversations = <Map<String, Object?>>[];
      for (final row in snapshot.conversations) {
        final plaintext = await _crypto.decryptForHandover(
          binding: source,
          keys: sourceKeys,
          recordId: row.conversationId,
          blob: row.lastMessageCipher,
        );
        final targetCipher = await _crypto.encryptForHandover(
          binding: target,
          keys: targetKeys,
          recordId: row.conversationId,
          plaintext: plaintext,
        );
        final verified = await _crypto.decryptForHandover(
          binding: target,
          keys: targetKeys,
          recordId: row.conversationId,
          blob: targetCipher,
        );
        if (verified != plaintext) {
          throw StateError('聊天会话摘要新账户密文回读不一致');
        }
        conversations.add(<String, Object?>{
          'id': row.id,
          'cipher': targetCipher,
        });
      }
      final messages = <Map<String, Object?>>[];
      for (final row in snapshot.messages) {
        final blob = row.plaintextCipher;
        if (blob == null || blob.isEmpty) {
          messages.add(<String, Object?>{
            'id': row.id,
            'cipher': null,
            'tokens': const <String>[],
          });
          continue;
        }
        final plaintext = await _crypto.decryptForHandover(
          binding: source,
          keys: sourceKeys,
          recordId: row.envelopeId,
          blob: blob,
        );
        final targetCipher = await _crypto.encryptForHandover(
          binding: target,
          keys: targetKeys,
          recordId: row.envelopeId,
          plaintext: plaintext,
        );
        final verified = await _crypto.decryptForHandover(
          binding: target,
          keys: targetKeys,
          recordId: row.envelopeId,
          blob: targetCipher,
        );
        if (verified != plaintext) {
          throw StateError('聊天正文新账户密文回读不一致');
        }
        messages.add(<String, Object?>{
          'id': row.id,
          'cipher': targetCipher,
          'tokens': await _crypto.searchTokensForHandover(
            keys: targetKeys,
            text: _messageSummary(plaintext),
          ),
        });
      }
      final key = _handoverKey(target);
      final value = jsonEncode(<String, Object?>{
        'source': source.toJson(),
        'target': target.toJson(),
        'conversations': conversations,
        'messages': messages,
      });
      await _walletIsar.writeTxn((isar) async {
        final row = await isar.appKvEntitys.getByKey(key) ?? AppKvEntity();
        row
          ..key = key
          ..stringValue = value;
        await isar.appKvEntitys.put(row);
      });
    } finally {
      sourceKeys.dispose();
      targetKeys.dispose();
    }
  }

  /// finalized 后一次 Isar 事务切换全部聊天密文；可安全重试。
  Future<void> commitAccountHandover({
    required AccountDataBinding source,
    required AccountDataBinding target,
  }) async {
    _validateHandover(source, target);
    final key = _handoverKey(target);
    final raw = await _walletIsar.read(
        (isar) async => (await isar.appKvEntitys.getByKey(key))?.stringValue);
    if (raw == null || raw.isEmpty) return;
    final value = jsonDecode(raw);
    if (value is! Map<String, dynamic>) {
      throw const FormatException('聊天换绑交接清单损坏');
    }
    final conversations = (value['conversations'] as List?)
            ?.whereType<Map<String, dynamic>>()
            .toList(growable: false) ??
        const <Map<String, dynamic>>[];
    final messages = (value['messages'] as List?)
            ?.whereType<Map<String, dynamic>>()
            .toList(growable: false) ??
        const <Map<String, dynamic>>[];
    await _walletIsar.writeTxn((isar) async {
      for (final item in conversations) {
        final id = item['id'];
        final cipher = item['cipher'];
        if (id is! int || cipher is! String) {
          throw const FormatException('聊天会话交接项损坏');
        }
        final row = await isar.chatConversationEntitys.get(id);
        if (row == null || row.ownerCidNumber != target.cidNumber) {
          throw const FormatException('聊天会话交接目标不存在');
        }
        row
          ..bindingRevision = target.bindingRevision
          ..accountId = target.accountId
          ..lastMessageCipher = cipher;
        await isar.chatConversationEntitys.put(row);
      }
      for (final item in messages) {
        final id = item['id'];
        final tokens = item['tokens'];
        if (id is! int || tokens is! List) {
          throw const FormatException('聊天消息交接项损坏');
        }
        final row = await isar.chatMessageEntitys.get(id);
        if (row == null || row.ownerCidNumber != target.cidNumber) {
          throw const FormatException('聊天消息交接目标不存在');
        }
        final cipher = item['cipher'];
        if (cipher != null && cipher is! String) {
          throw const FormatException('聊天消息密文交接项损坏');
        }
        row
          ..bindingRevision = target.bindingRevision
          ..accountId = target.accountId
          ..plaintextCipher = cipher as String?
          ..searchTokens = tokens.whereType<String>().toList(growable: false);
        await isar.chatMessageEntitys.put(row);
      }
      final manifest = await isar.appKvEntitys.getByKey(key);
      if (manifest != null) await isar.appKvEntitys.delete(manifest.id);
    });
  }

  Future<void> discardAccountHandover(AccountDataBinding target) async {
    final key = _handoverKey(target);
    await _walletIsar.writeTxn((isar) async {
      final row = await isar.appKvEntitys.getByKey(key);
      if (row != null) await isar.appKvEntitys.delete(row.id);
    });
  }

  static void _validateHandover(
    AccountDataBinding source,
    AccountDataBinding target,
  ) {
    source.validate();
    target.validate();
    if (source.genesisHash != target.genesisHash ||
        source.cidNumber != target.cidNumber ||
        target.bindingRevision != source.bindingRevision + 1 ||
        source.accountId == target.accountId) {
      throw const FormatException('聊天换绑交接上下文不合法');
    }
  }

  static String _handoverKey(AccountDataBinding target) =>
      '$_handoverPrefix${target.cidNumber}:${target.bindingRevision}:${target.accountId}';

  Future<String?> _openMessage(
    ChatMessageEntity row,
    String currentAccountId,
  ) async {
    final cipher = row.plaintextCipher;
    if (cipher == null || cipher.isEmpty) return null;
    return _crypto.decryptText(
      ownerCidNumber: row.ownerCidNumber,
      currentAccountId: currentAccountId,
      recordId: row.envelopeId,
      blob: cipher,
    );
  }

  Future<String> _openSummary(
    ChatConversationEntity row,
    String currentAccountId,
  ) =>
      _crypto.decryptText(
        ownerCidNumber: row.ownerCidNumber,
        currentAccountId: currentAccountId,
        recordId: row.conversationId,
        blob: row.lastMessageCipher,
      );

  Future<List<ChatConversationPreview>> readConversationPreviews({
    required String ownerCidNumber,
    required String currentAccountId,
  }) async {
    final binding = await _crypto.resolveCipherBinding(
      ownerCidNumber: ownerCidNumber,
      currentAccountId: currentAccountId,
    );
    return _walletIsar.read((isar) async {
      final rows = await isar.chatConversationEntitys
          .filter()
          .idGreaterThan(0, include: true)
          .findAll();
      final filtered = rows
          .where((row) =>
              row.ownerCidNumber == ownerCidNumber &&
              row.bindingRevision == binding.bindingRevision &&
              row.accountId == binding.accountId)
          .toList(growable: false);
      filtered.sort(
          (a, b) => b.lastUpdatedAtMillis.compareTo(a.lastUpdatedAtMillis));
      final out = <ChatConversationPreview>[];
      for (final row in filtered) {
        out.add(_conversationPreviewFromEntity(
          row,
          await _openSummary(row, currentAccountId),
        ));
      }
      return List<ChatConversationPreview>.unmodifiable(out);
    });
  }

  Future<List<ChatRoute>> readRouteRecords(String ownerCidNumber) {
    return _walletIsar.read((isar) async {
      final rows = await isar.chatRouteCacheEntitys
          .filter()
          .idGreaterThan(0, include: true)
          .findAll();
      final owned = rows
          .where((row) => row.ownerCidNumber == ownerCidNumber)
          .toList(growable: false)
        ..sort((a, b) => a.routeDisplayName.compareTo(b.routeDisplayName));
      return owned.map(_routeFromEntity).toList(growable: false);
    });
  }

  Future<ChatRoute?> getRouteRecord(
    String ownerCidNumber,
    String peerCidNumber,
  ) {
    return _walletIsar.read((isar) async {
      final row =
          await isar.chatRouteCacheEntitys.getByOwnerCidNumberPeerCidNumber(
        ownerCidNumber,
        peerCidNumber,
      );
      return row == null ? null : _routeFromEntity(row);
    });
  }

  Future<void> upsertRouteRecord(String ownerCidNumber, ChatRoute route) {
    return _walletIsar.writeTxn((isar) async {
      final now = DateTime.now().millisecondsSinceEpoch;
      final existing =
          await isar.chatRouteCacheEntitys.getByOwnerCidNumberPeerCidNumber(
        ownerCidNumber,
        route.peerCidNumber,
      );
      final entity = existing ?? ChatRouteCacheEntity();
      entity
        ..ownerCidNumber = ownerCidNumber
        ..peerCidNumber = route.peerCidNumber
        ..routeDisplayName = route.routeDisplayName
        ..deviceId = route.deviceId
        ..devicePublicKey = route.devicePublicKey
        ..safetyNumber = route.safetyNumber
        ..nearbyPeerHint = route.nearbyPeerHint
        ..note = route.note
        ..createdAtMillis =
            existing?.createdAtMillis ?? route.createdAtMillis ?? now
        ..updatedAtMillis = route.updatedAtMillis ?? now;
      await isar.chatRouteCacheEntitys.putByOwnerCidNumberPeerCidNumber(entity);
    });
  }

  Future<List<ChatStoredMessage>> readMessages({
    required String ownerCidNumber,
    required String currentAccountId,
    required String conversationId,
  }) async {
    final binding = await _crypto.resolveCipherBinding(
      ownerCidNumber: ownerCidNumber,
      currentAccountId: currentAccountId,
    );
    return _walletIsar.read((isar) async {
      final rows = await isar.chatMessageEntitys
          .filter()
          .idGreaterThan(0, include: true)
          .findAll();
      final filtered = rows
          .where((row) =>
              row.ownerCidNumber == ownerCidNumber &&
              row.bindingRevision == binding.bindingRevision &&
              row.accountId == binding.accountId &&
              row.conversationId == conversationId)
          .toList(growable: false)
        ..sort((a, b) => a.createdAtMillis.compareTo(b.createdAtMillis));
      final out = <ChatStoredMessage>[];
      for (final row in filtered) {
        out.add(_messageFromEntity(
          row,
          await _openMessage(row, currentAccountId),
        ));
      }
      return List<ChatStoredMessage>.unmodifiable(out);
    });
  }

  /// 跨会话搜索本机聊天记录（聊天搜索页的「聊天记录」段）。
  ///
  /// 正文已在磁盘上加密，无法再做明文子串匹配，改为**两段式**：
  /// 1. 用 `LocalKeyPurpose.chatIndex` 子钥把查询串切成 HMAC bigram token，
  ///    经 `searchTokens` 多值索引取出**同时命中全部 token** 的候选；
  /// 2. 只对候选解密，再验一次真实子串。
  ///
  /// 第 2 步不可省：token 是 HMAC **截断值**，存在假阳性；且 bigram 命中不等于
  /// 原串顺序命中（查 "abc" 会命中含 "ab"、"bc" 但实为 "bcab" 的记录）。
  /// 复验保证结果与此前明文 `contains` 语义完全一致。
  ///
  /// 匹配口径仍是**摘要**（文本取正文，媒体/贴纸取类型化占位），与建索引时一致；
  /// 大小写不敏感。查询不足 2 字符时无 bigram 可用，回落到按属主 CID 收窄后
  /// 解密扫描——单字符查询在中文里很常见，不能直接拒绝。
  Future<List<ChatStoredMessage>> searchMessages({
    required String ownerCidNumber,
    required String currentAccountId,
    required String keyword,
    int limit = 50,
  }) async {
    final needle = keyword.trim().toLowerCase();
    if (needle.isEmpty || ownerCidNumber.isEmpty || currentAccountId.isEmpty) {
      return const <ChatStoredMessage>[];
    }
    final binding = await _crypto.resolveCipherBinding(
      ownerCidNumber: ownerCidNumber,
      currentAccountId: currentAccountId,
    );
    final tokens = await _crypto.buildQueryTokens(
      ownerCidNumber: ownerCidNumber,
      currentAccountId: currentAccountId,
      query: needle,
    );
    return _walletIsar.read((isar) async {
      List<ChatMessageEntity> candidates;
      if (tokens.isEmpty) {
        candidates = await isar.chatMessageEntitys
            .filter()
            .ownerCidNumberEqualTo(ownerCidNumber)
            .findAll();
      } else {
        var query = isar.chatMessageEntitys
            .filter()
            .ownerCidNumberEqualTo(ownerCidNumber)
            .and()
            .searchTokensElementEqualTo(tokens.first);
        for (final token in tokens.skip(1)) {
          query = query.and().searchTokensElementEqualTo(token);
        }
        candidates = await query.findAll();
      }
      candidates.sort((a, b) => b.createdAtMillis.compareTo(a.createdAtMillis));

      final hits = <ChatStoredMessage>[];
      for (final row in candidates) {
        if (hits.length >= limit) break;
        if (row.bindingRevision != binding.bindingRevision ||
            row.accountId != binding.accountId) {
          continue;
        }
        final plaintext = await _openMessage(row, currentAccountId);
        if (!_messageSummary(plaintext).toLowerCase().contains(needle)) {
          continue; // 索引假阳性，复验滤掉
        }
        hits.add(_messageFromEntity(row, plaintext));
      }
      return List<ChatStoredMessage>.unmodifiable(hits);
    });
  }

  /// 彻底删除本机会话记录。
  ///
  /// Cloudflare 不保存聊天内容；用户删除聊天记录时，本地 Isar 是唯一
  /// 需要清理的聊天历史真源，附件缓存目录由运行态在同一操作中删除。
  Future<void> deleteConversation(
    String ownerCidNumber,
    String conversationId,
  ) {
    return _walletIsar.writeTxn((isar) async {
      final messages = await isar.chatMessageEntitys
          .filter()
          .idGreaterThan(0, include: true)
          .findAll();
      for (final message in messages.where(
        (message) =>
            message.ownerCidNumber == ownerCidNumber &&
            message.conversationId == conversationId,
      )) {
        await isar.chatMessageEntitys.delete(message.id);
      }

      final conversations = await isar.chatConversationEntitys
          .filter()
          .idGreaterThan(0, include: true)
          .findAll();
      for (final conversation in conversations.where(
        (conversation) =>
            conversation.ownerCidNumber == ownerCidNumber &&
            conversation.conversationId == conversationId,
      )) {
        await isar.chatConversationEntitys.delete(conversation.id);
      }

      final outboundRows = await isar.chatOutboundQueueEntitys
          .filter()
          .idGreaterThan(0, include: true)
          .findAll();
      for (final row in outboundRows.where(
        (row) =>
            row.ownerCidNumber == ownerCidNumber &&
            row.conversationId == conversationId,
      )) {
        await isar.chatOutboundQueueEntitys.delete(row.id);
      }

      final pendingRows = await isar.chatPendingInboundEntitys
          .filter()
          .idGreaterThan(0, include: true)
          .findAll();
      for (final row in pendingRows.where(
        (row) =>
            row.ownerCidNumber == ownerCidNumber &&
            row.conversationId == conversationId,
      )) {
        await isar.chatPendingInboundEntitys.delete(row.id);
      }

      final outgoingMediaRows = await isar.chatOutgoingMediaEntitys
          .filter()
          .idGreaterThan(0, include: true)
          .findAll();
      for (final row in outgoingMediaRows.where(
        (row) =>
            row.ownerCidNumber == ownerCidNumber &&
            row.conversationId == conversationId,
      )) {
        await isar.chatOutgoingMediaEntitys.delete(row.id);
      }
    });
  }

  /// 注销用户：清除该 CID 在本机的全部 Chat 历史与队列。
  ///
  /// Cloudflare 端 A 的设备登记由 Worker purge 删除；本地 Isar 是 A 私信密文与
  /// 本地队列的唯一残留处，须一并清空以做到零残留。
  Future<void> clearAllForCidNumber(String cidNumber) {
    return _walletIsar.writeTxn((isar) async {
      final conversations = await isar.chatConversationEntitys
          .filter()
          .idGreaterThan(0, include: true)
          .findAll();
      final owned = conversations
          .where((c) => c.ownerCidNumber == cidNumber)
          .toList(growable: false);
      for (final c in owned) {
        await isar.chatConversationEntitys.delete(c.id);
      }

      final messages = await isar.chatMessageEntitys
          .filter()
          .idGreaterThan(0, include: true)
          .findAll();
      for (final m in messages.where(
        (m) => m.ownerCidNumber == cidNumber,
      )) {
        await isar.chatMessageEntitys.delete(m.id);
      }

      final outbound = await isar.chatOutboundQueueEntitys
          .filter()
          .idGreaterThan(0, include: true)
          .findAll();
      for (final o in outbound.where(
        (o) => o.ownerCidNumber == cidNumber,
      )) {
        await isar.chatOutboundQueueEntitys.delete(o.id);
      }

      final pending = await isar.chatPendingInboundEntitys
          .filter()
          .idGreaterThan(0, include: true)
          .findAll();
      for (final p in pending.where(
        (p) => p.ownerCidNumber == cidNumber,
      )) {
        await isar.chatPendingInboundEntitys.delete(p.id);
      }

      final outgoingMedia = await isar.chatOutgoingMediaEntitys
          .filter()
          .idGreaterThan(0, include: true)
          .findAll();
      for (final row in outgoingMedia.where(
        (row) => row.ownerCidNumber == cidNumber,
      )) {
        await isar.chatOutgoingMediaEntitys.delete(row.id);
      }
      final routes = await isar.chatRouteCacheEntitys
          .filter()
          .ownerCidNumberEqualTo(cidNumber)
          .findAll();
      for (final row in routes) {
        await isar.chatRouteCacheEntitys.delete(row.id);
      }
    });
  }

  /// 无私有数据交接的新绑定只清理不可安全续用的瞬时/派生状态。
  ///
  /// 聊天正文与会话摘要密文继续保留在 Isar，读取时由当前账户认证失败而保持不可见；
  /// 出站队列、入站乱序缓冲、媒体补发、路由和群镜像必须清理，禁止新账户自动发送或
  /// 继续处理此前 MLS 上下文产生的任务。
  Future<void> isolateInaccessibleBinding(String cidNumber) {
    return _walletIsar.writeTxn((isar) async {
      final outbound = await isar.chatOutboundQueueEntitys
          .filter()
          .ownerCidNumberEqualTo(cidNumber)
          .findAll();
      for (final row in outbound) {
        await isar.chatOutboundQueueEntitys.delete(row.id);
      }
      final pending = await isar.chatPendingInboundEntitys
          .filter()
          .ownerCidNumberEqualTo(cidNumber)
          .findAll();
      for (final row in pending) {
        await isar.chatPendingInboundEntitys.delete(row.id);
      }
      final outgoingMedia = await isar.chatOutgoingMediaEntitys
          .filter()
          .ownerCidNumberEqualTo(cidNumber)
          .findAll();
      for (final row in outgoingMedia) {
        await isar.chatOutgoingMediaEntitys.delete(row.id);
      }
      final routes = await isar.chatRouteCacheEntitys
          .filter()
          .ownerCidNumberEqualTo(cidNumber)
          .findAll();
      for (final row in routes) {
        await isar.chatRouteCacheEntitys.delete(row.id);
      }
      final groups = await isar.chatGroupEntitys
          .filter()
          .ownerCidNumberEqualTo(cidNumber)
          .findAll();
      for (final row in groups) {
        await isar.chatGroupEntitys.delete(row.id);
      }
      final members = await isar.chatGroupMemberEntitys
          .filter()
          .ownerCidNumberEqualTo(cidNumber)
          .findAll();
      for (final row in members) {
        await isar.chatGroupMemberEntitys.delete(row.id);
      }
      final commits = await isar.chatGroupPendingCommitEntitys
          .filter()
          .ownerCidNumberEqualTo(cidNumber)
          .findAll();
      for (final row in commits) {
        await isar.chatGroupPendingCommitEntitys.delete(row.id);
      }
    });
  }

  Future<void> saveOutgoingEnvelope({
    required String ownerCidNumber,
    required String currentAccountId,
    required ChatEnvelope envelope,
    required List<int> envelopeBytes,
    required String recipientCidNumber,
    required ChatMessageKind messageKind,
    required ChatMessageDeliveryState deliveryState,
    String? plaintext,
  }) async {
    final binding = await _crypto.resolveCipherBinding(
      ownerCidNumber: ownerCidNumber,
      currentAccountId: currentAccountId,
    );
    // 加解密在事务外完成，避免密码学运算占住 Isar 写事务。
    final sealed = await _sealMessage(
      ownerCidNumber: ownerCidNumber,
      currentAccountId: currentAccountId,
      envelopeId: envelope.envelopeId,
      plaintext: plaintext,
    );
    final summaryCipher = await _sealSummary(
      ownerCidNumber: ownerCidNumber,
      currentAccountId: currentAccountId,
      conversationId: envelope.conversationId,
      plaintext: plaintext,
    );
    return _walletIsar.writeTxn((isar) async {
      await _putConversationInTxn(
        isar: isar,
        ownerCidNumber: ownerCidNumber,
        bindingRevision: binding.bindingRevision,
        accountId: binding.accountId,
        conversationId: envelope.conversationId,
        peerCidNumber: envelope.recipientCidNumber,
        title: envelope.recipientCidNumber,
        lastMessageCipher: summaryCipher,
        lastUpdatedAtMillis: envelope.createdAtMillis.toInt(),
        unreadDelta: 0,
        deliveryState: deliveryState,
      );
      await isar.chatMessageEntitys.putByOwnerCidNumberEnvelopeId(
        _messageEntity(
          ownerCidNumber: ownerCidNumber,
          bindingRevision: binding.bindingRevision,
          accountId: binding.accountId,
          envelope: envelope,
          envelopeBytes: envelopeBytes,
          direction: 'outgoing',
          messageKind: messageKind,
          deliveryState: deliveryState,
          plaintextCipher: sealed.cipher,
          searchTokens: sealed.tokens,
        ),
      );
      await isar.chatOutboundQueueEntitys.putByOwnerCidNumberEnvelopeId(
        ChatOutboundQueueEntity()
          ..ownerCidNumber = ownerCidNumber
          ..envelopeId = envelope.envelopeId
          ..conversationId = envelope.conversationId
          ..recipientCidNumber = recipientCidNumber
          ..envelopeBytesHex = _bytesToHex(envelopeBytes)
          ..deliveryState = deliveryState.name
          ..attemptCount = 0
          ..lastError = null
          ..updatedAtMillis = DateTime.now().millisecondsSinceEpoch,
      );
    });
  }

  Future<void> queueOutgoingEnvelope({
    required String ownerCidNumber,
    required ChatEnvelope envelope,
    required List<int> envelopeBytes,
    required String recipientCidNumber,
    required ChatMessageDeliveryState deliveryState,
  }) {
    return _walletIsar.writeTxn((isar) async {
      await isar.chatOutboundQueueEntitys.putByOwnerCidNumberEnvelopeId(
        ChatOutboundQueueEntity()
          ..ownerCidNumber = ownerCidNumber
          ..envelopeId = envelope.envelopeId
          ..conversationId = envelope.conversationId
          ..recipientCidNumber = recipientCidNumber
          ..envelopeBytesHex = _bytesToHex(envelopeBytes)
          ..deliveryState = deliveryState.name
          ..attemptCount = 0
          ..lastError = null
          ..updatedAtMillis = DateTime.now().millisecondsSinceEpoch,
      );
    });
  }

  Future<void> saveIncomingEnvelope({
    required String ownerCidNumber,
    required String currentAccountId,
    required ChatEnvelope envelope,
    required List<int> envelopeBytes,
    required ChatMessageKind messageKind,
    required String plaintext,
  }) async {
    final binding = await _crypto.resolveCipherBinding(
      ownerCidNumber: ownerCidNumber,
      currentAccountId: currentAccountId,
    );
    final sealed = await _sealMessage(
      ownerCidNumber: ownerCidNumber,
      currentAccountId: currentAccountId,
      envelopeId: envelope.envelopeId,
      plaintext: plaintext,
    );
    final summaryCipher = await _sealSummary(
      ownerCidNumber: ownerCidNumber,
      currentAccountId: currentAccountId,
      conversationId: envelope.conversationId,
      plaintext: plaintext,
    );
    return _walletIsar.writeTxn((isar) async {
      await _putConversationInTxn(
        isar: isar,
        ownerCidNumber: ownerCidNumber,
        bindingRevision: binding.bindingRevision,
        accountId: binding.accountId,
        conversationId: envelope.conversationId,
        peerCidNumber: envelope.senderCidNumber,
        title: envelope.senderCidNumber,
        lastMessageCipher: summaryCipher,
        lastUpdatedAtMillis: envelope.createdAtMillis.toInt(),
        unreadDelta: 1,
        deliveryState: ChatMessageDeliveryState.receivedByDevice,
      );
      await isar.chatMessageEntitys.putByOwnerCidNumberEnvelopeId(
        _messageEntity(
          ownerCidNumber: ownerCidNumber,
          bindingRevision: binding.bindingRevision,
          accountId: binding.accountId,
          envelope: envelope,
          envelopeBytes: envelopeBytes,
          direction: 'incoming',
          messageKind: messageKind,
          deliveryState: ChatMessageDeliveryState.receivedByDevice,
          plaintextCipher: sealed.cipher,
          searchTokens: sealed.tokens,
        ),
      );
    });
  }

  Future<void> markOutgoingDelivery({
    required String ownerCidNumber,
    required String envelopeId,
    required ChatMessageDeliveryState state,
    String? errorMessage,
  }) {
    return _walletIsar.writeTxn((isar) async {
      final queue = await isar.chatOutboundQueueEntitys
          .getByOwnerCidNumberEnvelopeId(ownerCidNumber, envelopeId);
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
          await isar.chatOutboundQueueEntitys
              .putByOwnerCidNumberEnvelopeId(queue);
        }
      }
      final message = await isar.chatMessageEntitys
          .getByOwnerCidNumberEnvelopeId(ownerCidNumber, envelopeId);
      if (message != null) {
        message.deliveryState = state.name;
        await isar.chatMessageEntitys.putByOwnerCidNumberEnvelopeId(message);
        final conversation = await isar.chatConversationEntitys
            .getByOwnerCidNumberConversationId(
          ownerCidNumber,
          message.conversationId,
        );
        if (conversation != null) {
          conversation.lastDeliveryState = state.name;
          await isar.chatConversationEntitys
              .putByOwnerCidNumberConversationId(conversation);
        }
      }
    });
  }

  Future<void> savePendingInbound({
    required String ownerCidNumber,
    required ChatEnvelope envelope,
    required List<int> envelopeBytes,
    required String reason,
  }) {
    return _walletIsar.writeTxn((isar) async {
      await isar.chatPendingInboundEntitys.putByOwnerCidNumberEnvelopeId(
        ChatPendingInboundEntity()
          ..ownerCidNumber = ownerCidNumber
          ..envelopeId = envelope.envelopeId
          ..conversationId = envelope.conversationId
          ..envelopeBytesHex = _bytesToHex(envelopeBytes)
          ..reason = reason
          ..createdAtMillis = DateTime.now().millisecondsSinceEpoch,
      );
    });
  }

  Future<List<ChatEnvelope>> takePendingInbound(
    String ownerCidNumber,
    String conversationId,
  ) {
    return _walletIsar.writeTxn((isar) async {
      final rows = await isar.chatPendingInboundEntitys
          .filter()
          .idGreaterThan(0, include: true)
          .findAll();
      final matched = rows
          .where((row) =>
              row.ownerCidNumber == ownerCidNumber &&
              row.conversationId == conversationId)
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

  Future<int> pendingInboundCount(String ownerCidNumber) {
    return _walletIsar.read((isar) async {
      final rows = await isar.chatPendingInboundEntitys
          .filter()
          .idGreaterThan(0, include: true)
          .findAll();
      return rows.where((row) => row.ownerCidNumber == ownerCidNumber).length;
    });
  }

  Future<int> outboundQueueCount(String ownerCidNumber) {
    return _walletIsar.read((isar) async {
      final rows = await isar.chatOutboundQueueEntitys
          .filter()
          .idGreaterThan(0, include: true)
          .findAll();
      return rows.where((row) => row.ownerCidNumber == ownerCidNumber).length;
    });
  }

  /// 读取发送设备上的待重试密文；Cloudflare 不提供远程补拉。
  Future<List<ChatQueuedEnvelope>> readQueuedEnvelopes({
    required String ownerCidNumber,
    String? recipientCidNumber,
  }) {
    return _walletIsar.read((isar) async {
      final rows = await isar.chatOutboundQueueEntitys
          .filter()
          .idGreaterThan(0, include: true)
          .findAll();
      final owned =
          rows.where((row) => row.ownerCidNumber == ownerCidNumber).toList();
      final matched = recipientCidNumber == null
          ? owned
          : owned
              .where((row) => row.recipientCidNumber == recipientCidNumber)
              .toList(growable: false);
      matched.sort((a, b) => a.updatedAtMillis.compareTo(b.updatedAtMillis));
      return matched
          .map(
            (row) => ChatQueuedEnvelope(
              envelopeId: row.envelopeId,
              recipientCidNumber: row.recipientCidNumber,
              envelopeBytes: _hexToBytes(row.envelopeBytesHex),
            ),
          )
          .toList(growable: false);
    });
  }

  /// 登记一条待设备投递的媒体(字节未送达对方设备,留待上线补发)。
  Future<void> recordOutgoingMedia({
    required String ownerCidNumber,
    required String attachmentId,
    required String recipientCidNumber,
    required String conversationId,
    required String fileName,
    required String contentType,
    required int byteSize,
  }) {
    return _walletIsar.writeTxn((isar) async {
      await isar.chatOutgoingMediaEntitys.putByOwnerCidNumberPendingKey(
        ChatOutgoingMediaEntity()
          ..ownerCidNumber = ownerCidNumber
          ..pendingKey = '$attachmentId|$recipientCidNumber'
          ..attachmentId = attachmentId
          ..recipientCidNumber = recipientCidNumber
          ..conversationId = conversationId
          ..fileName = fileName
          ..contentType = contentType
          ..byteSize = byteSize
          ..createdAtMillis = DateTime.now().millisecondsSinceEpoch,
      );
    });
  }

  /// 字节已送达某成员设备(收到 WebRTC ack)后删除该 (媒体, 成员) 待投递行。
  Future<void> deleteOutgoingMedia(
    String ownerCidNumber,
    String attachmentId,
    String recipientCidNumber,
  ) {
    return _walletIsar.writeTxn((isar) async {
      await isar.chatOutgoingMediaEntitys.deleteByOwnerCidNumberPendingKey(
        ownerCidNumber,
        '$attachmentId|$recipientCidNumber',
      );
    });
  }

  /// 读取待设备投递的媒体(可按对端过滤),供上线补发。
  Future<List<ChatPendingMedia>> readPendingOutgoingMedia({
    required String ownerCidNumber,
    String? recipientCidNumber,
  }) {
    return _walletIsar.read((isar) async {
      final rows = await isar.chatOutgoingMediaEntitys
          .filter()
          .idGreaterThan(0, include: true)
          .findAll();
      final owned =
          rows.where((row) => row.ownerCidNumber == ownerCidNumber).toList();
      final matched = recipientCidNumber == null
          ? owned
          : owned
              .where((row) => row.recipientCidNumber == recipientCidNumber)
              .toList(growable: false);
      matched.sort((a, b) => a.createdAtMillis.compareTo(b.createdAtMillis));
      return matched
          .map(
            (row) => ChatPendingMedia(
              attachmentId: row.attachmentId,
              recipientCidNumber: row.recipientCidNumber,
              conversationId: row.conversationId,
              fileName: row.fileName,
              contentType: row.contentType,
              byteSize: row.byteSize,
            ),
          )
          .toList(growable: false);
    });
  }

  Future<int> outgoingMediaCount(String ownerCidNumber) {
    return _walletIsar.read((isar) async {
      final rows = await isar.chatOutgoingMediaEntitys
          .filter()
          .idGreaterThan(0, include: true)
          .findAll();
      return rows.where((row) => row.ownerCidNumber == ownerCidNumber).length;
    });
  }

  // ==== 私密小群 ====

  /// 建群/入群时落群会话壳 + 群会话记录(conversationKind=group,title=群名)。
  Future<void> upsertGroupShell({
    required String ownerCidNumber,
    required String currentAccountId,
    required String groupId,
    required String groupName,
    required String creatorCidNumber,
    required int epoch,
  }) async {
    final binding = await _crypto.resolveCipherBinding(
      ownerCidNumber: ownerCidNumber,
      currentAccountId: currentAccountId,
    );
    return _walletIsar.writeTxn((isar) async {
      final now = DateTime.now().millisecondsSinceEpoch;
      final existing = await isar.chatGroupEntitys
          .getByOwnerCidNumberGroupId(ownerCidNumber, groupId);
      final entity = existing ?? ChatGroupEntity();
      entity
        ..ownerCidNumber = ownerCidNumber
        ..groupId = groupId
        ..groupName = groupName
        ..creatorCidNumber = creatorCidNumber
        ..epoch = epoch
        ..memberCount = existing?.memberCount ?? 1
        ..leftLocally = existing?.leftLocally ?? false
        ..createdAtMillis = existing?.createdAtMillis ?? now
        ..updatedAtMillis = now;
      await isar.chatGroupEntitys.putByOwnerCidNumberGroupId(entity);

      final conversation = await isar.chatConversationEntitys
          .getByOwnerCidNumberConversationId(ownerCidNumber, groupId);
      final shell = conversation ?? ChatConversationEntity();
      shell
        ..ownerCidNumber = ownerCidNumber
        ..bindingRevision = binding.bindingRevision
        ..accountId = binding.accountId
        ..conversationId = groupId
        ..peerCidNumber = creatorCidNumber
        ..title = groupName
        ..conversationKind = 'group'
        ..lastMessageCipher = conversation?.lastMessageCipher ?? ''
        ..lastUpdatedAtMillis = conversation?.lastUpdatedAtMillis ?? now
        ..unreadCount = conversation?.unreadCount ?? 0
        ..lastDeliveryState = conversation?.lastDeliveryState ??
            ChatMessageDeliveryState.queued.name;
      await isar.chatConversationEntitys
          .putByOwnerCidNumberConversationId(shell);
    });
  }

  /// 按 MLS 名册（CID→角色）覆盖群成员镜像，并更新 epoch/人数。
  Future<void> reconcileGroupRoster({
    required String ownerCidNumber,
    required String groupId,
    required Map<String, GroupMemberRole> members,
    required int epoch,
  }) {
    return _walletIsar.writeTxn((isar) async {
      final now = DateTime.now().millisecondsSinceEpoch;
      final existing = await isar.chatGroupMemberEntitys
          .filter()
          .ownerCidNumberEqualTo(ownerCidNumber)
          .and()
          .groupIdEqualTo(groupId)
          .findAll();
      final joinedAt = <String, int>{
        for (final row in existing) row.memberCidNumber: row.joinedAtMillis,
      };
      for (final row in existing) {
        await isar.chatGroupMemberEntitys.delete(row.id);
      }
      for (final entry in members.entries) {
        await isar.chatGroupMemberEntitys.putByOwnerCidNumberMemberKey(
          ChatGroupMemberEntity()
            ..ownerCidNumber = ownerCidNumber
            ..memberKey = '$groupId|${entry.key}'
            ..groupId = groupId
            ..memberCidNumber = entry.key
            ..role = entry.value.wireName
            ..joinedAtMillis = joinedAt[entry.key] ?? now,
        );
      }
      final group = await isar.chatGroupEntitys
          .getByOwnerCidNumberGroupId(ownerCidNumber, groupId);
      if (group != null) {
        group
          ..epoch = epoch
          ..memberCount = members.length
          ..updatedAtMillis = now;
        await isar.chatGroupEntitys.putByOwnerCidNumberGroupId(group);
      }
    });
  }

  Future<ChatGroup?> readGroup(String ownerCidNumber, String groupId) {
    return _walletIsar.read((isar) async {
      final group = await isar.chatGroupEntitys
          .getByOwnerCidNumberGroupId(ownerCidNumber, groupId);
      if (group == null) return null;
      final members = await isar.chatGroupMemberEntitys
          .filter()
          .ownerCidNumberEqualTo(ownerCidNumber)
          .and()
          .groupIdEqualTo(groupId)
          .findAll();
      return _groupFromEntities(group, members);
    });
  }

  Future<List<ChatGroup>> readGroups(String ownerCidNumber) {
    return _walletIsar.read((isar) async {
      final groups = await isar.chatGroupEntitys
          .filter()
          .idGreaterThan(0, include: true)
          .findAll();
      final filtered = groups
          .where((row) => row.ownerCidNumber == ownerCidNumber)
          .toList(growable: false);
      final result = <ChatGroup>[];
      for (final group in filtered) {
        final members = await isar.chatGroupMemberEntitys
            .filter()
            .ownerCidNumberEqualTo(ownerCidNumber)
            .and()
            .groupIdEqualTo(group.groupId)
            .findAll();
        result.add(_groupFromEntities(group, members));
      }
      return result;
    });
  }

  /// 退群/被移除:本机标记已退,停止参与。
  Future<void> markGroupLeft(String ownerCidNumber, String groupId) {
    return _walletIsar.writeTxn((isar) async {
      final group = await isar.chatGroupEntitys
          .getByOwnerCidNumberGroupId(ownerCidNumber, groupId);
      if (group != null) {
        group
          ..leftLocally = true
          ..updatedAtMillis = DateTime.now().millisecondsSinceEpoch;
        await isar.chatGroupEntitys.putByOwnerCidNumberGroupId(group);
      }
    });
  }

  /// 改群名(群记录 + 群会话 title 同步)。空名忽略。
  Future<void> renameGroup(
    String ownerCidNumber,
    String groupId,
    String name,
  ) {
    final trimmed = name.trim();
    if (trimmed.isEmpty) {
      return Future<void>.value();
    }
    return _walletIsar.writeTxn((isar) async {
      final now = DateTime.now().millisecondsSinceEpoch;
      final group = await isar.chatGroupEntitys
          .getByOwnerCidNumberGroupId(ownerCidNumber, groupId);
      if (group != null) {
        group
          ..groupName = trimmed
          ..updatedAtMillis = now;
        await isar.chatGroupEntitys.putByOwnerCidNumberGroupId(group);
      }
      final conversation = await isar.chatConversationEntitys
          .getByOwnerCidNumberConversationId(ownerCidNumber, groupId);
      if (conversation != null) {
        conversation.title = trimmed;
        await isar.chatConversationEntitys
            .putByOwnerCidNumberConversationId(conversation);
      }
    });
  }

  /// 缓冲一条乱序群 Commit(键 groupId+messageEpoch)。
  Future<void> bufferGroupCommit({
    required String ownerCidNumber,
    required String groupId,
    required int messageEpoch,
    required ChatEnvelope envelope,
    required List<int> envelopeBytes,
  }) {
    return _walletIsar.writeTxn((isar) async {
      await isar.chatGroupPendingCommitEntitys.putByOwnerCidNumberEnvelopeId(
        ChatGroupPendingCommitEntity()
          ..ownerCidNumber = ownerCidNumber
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
    String ownerCidNumber,
    String groupId,
    int messageEpoch,
  ) {
    return _walletIsar.writeTxn((isar) async {
      final rows = await isar.chatGroupPendingCommitEntitys
          .filter()
          .ownerCidNumberEqualTo(ownerCidNumber)
          .and()
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
  ///
  /// [recipientCidByCidNumber] 固定按成员 CID 建立队列路由；账户不进入群消息身份。
  Future<void> saveOutgoingGroupMessage({
    required String ownerCidNumber,
    required String currentAccountId,
    required String groupId,
    required String senderCidNumber,
    required String senderDeviceId,
    required String logicalEnvelopeId,
    required ChatMessageKind messageKind,
    required String payload,
    required int createdAtMillis,
    required List<ChatEnvelope> envelopes,
    required Map<String, String> recipientCidByCidNumber,
  }) async {
    final binding = await _crypto.resolveCipherBinding(
      ownerCidNumber: ownerCidNumber,
      currentAccountId: currentAccountId,
    );
    final sealed = await _sealMessage(
      ownerCidNumber: ownerCidNumber,
      currentAccountId: currentAccountId,
      envelopeId: logicalEnvelopeId,
      plaintext: payload,
    );
    final summaryCipher = await _sealSummary(
      ownerCidNumber: ownerCidNumber,
      currentAccountId: currentAccountId,
      conversationId: groupId,
      plaintext: payload,
    );
    return _walletIsar.writeTxn((isar) async {
      await _touchGroupConversationInTxn(
        isar: isar,
        ownerCidNumber: ownerCidNumber,
        bindingRevision: binding.bindingRevision,
        accountId: binding.accountId,
        groupId: groupId,
        lastMessageCipher: summaryCipher,
        lastUpdatedAtMillis: createdAtMillis,
        unreadDelta: 0,
        deliveryState: ChatMessageDeliveryState.queued,
      );
      await isar.chatMessageEntitys.putByOwnerCidNumberEnvelopeId(
        ChatMessageEntity()
          ..ownerCidNumber = ownerCidNumber
          ..bindingRevision = binding.bindingRevision
          ..accountId = binding.accountId
          ..envelopeId = logicalEnvelopeId
          ..conversationId = groupId
          ..direction = 'outgoing'
          ..senderCidNumber = senderCidNumber
          ..recipientCidNumber = groupId
          ..senderDeviceId = senderDeviceId
          ..messageKind = messageKind.name
          ..mlsMessageKind =
              MlsWireMessageKind.MLS_WIRE_MESSAGE_KIND_APPLICATION.name
          ..deliveryState = ChatMessageDeliveryState.queued.name
          ..plaintextCipher = sealed.cipher
          ..searchTokens = sealed.tokens
          ..envelopeBytesHex = ''
          ..createdAtMillis = createdAtMillis,
      );
      for (final envelope in envelopes) {
        final recipientCidNumber =
            recipientCidByCidNumber[envelope.recipientCidNumber];
        if (recipientCidNumber == null || recipientCidNumber.isEmpty) {
          throw StateError('群出站队列缺少收件人 CID 映射: ${envelope.recipientCidNumber}');
        }
        await isar.chatOutboundQueueEntitys.putByOwnerCidNumberEnvelopeId(
          ChatOutboundQueueEntity()
            ..ownerCidNumber = ownerCidNumber
            ..envelopeId = envelope.envelopeId
            ..conversationId = groupId
            ..recipientCidNumber = recipientCidNumber
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
    required String ownerCidNumber,
    required String currentAccountId,
    required ChatEnvelope envelope,
    required List<int> envelopeBytes,
    required ChatMessageKind messageKind,
    required String plaintext,
  }) async {
    final binding = await _crypto.resolveCipherBinding(
      ownerCidNumber: ownerCidNumber,
      currentAccountId: currentAccountId,
    );
    final sealed = await _sealMessage(
      ownerCidNumber: ownerCidNumber,
      currentAccountId: currentAccountId,
      envelopeId: envelope.envelopeId,
      plaintext: plaintext,
    );
    final summaryCipher = await _sealSummary(
      ownerCidNumber: ownerCidNumber,
      currentAccountId: currentAccountId,
      conversationId: envelope.conversationId,
      plaintext: plaintext,
    );
    return _walletIsar.writeTxn((isar) async {
      await _touchGroupConversationInTxn(
        isar: isar,
        ownerCidNumber: ownerCidNumber,
        bindingRevision: binding.bindingRevision,
        accountId: binding.accountId,
        groupId: envelope.conversationId,
        lastMessageCipher: summaryCipher,
        lastUpdatedAtMillis: envelope.createdAtMillis.toInt(),
        unreadDelta: 1,
        deliveryState: ChatMessageDeliveryState.receivedByDevice,
      );
      await isar.chatMessageEntitys.putByOwnerCidNumberEnvelopeId(
        _messageEntity(
          ownerCidNumber: ownerCidNumber,
          bindingRevision: binding.bindingRevision,
          accountId: binding.accountId,
          envelope: envelope,
          envelopeBytes: envelopeBytes,
          direction: 'incoming',
          messageKind: messageKind,
          deliveryState: ChatMessageDeliveryState.receivedByDevice,
          plaintextCipher: sealed.cipher,
          searchTokens: sealed.tokens,
        ),
      );
    });
  }

  /// 更新群会话的 lastMessage/未读/投递态,但保留群名 title 与 conversationKind。
  Future<void> _touchGroupConversationInTxn({
    required Isar isar,
    required String ownerCidNumber,
    required int bindingRevision,
    required String accountId,
    required String groupId,
    required String lastMessageCipher,
    required int lastUpdatedAtMillis,
    required int unreadDelta,
    required ChatMessageDeliveryState deliveryState,
  }) async {
    final existing = await isar.chatConversationEntitys
        .getByOwnerCidNumberConversationId(ownerCidNumber, groupId);
    final group = await isar.chatGroupEntitys
        .getByOwnerCidNumberGroupId(ownerCidNumber, groupId);
    final entity = existing ?? ChatConversationEntity();
    entity
      ..ownerCidNumber = ownerCidNumber
      ..bindingRevision = bindingRevision
      ..accountId = accountId
      ..conversationId = groupId
      ..peerCidNumber =
          existing?.peerCidNumber ?? (group?.creatorCidNumber ?? '')
      ..title = group?.groupName ?? existing?.title ?? groupId
      ..conversationKind = 'group'
      ..lastMessageCipher = lastMessageCipher
      ..lastUpdatedAtMillis = lastUpdatedAtMillis
      ..unreadCount = (existing?.unreadCount ?? 0) + unreadDelta
      ..lastDeliveryState = deliveryState.name;
    await isar.chatConversationEntitys
        .putByOwnerCidNumberConversationId(entity);
  }

  ChatGroup _groupFromEntities(
    ChatGroupEntity group,
    List<ChatGroupMemberEntity> members,
  ) {
    return ChatGroup(
      groupId: group.groupId,
      name: group.groupName,
      creatorCidNumber: group.creatorCidNumber,
      epoch: group.epoch,
      leftLocally: group.leftLocally,
      roster: members
          .map((row) => GroupMember(
                cidNumber: row.memberCidNumber,
                role: GroupMemberRole.fromName(row.role),
              ))
          .toList(growable: false),
    );
  }

  Future<void> _putConversationInTxn({
    required Isar isar,
    required String ownerCidNumber,
    required int bindingRevision,
    required String accountId,
    required String conversationId,
    required String peerCidNumber,
    required String title,
    required String lastMessageCipher,
    required int lastUpdatedAtMillis,
    required int unreadDelta,
    required ChatMessageDeliveryState deliveryState,
  }) async {
    final existing =
        await isar.chatConversationEntitys.getByOwnerCidNumberConversationId(
      ownerCidNumber,
      conversationId,
    );
    final entity = existing ?? ChatConversationEntity();
    entity
      ..ownerCidNumber = ownerCidNumber
      ..bindingRevision = bindingRevision
      ..accountId = accountId
      ..conversationId = conversationId
      ..peerCidNumber = peerCidNumber
      ..title = title
      ..lastMessageCipher = lastMessageCipher
      ..lastUpdatedAtMillis = lastUpdatedAtMillis
      ..unreadCount = (existing?.unreadCount ?? 0) + unreadDelta
      ..lastDeliveryState = deliveryState.name;
    await isar.chatConversationEntitys
        .putByOwnerCidNumberConversationId(entity);
  }
}

/// [lastMessage] 由 `ChatStore` 解密后传入——本函数不接触密钥。
ChatConversationPreview _conversationPreviewFromEntity(
    ChatConversationEntity row, String lastMessage) {
  return ChatConversationPreview(
    conversationId: row.conversationId,
    title: row.title,
    peerCidNumber: row.peerCidNumber,
    lastMessage: lastMessage,
    lastUpdatedAt: DateTime.fromMillisecondsSinceEpoch(row.lastUpdatedAtMillis),
    unreadCount: row.unreadCount,
    deliveryState: _deliveryStateFromName(row.lastDeliveryState),
    conversationKind: row.conversationKind ?? 'dm',
  );
}

/// [plaintext] 由 `ChatStore` 解密后传入——本函数不接触密钥。
ChatStoredMessage _messageFromEntity(ChatMessageEntity row, String? plaintext) {
  return ChatStoredMessage(
    envelopeId: row.envelopeId,
    conversationId: row.conversationId,
    direction: row.direction,
    senderCidNumber: row.senderCidNumber,
    recipientCidNumber: row.recipientCidNumber,
    messageKind: _messageKindFromName(row.messageKind),
    deliveryState: _deliveryStateFromName(row.deliveryState),
    createdAtMillis: row.createdAtMillis,
    plaintext: plaintext,
  );
}

ChatRoute _routeFromEntity(ChatRouteCacheEntity row) {
  return ChatRoute(
    peerCidNumber: row.peerCidNumber,
    routeDisplayName: row.routeDisplayName,
    deviceId: row.deviceId,
    devicePublicKey: row.devicePublicKey,
    safetyNumber: row.safetyNumber,
    nearbyPeerHint: row.nearbyPeerHint,
    note: row.note,
    createdAtMillis: row.createdAtMillis,
    updatedAtMillis: row.updatedAtMillis,
  );
}

ChatMessageEntity _messageEntity({
  required String ownerCidNumber,
  required int bindingRevision,
  required String accountId,
  required ChatEnvelope envelope,
  required List<int> envelopeBytes,
  required String direction,
  required ChatMessageKind messageKind,
  required ChatMessageDeliveryState deliveryState,
  String? plaintextCipher,
  List<String> searchTokens = const <String>[],
}) {
  return ChatMessageEntity()
    ..ownerCidNumber = ownerCidNumber
    ..bindingRevision = bindingRevision
    ..accountId = accountId
    ..envelopeId = envelope.envelopeId
    ..conversationId = envelope.conversationId
    ..direction = direction
    ..senderCidNumber = envelope.senderCidNumber
    ..recipientCidNumber = envelope.recipientCidNumber
    ..senderDeviceId = envelope.senderDeviceId
    ..messageKind = messageKind.name
    ..mlsMessageKind = envelope.mlsMessageKind.name
    ..deliveryState = deliveryState.name
    ..plaintextCipher = plaintextCipher
    ..searchTokens = searchTokens
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

/// 一条消息落盘所需的密文与搜索索引。
class _SealedMessage {
  const _SealedMessage({required this.cipher, required this.tokens});

  /// 正文密文；正文为空时为 null。
  final String? cipher;

  /// HMAC 分词索引（去重后的 bigram token）。
  final List<String> tokens;
}
