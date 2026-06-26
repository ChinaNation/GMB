import 'package:citizenapp/transaction/personal-manage/personal_manage_models.dart';
import 'package:citizenapp/citizen/shared/proposal/proposal_models.dart';

/// 提案内存缓存。
///
/// App 生命周期内有效，下拉刷新时清空重载。
/// 提案数据量小（几百字节/个），内存缓存即可。
class ProposalCache {
  // ──── 内存缓存 ────

  static final Map<int, ProposalMeta> _metaCache = {};
  static final Map<int, RuntimeUpgradeProposalInfo> _runtimeUpgradeDetailCache =
      {};
  static final Map<int, CreateMultisigProposalInfo> _createMultisigDetailCache =
      {};
  static final Map<int, CloseMultisigProposalInfo> _closeMultisigDetailCache = {};

  // ──── 读取 ────

  /// 获取提案元数据，命中返回缓存，未命中返回 null。
  static ProposalMeta? getMeta(int proposalId) => _metaCache[proposalId];

  /// 获取协议升级提案详情，命中返回缓存，未命中返回 null。
  static RuntimeUpgradeProposalInfo? getRuntimeUpgradeDetail(int proposalId) =>
      _runtimeUpgradeDetailCache[proposalId];

  /// 获取创建多签提案详情。
  static CreateMultisigProposalInfo? getCreateMultisigDetail(int proposalId) =>
      _createMultisigDetailCache[proposalId];

  /// 获取关闭多签提案详情。
  static CloseMultisigProposalInfo? getCloseMultisigDetail(int proposalId) =>
      _closeMultisigDetailCache[proposalId];

  // ──── 写入 ────

  /// 存入提案元数据。
  static void putMeta(int proposalId, ProposalMeta meta) =>
      _metaCache[proposalId] = meta;

  /// 存入协议升级提案详情。
  static void putRuntimeUpgradeDetail(
          int proposalId, RuntimeUpgradeProposalInfo detail) =>
      _runtimeUpgradeDetailCache[proposalId] = detail;

  /// 存入创建多签提案详情。
  static void putCreateMultisigDetail(
          int proposalId, CreateMultisigProposalInfo detail) =>
      _createMultisigDetailCache[proposalId] = detail;

  /// 存入关闭多签提案详情。
  static void putCloseMultisigDetail(
          int proposalId, CloseMultisigProposalInfo detail) =>
      _closeMultisigDetailCache[proposalId] = detail;

  // ──── 清除 ────

  /// 清空所有缓存（下拉刷新时调用）。
  static void clear() {
    _metaCache.clear();
    _runtimeUpgradeDetailCache.clear();
    _createMultisigDetailCache.clear();
    _closeMultisigDetailCache.clear();
  }

  /// 使单个提案缓存失效（轻节点推送新区块时用）。
  static void invalidate(int proposalId) {
    _metaCache.remove(proposalId);
    _runtimeUpgradeDetailCache.remove(proposalId);
    _createMultisigDetailCache.remove(proposalId);
    _closeMultisigDetailCache.remove(proposalId);
  }
}
