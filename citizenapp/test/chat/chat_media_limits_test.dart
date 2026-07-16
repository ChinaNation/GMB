import 'package:flutter_test/flutter_test.dart';
import 'package:citizenapp/chat/chat_media_limits.dart';
import 'package:citizenapp/chat/chat_models.dart';

void main() {
  const mib = 1024 * 1024;

  test('限额定稿:图片 100MB、视频/文件 5GB', () {
    expect(ChatMediaLimits.imageMaxBytes, 100 * mib);
    expect(ChatMediaLimits.videoMaxBytes, 5120 * mib);
    expect(ChatMediaLimits.fileMaxBytes, 5120 * mib);
    expect(ChatMediaLimits.absoluteMaxBytes, 5120 * mib);
  });

  test('forKind:按类型返回上限;text/sticker 无字节返回 0', () {
    expect(ChatMediaLimits.forKind(ChatMessageKind.image), 100 * mib);
    expect(ChatMediaLimits.forKind(ChatMessageKind.video), 5120 * mib);
    expect(ChatMediaLimits.forKind(ChatMessageKind.file), 5120 * mib);
    expect(ChatMediaLimits.forKind(ChatMessageKind.text), 0);
    expect(ChatMediaLimits.forKind(ChatMessageKind.sticker), 0);
  });

  test('forMime:按前缀取上限,未知类型按文件上限', () {
    expect(ChatMediaLimits.forMime('image/png'), 100 * mib);
    expect(ChatMediaLimits.forMime('video/quicktime'), 5120 * mib);
    expect(ChatMediaLimits.forMime('application/pdf'), 5120 * mib);
    expect(ChatMediaLimits.forMime('application/octet-stream'), 5120 * mib);
  });

  test('exceedsForKind:精确边界', () {
    expect(
      ChatMediaLimits.exceedsForKind(ChatMessageKind.image, 100 * mib),
      isFalse,
    );
    expect(
      ChatMediaLimits.exceedsForKind(ChatMessageKind.image, 100 * mib + 1),
      isTrue,
    );
    expect(
      ChatMediaLimits.exceedsForKind(ChatMessageKind.video, 5120 * mib),
      isFalse,
    );
    expect(
      ChatMediaLimits.exceedsForKind(ChatMessageKind.video, 5120 * mib + 1),
      isTrue,
    );
    // text/sticker 无字节,任何大小都视为不超限(它们不携带媒体字节)。
    expect(
        ChatMediaLimits.exceedsForKind(ChatMessageKind.text, 1 << 40), isFalse);
    expect(
      ChatMediaLimits.exceedsForKind(ChatMessageKind.sticker, 1 << 40),
      isFalse,
    );
  });
}
