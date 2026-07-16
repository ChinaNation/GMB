import 'chat_models.dart';

/// 聊天媒体大小上限的单一真源(收发两端共用),按会员档动态(ADR-036,会员与身份解耦)。
///
/// 会员权益之一 = 单个聊天文件上限:无订阅/自由 10MB、民主 100MB、薪火 5GB,与会员套餐
/// `chat_file_max_bytes` 同源。媒体字节走 WebRTC 设备直连,Cloudflare 从不经手,故上限
/// 只考验用户网络与设备内存,不影响服务端资源;也正因服务端不在字节路径上,大小门控
/// **只能且必须**由收发两端各自强制:发送端拒发、接收端在字节管道层拒收——被篡改的发送
/// 方也无法把超限媒体塞给诚实的接收方。
///
/// >100MB(仅薪火,100MB–5GB)的 Cloudflare 瞬时"群密钥密文"中转 transport 归卡2 阶段3
/// 实现,本表只定各档单个文件上限值。当前档由 [applyMembershipLevel] 在会员状态载入时
/// 设置(见 `SquareApiClient.fetchMembership`),未知时 fail-closed 取自由档 10MB。
class ChatMediaLimits {
  ChatMediaLimits._();

  static const int _mib = 1024 * 1024;

  /// 三档单个文件上限(字节),与会员套餐 `chat_file_max_bytes` 单源对齐。
  static const int freedomMaxBytes = 10 * _mib;
  static const int democracyMaxBytes = 100 * _mib;
  static const int sparkMaxBytes = 5120 * _mib;

  /// 传输字节层的硬顶(= 最高档上限)。mime / 档未知时的兜底上界。
  static const int absoluteMaxBytes = sparkMaxBytes;

  /// P2P / Cloudflare 中转的分界(**固定 100MB,不随档变**):
  /// ≤100MB 走 WebRTC P2P(永不触 Cloudflare 字节路径);>100MB 走 R2 瞬时中转。
  /// 因非薪火档上限 <100MB(门①天然挡),只有薪火能产出 >100MB、走中转。
  static const int relayThresholdBytes = democracyMaxBytes;

  /// 该字节数是否需走 Cloudflare 中转(仅 >100MB)。
  static bool needsRelay(int byteSize) => byteSize > relayThresholdBytes;

  /// 当前默认钱包会员档的单个文件上限;会员载入后由 [applyMembershipLevel] 更新,
  /// 未知或订阅失效时 fail-closed 为自由档 10MB。
  static int _currentMaxBytes = freedomMaxBytes;

  /// 会员档 → 单个文件上限(纯函数,可测)。未知 / 无订阅一律归自由档。
  static int maxBytesForLevel(String? level) => switch (level) {
        'spark' => sparkMaxBytes,
        'democracy' => democracyMaxBytes,
        _ => freedomMaxBytes,
      };

  /// 会员状态载入后设置当前档上限(fail-closed 到自由档);订阅失效传 null。
  static void applyMembershipLevel(String? level) {
    _currentMaxBytes = maxBytesForLevel(level);
  }

  /// 当前档单个文件上限(字节)。
  static int get currentMaxBytes => _currentMaxBytes;

  /// 按消息类型取上限。媒体(image/video/file)= 当前档上限;text / sticker 无字节返回 0。
  static int forKind(ChatMessageKind kind) => switch (kind) {
        ChatMessageKind.image ||
        ChatMessageKind.video ||
        ChatMessageKind.file =>
          _currentMaxBytes,
        ChatMessageKind.text || ChatMessageKind.sticker => 0,
      };

  /// 按 MIME 取上限。传输字节层(WebRTC)只有 content_type;媒体一律取当前档上限。
  static int forMime(String mime) => _currentMaxBytes;

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
