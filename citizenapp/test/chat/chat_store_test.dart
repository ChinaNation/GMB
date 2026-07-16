import 'package:flutter_test/flutter_test.dart';
import 'package:citizenapp/chat/crypto/mls_session.dart';
import 'package:citizenapp/chat/chat_models.dart';
import 'package:citizenapp/chat/storage/chat_store.dart';

import '../support/isar_test_env.dart';

void main() {
  useIsolatedIsar();

  test('Isar store persists outgoing, pending, and incoming Chat records',
      () async {
    final store = ChatStore();
    final envelope = const MlsWireMessage(
      wireBytes: [0x68, 0x69],
      cipherSuite: 'MLS_128',
      conversationId: 'conv-store',
      messageKind: MlsMessageKind.application,
    ).toEnvelope(
      envelopeId: 'env-store',
      senderAccount: 'alice-wallet',
      recipientAccount: 'bob-wallet',
      senderDeviceId: 'alice-phone',
      createdAtMillis: 10,
      ttlMillis: 60000,
    );

    await store.saveOutgoingEnvelope(
      envelope: envelope,
      envelopeBytes: envelope.writeToBuffer(),
      messageKind: ChatMessageKind.text,
      deliveryState: ChatMessageDeliveryState.queued,
      plaintext: 'hi',
    );
    await store.markOutgoingDelivery(
      envelopeId: 'env-store',
      state: ChatMessageDeliveryState.sent,
    );

    final outgoing = await store.readMessages('conv-store');
    expect(outgoing.single.deliveryState, ChatMessageDeliveryState.sent);
    expect(outgoing.single.plaintext, 'hi');

    await store.savePendingInbound(
      envelope: envelope,
      envelopeBytes: envelope.writeToBuffer(),
      reason: 'waiting for welcome',
    );
    expect(await store.pendingInboundCount(), 1);

    final pending = await store.takePendingInbound('conv-store');
    expect(pending.single.envelopeId, 'env-store');
    expect(await store.pendingInboundCount(), 0);

    await store.saveIncomingEnvelope(
      envelope: envelope,
      envelopeBytes: envelope.writeToBuffer(),
      messageKind: ChatMessageKind.text,
      plaintext: 'hi back',
    );
    final conversations = await store.readConversationPreviews();
    expect(conversations.single.unreadCount, 1);
    expect(conversations.single.lastMessage, 'hi back');
  });

  test('Isar store deletes one local conversation without touching others',
      () async {
    final store = ChatStore();
    final targetEnvelope = const MlsWireMessage(
      wireBytes: [0x01],
      cipherSuite: 'MLS_128',
      conversationId: 'conv-delete',
      messageKind: MlsMessageKind.application,
    ).toEnvelope(
      envelopeId: 'env-delete',
      senderAccount: 'alice-wallet',
      recipientAccount: 'bob-wallet',
      senderDeviceId: 'alice-phone',
      createdAtMillis: 10,
      ttlMillis: 60000,
    );
    final keptEnvelope = const MlsWireMessage(
      wireBytes: [0x02],
      cipherSuite: 'MLS_128',
      conversationId: 'conv-keep',
      messageKind: MlsMessageKind.application,
    ).toEnvelope(
      envelopeId: 'env-keep',
      senderAccount: 'alice-wallet',
      recipientAccount: 'carol-wallet',
      senderDeviceId: 'alice-phone',
      createdAtMillis: 20,
      ttlMillis: 60000,
    );

    await store.saveOutgoingEnvelope(
      envelope: targetEnvelope,
      envelopeBytes: targetEnvelope.writeToBuffer(),
      messageKind: ChatMessageKind.text,
      deliveryState: ChatMessageDeliveryState.queued,
      plaintext: 'delete me',
    );
    await store.savePendingInbound(
      envelope: targetEnvelope,
      envelopeBytes: targetEnvelope.writeToBuffer(),
      reason: 'waiting',
    );
    await store.saveOutgoingEnvelope(
      envelope: keptEnvelope,
      envelopeBytes: keptEnvelope.writeToBuffer(),
      messageKind: ChatMessageKind.text,
      deliveryState: ChatMessageDeliveryState.sent,
      plaintext: 'keep me',
    );

    expect(await store.outboundQueueCount(), 2);
    expect(await store.pendingInboundCount(), 1);

    await store.deleteConversation('conv-delete');

    expect(await store.readMessages('conv-delete'), isEmpty);
    expect(await store.pendingInboundCount(), 0);
    expect(await store.outboundQueueCount(), 1);

    final conversations = await store.readConversationPreviews();
    expect(conversations.single.conversationId, 'conv-keep');
    expect((await store.readMessages('conv-keep')).single.plaintext, 'keep me');
  });

  test('待设备投递媒体队列:登记 / 按对端读 / 删 / 会话删连带清理', () async {
    final store = ChatStore();
    await store.recordOutgoingMedia(
      attachmentId: 'att-1',
      recipientAccount: 'bob-wallet',
      conversationId: 'conv-a',
      fileName: 'p.jpg',
      contentType: 'image/jpeg',
      byteSize: 100,
    );
    await store.recordOutgoingMedia(
      attachmentId: 'att-2',
      recipientAccount: 'carol-wallet',
      conversationId: 'conv-b',
      fileName: 'v.mp4',
      contentType: 'video/mp4',
      byteSize: 200,
    );
    expect(await store.outgoingMediaCount(), 2);

    final forBob = await store.readPendingOutgoingMedia(
      recipientAccount: 'bob-wallet',
    );
    expect(forBob.single.attachmentId, 'att-1');
    expect(forBob.single.fileName, 'p.jpg');
    expect(forBob.single.conversationId, 'conv-a');
    expect(forBob.single.byteSize, 100);

    await store.deleteOutgoingMedia('att-1', 'bob-wallet'); // 收到 ack 后删除
    expect(await store.outgoingMediaCount(), 1);

    // 删会话 conv-b 连带清理其待投递媒体,不留孤儿。
    await store.deleteConversation('conv-b');
    expect(await store.outgoingMediaCount(), 0);
  });

  test('群媒体:同一 attachmentId 发多成员各占一行,按成员删', () async {
    final store = ChatStore();
    for (final member in ['b-wallet', 'c-wallet', 'd-wallet']) {
      await store.recordOutgoingMedia(
        attachmentId: 'att-grp',
        recipientAccount: member,
        conversationId: 'grp:a:n',
        fileName: 'g.jpg',
        contentType: 'image/jpeg',
        byteSize: 100,
      );
    }
    expect(await store.outgoingMediaCount(), 3);
    // 仅 c 收到 ack → 删 c 的行,b/d 待投递保留。
    await store.deleteOutgoingMedia('att-grp', 'c-wallet');
    expect(await store.outgoingMediaCount(), 2);
    final forB = await store.readPendingOutgoingMedia(recipientAccount: 'b-wallet');
    expect(forB.single.attachmentId, 'att-grp');
  });

  test('clearAllForOwner 连带清理该 owner 会话的待投递媒体', () async {
    final store = ChatStore();
    // 以出站信封建立 owner=alice 的会话行(conversationId=conv-own)。
    final envelope = const MlsWireMessage(
      wireBytes: [1],
      cipherSuite: 'MLS_128',
      conversationId: 'conv-own',
      messageKind: MlsMessageKind.application,
    ).toEnvelope(
      envelopeId: 'env-own',
      senderAccount: 'alice-wallet',
      recipientAccount: 'bob-wallet',
      senderDeviceId: 'alice-phone',
      createdAtMillis: 1,
      ttlMillis: 60000,
    );
    await store.saveOutgoingEnvelope(
      envelope: envelope,
      envelopeBytes: envelope.writeToBuffer(),
      messageKind: ChatMessageKind.text,
      deliveryState: ChatMessageDeliveryState.queued,
      plaintext: 'hi',
    );
    await store.recordOutgoingMedia(
      attachmentId: 'att-own',
      recipientAccount: 'bob-wallet',
      conversationId: 'conv-own',
      fileName: 'p.jpg',
      contentType: 'image/jpeg',
      byteSize: 5,
    );
    expect(await store.outgoingMediaCount(), 1);

    await store.clearAllForOwner('alice-wallet');
    expect(await store.outgoingMediaCount(), 0);
  });
}
