import 'dart:convert';

import 'package:citizenapp/chat/chat_flow.dart';
import 'package:citizenapp/chat/chat_media_limits.dart';
import 'package:citizenapp/chat/chat_models.dart';
import 'package:citizenapp/chat/chat_payload.dart';
import 'package:citizenapp/chat/crypto/mls_boundary.dart';
import 'package:citizenapp/chat/crypto/mls_group_boundary.dart';
import 'package:citizenapp/chat/group/group_control.dart';
import 'package:citizenapp/chat/group/group_flow.dart';
import 'package:citizenapp/chat/group/group_membership.dart';
import 'package:citizenapp/chat/proto/chat_envelope.pb.dart';
import 'package:citizenapp/chat/storage/chat_store.dart';
import 'package:citizenapp/chat/transport/chat_transport.dart';
import 'package:flutter_test/flutter_test.dart';

import '../../support/isar_test_env.dart';

/// 内存态 fake:模拟 MLS 群语义(roster + epoch),不做真加密。
class _FakeGroupCrypto implements MlsGroupCrypto {
  _FakeGroupCrypto({required this.accountId, required this.localDeviceId});

  final String accountId;
  final String localDeviceId;
  final Map<String, List<String>> _roster = {};
  final Map<String, int> _epoch = {};

  String get _localIdentity => '$accountId:$localDeviceId';

  @override
  Future<GroupCreated> createGroup(String groupId) async {
    _roster[groupId] = [_localIdentity];
    _epoch[groupId] = 0;
    return GroupCreated(groupId: groupId, epoch: 0);
  }

  @override
  Future<GroupCommitBundle> addMembers(
    String groupId,
    List<MlsKeyPackage> keyPackages,
  ) async {
    final roster = _roster[groupId]!;
    for (final keyPackage in keyPackages) {
      roster.add('${keyPackage.accountId}:${keyPackage.deviceId}');
    }
    _epoch[groupId] = (_epoch[groupId] ?? 0) + 1;
    return GroupCommitBundle(
      groupId: groupId,
      epoch: _epoch[groupId]!,
      commit: _wire(groupId, 'commit'),
      welcome: _wire(groupId, 'welcome'),
    );
  }

  @override
  Future<GroupCommitBundle> removeMembers(
    String groupId,
    List<String> memberAccountIds,
  ) async {
    final roster = _roster[groupId]!;
    roster.removeWhere(
        (identity) => memberAccountIds.contains(identity.split(':').first));
    _epoch[groupId] = (_epoch[groupId] ?? 0) + 1;
    return GroupCommitBundle(
      groupId: groupId,
      epoch: _epoch[groupId]!,
      commit: _wire(groupId, 'commit'),
      removedAccounts: memberAccountIds,
    );
  }

  @override
  Future<MlsWireMessage> groupCreateMessage(
    String groupId,
    List<int> plaintext,
  ) async {
    return MlsWireMessage(
      wireBytes: plaintext,
      cipherSuite: '',
      conversationId: groupId,
      messageKind: MlsMessageKind.application,
    );
  }

  @override
  Future<GroupInbound> groupProcess(MlsWireMessage wire) async {
    // 测试只驱动 application 入站:回显明文。
    final epoch = _epoch[wire.conversationId] ?? 0;
    return GroupInbound(
      groupId: wire.conversationId,
      kind: GroupInboundKind.application,
      status: GroupProcessStatus.applied,
      messageEpoch: epoch,
      groupEpoch: epoch,
      selfRemoved: false,
      plaintext: wire.wireBytes,
    );
  }

  @override
  Future<GroupState> groupState(String groupId) async {
    return GroupState(
      groupId: groupId,
      epoch: _epoch[groupId] ?? 0,
      memberIdentities: List.of(_roster[groupId] ?? const []),
    );
  }

  MlsWireMessage _wire(String groupId, String tag) => MlsWireMessage(
        wireBytes: utf8.encode(tag),
        cipherSuite: '',
        conversationId: groupId,
        messageKind: MlsMessageKind.application,
      );
}

ChatMediaDraft _mediaDraft(int byteSize) => ChatMediaDraft(
      kind: ChatMessageKind.image,
      fileName: 'g.jpg',
      contentType: 'image/jpeg',
      sourcePath: '/dev/null',
      byteSize: byteSize,
    );

