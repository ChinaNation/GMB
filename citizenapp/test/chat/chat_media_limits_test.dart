import 'package:flutter_test/flutter_test.dart';
import 'package:citizenapp/chat/chat_media_limits.dart';
import 'package:citizenapp/chat/chat_models.dart';

void main() {
  const mib = 1024 * 1024;

  // 每个用例后复位到 fail-closed 的自由档，避免用例间的档位串扰。
  tearDown(() => ChatMediaLimits.applyMembershipLevel(null));

  test('单个文件上限按会员档（ADR-036）:自由 10MB、民主 100MB、薪火 5GB', () {
    expect(ChatMediaLimits.maxBytesForLevel('freedom'), 10 * mib);
    expect(ChatMediaLimits.maxBytesForLevel('democracy'), 100 * mib);
    expect(ChatMediaLimits.maxBytesForLevel('spark'), 5120 * mib);
    // 未知 / 无订阅 fail-closed 到自由档。
    expect(ChatMediaLimits.maxBytesForLevel(null), 10 * mib);
    expect(ChatMediaLimits.maxBytesForLevel('voting'), 10 * mib);
    expect(ChatMediaLimits.absoluteMaxBytes, 5120 * mib);
  });

  test('applyMembershipLevel 设置当前档上限;默认 fail-closed 自由档', () {
    ChatMediaLimits.applyMembershipLevel(null);
    expect(ChatMediaLimits.currentMaxBytes, 10 * mib);
    ChatMediaLimits.applyMembershipLevel('democracy');
    expect(ChatMediaLimits.currentMaxBytes, 100 * mib);
    ChatMediaLimits.applyMembershipLevel('spark');
    expect(ChatMediaLimits.currentMaxBytes, 5120 * mib);
  });

  test('forKind:媒体取当前档上限;text/sticker 无字节返回 0', () {
    ChatMediaLimits.applyMembershipLevel('democracy');
    expect(ChatMediaLimits.forKind(ChatMessageKind.image), 100 * mib);
    expect(ChatMediaLimits.forKind(ChatMessageKind.video), 100 * mib);
    expect(ChatMediaLimits.forKind(ChatMessageKind.file), 100 * mib);
    expect(ChatMediaLimits.forKind(ChatMessageKind.text), 0);
    expect(ChatMediaLimits.forKind(ChatMessageKind.sticker), 0);
  });

  test('forMime:任何媒体 mime 取当前档上限', () {
    ChatMediaLimits.applyMembershipLevel('spark');
    expect(ChatMediaLimits.forMime('image/png'), 5120 * mib);
    expect(ChatMediaLimits.forMime('video/quicktime'), 5120 * mib);
    expect(ChatMediaLimits.forMime('application/pdf'), 5120 * mib);
  });

  test('exceedsForKind:精确边界(按当前档)', () {
    ChatMediaLimits.applyMembershipLevel('freedom');
    expect(
      ChatMediaLimits.exceedsForKind(ChatMessageKind.image, 10 * mib),
      isFalse,
    );
    expect(
      ChatMediaLimits.exceedsForKind(ChatMessageKind.image, 10 * mib + 1),
      isTrue,
    );
    ChatMediaLimits.applyMembershipLevel('spark');
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
