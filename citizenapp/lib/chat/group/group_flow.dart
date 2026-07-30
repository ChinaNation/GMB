// 私密小群收发编排。串起 MlsGroupCrypto(密码学)、GroupFanout(扇出)、
// GroupEpochOrdering(有序)、ChatStore(落库)与 deliverer(投递)。
// 本层不实现密码学;核心可注入 fake 单测。
// 详见 memory/05-modules/citizenapp/chat/CHAT_GROUP_TECHNICAL.md。

import 'dart:convert';
import 'dart:math';

import '../chat_flow.dart';
import '../chat_media_limits.dart';
import '../chat_models.dart';
import '../chat_payload.dart';
import '../crypto/mls_boundary.dart';
import '../crypto/mls_group_boundary.dart';
import '../proto/chat_envelope.pb.dart';
import '../storage/chat_store.dart';
import '../transport/chat_transport.dart';
import 'group_control.dart';
import 'group_epoch.dart';
import 'group_fanout.dart';
import 'group_membership.dart';
import 'group_model.dart';

/// 群 ID 形如 `grp:<creator CID>:<nonce>`。
String creatorCidNumberFromGroupId(String groupId) {
  final parts = groupId.split(':');
  return parts.length >= 2 ? parts[1] : '';
}

/// 生成群 ID（创建者 CID + 随机 nonce）。
String newGroupId(String creatorCidNumber) {
  return 'grp:$creatorCidNumber:${_nonce()}';
}

/// 登记/清除某成员的待投递群媒体(离线补发按成员;键 attachmentId+成员 CID)。
typedef GroupMemberMediaRecorder = Future<void> Function(
  String attachmentId,
  String memberCidNumber,
);

class ChatGroupFlow {
  const ChatGroupFlow({
    required MlsGroupCrypto crypto,
    required ChatStore store,
    required ChatEnvelopeDeliverer deliverer,
    required String ownerCidNumber,
    required String cidNumber,
    required String currentAccountId,
    required String localDeviceId,
    this.defaultTtlMillis = 30 * 24 * 60 * 60 * 1000,
  })  : _crypto = crypto,
        _store = store,
        _deliverer = deliverer,
        _ownerCidNumber = ownerCidNumber,
        _cidNumber = cidNumber,
        _currentAccountId = currentAccountId,
        _localDeviceId = localDeviceId;

  final MlsGroupCrypto _crypto;
  final ChatStore _store;
  final ChatEnvelopeDeliverer _deliverer;

  final String _ownerCidNumber;

  /// 本机聊天 CID 与设备 ID（入站处理判定自身、代提交退群移除的 fanout 发送者）。
  final String _cidNumber;
  final String _currentAccountId;
  final String _localDeviceId;
  final int defaultTtlMillis;

  /// 建群:创建者为唯一成员(admin),可选带初始邀请。
  Future<ChatGroup> createGroup({
    required String groupId,
    required String name,
    required String cidNumber,
    required String localDeviceId,
    List<MlsKeyPackage> invitees = const [],
  }) async {
    GroupMembership.ensureCanCreate(inviteeCount: invitees.length);
    final created = await _crypto.createGroup(groupId);
    await _store.upsertGroupShell(
      ownerCidNumber: _ownerCidNumber,
      groupId: groupId,
      groupName: name,
      creatorCidNumber: cidNumber,
      epoch: created.epoch,
    );
    await _store.reconcileGroupRoster(
      ownerCidNumber: _ownerCidNumber,
      groupId: groupId,
      members: {cidNumber: GroupMemberRole.admin},
      epoch: created.epoch,
    );
    if (invitees.isNotEmpty) {
      await _addMembersInternal(
        groupId: groupId,
        actorCidNumber: cidNumber,
        actorDeviceId: localDeviceId,
        creatorCidNumber: cidNumber,
        existingCidNumbers: [cidNumber],
        invitees: invitees,
      );
    }
    final group = await _store.readGroup(_ownerCidNumber, groupId);
    return group!;
  }

