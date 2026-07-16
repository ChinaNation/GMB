import 'dart:convert';
import 'dart:io';
import 'dart:math';

import 'package:citizenapp/chat/chat_media_limits.dart';
import 'package:citizenapp/chat/chat_models.dart';
import 'package:citizenapp/chat/chat_payload.dart';
import 'package:citizenapp/chat/media/media_relay_crypto.dart';
import 'package:flutter_test/flutter_test.dart';

void main() {
  group('ChatMediaLimits.needsRelay 分界(固定 100MB)', () {
    test('≤100MB 不走中转、>100MB 走中转', () {
      expect(ChatMediaLimits.needsRelay(ChatMediaLimits.relayThresholdBytes),
          isFalse);
      expect(
          ChatMediaLimits.needsRelay(ChatMediaLimits.relayThresholdBytes + 1),
          isTrue);
      expect(ChatMediaLimits.relayThresholdBytes,
          ChatMediaLimits.democracyMaxBytes);
    });
  });

  group('MediaRelayCrypto 流式 AES-256-GCM', () {
    test('单块加解密 round-trip', () async {
      final key = MediaRelayCrypto.newContentKey();
      final plain = List<int>.generate(5000, (i) => i % 256);
      final frame = await MediaRelayCrypto.encryptChunk(key, 3, plain);
      expect(frame.length, plain.length + MediaRelayCrypto.macBytes);
      final back = await MediaRelayCrypto.decryptChunk(key, 3, frame);
      expect(back, plain);
    });

    test('密文被篡改 → 解密失败(GCM tag 挡住)', () async {
      final key = MediaRelayCrypto.newContentKey();
      final frame = await MediaRelayCrypto.encryptChunk(key, 0, [1, 2, 3, 4]);
      final tampered = List<int>.of(frame)..[0] ^= 0xFF;
      await expectLater(
        MediaRelayCrypto.decryptChunk(key, 0, tampered),
        throwsA(anything),
      );
    });

    test('错误密钥 → 解密失败', () async {
      final key = MediaRelayCrypto.newContentKey();
      final wrong = MediaRelayCrypto.newContentKey();
      final frame = await MediaRelayCrypto.encryptChunk(key, 0, [9, 8, 7]);
      await expectLater(
        MediaRelayCrypto.decryptChunk(wrong, 0, frame),
        throwsA(anything),
      );
    });

    test('多块文件流式加解密 round-trip(跨块边界)', () async {
      final dir = await Directory.systemTemp.createTemp('relay_crypto_');
      try {
        final key = MediaRelayCrypto.newContentKey();
        final source = File('${dir.path}/plain.bin');
        // 2.5 块:验证定长分块 + 末块处理。
        final random = Random(42);
        final bytes = List<int>.generate(
          (2.5 * 4096).toInt(),
          (_) => random.nextInt(256),
        );
        await source.writeAsBytes(bytes);
        final enc = '${dir.path}/enc.bin';
        final dec = '${dir.path}/dec.bin';

        final encSize = await MediaRelayCrypto.encryptFile(
          sourcePath: source.path,
          destPath: enc,
          key: key,
          chunkSize: 4096,
        );
        expect(encSize, await File(enc).length());
        expect(encSize, greaterThan(bytes.length)); // 含帧头 + tag

        await MediaRelayCrypto.decryptFile(
          sourcePath: enc,
          destPath: dec,
          key: key,
        );
        expect(await File(dec).readAsBytes(), bytes);
      } finally {
        await dir.delete(recursive: true);
      }
    });
  });

  group('ChatPayload relay 字段', () {
    test('relay 媒体编解码保真 + isRelayMedia', () {
      final content = ChatContent.media(
        kind: ChatMessageKind.video,
        attachmentId: 'att-1',
        fileName: 'big.mp4',
        mime: 'video/mp4',
        byteSize: 500 * 1024 * 1024,
        relayObjectKey: 'chat-relay/abc123',
        contentKeyB64: base64Encode(List<int>.filled(32, 7)),
        chunkSize: 1048576,
        encSize: 500 * 1024 * 1024 + 8192,
      );
      expect(content.isRelayMedia, isTrue);

      final decoded = ChatPayloadCodec.decode(ChatPayloadCodec.encode(content));
      expect(decoded.isRelayMedia, isTrue);
      expect(decoded.relayObjectKey, 'chat-relay/abc123');
      expect(decoded.contentKeyB64, content.contentKeyB64);
      expect(decoded.chunkSize, 1048576);
      expect(decoded.encSize, content.encSize);
      expect(decoded.byteSize, content.byteSize);
    });

    test('普通(WebRTC)媒体无 relay 字段', () {
      final content = ChatContent.media(
        kind: ChatMessageKind.image,
        attachmentId: 'att-2',
        fileName: 'p.jpg',
        mime: 'image/jpeg',
        byteSize: 2048,
      );
      expect(content.isRelayMedia, isFalse);
      final json = ChatPayloadCodec.encode(content);
      expect(json.contains('relay_object_key'), isFalse);
      final decoded = ChatPayloadCodec.decode(json);
      expect(decoded.isRelayMedia, isFalse);
      expect(decoded.relayObjectKey, isNull);
    });
  });
}
