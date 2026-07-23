/// Chat 会话与投递状态的前端基础模型。
///
/// 本文件只定义公民端展示和状态机所需的轻量模型；真实消息持久化
/// 后续进入 Isar schema 前必须单独确认，避免擅自改变本地数据库结构。
enum ChatMessageKind {
  /// 普通文本消息。
  text,

  /// 相册/相机图片消息。字节经 WebRTC 端到端直传,内联展示。
  image,

  /// 视频消息。字节经 WebRTC 端到端直传,展示封面并可播放。
  video,

  /// 通用文件附件消息(非图片/视频)。
  file,

  /// 内置贴纸消息。只传贴纸 id,字节不过网,本地资源渲染。
  sticker,
}

/// Chat 消息发送状态。
enum ChatMessageDeliveryState {
  /// 已写入本机发送队列。
  queued,

  /// 正在通过 Cloudflare 瞬时转发或近场链路发送。
  sending,

  /// 已交给目标在线设备或近场对端。
  sent,

  /// 对方设备已经收到密文消息。
  receivedByDevice,

  /// 本机确认通信结果失败。
  failed,
}

/// 会话列表展示用快照。
class ChatConversationPreview {
  const ChatConversationPreview({
    required this.conversationId,
    required this.title,
    required this.peerAccountId,
    required this.lastMessage,
    required this.lastUpdatedAt,
    required this.unreadCount,
    required this.deliveryState,
    this.conversationKind = 'dm',
  });

  /// 会话 ID，由 Chat 层生成，不复用链上交易哈希。
  final String conversationId;

  /// 用户可见名称，默认可以来自钱包地址短展示。
  final String title;

  /// 联系人的钱包账户地址；聊天账户与收付款账户共用该账户。
  final String peerAccountId;

  /// 最近一条消息摘要。真实明文只允许存在于手机端本地。
  final String lastMessage;

  /// 最近更新时间。
  final DateTime lastUpdatedAt;

  /// 未读消息数量。
  final int unreadCount;

  /// 最近一条消息投递状态。
  final ChatMessageDeliveryState deliveryState;

  /// 会话类型:"dm"=私聊,"group"=私密小群。
  final String conversationKind;

  bool get isGroup => conversationKind == 'group';
}

/// 聊天 Tab 顶部状态快照。
class ChatInboxOverview {
  const ChatInboxOverview({
    required this.accountId,
    required this.pendingOutgoing,
    required this.unreadCount,
  });

  /// 当前作为聊天账户的钱包地址。
  final String? accountId;

  /// 等待发送或重试的密文消息数量。
  final int pendingOutgoing;

  /// 所有会话未读数量。
  final int unreadCount;

  /// 当前没有聊天账户时使用的安全空快照。
  static const empty = ChatInboxOverview(
    accountId: null,
    pendingOutgoing: 0,
    unreadCount: 0,
  );
}
