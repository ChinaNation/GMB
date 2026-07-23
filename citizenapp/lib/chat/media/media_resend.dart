import '../storage/chat_store.dart';

/// 上线补发媒体字节的**可测核心**(与 WebRTC / 文件系统 / Documents 目录解耦)。
///
/// 对每条待投递媒体:在途中的(初始发送或另一次补发正在传)跳过去重;缓存副本已丢
/// (如会话删除)清孤儿行;否则重发,收到 ack(成功)删行、失败保留待下次 peer_ready。
/// 路径由调用方按当前 Documents 目录重算传入(不依赖持久化的绝对路径)。
class MediaResend {
  MediaResend._();

  /// 在途去重键 = (attachmentId, recipient)。群里同一媒体发多成员,按成员去重,
  /// 不能只按 attachmentId(否则发给 B 时会误跳过发给 C)。
  static String inFlightKey(String attachmentId, String recipientAccountId) =>
      '$attachmentId|$recipientAccountId';

  static Future<void> run({
    required List<ChatPendingMedia> pending,
    required Set<String> inFlight,
    required String Function(ChatPendingMedia media) resolveCachePath,
    required Future<bool> Function(String path) cacheFileExists,
    required Future<void> Function(ChatPendingMedia media, String path)
        sendBytes,
    required Future<void> Function(ChatPendingMedia media) deletePending,
  }) async {
    for (final media in pending) {
      final key = inFlightKey(media.attachmentId, media.recipientAccountId);
      if (inFlight.contains(key)) {
        continue; // 在途,别重发(按 (媒体,成员) 去重)
      }
      final path = resolveCachePath(media);
      if (!await cacheFileExists(path)) {
        await deletePending(media); // 缓存副本已丢,清孤儿
        continue;
      }
      inFlight.add(key);
      try {
        await sendBytes(media, path);
        await deletePending(media); // 收到 ack,删行
      } on Exception {
        // 仍离线 / 直连失败:保留 pending,等下次 peer_ready 再补发。
      } finally {
        inFlight.remove(key);
      }
    }
  }
}
