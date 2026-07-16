import 'package:flutter_test/flutter_test.dart';
import 'package:citizenapp/chat/media/media_resend.dart';
import 'package:citizenapp/chat/storage/chat_store.dart';

ChatPendingMedia _pending(String id) => ChatPendingMedia(
      attachmentId: id,
      recipientAccount: 'bob',
      conversationId: 'conv',
      fileName: '$id.jpg',
      contentType: 'image/jpeg',
      byteSize: 10,
    );

void main() {
  test('存在缓存 + 发送成功 → 删待投递行,不残留在途', () async {
    final inFlight = <String>{};
    final sent = <String>[];
    final deleted = <String>[];
    await MediaResend.run(
      pending: [_pending('att-1')],
      inFlight: inFlight,
      resolveCachePath: (m) => '/cache/${m.attachmentId}',
      cacheFileExists: (_) async => true,
      sendBytes: (m, path) async => sent.add(m.attachmentId),
      deletePending: (media) async => deleted.add(media.attachmentId),
    );
    expect(sent, ['att-1']);
    expect(deleted, ['att-1']);
    expect(inFlight, isEmpty);
  });

  test('缓存副本已丢 → 清孤儿删行,不发送', () async {
    final sent = <String>[];
    final deleted = <String>[];
    await MediaResend.run(
      pending: [_pending('att-1')],
      inFlight: <String>{},
      resolveCachePath: (m) => '/cache/${m.attachmentId}',
      cacheFileExists: (_) async => false,
      sendBytes: (m, path) async => sent.add(m.attachmentId),
      deletePending: (media) async => deleted.add(media.attachmentId),
    );
    expect(sent, isEmpty);
    expect(deleted, ['att-1']);
  });

  test('发送失败 → 保留待投递行(不删),不残留在途', () async {
    final inFlight = <String>{};
    final deleted = <String>[];
    await MediaResend.run(
      pending: [_pending('att-1')],
      inFlight: inFlight,
      resolveCachePath: (m) => '/cache/${m.attachmentId}',
      cacheFileExists: (_) async => true,
      sendBytes: (m, path) async => throw Exception('offline'),
      deletePending: (media) async => deleted.add(media.attachmentId),
    );
    expect(deleted, isEmpty); // 保留待下次 peer_ready
    expect(inFlight, isEmpty);
  });

  test('在途中的媒体被跳过(去重):不重发、不删、不动在途集合', () async {
    // 在途去重按 (attachmentId, recipient) 复合键。
    final inFlight = {MediaResend.inFlightKey('att-1', 'bob')};
    final sent = <String>[];
    final deleted = <String>[];
    await MediaResend.run(
      pending: [_pending('att-1')],
      inFlight: inFlight,
      resolveCachePath: (m) => '/cache/${m.attachmentId}',
      cacheFileExists: (_) async => true,
      sendBytes: (m, path) async => sent.add(m.attachmentId),
      deletePending: (media) async => deleted.add(media.attachmentId),
    );
    expect(sent, isEmpty);
    expect(deleted, isEmpty);
    expect(inFlight, {MediaResend.inFlightKey('att-1', 'bob')});
  });
}
