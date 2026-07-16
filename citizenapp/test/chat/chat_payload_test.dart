import 'package:flutter_test/flutter_test.dart';
import 'package:citizenapp/chat/chat_models.dart';
import 'package:citizenapp/chat/chat_payload.dart';

void main() {
  test('text content round-trips through the codec', () {
    final encoded = ChatPayloadCodec.encode(ChatContent.text('你好'));
    final decoded = ChatPayloadCodec.decode(encoded);
    expect(decoded.kind, ChatMessageKind.text);
    expect(decoded.text, '你好');
    expect(decoded.summary, '你好');
  });

  test('image/video/file media round-trip with control metadata', () {
    for (final kind in const [
      ChatMessageKind.image,
      ChatMessageKind.video,
      ChatMessageKind.file,
    ]) {
      final encoded = ChatPayloadCodec.encode(
        ChatContent.media(
          kind: kind,
          attachmentId: 'att-1',
          fileName: 'IMG.jpg',
          mime: 'image/jpeg',
          byteSize: 2048,
          width: 1080,
          height: 1920,
          durationMs: kind == ChatMessageKind.video ? 4200 : null,
          blurhash: 'L6Pj0',
        ),
      );
      final decoded = ChatPayloadCodec.decode(encoded);
      expect(decoded.kind, kind);
      expect(decoded.isMedia, isTrue);
      expect(decoded.attachmentId, 'att-1');
      expect(decoded.fileName, 'IMG.jpg');
      expect(decoded.mime, 'image/jpeg');
      expect(decoded.byteSize, 2048);
      expect(decoded.width, 1080);
      expect(decoded.height, 1920);
      expect(decoded.blurhash, 'L6Pj0');
      expect(decoded.durationMs, kind == ChatMessageKind.video ? 4200 : null);
    }
  });

  test('sticker carries only ids, no bytes/metadata', () {
    final encoded = ChatPayloadCodec.encode(
      ChatContent.sticker(packId: 'fluent3d', stickerId: 'grinning_face'),
    );
    final decoded = ChatPayloadCodec.decode(encoded);
    expect(decoded.kind, ChatMessageKind.sticker);
    expect(decoded.packId, 'fluent3d');
    expect(decoded.stickerId, 'grinning_face');
    expect(decoded.summary, '[贴纸]');
    expect(decoded.isMedia, isFalse);
  });

  test('plain text that looks like JSON is NOT misclassified as media', () {
    // 早期"能否 jsonDecode"启发式会把这类文本误判为附件;显式 kind 修掉该隐患。
    const jsonyText = '{"type":"gmb_chat_attachment_v2","file_name":"x"}';
    final decoded = ChatPayloadCodec.decode(jsonyText);
    expect(decoded.kind, ChatMessageKind.text);
    expect(decoded.text, jsonyText);
  });

  test('garbage / empty decode to plain text without throwing', () {
    expect(ChatPayloadCodec.decode('').kind, ChatMessageKind.text);
    final broken = ChatPayloadCodec.decode('not json {{{');
    expect(broken.kind, ChatMessageKind.text);
    expect(broken.text, 'not json {{{');
  });

  test('media summaries are typed placeholders', () {
    String summaryOf(ChatContent c) =>
        ChatPayloadCodec.decode(ChatPayloadCodec.encode(c)).summary;
    expect(
      summaryOf(ChatContent.media(
        kind: ChatMessageKind.image,
        attachmentId: 'a',
        fileName: 'p.png',
        mime: 'image/png',
        byteSize: 1,
      )),
      '[图片]',
    );
    expect(
      summaryOf(ChatContent.media(
        kind: ChatMessageKind.video,
        attachmentId: 'a',
        fileName: 'v.mp4',
        mime: 'video/mp4',
        byteSize: 1,
      )),
      '[视频]',
    );
    expect(
      summaryOf(ChatContent.media(
        kind: ChatMessageKind.file,
        attachmentId: 'a',
        fileName: 'doc.pdf',
        mime: 'application/pdf',
        byteSize: 1,
      )),
      '[文件] doc.pdf',
    );
  });
}
