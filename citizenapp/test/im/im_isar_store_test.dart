import 'package:flutter_test/flutter_test.dart';
import 'package:citizenapp/im/crypto/im_mls_session.dart';
import 'package:citizenapp/im/im_session_models.dart';
import 'package:citizenapp/im/storage/im_isar_store.dart';
import 'package:citizenapp/isar/wallet_isar.dart';

void main() {
  setUp(() async {
    await WalletIsar.instance.resetForTest();
  });

  tearDown(() async {
    await WalletIsar.instance.resetForTest();
  });

  test('Isar store persists outgoing, pending, and incoming IM records',
      () async {
    final store = ImIsarStore();
    final envelope = const ImMlsWireMessage(
      wireBytes: [0x68, 0x69],
      cipherSuite: 'MLS_128',
      conversationId: 'conv-store',
      messageKind: ImMlsMessageKind.application,
    ).toEnvelope(
      envelopeId: 'env-store',
      senderChatAccount: 'alice-wallet',
      recipientChatAccount: 'bob-wallet',
      senderDeviceId: 'alice-phone',
      createdAtMillis: 10,
      ttlMillis: 60000,
    );

    await store.saveOutgoingEnvelope(
      envelope: envelope,
      envelopeBytes: envelope.writeToBuffer(),
      messageKind: ImMessageKind.text,
      deliveryState: ImMessageDeliveryState.queued,
      plaintext: 'hi',
    );
    await store.markOutgoingDelivery(
      envelopeId: 'env-store',
      state: ImMessageDeliveryState.sent,
    );

    final outgoing = await store.readMessages('conv-store');
    expect(outgoing.single.deliveryState, ImMessageDeliveryState.sent);
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
      messageKind: ImMessageKind.text,
      plaintext: 'hi back',
    );
    final conversations = await store.readConversationPreviews();
    expect(conversations.single.unreadCount, 1);
    expect(conversations.single.lastMessage, 'hi back');
  });
}