Future<ChatDeliveryResult> _okDeliverer(
  ChatEnvelope envelope,
  List<int> bytes,
) async =>
    ChatDeliveryResult(
      envelopeId: envelope.envelopeId,
      transportType: ChatTransportType.cloudflare,
      state: ChatMessageDeliveryState.sent,
    );

MlsKeyPackage _keyPackage(String account, String device) => MlsKeyPackage(
      accountId: account,
      deviceId: device,
      keyPackageId: 'kp-$account',
      keyPackageBytes: const [1, 2],
      cipherSuite: '',
      createdAtMillis: 0,
      expiresAtMillis: 0,
    );

void main() {
  useIsolatedIsar();

  test('建群→发文本→收文本→删人 全链路(fake 密码学 + 真 Isar)', () async {
    final store = ChatStore();
    final crypto = _FakeGroupCrypto(
        accountId:
            '0x3333333333333333333333333333333333333333333333333333333333333333',
        localDeviceId: 'devA');
    final delivered = <ChatEnvelope>[];
    Future<ChatDeliveryResult> deliverer(
      ChatEnvelope envelope,
      List<int> bytes,
    ) async {
      delivered.add(envelope);
      return ChatDeliveryResult(
        envelopeId: envelope.envelopeId,
        transportType: ChatTransportType.cloudflare,
        state: ChatMessageDeliveryState.sent,
      );
    }

    final flow = ChatGroupFlow(
      crypto: crypto,
      store: store,
      deliverer: deliverer,
      accountId:
          '0x3333333333333333333333333333333333333333333333333333333333333333',
      localDeviceId: 'devA',
    );
    const groupId = 'grp:acctA:testnonce';

    // 建群 + 邀请 B、C。
    final group = await flow.createGroup(
      groupId: groupId,
      name: '测试群',
      accountId:
          '0x3333333333333333333333333333333333333333333333333333333333333333',
      localDeviceId: 'devA',
      invitees: [
        _keyPackage(
            '0x4444444444444444444444444444444444444444444444444444444444444444',
            'devB'),
        _keyPackage('acctC', 'devC')
      ],
    );
    expect(group.memberAccountIds.toSet(), {
      '0x3333333333333333333333333333333333333333333333333333333333333333',
      '0x4444444444444444444444444444444444444444444444444444444444444444',
      'acctC'
    });
    expect(group.adminSet,
        {'0x3333333333333333333333333333333333333333333333333333333333333333'});
    // Welcome 扇给 B、C(建群时无其他现有成员,无 Commit 扇出)。
    expect(delivered.map((e) => e.recipientAccountId).toSet(), {
      '0x4444444444444444444444444444444444444444444444444444444444444444',
      'acctC'
    });

    // 群发文本 → 扇给 B、C,落 1 条逻辑消息。
    delivered.clear();
    final results = await flow.sendGroupText(
      groupId: groupId,
      senderAccountId:
          '0x3333333333333333333333333333333333333333333333333333333333333333',
      senderDeviceId: 'devA',
      text: '大家好',
    );
    expect(results.length, 2);
    expect(delivered.map((e) => e.recipientAccountId).toSet(), {
      '0x4444444444444444444444444444444444444444444444444444444444444444',
      'acctC'
    });
    // 同一份密文扇 2 封。
    expect(delivered[0].mlsWireMessage, delivered[1].mlsWireMessage);
    final afterSend = await store.readMessages(groupId);
    final outgoing = afterSend.where((m) => m.direction == 'outgoing').toList();
    expect(outgoing.length, 1);
    expect(outgoing.single.plaintext, contains('大家好'));

    // 收到 B 的文本。
    final payload = ChatPayloadCodec.encode(ChatContent.text('收到'));
    final inboundWire = MlsWireMessage(
      wireBytes: utf8.encode(payload),
      cipherSuite: '',
      conversationId: groupId,
      messageKind: MlsMessageKind.application,
    );
    final inbound = inboundWire.toEnvelope(
      envelopeId: 'in-1',
      senderAccountId:
          '0x4444444444444444444444444444444444444444444444444444444444444444',
      recipientAccountId:
          '0x3333333333333333333333333333333333333333333333333333333333333333',
      senderDeviceId: 'devB',
      createdAtMillis: 100,
      ttlMillis: 60,
    );
    await flow.processIncomingGroupEnvelope(inbound.writeToBuffer());
    final afterIncoming = await store.readMessages(groupId);
    final incoming =
        afterIncoming.where((m) => m.direction == 'incoming').toList();
    expect(incoming.length, 1);
    expect(incoming.single.plaintext, contains('收到'));

    // 删除 C → 名册剩 A、B;Commit 扇给删前成员 B、C(减自己)。
    delivered.clear();
    await flow.removeMembers(
      groupId: groupId,
      actorAccountId:
          '0x3333333333333333333333333333333333333333333333333333333333333333',
      actorDeviceId: 'devA',
      targetAccounts: ['acctC'],
    );
    final afterRemove = await store.readGroup(groupId);
    expect(afterRemove!.memberAccountIds.toSet(), {
      '0x3333333333333333333333333333333333333333333333333333333333333333',
      '0x4444444444444444444444444444444444444444444444444444444444444444'
    });
    expect(delivered.map((e) => e.recipientAccountId).toSet(), {
      '0x4444444444444444444444444444444444444444444444444444444444444444',
      'acctC'
    });
  });

  test('非 admin 加人被拒', () async {
    final store = ChatStore();
    final crypto = _FakeGroupCrypto(
        accountId:
            '0x3333333333333333333333333333333333333333333333333333333333333333',
        localDeviceId: 'devA');
    Future<ChatDeliveryResult> deliverer(
      ChatEnvelope envelope,
      List<int> bytes,
    ) async =>
        ChatDeliveryResult(
          envelopeId: envelope.envelopeId,
          transportType: ChatTransportType.cloudflare,
          state: ChatMessageDeliveryState.sent,
        );
    final flow = ChatGroupFlow(
      crypto: crypto,
      store: store,
      deliverer: deliverer,
      accountId:
          '0x3333333333333333333333333333333333333333333333333333333333333333',
      localDeviceId: 'devA',
    );
    const groupId = 'grp:acctA:n';
    await flow.createGroup(
      groupId: groupId,
      name: 'g',
      accountId:
          '0x3333333333333333333333333333333333333333333333333333333333333333',
      localDeviceId: 'devA',
      invitees: [
        _keyPackage(
            '0x4444444444444444444444444444444444444444444444444444444444444444',
            'devB')
      ],
    );

    await expectLater(
      flow.addMembers(
        groupId: groupId,
        actorAccountId:
            '0x4444444444444444444444444444444444444444444444444444444444444444', // 非 admin
        actorDeviceId: 'devB',
        invitees: [_keyPackage('acctD', 'devD')],
      ),
      throwsA(isA<GroupMembershipException>()),
    );
  });

  test('admin 收到 leave_request → 自动移除退群者(后向保密)', () async {
    final store = ChatStore();
    final crypto = _FakeGroupCrypto(
        accountId:
            '0x3333333333333333333333333333333333333333333333333333333333333333',
        localDeviceId: 'devA');
    final flow = ChatGroupFlow(
      crypto: crypto,
      store: store,
      deliverer: (envelope, bytes) async => ChatDeliveryResult(
        envelopeId: envelope.envelopeId,
        transportType: ChatTransportType.cloudflare,
        state: ChatMessageDeliveryState.sent,
      ),
      accountId:
          '0x3333333333333333333333333333333333333333333333333333333333333333',
      localDeviceId: 'devA',
    );
    const groupId = 'grp:acctA:n';
    await flow.createGroup(
      groupId: groupId,
      name: 'g',
      accountId:
          '0x3333333333333333333333333333333333333333333333333333333333333333',
      localDeviceId: 'devA',
      invitees: [
        _keyPackage(
            '0x4444444444444444444444444444444444444444444444444444444444444444',
            'devB'),
        _keyPackage('acctC', 'devC')
      ],
    );

    // acctB 发来退群请求(fake groupProcess 回显 wire 明文)。
    final payload = GroupControlCodec.encode(const GroupControl.leaveRequest());
    final wire = MlsWireMessage(
      wireBytes: utf8.encode(payload),
      cipherSuite: '',
      conversationId: groupId,
      messageKind: MlsMessageKind.application,
    );
    final envelope = wire.toEnvelope(
      envelopeId: 'lr-1',
      senderAccountId:
          '0x4444444444444444444444444444444444444444444444444444444444444444',
      recipientAccountId:
          '0x3333333333333333333333333333333333333333333333333333333333333333',
      senderDeviceId: 'devB',
      createdAtMillis: 1,
      ttlMillis: 60,
    );
    await flow.processIncomingGroupEnvelope(envelope.writeToBuffer());
    final group = await store.readGroup(groupId);
    expect(group!.memberAccountIds.toSet(), {
      '0x3333333333333333333333333333333333333333333333333333333333333333',
      'acctC'
    }); // B 被移除
  });

  test('收到 rename → 群名更新(非 admin 收端也同步)', () async {
    final store = ChatStore();
    final crypto = _FakeGroupCrypto(
        accountId:
            '0x4444444444444444444444444444444444444444444444444444444444444444',
        localDeviceId: 'devB');
    final flow = ChatGroupFlow(
      crypto: crypto,
      store: store,
      deliverer: (envelope, bytes) async => ChatDeliveryResult(
        envelopeId: envelope.envelopeId,
        transportType: ChatTransportType.cloudflare,
        state: ChatMessageDeliveryState.sent,
      ),
      accountId:
          '0x4444444444444444444444444444444444444444444444444444444444444444',
      localDeviceId: 'devB',
    );
    const groupId = 'grp:acctA:n2';
    await store.upsertGroupShell(
      groupId: groupId,
      groupName: '旧名',
      creatorAccountId:
          '0x3333333333333333333333333333333333333333333333333333333333333333',
      accountId:
          '0x4444444444444444444444444444444444444444444444444444444444444444',
      epoch: 1,
    );

    final payload = GroupControlCodec.encode(GroupControl.rename('新群名'));
    final wire = MlsWireMessage(
      wireBytes: utf8.encode(payload),
      cipherSuite: '',
      conversationId: groupId,
      messageKind: MlsMessageKind.application,
    );
    final envelope = wire.toEnvelope(
      envelopeId: 'rn-1',
      senderAccountId:
          '0x3333333333333333333333333333333333333333333333333333333333333333',
      recipientAccountId:
          '0x4444444444444444444444444444444444444444444444444444444444444444',
      senderDeviceId: 'devA',
      createdAtMillis: 1,
      ttlMillis: 60,
    );
    await flow.processIncomingGroupEnvelope(envelope.writeToBuffer());
    final group = await store.readGroup(groupId);
    expect(group!.name, '新群名');
  });

  test('群发贴纸:落 sticker 消息 + 扇出', () async {
    final store = ChatStore();
    final crypto = _FakeGroupCrypto(
        accountId:
            '0x3333333333333333333333333333333333333333333333333333333333333333',
        localDeviceId: 'devA');
    final delivered = <ChatEnvelope>[];
    final flow = ChatGroupFlow(
      crypto: crypto,
      store: store,
      deliverer: (envelope, bytes) async {
        delivered.add(envelope);
        return ChatDeliveryResult(
          envelopeId: envelope.envelopeId,
          transportType: ChatTransportType.cloudflare,
          state: ChatMessageDeliveryState.sent,
        );
      },
      accountId:
          '0x3333333333333333333333333333333333333333333333333333333333333333',
      localDeviceId: 'devA',
    );
    const groupId = 'grp:acctA:ns';
    await flow.createGroup(
      groupId: groupId,
      name: 'g',
      accountId:
          '0x3333333333333333333333333333333333333333333333333333333333333333',
      localDeviceId: 'devA',
      invitees: [
        _keyPackage(
            '0x4444444444444444444444444444444444444444444444444444444444444444',
            'devB')
      ],
    );

    delivered.clear();
    await flow.sendGroupSticker(
      groupId: groupId,
      senderAccountId:
          '0x3333333333333333333333333333333333333333333333333333333333333333',
      senderDeviceId: 'devA',
      packId: 'fluent3d',
      stickerId: 'grinning_face',
    );
    expect(delivered.map((e) => e.recipientAccountId).toSet(),
        {'0x4444444444444444444444444444444444444444444444444444444444444444'});
    final messages = await store.readMessages(groupId);
    final sticker =
        messages.firstWhere((m) => m.messageKind == ChatMessageKind.sticker);
    expect(sticker.direction, 'outgoing');
  });

  group('群媒体 sendGroupMedia', () {
    setUp(() => ChatMediaLimits.applyMembershipLevel('spark')); // 5GB 档
    tearDown(() => ChatMediaLimits.applyMembershipLevel(null));

    Future<ChatGroupFlow> buildGroup(ChatStore store) async {
      final crypto = _FakeGroupCrypto(
          accountId:
              '0x3333333333333333333333333333333333333333333333333333333333333333',
          localDeviceId: 'devA');
      final flow = ChatGroupFlow(
        crypto: crypto,
        store: store,
        deliverer: _okDeliverer,
        accountId:
            '0x3333333333333333333333333333333333333333333333333333333333333333',
        localDeviceId: 'devA',
      );
      await flow.createGroup(
        groupId: 'grp:acctA:nm',
        name: 'g',
        accountId:
            '0x3333333333333333333333333333333333333333333333333333333333333333',
        localDeviceId: 'devA',
        invitees: [
          _keyPackage(
              '0x4444444444444444444444444444444444444444444444444444444444444444',
              'devB'),
          _keyPackage('acctC', 'devC')
        ],
      );
      return flow;
    }

    test('≤100MB → 对每个成员各发一次 WebRTC + 按成员登记 pending', () async {
      final store = ChatStore();
      final flow = await buildGroup(store);
      final webrtcTo = <String>[];
      final pending = <String>[];
      var relayUploads = 0;

      await flow.sendGroupMedia(
        groupId: 'grp:acctA:nm',
        senderAccountId:
            '0x3333333333333333333333333333333333333333333333333333333333333333',
        senderDeviceId: 'devA',
        media: _mediaDraft(50 * 1024 * 1024),
        sendMemberAttachment: ({
          required recipientAccountId,
          required conversationId,
          required attachmentId,
          required fileName,
          required contentType,
          required sourcePath,
          required byteSize,
        }) async {
          webrtcTo.add(recipientAccountId);
        },
        uploadRelayMedia: ({
          required conversationId,
          required attachmentId,
          required media,
          int recipientCount = 1,
        }) async {
          relayUploads++;
          return const ChatRelayDescriptor(
              relayObjectKey: '', contentKeyB64: '', chunkSize: 0, encSize: 0);
        },
        recordPendingMember: (attachmentId, member) async =>
            pending.add(member),
        markMemberDelivered: (attachmentId, member) async {},
      );

      expect(webrtcTo.toSet(), {
        '0x4444444444444444444444444444444444444444444444444444444444444444',
        'acctC'
      }); // 每成员各一次
      expect(relayUploads, 0);
      expect(pending.toSet(), {
        '0x4444444444444444444444444444444444444444444444444444444444444444',
        'acctC'
      });
    });

    test('>100MB → 中转一次上传,不走 WebRTC', () async {
      final store = ChatStore();
      final flow = await buildGroup(store);
      final webrtcTo = <String>[];
      var relayUploads = 0;

      await flow.sendGroupMedia(
        groupId: 'grp:acctA:nm',
        senderAccountId:
            '0x3333333333333333333333333333333333333333333333333333333333333333',
        senderDeviceId: 'devA',
        media: _mediaDraft(200 * 1024 * 1024),
        sendMemberAttachment: ({
          required recipientAccountId,
          required conversationId,
          required attachmentId,
          required fileName,
          required contentType,
          required sourcePath,
          required byteSize,
        }) async {
          webrtcTo.add(recipientAccountId);
        },
        uploadRelayMedia: ({
          required conversationId,
          required attachmentId,
          required media,
          int recipientCount = 1,
        }) async {
          relayUploads++;
          return const ChatRelayDescriptor(
            relayObjectKey: 'chat-relay/x',
            contentKeyB64: 'a2V5',
            chunkSize: 1048576,
            encSize: 200 * 1024 * 1024 + 8192,
          );
        },
        recordPendingMember: (attachmentId, member) async {},
        markMemberDelivered: (attachmentId, member) async {},
      );

      expect(relayUploads, 1); // 一次上传
      expect(webrtcTo, isEmpty); // 不走 WebRTC
      final content = ChatPayloadCodec.decode(
        (await store.readMessages('grp:acctA:nm')).last.plaintext ?? '',
      );
      expect(content.isRelayMedia, isTrue);
    });
  });
}
