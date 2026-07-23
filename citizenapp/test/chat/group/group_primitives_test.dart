import 'package:citizenapp/chat/crypto/mls_group_boundary.dart';
import 'package:citizenapp/chat/crypto/mls_session.dart';
import 'package:citizenapp/chat/group/chat_group_limits.dart';
import 'package:citizenapp/chat/group/group_epoch.dart';
import 'package:citizenapp/chat/group/group_fanout.dart';
import 'package:citizenapp/chat/group/group_membership.dart';
import 'package:citizenapp/chat/proto/chat_envelope.pb.dart';
import 'package:flutter_test/flutter_test.dart';

void main() {
  group('GroupFanout', () {
    test('单密文扇成 N 信封:同 wire 异 recipient、envelope_id 唯一', () {
      const wire = MlsWireMessage(
        wireBytes: [1, 2, 3, 4],
        cipherSuite: '',
        conversationId: 'grp:acctA:n1',
        messageKind: MlsMessageKind.application,
      );
      final envelopes = GroupFanout.fanOut(
        wire: wire,
        recipientAccountIds: const [
          '0x4444444444444444444444444444444444444444444444444444444444444444',
          'acctC',
          'acctD'
        ],
        senderAccountId:
            '0x3333333333333333333333333333333333333333333333333333333333333333',
        senderDeviceId: 'devA',
        messageId: 'msg-1',
        nowMillis: 1000,
        ttlMillis: 60,
      );

      expect(envelopes.length, 3);
      expect(
        envelopes.map((e) => e.recipientAccountId).toList(),
        [
          '0x4444444444444444444444444444444444444444444444444444444444444444',
          'acctC',
          'acctD'
        ],
      );
      // 同一份密文。
      for (final envelope in envelopes) {
        expect(envelope.mlsWireMessage, const [1, 2, 3, 4]);
        expect(envelope.senderAccountId,
            '0x3333333333333333333333333333333333333333333333333333333333333333');
        expect(envelope.conversationId, 'grp:acctA:n1');
      }
      // envelope_id 唯一。
      final ids = envelopes.map((e) => e.envelopeId).toSet();
      expect(ids.length, 3);
    });

    test('空收件人(仅自己)返回空扇出', () {
      const wire = MlsWireMessage(
        wireBytes: [9],
        cipherSuite: '',
        conversationId: 'grp:acctA:n1',
        messageKind: MlsMessageKind.application,
      );
      final envelopes = GroupFanout.fanOut(
        wire: wire,
        recipientAccountIds: const [],
        senderAccountId:
            '0x3333333333333333333333333333333333333333333333333333333333333333',
        senderDeviceId: 'devA',
        messageId: 'msg-2',
        nowMillis: 1,
        ttlMillis: 1,
      );
      expect(envelopes, isEmpty);
    });
  });

  group('GroupMembership 上限/权限', () {
    test('建群人数达上限通过、超限拒', () {
      expect(
        () =>
            GroupMembership.ensureCanCreate(inviteeCount: kMaxGroupMembers - 1),
        returnsNormally,
      );
      expect(
        () => GroupMembership.ensureCanCreate(inviteeCount: kMaxGroupMembers),
        throwsA(isA<GroupMembershipException>()),
      );
    });

    test('加人到 1989 通过、第 1990 人拒', () {
      expect(
        () => GroupMembership.ensureCanAdd(
            currentCount: kMaxGroupMembers - 1, addingCount: 1),
        returnsNormally,
      );
      expect(
        () => GroupMembership.ensureCanAdd(
            currentCount: kMaxGroupMembers, addingCount: 1),
        throwsA(isA<GroupMembershipException>()),
      );
      expect(
        () => GroupMembership.ensureCanAdd(currentCount: 5, addingCount: 0),
        throwsA(isA<GroupMembershipException>()),
      );
    });

    test('仅 admin 可加/删', () {
      expect(
        () => GroupMembership.ensureAdmin(
            adminSet: {
              '0x3333333333333333333333333333333333333333333333333333333333333333'
            },
            actorAccountId:
                '0x3333333333333333333333333333333333333333333333333333333333333333'),
        returnsNormally,
      );
      expect(
        () => GroupMembership.ensureAdmin(
            adminSet: {
              '0x3333333333333333333333333333333333333333333333333333333333333333'
            },
            actorAccountId:
                '0x4444444444444444444444444444444444444444444444444444444444444444'),
        throwsA(isA<GroupMembershipException>()),
      );
    });
  });

  group('GroupEpochOrdering 乱序 Commit 缓冲/回放', () {
    test('未来 epoch Commit 先到被缓冲,前序补齐后按序回放', () async {
      const groupId = 'grp:acctA:n1';
      var current = 5;
      final buffer = <int, List<ChatEnvelope>>{};

      MlsWireMessage commitWire(int messageEpoch) => MlsWireMessage(
            wireBytes: [messageEpoch],
            cipherSuite: '',
            conversationId: groupId,
            messageKind: MlsMessageKind.application,
          );
      ChatEnvelope envelopeFor(int messageEpoch) =>
          commitWire(messageEpoch).toEnvelope(
            envelopeId: 'e$messageEpoch',
            senderAccountId: 'acctS',
            recipientAccountId: 'acctR',
            senderDeviceId: 'devS',
            createdAtMillis: 0,
            ttlMillis: 0,
          );

      Future<GroupInbound> process(MlsWireMessage wire) async {
        final messageEpoch = wire.wireBytes.first;
        if (messageEpoch > current) {
          return GroupInbound(
            groupId: groupId,
            kind: GroupInboundKind.unknown,
            status: GroupProcessStatus.outOfOrder,
            messageEpoch: messageEpoch,
            groupEpoch: current,
            selfRemoved: false,
          );
        }
        if (messageEpoch < current) {
          return GroupInbound(
            groupId: groupId,
            kind: GroupInboundKind.unknown,
            status: GroupProcessStatus.stale,
            messageEpoch: messageEpoch,
            groupEpoch: current,
            selfRemoved: false,
          );
        }
        current = messageEpoch + 1;
        return GroupInbound(
          groupId: groupId,
          kind: GroupInboundKind.commit,
          status: GroupProcessStatus.applied,
          messageEpoch: messageEpoch,
          groupEpoch: current,
          selfRemoved: false,
          memberIdentities: const [],
        );
      }

      Future<void> put(String g, int e, ChatEnvelope en) async {
        (buffer[e] ??= <ChatEnvelope>[]).add(en);
      }

      Future<ChatEnvelope?> take(String g, int e) async {
        final list = buffer[e];
        if (list == null || list.isEmpty) return null;
        return list.removeAt(0);
      }

      // 未来 epoch 6 的 Commit 先到 → 缓冲,epoch 不动。
      final first = await GroupEpochOrdering.processOrdered(
        wire: commitWire(6),
        envelope: envelopeFor(6),
        process: process,
        bufferPut: put,
        bufferTake: take,
        wireFromEnvelope: imMlsWireMessageFromEnvelope,
      );
      expect(first.status, GroupProcessStatus.outOfOrder);
      expect(current, 5);

      // 前序 epoch 5 的 Commit 到 → 应用(→6),随即回放缓冲中的 6(→7)。
      final second = await GroupEpochOrdering.processOrdered(
        wire: commitWire(5),
        envelope: envelopeFor(5),
        process: process,
        bufferPut: put,
        bufferTake: take,
        wireFromEnvelope: imMlsWireMessageFromEnvelope,
      );
      expect(second.status, GroupProcessStatus.applied);
      expect(current, 7);
      expect(buffer.values.every((list) => list.isEmpty), isTrue);
    });
  });
}
