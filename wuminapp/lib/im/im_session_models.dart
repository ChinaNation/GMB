/// IM 会话与节点状态的前端基础模型。
///
/// 本文件只定义公民端展示和状态机所需的轻量模型；真实消息持久化
/// 后续进入 Isar schema 前必须单独确认，避免擅自改变本地数据库结构。
enum ImMessageKind {
  /// 普通文本消息。
  text,

  /// 图片、视频或文件附件消息。
  attachment,

  /// 聊天窗口内发起的公民币转账提示。
  paymentNotice,
}

/// IM 消息发送状态。
enum ImMessageDeliveryState {
  /// 已写入本机发送队列。
  queued,

  /// 正在通过私人通信全节点或近场链路发送。
  sending,

  /// 已交给目标私人通信全节点或近场对端。
  sent,

  /// 对方设备已经拉取到密文消息。
  receivedByDevice,

  /// 本机确认链上或通信结果失败。
  failed,
}

/// 私人通信全节点绑定状态。
enum ImNodeBindingStatus {
  /// 尚未绑定自己的通信全节点。
  unbound,

  /// 已绑定但当前不可达。
  offline,

  /// 已绑定且可达。
  online,

  /// 正在同步密文收件箱。
  syncing,
}

/// 会话列表展示用快照。
class ImConversationPreview {
  const ImConversationPreview({
    required this.conversationId,
    required this.title,
    required this.walletAddress,
    required this.lastMessage,
    required this.lastUpdatedAt,
    required this.unreadCount,
    required this.deliveryState,
  });

  /// 会话 ID，由 IM 层生成，不复用链上交易哈希。
  final String conversationId;

  /// 用户可见名称，默认可以来自钱包地址短展示。
  final String title;

  /// 联系人的钱包账户地址；聊天账户与收付款账户共用该账户。
  final String walletAddress;

  /// 最近一条消息摘要。真实明文只允许存在于手机端本地。
  final String lastMessage;

  /// 最近更新时间。
  final DateTime lastUpdatedAt;

  /// 未读消息数量。
  final int unreadCount;

  /// 最近一条消息投递状态。
  final ImMessageDeliveryState deliveryState;
}

/// 信息 Tab 顶部状态快照。
class ImInboxOverview {
  const ImInboxOverview({
    required this.nodeStatus,
    required this.boundWalletAddress,
    required this.nodeEndpoint,
    required this.pendingOutgoing,
    required this.unreadCount,
  });

  /// 当前私人通信全节点绑定状态。
  final ImNodeBindingStatus nodeStatus;

  /// 当前作为聊天账户的钱包地址。
  final String? boundWalletAddress;

  /// 自己通信全节点的 IPv4 / IPv6 / dnsaddr 端点展示。
  final String? nodeEndpoint;

  /// 等待发送或重试的密文消息数量。
  final int pendingOutgoing;

  /// 所有会话未读数量。
  final int unreadCount;

  /// 当前没有真实 IM 后端时使用的安全空快照。
  static const empty = ImInboxOverview(
    nodeStatus: ImNodeBindingStatus.unbound,
    boundWalletAddress: null,
    nodeEndpoint: null,
    pendingOutgoing: 0,
    unreadCount: 0,
  );
}
