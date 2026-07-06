import 'package:flutter_chat_core/flutter_chat_core.dart';
import 'package:flutter_test/flutter_test.dart';
import 'package:citizenapp/im/im_chat_ui_adapter.dart';
import 'package:citizenapp/im/im_session_models.dart';
import 'package:citizenapp/im/proto/im_envelope.pb.dart' as pb;
import 'package:citizenapp/im/storage/im_isar_store.dart';

void main() {
  test('IM stored messages map to flutter_chat_core text messages', () {
    final outgoing = imStoredMessageToChatMessage(
      const ImStoredMessage(
        envelopeId: 'env-out',
        conversationId: 'dm:alice:bob',
        direction: 'outgoing',
        senderChatAccount: 'alice-wallet',
        recipientChatAccount: 'bob-wallet',
        messageKind: ImMessageKind.text,
        deliveryState: ImMessageDeliveryState.sent,
        createdAtMillis: 1000,
        plaintext: 'hello',
      ),
      currentUserId: 'alice-wallet',
    ) as TextMessage;

    expect(outgoing.id, 'env-out');
    expect(outgoing.authorId, 'alice-wallet');
    expect(outgoing.text, 'hello');
    expect(outgoing.status, MessageStatus.sent);
    expect(outgoing.sentAt, isNotNull);
    expect(outgoing.metadata?['is_mine'], isTrue);

    final incoming = imStoredMessageToChatMessage(
      const ImStoredMessage(
        envelopeId: 'env-in',
        conversationId: 'dm:alice:bob',
        direction: 'incoming',
        senderChatAccount: 'bob-wallet',
        recipientChatAccount: 'alice-wallet',
        messageKind: ImMessageKind.text,
        deliveryState: ImMessageDeliveryState.receivedByDevice,
        createdAtMillis: 2000,
        plaintext: 'hi',
      ),
      currentUserId: 'alice-wallet',
    ) as TextMessage;

    expect(incoming.authorId, 'bob-wallet');
    expect(incoming.status, MessageStatus.delivered);
    expect(incoming.deliveredAt, isNotNull);
    expect(incoming.metadata?['is_mine'], isFalse);
  });

  test('attachment IM message shows safe visible filename placeholder', () {
    final attachment = imStoredMessageToChatMessage(
      const ImStoredMessage(
        envelopeId: 'env-attachment',
        conversationId: 'dm:alice:bob',
        direction: 'outgoing',
        senderChatAccount: 'alice-wallet',
        recipientChatAccount: 'bob-wallet',
        messageKind: ImMessageKind.attachment,
        deliveryState: ImMessageDeliveryState.failed,
        createdAtMillis: 3000,
        plaintext: '{"type":"gmb_im_attachment_v1","file_name":"photo.txt"}',
      ),
      currentUserId: 'alice-wallet',
    ) as TextMessage;

    expect(attachment.text, '[附件] photo.txt');
    expect(attachment.metadata?['message_kind'], 'attachment');
    expect(
      attachment.metadata?['attachment_control_plaintext'],
      contains('gmb_im_attachment_v1'),
    );
    expect(attachment.status, MessageStatus.error);
    expect(attachment.failedAt, isNotNull);
  });

  test('generated protobuf export remains available to adapter callers', () {
    expect(
      pb.ImMlsWireMessageKind.IM_MLS_WIRE_MESSAGE_KIND_APPLICATION.name,
      contains('APPLICATION'),
    );
  });
}
