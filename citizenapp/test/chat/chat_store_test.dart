import 'package:flutter_test/flutter_test.dart';
import 'package:citizenapp/chat/crypto/mls_session.dart';
import 'package:citizenapp/chat/chat_models.dart';
import 'package:citizenapp/chat/storage/chat_store.dart';
import 'package:citizenapp/isar/app_isar.dart';

void main() {
  setUp(() async {
    await WalletIsar.instance.resetForTest();
  });

  tearDown(() async {
    await WalletIsar.instance.resetForTest();
  });

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
}
