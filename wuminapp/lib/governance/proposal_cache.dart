import 'transfer_proposal_service.dart';

/// 提案内存缓存。
///
/// App 生命周期内有效，下拉刷新时清空重载。
/// 提案数据量小（几百字节/个），内存缓存即可。
class ProposalCache {
  // ──── 内存缓存 ────

  static final Map<int, ProposalMeta> _metaCache = {};
  static final Map<int, TransferProposalInfo> _detailCache = {};

  // ──── 读取 ────

  /// 获取提案元数据，命中返回缓存，未命中返回 null。
  static ProposalMeta? getMeta(int proposalId) => _metaCache[proposalId];

  /// 获取转账提案详情，命中返回缓存，未命中返回 null。
  static TransferProposalInfo? getDetail(int proposalId) =>
      _detailCache[proposalId];

  // ──── 写入 ────

  /// 存入提案元数据。
  static void putMeta(int proposalId, ProposalMeta meta) =>
      _metaCache[proposalId] = meta;

  /// 存入转账提案详情。
  static void putDetail(int proposalId, TransferProposalInfo detail) =>
      _detailCache[proposalId] = detail;

  // ──── 清除 ────

  /// 清空所有缓存（下拉刷新时调用）。
  static void clear() {
    _metaCache.clear();
    _detailCache.clear();
  }

  /// 使单个提案缓存失效（WebSocket 推送新区块时用）。
  static void invalidate(int proposalId) {
    _metaCache.remove(proposalId);
    _detailCache.remove(proposalId);
  }
}
