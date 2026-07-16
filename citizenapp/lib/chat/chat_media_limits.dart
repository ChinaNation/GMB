import 'chat_models.dart';

/// 聊天媒体大小上限的单一真源,收发两端共用。
///
/// 媒体字节走 WebRTC 设备直连,Cloudflare 从不经手,故上限只考验用户网络与设备
/// 内存,不影响服务端资源。也正因为服务端不在字节路径上,大小门控**只能且必须**
/// 由收发两端各自强制:发送端拒发、接收端在字节管道层拒收——被篡改的发送方也
/// 无法把超限媒体塞给诚实的接收方。
class ChatMediaLimits {
  ChatMediaLimits._();

  static const int _mib = 1024 * 1024;

  /// 图片 100MB;视频、文件 5GB。
  static const int imageMaxBytes = 100 * _mib;
  static const int videoMaxBytes = 5120 * _mib;
  static const int fileMaxBytes = 5120 * _mib;

  /// 传输字节层的硬顶(= 各类上限的最大值)。mime 未知时的兜底上界。
  static const int absoluteMaxBytes = 5120 * _mib;

  /// 按消息类型取上限。text / sticker 无字节,返回 0(表示"不接受任何字节")。
  static int forKind(ChatMessageKind kind) => switch (kind) {
        ChatMessageKind.image => imageMaxBytes,
        ChatMessageKind.video => videoMaxBytes,
        ChatMessageKind.file => fileMaxBytes,
        ChatMessageKind.text || ChatMessageKind.sticker => 0,
      };

  /// 按 MIME 取上限。传输字节层(WebRTC)只有 content_type,用它判额。
  static int forMime(String mime) {
    if (mime.startsWith('image/')) return imageMaxBytes;
    if (mime.startsWith('video/')) return videoMaxBytes;
    return fileMaxBytes;
  }

  /// 该类消息的 [byteSize] 是否超限。0 上限(text/sticker)一律视为不超限,
  /// 因为它们不携带媒体字节。
  static bool exceedsForKind(ChatMessageKind kind, int byteSize) {
    final limit = forKind(kind);
    return limit > 0 && byteSize > limit;
  }
}

/// 媒体超出大小上限。发送端在把任何字节送入通道前抛出;UI 据此给用户明确提示。
class ChatMediaTooLargeException implements Exception {
  const ChatMediaTooLargeException({
    required this.byteSize,
    required this.limitBytes,
    this.kind,
  });

  final int byteSize;
  final int limitBytes;
  final ChatMessageKind? kind;

  @override
  String toString() => '媒体大小 $byteSize 字节超出上限 $limitBytes 字节';
}
