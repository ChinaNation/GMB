import 'package:wuminapp_mobile/transaction/duoqian-transfer/duoqian_transfer_models.dart';

/// 多签转账提案详情缓存。
///
/// 只缓存 DuoqianTransfer pallet 解码出的业务详情；通用提案 meta、
/// runtime 升级和多签管理缓存仍由 proposal/shared 自己维护。
class DuoqianTransferCache {
  static final Map<int, TransferProposalInfo> _transferDetailCache = {};

  /// 获取普通多签转账提案详情。
  static TransferProposalInfo? getTransferDetail(int proposalId) =>
      _transferDetailCache[proposalId];

  /// 存入普通多签转账提案详情。
  static void putTransferDetail(int proposalId, TransferProposalInfo detail) =>
      _transferDetailCache[proposalId] = detail;

  /// 清空多签转账缓存。
  static void clear() {
    _transferDetailCache.clear();
  }

  /// 使单个多签转账提案缓存失效。
  static void invalidate(int proposalId) {
    _transferDetailCache.remove(proposalId);
  }
}