  /// 加人(仅 admin)。
  Future<void> addMembers({
    required String groupId,
    required String actorCidNumber,
    required String actorDeviceId,
    required List<MlsKeyPackage> invitees,
  }) async {
    final group = await _requireGroup(groupId);
    GroupMembership.ensureAdmin(
        adminSet: group.adminSet, actorCidNumber: actorCidNumber);
    GroupMembership.ensureCanAdd(
      currentCount: group.roster.length,
      addingCount: invitees.length,
    );
    await _addMembersInternal(
      groupId: groupId,
      actorCidNumber: actorCidNumber,
      actorDeviceId: actorDeviceId,
      creatorCidNumber: group.creatorCidNumber,
      existingCidNumbers: group.memberCidNumbers,
      invitees: invitees,
    );
  }

  Future<void> _addMembersInternal({
    required String groupId,
    required String actorCidNumber,
    required String actorDeviceId,
    required String creatorCidNumber,
    required List<String> existingCidNumbers,
    required List<MlsKeyPackage> invitees,
  }) async {
    final bundle = await _crypto.addMembers(groupId, invitees);
    final nowMillis = DateTime.now().millisecondsSinceEpoch;

    // Welcome → 全部新人;Commit → 现有成员(减自己)。
    final inviteeCidNumbers = cidNumbersFromMemberIdentities(
      invitees.map((keyPackage) => keyPackage.cidNumber),
      excludeCidNumber: actorCidNumber,
    );
    final welcome = bundle.welcome;
    if (welcome != null && inviteeCidNumbers.isNotEmpty) {
      await _fanoutHandshake(
        wire: welcome,
        recipients: inviteeCidNumbers,
        senderCidNumber: actorCidNumber,
        senderDeviceId: actorDeviceId,
        groupId: groupId,
        nowMillis: nowMillis,
        tag: 'welcome',
      );
    }
    final commitRecipients = existingCidNumbers
        .where((cidNumber) => cidNumber != actorCidNumber)
        .toList();
    if (commitRecipients.isNotEmpty) {
      await _fanoutHandshake(
        wire: bundle.commit,
        recipients: commitRecipients,
        senderCidNumber: actorCidNumber,
        senderDeviceId: actorDeviceId,
        groupId: groupId,
        nowMillis: nowMillis,
        tag: 'commit',
      );
    }
    await _reconcileFromChain(groupId, creatorCidNumber);
  }

  /// 删人（仅 admin，按 CID）。
  Future<void> removeMembers({
    required String groupId,
    required String actorCidNumber,
    required String actorDeviceId,
    required List<String> targetCidNumbers,
  }) async {
    final group = await _requireGroup(groupId);
    GroupMembership.ensureAdmin(
        adminSet: group.adminSet, actorCidNumber: actorCidNumber);
    final bundle = await _crypto.removeMembers(groupId, targetCidNumbers);
    final nowMillis = DateTime.now().millisecondsSinceEpoch;

    // Commit → 剩余成员 + 被删者(镜像此刻仍含被删者),都减自己。
    final recipients = group.memberCidNumbers
        .where((cidNumber) => cidNumber != actorCidNumber)
        .toList();
    if (recipients.isNotEmpty) {
      await _fanoutHandshake(
        wire: bundle.commit,
        recipients: recipients,
        senderCidNumber: actorCidNumber,
        senderDeviceId: actorDeviceId,
        groupId: groupId,
        nowMillis: nowMillis,
        tag: 'commit',
      );
    }
    await _reconcileFromChain(groupId, group.creatorCidNumber);
  }

  /// 退群:先发退群请求(群 admin 收到后自动 removeMembers 重钥,保证后向保密),
  /// 再本机即刻标记已退、停止参与。发送失败不阻断本机退出。
  Future<void> leaveGroup(String groupId) async {
    final group = await _store.readGroup(_ownerCidNumber, groupId);
    if (group != null && !group.leftLocally) {
      try {
        await sendGroupControl(groupId, const GroupControl.leaveRequest());
      } catch (_) {
        // 控制消息发送失败(离线等)不阻断本机退出;后向保密待 admin 后续收敛。
      }
    }
    await _store.markGroupLeft(_ownerCidNumber, groupId);
  }

