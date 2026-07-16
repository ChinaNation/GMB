import 'package:flutter_chat_core/flutter_chat_core.dart';
import 'package:flutter_test/flutter_test.dart';
import 'package:citizenapp/chat/chat_ui_adapter.dart';
import 'package:citizenapp/chat/chat_media_limits.dart';
import 'package:citizenapp/chat/chat_models.dart';
import 'package:citizenapp/chat/chat_payload.dart';
import 'package:citizenapp/chat/proto/chat_envelope.pb.dart' as pb;
import 'package:citizenapp/chat/storage/chat_store.dart';

ChatStoredMessage _stored({
  required String envelopeId,
  required ChatMessageKind kind,
  required String plaintext,
  String direction = 'outgoing',
  ChatMessageDeliveryState state = ChatMessageDeliveryState.sent,
}) {
  final outgoing = direction == 'outgoing';
  return ChatStoredMessage(
    envelopeId: envelopeId,
    conversationId: 'dm:alice:bob',
    direction: direction,
    senderAccount: outgoing ? 'alice-wallet' : 'bob-wallet',
    recipientAccount: outgoing ? 'bob-wallet' : 'alice-wallet',
    messageKind: kind,
    deliveryState: state,
    createdAtMillis: 1000,
    plaintext: plaintext,
  );
}

