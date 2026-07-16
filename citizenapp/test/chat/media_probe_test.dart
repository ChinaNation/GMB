import 'dart:typed_data';

import 'package:flutter_test/flutter_test.dart';
import 'package:image/image.dart' as img;
import 'package:citizenapp/chat/chat_models.dart';
import 'package:citizenapp/chat/media/media_probe.dart';

Uint8List _tinyPng() {
  final image = img.Image(width: 8, height: 8);
  img.fill(image, color: img.ColorRgb8(120, 80, 200));
  return img.encodePng(image);
}

void main() {
  test('encodeBlurhash 由小图字节产出非空 hash', () {
    final hash = MediaProbe.encodeBlurhash(_tinyPng());
    expect(hash, isNotNull);
    expect(hash!.isNotEmpty, isTrue);
  });

  test('encodeBlurhash 坏字节返回 null 且不抛', () {
    expect(MediaProbe.encodeBlurhash(const [1, 2, 3]), isNull);
  });

  test('probe(image) 装配宽高与 blurhash(缩略字节走注入)', () async {
    final png = _tinyPng();
    final probe = MediaProbe(
      imageSize: (_) => (width: 800, height: 600),
      imageThumbBytes: (_) async => png,
    );
    final result =
        await probe.probe(path: '/p.jpg', kind: ChatMessageKind.image);
    expect(result.width, 800);
    expect(result.height, 600);
    expect(result.durationMs, isNull);
    expect(result.blurhash, isNotNull);
  });

  test('probe(video) 装配宽高/时长/blurhash', () async {
    final png = _tinyPng();
    final probe = MediaProbe(
      videoInfo: (_) async => (width: 1920, height: 1080, durationMs: 4200),
      videoThumbBytes: (_) async => png,
    );
    final result =
        await probe.probe(path: '/v.mp4', kind: ChatMessageKind.video);
    expect(result.width, 1920);
    expect(result.height, 1080);
    expect(result.durationMs, 4200);
    expect(result.blurhash, isNotNull);
  });

  test('probe(file) 不探测,字段留空', () async {
    final result = await MediaProbe().probe(
      path: '/x.pdf',
      kind: ChatMessageKind.file,
    );
    expect(result.width, isNull);
    expect(result.height, isNull);
    expect(result.blurhash, isNull);
  });

  test('probe:缩略图为空时 blurhash 降级为 null,宽高仍在', () async {
    final probe = MediaProbe(
      imageSize: (_) => (width: 800, height: 600),
      imageThumbBytes: (_) async => null,
    );
    final result =
        await probe.probe(path: '/p.jpg', kind: ChatMessageKind.image);
    expect(result.width, 800);
    expect(result.height, 600);
    expect(result.blurhash, isNull);
  });

  test('encodeBlurhash 对纵向缩略图也降到 ≤64 并产出 hash', () {
    final tall = img.Image(width: 40, height: 320);
    img.fill(tall, color: img.ColorRgb8(10, 200, 90));
    final hash = MediaProbe.encodeBlurhash(img.encodePng(tall));
    expect(hash, isNotNull);
    expect(hash!.isNotEmpty, isTrue);
  });

  test('探测抛错时不阻断,返回空结果', () async {
    final probe = MediaProbe(
      imageSize: (_) => throw StateError('boom'),
      imageThumbBytes: (_) async => throw StateError('boom'),
    );
    final result =
        await probe.probe(path: '/p.jpg', kind: ChatMessageKind.image);
    expect(result.width, isNull);
    expect(result.blurhash, isNull);
  });
}
