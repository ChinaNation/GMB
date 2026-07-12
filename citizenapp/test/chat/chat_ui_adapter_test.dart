import 'package:flutter_chat_core/flutter_chat_core.dart';
import 'package:flutter_test/flutter_test.dart';
import 'package:citizenapp/chat/chat_ui_adapter.dart';
import 'package:citizenapp/chat/chat_models.dart';
import 'package:citizenapp/chat/proto/chat_envelope.pb.dart' as pb;
import 'package:citizenapp/chat/storage/chat_store.dart';

void main() {
  test('Chat stored messages map to flutter_chat_core text messages', () {
    final outgoing = storedMessageToChatMessage(
      const ChatStoredMessage(
        envelopeId: 'env-out',
        conversationId: 'dm:alice:bob',
        direction: 'outgoing',
        senderAccount: 'alice-wallet',
        recipientAccount: 'bob-wallet',
        messageKind: ChatMessageKind.text,
        deliveryState: ChatMessageDeliveryState.sent,
        createdAtMillis: 1000,
        plaintext: 'hello',
      ),
      ownerAccount: 'alice-wallet',
    ) as TextMessage;

    expect(outgoing.id, 'env-out');
    expect(outgoing.authorId, 'alice-wallet');
    expect(outgoing.text, 'hello');
    expect(outgoing.status, MessageStatus.sent);
    expect(outgoing.sentAt, isNotNull);
    expect(outgoing.metadata?['is_mine'], isTrue);

    final incoming = storedMessageToChatMessage(
      const ChatStoredMessage(
        envelopeId: 'env-in',
        conversationId: 'dm:alice:bob',
        direction: 'incoming',
        senderAccount: 'bob-wallet',
        recipientAccount: 'alice-wallet',
        messageKind: ChatMessageKind.text,
        deliveryState: ChatMessageDeliveryState.receivedByDevice,
        createdAtMillis: 2000,
        plaintext: 'hi',
      ),
      ownerAccount: 'alice-wallet',
    ) as TextMessage;

    expect(incoming.authorId, 'bob-wallet');
    expect(incoming.status, MessageStatus.delivered);
    expect(incoming.deliveredAt, isNotNull);
    expect(incoming.metadata?['is_mine'], isFalse);
  });

  test('attachment Chat message shows safe visible filename placeholder', () {
    final attachment = storedMessageToChatMessage(
      const ChatStoredMessage(
        envelopeId: 'env-attachment',
        conversationId: 'dm:alice:bob',
        direction: 'outgoing',
        senderAccount: 'alice-wallet',
        recipientAccount: 'bob-wallet',
        messageKind: ChatMessageKind.attachment,
        deliveryState: ChatMessageDeliveryState.failed,
        createdAtMillis: 3000,
        plaintext: '{"type":"gmb_chat_attachment_v2","file_name":"photo.txt"}',
      ),
      ownerAccount: 'alice-wallet',
    ) as TextMessage;

    expect(attachment.text, '[附件] photo.txt');
    expect(attachment.metadata?['message_kind'], 'attachment');
    expect(
      attachment.metadata?['attachment_control_plaintext'],
      contains('gmb_chat_attachment_v2'),
    );
    expect(attachment.status, MessageStatus.error);
    expect(attachment.failedAt, isNotNull);
  });

  test('generated protobuf export remains available to adapter callers', () {
    expect(
      pb.MlsWireMessageKind.MLS_WIRE_MESSAGE_KIND_APPLICATION.name,
      contains('APPLICATION'),
    );
  });
}
