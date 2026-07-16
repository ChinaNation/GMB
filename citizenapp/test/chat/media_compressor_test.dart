import 'package:flutter_test/flutter_test.dart';
import 'package:citizenapp/chat/chat_media_limits.dart';
import 'package:citizenapp/chat/chat_models.dart';
import 'package:citizenapp/chat/media/media_compressor.dart';

void main() {
  test('正常大小直通,不触发压缩', () async {
    final compressor = MediaCompressor(
      sizeOf: (_) async => 5,
      compressImage: (_, __) async => throw StateError('不应压缩'),
    );
    final out = await compressor.ensureWithinLimit(
      path: '/p.jpg',
      kind: ChatMessageKind.image,
    );
    expect(out, '/p.jpg');
  });

  test('图片超限:压后达标 → 返回压缩后路径', () async {
    final sizes = {
      '/big.jpg': ChatMediaLimits.imageMaxBytes + 1,
      '/small.jpg': 10,
    };
    final compressor = MediaCompressor(
      sizeOf: (p) async => sizes[p]!,
      compressImage: (_, __) async => '/small.jpg',
    );
    final out = await compressor.ensureWithinLimit(
      path: '/big.jpg',
      kind: ChatMessageKind.image,
    );
    expect(out, '/small.jpg');
  });

  test('图片超限:压后仍超 → 抛 ChatMediaTooLargeException', () async {
    final compressor = MediaCompressor(
      sizeOf: (_) async => ChatMediaLimits.imageMaxBytes + 1,
      compressImage: (_, __) async => '/still-big.jpg',
    );
    await expectLater(
      compressor.ensureWithinLimit(
        path: '/big.jpg',
        kind: ChatMessageKind.image,
      ),
      throwsA(isA<ChatMediaTooLargeException>()),
    );
  });

  test('图片超限:压缩失败(null)→ 抛', () async {
    final compressor = MediaCompressor(
      sizeOf: (_) async => ChatMediaLimits.imageMaxBytes + 1,
      compressImage: (_, __) async => null,
    );
    await expectLater(
      compressor.ensureWithinLimit(
        path: '/big.jpg',
        kind: ChatMessageKind.image,
      ),
      throwsA(isA<ChatMediaTooLargeException>()),
    );
  });

  test('视频超限:不转码,直接抛,且不触发压缩', () async {
    var compressCalled = false;
    final compressor = MediaCompressor(
      sizeOf: (_) async => ChatMediaLimits.videoMaxBytes + 1,
      compressImage: (_, __) async {
        compressCalled = true;
        return null;
      },
    );
    await expectLater(
      compressor.ensureWithinLimit(
        path: '/big.mp4',
        kind: ChatMessageKind.video,
      ),
      throwsA(isA<ChatMediaTooLargeException>()),
    );
    expect(compressCalled, isFalse);
  });
}