  /// 改群名(仅 admin):本机改 + 广播 rename 让全员同步(补 Welcome 不带名的缺口)。
  Future<void> renameGroup(String groupId, String name) async {
    final group = await _requireGroup(groupId);
    GroupMembership.ensureAdmin(
        adminSet: group.adminSet, actorCidNumber: _cidNumber);
    await _store.renameGroup(_ownerCidNumber, groupId, name);
    await sendGroupControl(groupId, GroupControl.rename(name));
  }

  /// 广播群控制消息(改名/退群请求):走 E2E application 扇出,**不落聊天消息行**。
  Future<void> sendGroupControl(String groupId, GroupControl control) async {
    final group = await _requireGroup(groupId);
    final recipients = group.memberCidNumbers
        .where((cidNumber) => cidNumber != _cidNumber)
        .toList();
    if (recipients.isEmpty) {
      return;
    }
    final wire = await _crypto.groupCreateMessage(
      groupId,
      utf8.encode(GroupControlCodec.encode(control)),
    );
    await _fanoutHandshake(
      wire: wire,
      recipients: recipients,
      senderCidNumber: _cidNumber,
      senderDeviceId: _localDeviceId,
      groupId: groupId,
      nowMillis: DateTime.now().millisecondsSinceEpoch,
      tag: 'ctrl',
    );
  }

  /// 群发文本:单次加密 → 扇 N 信封 → 1 条逻辑消息 + N 出站队列。
  Future<List<ChatDeliveryResult>> sendGroupText({
    required String groupId,
    required String senderCidNumber,
    required String senderDeviceId,
    required String text,
  }) {
    return _sendGroupUserMessage(
      groupId: groupId,
      senderCidNumber: senderCidNumber,
      senderDeviceId: senderDeviceId,
      messageKind: ChatMessageKind.text,
      payload: ChatPayloadCodec.encode(ChatContent.text(text)),
    );
  }

  /// 群发内置贴纸(零字节,收端本地渲染;复用群发用户消息编排)。
  Future<List<ChatDeliveryResult>> sendGroupSticker({
    required String groupId,
    required String senderCidNumber,
    required String senderDeviceId,
    required String packId,
    required String stickerId,
  }) {
    return _sendGroupUserMessage(
      groupId: groupId,
      senderCidNumber: senderCidNumber,
      senderDeviceId: senderDeviceId,
      messageKind: ChatMessageKind.sticker,
      payload: ChatPayloadCodec.encode(
        ChatContent.sticker(packId: packId, stickerId: stickerId),
      ),
    );
  }

