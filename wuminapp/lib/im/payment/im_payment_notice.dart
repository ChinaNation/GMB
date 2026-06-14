/// 聊天窗口内公民币转账提示状态。
///
/// payment notice 只是聊天里的加密提示，不是到账真相；到账真相必须由
/// wuminapp 通过链上查询确认。
enum ImPaymentNoticeState {
  /// 等待用户钱包签名。
  waitingForSignature,

  /// 交易已广播到公民链网络。
  broadcast,

  /// 链上已确认。
  chainConfirmed,

  /// 链上确认失败或本地提交失败。
  failed,

  /// 聊天提示与链上结果不匹配，需要用户核验。
  mismatch,
}

/// 聊天消息中的公民币转账提示。
class ImPaymentNotice {
  const ImPaymentNotice({
    required this.noticeId,
    required this.fromWallet,
    required this.toWallet,
    required this.amountFen,
    required this.state,
    this.txHash,
    this.createdAt,
  });

  /// IM 内部提示 ID，不复用 txHash。
  final String noticeId;

  /// 付款钱包账户。
  final String fromWallet;

  /// 收款钱包账户，来自联系人聊天账户。
  final String toWallet;

  /// 金额，单位为分。
  final int amountFen;

  /// 当前链上确认状态。
  final ImPaymentNoticeState state;

  /// 已广播后拿到的链上交易哈希。
  final String? txHash;

  /// 创建时间。
  final DateTime? createdAt;
}