void main() {
  test('text messages map to flutter_chat_core text messages', () {
    final outgoing = storedMessageToChatMessage(
      _stored(
        envelopeId: 'env-out',
        kind: ChatMessageKind.text,
        plaintext: ChatPayloadCodec.encode(ChatContent.text('hello')),
      ),
      ownerAccount: 'alice-wallet',
    ) as TextMessage;
    expect(outgoing.text, 'hello');
    expect(outgoing.status, MessageStatus.sent);
    expect(outgoing.sentAt, isNotNull);
    expect(outgoing.metadata?['is_mine'], isTrue);

    final incoming = storedMessageToChatMessage(
      _stored(
        envelopeId: 'env-in',
        kind: ChatMessageKind.text,
        direction: 'incoming',
        state: ChatMessageDeliveryState.receivedByDevice,
        plaintext: ChatPayloadCodec.encode(ChatContent.text('hi')),
      ),
      ownerAccount: 'alice-wallet',
    ) as TextMessage;
    expect(incoming.text, 'hi');
    expect(incoming.status, MessageStatus.delivered);
    expect(incoming.metadata?['is_mine'], isFalse);
  });

  test('image maps to ImageMessage with resolved local path and dimensions',
      () {
    final payload = ChatPayloadCodec.encode(
      ChatContent.media(
        kind: ChatMessageKind.image,
        attachmentId: 'att-1',
        fileName: 'p.jpg',
        mime: 'image/jpeg',
        byteSize: 2048,
        width: 800,
        height: 600,
        blurhash: 'L6',
      ),
    );
    final msg = storedMessageToChatMessage(
      _stored(
        envelopeId: 'env-img',
        kind: ChatMessageKind.image,
        plaintext: payload,
      ),
      ownerAccount: 'alice-wallet',
      resolveLocalMediaPath: (c) =>
          c.attachmentId == 'att-1' ? '/cache/p.jpg' : null,
    ) as ImageMessage;
    expect(msg.source, '/cache/p.jpg');
    expect(msg.width, 800);
    expect(msg.height, 600);
    expect(msg.blurhash, 'L6');
    expect(msg.metadata?['message_kind'], 'image');
    expect(msg.metadata?['file_name'], 'p.jpg'); // 全屏查看/存相册用
    expect(
      msg.metadata?['attachment_control_plaintext'],
      contains('gmb.chat.msg'),
    );
  });

  test('image without cached bytes yields empty source for the placeholder',
      () {
    final payload = ChatPayloadCodec.encode(
      ChatContent.media(
        kind: ChatMessageKind.image,
        attachmentId: 'att-2',
        fileName: 'p.jpg',
        mime: 'image/jpeg',
        byteSize: 2048,
      ),
    );
    final msg = storedMessageToChatMessage(
      _stored(
        envelopeId: 'env-img2',
        kind: ChatMessageKind.image,
        plaintext: payload,
      ),
      ownerAccount: 'alice-wallet',
      resolveLocalMediaPath: (_) => null,
    ) as ImageMessage;
    expect(msg.source, '');
  });

  test('video maps to VideoMessage with dims and tap-to-save control metadata',
      () {
    final payload = ChatPayloadCodec.encode(
      ChatContent.media(
        kind: ChatMessageKind.video,
        attachmentId: 'att-v',
        fileName: 'clip.mp4',
        mime: 'video/mp4',
        byteSize: 8192,
        width: 1920,
        height: 1080,
        durationMs: 4200,
        blurhash: 'L6Pj0',
      ),
    );
    final msg = storedMessageToChatMessage(
      _stored(
        envelopeId: 'env-vid',
        kind: ChatMessageKind.video,
        plaintext: payload,
      ),
      ownerAccount: 'alice-wallet',
      resolveLocalMediaPath: (_) => '/cache/clip.mp4',
    ) as VideoMessage;
    expect(msg.name, 'clip.mp4');
    expect(msg.width, 1920);
    expect(msg.height, 1080);
    expect(msg.size, 8192);
    expect(msg.source, '/cache/clip.mp4');
    expect(msg.metadata?['message_kind'], 'video');
    // _buildVideoMessage 读这些键:blurhash 出封面占位、file_name 供播放页存相册、
    // 控制载荷供另存。
    expect(msg.metadata?['blurhash'], 'L6Pj0');
    expect(msg.metadata?['file_name'], 'clip.mp4');
    expect(
      msg.metadata?['attachment_control_plaintext'],
      contains('gmb.chat.msg'),
    );
  });

  test('file maps to FileMessage with name/size/mime', () {
    final payload = ChatPayloadCodec.encode(
      ChatContent.media(
        kind: ChatMessageKind.file,
        attachmentId: 'att-3',
        fileName: 'doc.pdf',
        mime: 'application/pdf',
        byteSize: 4096,
      ),
    );
    final msg = storedMessageToChatMessage(
      _stored(
        envelopeId: 'env-file',
        kind: ChatMessageKind.file,
        plaintext: payload,
      ),
      ownerAccount: 'alice-wallet',
      resolveLocalMediaPath: (_) => '/cache/doc.pdf',
    ) as FileMessage;
    expect(msg.name, 'doc.pdf');
    expect(msg.size, 4096);
    expect(msg.mimeType, 'application/pdf');
    expect(msg.source, '/cache/doc.pdf');
  });

  test('sticker maps to a placeholder text carrying pack/sticker ids', () {
    final payload = ChatPayloadCodec.encode(
      ChatContent.sticker(packId: 'fluent3d', stickerId: '1f600'),
    );
    final msg = storedMessageToChatMessage(
      _stored(
        envelopeId: 'env-st',
        kind: ChatMessageKind.sticker,
        plaintext: payload,
      ),
      ownerAccount: 'alice-wallet',
    ) as TextMessage;
    expect(msg.text, '[贴纸]');
    expect(msg.metadata?['pack_id'], 'fluent3d');
    expect(msg.metadata?['sticker_id'], '1f600');
  });

  test('门④:声明超限的媒体渲染为拒收占位,且不解析本机路径', () {
    final payload = ChatPayloadCodec.encode(
      ChatContent.media(
        kind: ChatMessageKind.image,
        attachmentId: 'att-big',
        fileName: 'big.jpg',
        mime: 'image/jpeg',
        byteSize: ChatMediaLimits.imageMaxBytes + 1,
      ),
    );
    var resolverCalled = false;
    final msg = storedMessageToChatMessage(
      _stored(
        envelopeId: 'env-big',
        kind: ChatMessageKind.image,
        plaintext: payload,
      ),
      ownerAccount: 'alice-wallet',
      resolveLocalMediaPath: (_) {
        resolverCalled = true;
        return '/cache/big.jpg';
      },
    ) as TextMessage;
    expect(msg.metadata?['oversized'], isTrue);
    expect(msg.text, contains('已拒收'));
    // 拒收媒体不解析路径,不诱导用户去拉取。
    expect(resolverCalled, isFalse);
  });

  test('generated protobuf export remains available to adapter callers', () {
    expect(
      pb.MlsWireMessageKind.MLS_WIRE_MESSAGE_KIND_APPLICATION.name,
      contains('APPLICATION'),
    );
  });
}