  /// 群发媒体:控制消息单次加密扇 N;字节 **>100MB 走已部署中转**(一次上传 + K 扇 N),
  /// **≤100MB 对每个成员逐个 WebRTC 直传**(口径 A,离线按成员补发)。四门按己档强制。
  Future<List<ChatDeliveryResult>> sendGroupMedia({
    required String groupId,
    required String senderCidNumber,
    required String senderDeviceId,
    required ChatMediaDraft media,
    required ChatAttachmentDeviceSender sendMemberAttachment,
    ChatRelayUploader? uploadRelayMedia,
    ChatLocalAttachmentSaver? saveLocalAttachment,
    GroupMemberMediaRecorder? recordPendingMember,
    GroupMemberMediaRecorder? markMemberDelivered,
  }) async {
    final group = await _requireGroup(groupId);
    if (group.leftLocally) {
      throw StateError('已退出该群，无法发送');
    }
    // 门①:己档硬拦(非薪火发不出 >100MB)。
    if (ChatMediaLimits.exceedsForKind(media.kind, media.byteSize)) {
      throw ChatMediaTooLargeException(
        byteSize: media.byteSize,
        limitBytes: ChatMediaLimits.forKind(media.kind),
        kind: media.kind,
      );
    }
    final nowMillis = DateTime.now().millisecondsSinceEpoch;
    final attachmentId = 'att-$nowMillis-${_nonce()}';

    // 路由:>100MB **必须**经中转(一次上传),绝不走 WebRTC。
    ChatRelayDescriptor? relay;
    if (ChatMediaLimits.needsRelay(media.byteSize)) {
      if (uploadRelayMedia == null) {
        throw StateError('>100MB 媒体必须经 Cloudflare 中转,但中转未配置');
      }
      relay = await uploadRelayMedia(
        conversationId: groupId,
        attachmentId: attachmentId,
        media: media,
        // 群删时机:全部收件人(减自己)ack 后删,避免首个 ack 即删。
        recipientCount: group.memberCidNumbers
            .where((account) => account != senderCidNumber)
            .length,
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
    // 控制消息单次加密扇 N + 落 1 逻辑媒体消息(复用共用编排)。
    final results = await _sendGroupUserMessage(
      groupId: groupId,
      senderCidNumber: senderCidNumber,
      senderDeviceId: senderDeviceId,
      messageKind: media.kind,
      payload: payload,
    );
    await saveLocalAttachment?.call(
      conversationId: groupId,
      attachmentId: attachmentId,
      fileName: media.fileName,
      contentType: media.contentType,
      sourcePath: media.sourcePath,
      byteSize: media.byteSize,
    );
    // 中转路径:密文已在 R2,收方按需拉;不走 WebRTC。
    if (relay != null) {
      return results;
    }
    // ≤100MB：对每个成员 CID 逐个 WebRTC 直传（离线按成员留 pending 补发）。
    final members = group.memberCidNumbers
        .where((account) => account != senderCidNumber)
        .toList(growable: false);
    for (final member in members) {
      await recordPendingMember?.call(attachmentId, member);
      try {
        await sendMemberAttachment(
          recipientCidNumber: member,
          conversationId: groupId,
          attachmentId: attachmentId,
          fileName: media.fileName,
          contentType: media.contentType,
          sourcePath: media.sourcePath,
          byteSize: media.byteSize,
        );
        await markMemberDelivered?.call(attachmentId, member);
      } on Exception {
        // 该成员离线/直连失败:留 pending,peer_ready 补发。
      }
    }
    return results;
  }

  /// 群发用户消息(文本/贴纸)共用编排:单次加密 → 扇 N → 1 逻辑消息 + N 出站队列。
  Future<List<ChatDeliveryResult>> _sendGroupUserMessage({
    required String groupId,
    required String senderCidNumber,
    required String senderDeviceId,
    required ChatMessageKind messageKind,
    required String payload,
  }) async {
    final group = await _requireGroup(groupId);
    if (group.leftLocally) {
      throw StateError('已退出该群，无法发送');
    }
    final nowMillis = DateTime.now().millisecondsSinceEpoch;
    final wire =
        await _crypto.groupCreateMessage(groupId, utf8.encode(payload));
    final recipients = group.memberCidNumbers
        .where((account) => account != senderCidNumber)
        .toList();
    final messageId = '$groupId-msg-$nowMillis-${_nonce()}';
    final envelopes = GroupFanout.fanOut(
      wire: wire,
      recipientCidNumbers: recipients,
      senderCidNumber: senderCidNumber,
      senderDeviceId: senderDeviceId,
      messageId: messageId,
      nowMillis: nowMillis,
      ttlMillis: defaultTtlMillis,
    );
    await _store.saveOutgoingGroupMessage(
      ownerCidNumber: _ownerCidNumber,
      currentAccountId: _currentAccountId,
      groupId: groupId,
      senderCidNumber: senderCidNumber,
      senderDeviceId: senderDeviceId,
      logicalEnvelopeId: messageId,
      messageKind: messageKind,
      payload: payload,
      createdAtMillis: nowMillis,
      envelopes: envelopes,
      recipientCidByCidNumber: {
        for (final cidNumber in recipients) cidNumber: cidNumber
      },
    );
    final results = <ChatDeliveryResult>[];
    for (final envelope in envelopes) {
      final result = await _deliverer(
        envelope,
        envelope.writeToBuffer(),
        envelope.recipientCidNumber,
      );
      await _store.markOutgoingDelivery(
        ownerCidNumber: _ownerCidNumber,
        envelopeId: envelope.envelopeId,
        state: result.state,
        errorMessage: result.errorMessage,
      );
      results.add(result);
    }
    // 逻辑消息态:任一投出即 sent,否则维持 queued。
    final anySent =
        results.any((result) => result.state == ChatMessageDeliveryState.sent);
    await _store.markOutgoingDelivery(
      ownerCidNumber: _ownerCidNumber,
      envelopeId: messageId,
      state: anySent
          ? ChatMessageDeliveryState.sent
          : ChatMessageDeliveryState.queued,
    );
    return results;
  }

  /// 处理入站群 envelope:经 epoch 有序处理后落地。
  ///
  /// 入群前(未处理 Welcome)到达的 Commit/Application 会让 Rust 报"群会话不存在",
  /// 此时存入 pending-inbound,由 Welcome 处理后回放(复用 1:1 机制)。
  Future<GroupInbound?> processIncomingGroupEnvelope(
    List<int> envelopeBytes,
  ) async {
    final envelope = ChatEnvelope.fromBuffer(envelopeBytes);
    final wire = imMlsWireMessageFromEnvelope(envelope);
    try {
      final result = await GroupEpochOrdering.processOrdered(
        wire: wire,
        envelope: envelope,
        process: _crypto.groupProcess,
        bufferPut: (groupId, messageEpoch, bufferedEnvelope) =>
            _store.bufferGroupCommit(
          ownerCidNumber: _ownerCidNumber,
          groupId: groupId,
          messageEpoch: messageEpoch,
          envelope: bufferedEnvelope,
          envelopeBytes: bufferedEnvelope.writeToBuffer(),
        ),
        bufferTake: (groupId, messageEpoch) => _store.takeGroupPendingCommit(
          _ownerCidNumber,
          groupId,
          messageEpoch,
        ),
        wireFromEnvelope: imMlsWireMessageFromEnvelope,
      );
      await _applyInbound(envelope, envelopeBytes, result);
      return result;
    } catch (error) {
      if (_needsWelcomeFirst(error)) {
        await _store.savePendingInbound(
          ownerCidNumber: _ownerCidNumber,
          envelope: envelope,
          envelopeBytes: envelopeBytes,
          reason: error.toString(),
        );
        return null;
      }
      rethrow;
    }
  }

  Future<void> _applyInbound(
    ChatEnvelope envelope,
    List<int> envelopeBytes,
    GroupInbound result,
  ) async {
    if (!result.isApplied) {
      return; // out_of_order 已缓冲;stale 丢弃。
    }
    final creator = creatorCidNumberFromGroupId(result.groupId);
    switch (result.kind) {
      case GroupInboundKind.welcome:
        await _store.upsertGroupShell(
          ownerCidNumber: _ownerCidNumber,
          groupId: result.groupId,
          groupName: '群聊',
          creatorCidNumber: creator,
          epoch: result.groupEpoch,
        );
        await _reconcileRosterFrom(result, creator);
        // 回放入群前缓冲的同群消息。
        final pending = await _store.takePendingInbound(
          _ownerCidNumber,
          result.groupId,
        );
        for (final buffered in pending) {
          await processIncomingGroupEnvelope(buffered.writeToBuffer());
        }
      case GroupInboundKind.commit:
        if (result.selfRemoved) {
          await _store.markGroupLeft(_ownerCidNumber, result.groupId);
          return;
        }
        await _reconcileRosterFrom(result, creator);
      case GroupInboundKind.application:
        final plaintext = utf8.decode(result.plaintext ?? const []);
        // 群控制消息先判别:是控制则处理、绝不当聊天消息显示;否则落普通消息。
        final control = GroupControlCodec.tryDecode(plaintext);
        if (control != null) {
          await _handleGroupControl(envelope, control);
          return;
        }
        await _store.saveIncomingGroupMessage(
          ownerCidNumber: _ownerCidNumber,
          currentAccountId: _currentAccountId,
          envelope: envelope,
          envelopeBytes: envelopeBytes,
          messageKind: ChatPayloadCodec.decode(plaintext).kind,
          plaintext: plaintext,
        );
      case GroupInboundKind.unknown:
        break;
    }
  }

  Future<void> _handleGroupControl(
    ChatEnvelope envelope,
    GroupControl control,
  ) async {
    final groupId = envelope.conversationId;
    switch (control.op) {
      case GroupControlOp.rename:
        await _store.renameGroup(
          _ownerCidNumber,
          groupId,
          control.groupName ?? '',
        );
      case GroupControlOp.leaveRequest:
        final group = await _store.readGroup(_ownerCidNumber, groupId);
        if (group == null || group.leftLocally) {
          return;
        }
        // 仅本机是 admin 时代提交移除退群者;其余成员忽略,靠 admin 的 Commit 收敛。
        if (group.adminSet.contains(_cidNumber)) {
          await removeMembers(
            groupId: groupId,
            actorCidNumber: _cidNumber,
            actorDeviceId: _localDeviceId,
            targetCidNumbers: [envelope.senderCidNumber],
          );
        }
    }
  }

  Future<void> _reconcileFromChain(
      String groupId, String creatorCidNumber) async {
    final state = await _crypto.groupState(groupId);
    await _store.reconcileGroupRoster(
      ownerCidNumber: _ownerCidNumber,
      groupId: groupId,
      members: _rolesFor(state.memberIdentities, creatorCidNumber),
      epoch: state.epoch,
    );
  }

  Future<void> _reconcileRosterFrom(
    GroupInbound result,
    String creatorCidNumber,
  ) async {
    final identities = result.memberIdentities ?? const [];
    await _store.reconcileGroupRoster(
      ownerCidNumber: _ownerCidNumber,
      groupId: result.groupId,
      members: _rolesFor(identities, creatorCidNumber),
      epoch: result.groupEpoch,
    );
  }

  Map<String, GroupMemberRole> _rolesFor(
    Iterable<String> identities,
    String creatorCidNumber,
  ) {
    final cidNumbers = cidNumbersFromMemberIdentities(identities);
    return {
      for (final cidNumber in cidNumbers)
        cidNumber: cidNumber == creatorCidNumber
            ? GroupMemberRole.admin
            : GroupMemberRole.member,
    };
  }

  Future<void> _fanoutHandshake({
    required MlsWireMessage wire,
    required List<String> recipients,
    required String senderCidNumber,
    required String senderDeviceId,
    required String groupId,
    required int nowMillis,
    required String tag,
  }) async {
    final messageId = '$groupId-$tag-$nowMillis-${_nonce()}';
    final envelopes = GroupFanout.fanOut(
      wire: wire,
      recipientCidNumbers: recipients,
      senderCidNumber: senderCidNumber,
      senderDeviceId: senderDeviceId,
      messageId: messageId,
      nowMillis: nowMillis,
      ttlMillis: defaultTtlMillis,
    );
    for (final envelope in envelopes) {
      final bytes = envelope.writeToBuffer();
      final recipientCidNumber = envelope.recipientCidNumber;
      await _store.queueOutgoingEnvelope(
        ownerCidNumber: _ownerCidNumber,
        envelope: envelope,
        envelopeBytes: bytes,
        recipientCidNumber: recipientCidNumber,
        deliveryState: ChatMessageDeliveryState.queued,
      );
      final result = await _deliverer(envelope, bytes, recipientCidNumber);
      await _store.markOutgoingDelivery(
        ownerCidNumber: _ownerCidNumber,
        envelopeId: envelope.envelopeId,
        state: result.state,
        errorMessage: result.errorMessage,
      );
    }
  }

  Future<ChatGroup> _requireGroup(String groupId) async {
    final group = await _store.readGroup(_ownerCidNumber, groupId);
    if (group == null) {
      throw StateError('群不存在: $groupId');
    }
    return group;
  }
}

bool _needsWelcomeFirst(Object error) {
  return error.toString().contains('群会话不存在');
}

String _nonce() {
  final random = Random.secure();
  final bytes = List<int>.generate(8, (_) => random.nextInt(256));
  return bytes.map((byte) => byte.toRadixString(16).padLeft(2, '0')).join();
}
