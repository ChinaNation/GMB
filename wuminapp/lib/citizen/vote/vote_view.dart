import 'dart:async';
import 'dart:io' show Platform;

import 'package:flutter/material.dart';
import 'package:wuminapp_mobile/governance/institution_manage_detail_page.dart';

import 'package:wuminapp_mobile/ui/app_theme.dart';
import 'package:wuminapp_mobile/ui/widgets/pressable_card.dart';
import 'package:wuminapp_mobile/ui/widgets/shimmer_loading.dart';
import 'package:wuminapp_mobile/rpc/chain_event_subscription.dart';
import 'package:wuminapp_mobile/rpc/smoldot_client.dart';
import 'package:wuminapp_mobile/governance/admins-change/services/institution_admin_service.dart';
import 'package:wuminapp_mobile/governance/shared/proposal/proposal_cache.dart';
import 'package:wuminapp_mobile/governance/shared/proposal/proposal_context.dart';
import 'package:wuminapp_mobile/governance/shared/proposal/proposal_local_store.dart';
import 'package:wuminapp_mobile/governance/runtime-upgrade/runtime_upgrade_detail_page.dart';
import 'package:wuminapp_mobile/governance/shared/proposal/proposal_models.dart';
import 'package:wuminapp_mobile/transaction/duoqian-transfer/duoqian_transfer_proposal_adapter.dart';

/// 全局治理提案列表:展示 NRC / PRC / PRB 三类机构所有提案,按 ID 倒序。
///
/// **数据源(v1 双层 ID + 反向索引)**:
/// - `ProposalsByOrg[NRC] ∪ ByOrg[PRC] ∪ ByOrg[PRB]` 取所有治理类提案 ID
/// - **不再扫主键 + 客户端过滤**;ORG_REN 多签提案天然不进列表
///
/// **分页**:cursor 模式按 `_allIds` 切分,翻页天然不会卡空页。
/// **新区块订阅**:周期性重 fetch 三 org id 列表,补差异。
class VoteView extends StatefulWidget {
  const VoteView({
    super.key,
    this.onPendingVoteCountChanged,
  });

  /// 待投票数变化时的回调（用于底部 tab 红点数字）。
  final ValueChanged<int>? onPendingVoteCountChanged;

  @override
  State<VoteView> createState() => _VoteViewState();
}

class _VoteViewState extends State<VoteView> {
  static const int _pageSize = 10;
  static const Duration _newBlockIndexCheckMinInterval = Duration(seconds: 60);

  // 治理类 org 编码(与 votingengine::internal_vote::ORG_* 对齐)
  static const int _orgNrc = 0;
  static const int _orgPrc = 1;
  static const int _orgPrb = 2;

  final DuoqianTransferProposalFeed _duoqianTransferFeed =
      DuoqianTransferProposalFeed();
  final InstitutionAdminService _adminService = InstitutionAdminService();
  final ProposalContextResolver _contextResolver = ProposalContextResolver();
  final VoteChecker _voteChecker = VoteChecker();
  final ScrollController _scrollController = ScrollController();

  // 轻节点新区块订阅
  ChainEventSubscription? _subscription;
  StreamSubscription<ChainEvent>? _eventSub;

  // 分页状态
  bool _loading = true;
  bool _loadingMore = false;
  String? _error;
  List<_ProposalDisplayItem> _items = [];

  /// 通过反向索引取到的全部治理提案 ID(降序排列)。
  /// 列表页基于此切分翻页 — cursor `_items.length` 标记已加载到第几条,
  /// `_hasMore = _items.length < _allIds.length`。
  List<int> _allIds = const [];

  /// 待投票计数。
  int _pendingVoteCount = 0;

  DateTime? _lastProposalIndexCheckAt;

  bool get _hasMore => _items.length < _allIds.length;
  bool get _isFlutterTest => Platform.environment.containsKey('FLUTTER_TEST');

  @override
  void initState() {
    super.initState();
    _scrollController.addListener(_onScroll);
    if (_isFlutterTest) {
      // 中文注释：App 启动 widget test 只验证首屏结构，不验证隐藏广场页的轻节点订阅。
      // 测试环境没有真实 smoldot 链路，继续加载链上提案会让 pumpAndSettle 等不到稳定帧。
      _loading = false;
      return;
    }
    _loadFirstPage();
    _startChainSubscription();
  }

  @override
  void dispose() {
    _scrollController.dispose();
    _eventSub?.cancel();
    _subscription?.disconnect();
    super.dispose();
  }

  // ──── 轻节点订阅 ────

  void _startChainSubscription() {
    _subscription = ChainEventSubscription();
    _subscription!.connect();
    _eventSub = _subscription!.events.listen((event) {
      if (event.type == ChainEventType.newBlock) {
        _checkForNewProposals();
      }
    });
  }

  Future<void> _checkForNewProposals() async {
    final now = DateTime.now();
    final lastCheck = _lastProposalIndexCheckAt;
    if (lastCheck != null &&
        now.difference(lastCheck) < _newBlockIndexCheckMinInterval) {
      return;
    }
    _lastProposalIndexCheckAt = now;

    try {
      final fresh = await _fetchAllGovernanceIds();
      await ProposalLocalStore.instance.putGlobalIndex(fresh);
      final knownSet = _allIds.toSet();
      final newIds = fresh.where((id) => !knownSet.contains(id)).toList();
      if (newIds.isEmpty) return;

      // 新增提案插到列表顶部
      final newItems = await _loadItemsForIds(newIds);
      if (mounted) {
        setState(() {
          _items = [...newItems, ..._items];
          _allIds = fresh;
        });
        _updatePendingVoteCount();
      }
    } catch (_) {
      // 静默忽略,不阻塞 UI
    }
  }

  // ──── 分页加载 ────

  void _onScroll() {
    if (_scrollController.position.pixels >=
        _scrollController.position.maxScrollExtent - 200) {
      _loadNextPage();
    }
  }

  /// 拉三 org 反向索引并集,降序返回(主键单调,降序即按时间倒序)。
  Future<List<int>> _fetchAllGovernanceIds() async {
    final results = await Future.wait([
      _duoqianTransferFeed.fetchProposalIdsByOrg(_orgNrc),
      _duoqianTransferFeed.fetchProposalIdsByOrg(_orgPrc),
      _duoqianTransferFeed.fetchProposalIdsByOrg(_orgPrb),
    ]);
    final all = <int>{...results[0], ...results[1], ...results[2]}.toList();
    all.sort((a, b) => b.compareTo(a));
    return all;
  }

  Future<void> _loadFirstPage({bool force = false}) async {
    setState(() {
      _loading = _items.isEmpty;
      _error = null;
      _loadingMore = false;
    });

    final localLoaded = !force && await _loadFirstPageFromLocal();
    if (localLoaded && await ProposalLocalStore.instance.isGlobalIndexFresh()) {
      if (mounted) {
        setState(() => _loading = false);
      }
      return;
    }

    try {
      final ids = await _fetchAllGovernanceIds();

      if (ids.isEmpty) {
        await ProposalLocalStore.instance.putGlobalIndex(const []);
        if (!mounted) return;
        setState(() {
          _allIds = const [];
          _items = const [];
          _loading = false;
        });
        widget.onPendingVoteCountChanged?.call(0);
        return;
      }

      // 切前 _pageSize 条
      final firstPageIds =
          ids.sublist(0, ids.length < _pageSize ? ids.length : _pageSize);
      final items = await _loadItemsForIds(firstPageIds);
      await ProposalLocalStore.instance.putGlobalIndex(ids);

      if (!mounted) return;
      setState(() {
        _allIds = ids;
        _items = items;
        _loading = false;
      });

      _updatePendingVoteCount();
    } catch (e) {
      if (!mounted) return;
      if (localLoaded) {
        setState(() => _loading = false);
        return;
      }
      setState(() {
        _error = SmoldotClientManager.instance.buildUserFacingError(e);
        _loading = false;
      });
      widget.onPendingVoteCountChanged?.call(0);
    }
  }

  Future<bool> _loadFirstPageFromLocal() async {
    try {
      final index = await ProposalLocalStore.instance.readGlobalIndex();
      if (index == null || index.ids.isEmpty) return false;
      final summaries = await ProposalLocalStore.instance.readGlobalPage(
        limit: _pageSize,
      );
      if (!mounted || summaries.isEmpty) return summaries.isNotEmpty;
      setState(() {
        _allIds = index.ids;
        _items = summaries
            .map(_ProposalDisplayItem.fromLocalSummary)
            .toList(growable: false);
        _loading = false;
      });
      widget.onPendingVoteCountChanged?.call(0);
      return true;
    } catch (_) {
      return false;
    }
  }

  Future<void> _loadNextPage() async {
    if (_loadingMore || !_hasMore) return;

    setState(() => _loadingMore = true);

    try {
      final from = _items.length;
      final to = (from + _pageSize) > _allIds.length
          ? _allIds.length
          : (from + _pageSize);
      final pageIds = _allIds.sublist(from, to);
      final localItems = await _loadLocalItemsForIds(pageIds);
      final useLocalOnly = localItems.length == pageIds.length &&
          await ProposalLocalStore.instance.isGlobalIndexFresh();
      final newItems =
          useLocalOnly ? localItems : await _loadItemsForIds(pageIds);

      if (!mounted) return;
      setState(() {
        _items = [..._items, ...newItems];
        _loadingMore = false;
      });

      _updatePendingVoteCount();
    } catch (e) {
      if (mounted) {
        setState(() => _loadingMore = false);
      }
    }
  }

  Future<List<_ProposalDisplayItem>> _loadLocalItemsForIds(
      List<int> ids) async {
    final summaries =
        await ProposalLocalStore.instance.readSummariesForIds(ids);
    return summaries
        .map(_ProposalDisplayItem.fromLocalSummary)
        .toList(growable: false);
  }

  /// 给定一组 proposal_id,batch fetch 详情 + 上下文 + 待投票判定,
  /// 返回 `_ProposalDisplayItem` 列表(顺序与入参一致)。
  Future<List<_ProposalDisplayItem>> _loadItemsForIds(List<int> ids) async {
    if (ids.isEmpty) return const [];

    // 批量取提案详情(meta + 业务详情)
    final proposals = await _duoqianTransferFeed.fetchProposalsByIds(ids);

    // 批量解析提案上下文
    final contexts = await _contextResolver.resolveBatch(
      proposals.map((p) => p.meta.institutionBytes?.toList()).toList(),
      internalOrgList: proposals.map((p) => p.meta.internalOrg).toList(),
    );

    final items = <_ProposalDisplayItem>[];

    for (var i = 0; i < proposals.length; i++) {
      final p = proposals[i];
      final ctx = contexts[i];

      // 检查是否有未投票的钱包(统一使用 VoteChecker)
      bool needsVote = false;
      if (ctx.hasAdminWallets && p.meta.status == 0) {
        needsVote = await _voteChecker.hasUnvotedWallet(
          proposalId: p.meta.proposalId,
          kind: p.meta.kind,
          adminWallets: ctx.adminWallets,
          institution: ctx.institution,
        );
      }

      items.add(_ProposalDisplayItem.fromProposal(
        proposal: p,
        context: ctx,
        needsVote: needsVote,
      ));
    }

    await ProposalLocalStore.instance.upsertSummaries(
      items.map((item) => item.summary).toList(growable: false),
    );
    return items;
  }

  void _updatePendingVoteCount() {
    _pendingVoteCount = _items.where((i) => i.needsVote).length;
    widget.onPendingVoteCountChanged?.call(_pendingVoteCount);
  }

  // ──── UI ────

  @override
  Widget build(BuildContext context) {
    return _buildForeground();
  }

  Widget _buildForeground() {
    if (_loading) {
      return ListSkeleton(
        itemCount: 5,
        itemBuilder: (_, __) => const ProposalCardSkeleton(),
      );
    }
    if (_error != null) {
      return Center(
        child: Padding(
          padding: const EdgeInsets.all(24),
          child: Column(
            mainAxisSize: MainAxisSize.min,
            children: [
              const Icon(Icons.error_outline, size: 48, color: AppTheme.danger),
              const SizedBox(height: 12),
              const Text('加载失败',
                  style:
                      TextStyle(fontSize: 16, color: AppTheme.textSecondary)),
              const SizedBox(height: 6),
              Text(_error!,
                  style: const TextStyle(
                      fontSize: 12, color: AppTheme.textTertiary),
                  textAlign: TextAlign.center,
                  maxLines: 4,
                  overflow: TextOverflow.ellipsis),
              const SizedBox(height: 16),
              OutlinedButton(
                  onPressed: _loadFirstPage, child: const Text('重试')),
            ],
          ),
        ),
      );
    }
    if (_items.isEmpty && !_hasMore) {
      // 空态:留给水印,前景透明占位以承接下拉刷新。
      return RefreshIndicator(
        onRefresh: () async {
          _adminService.clearCache();
          _contextResolver.clearWalletCache();
          ProposalCache.clear();
          DuoqianTransferProposalAdapter.clearCache();
          await _loadFirstPage(force: true);
        },
        child: ListView(
          physics: const AlwaysScrollableScrollPhysics(),
          children: const [SizedBox(height: 400)],
        ),
      );
    }

    return RefreshIndicator(
      onRefresh: () async {
        _adminService.clearCache();
        _contextResolver.clearWalletCache();
        ProposalCache.clear();
        DuoqianTransferProposalAdapter.clearCache();
        await _loadFirstPage(force: true);
      },
      child: ListView.separated(
        controller: _scrollController,
        padding: const EdgeInsets.fromLTRB(16, 24, 16, 32),
        itemCount: _items.length + (_hasMore ? 1 : 0),
        separatorBuilder: (_, __) => const SizedBox(height: 8),
        itemBuilder: (context, index) {
          if (index < _items.length) {
            return _buildProposalCard(_items[index]);
          }
          // 底部加载指示器
          return const Padding(
            padding: EdgeInsets.symmetric(vertical: 16),
            child: Center(
              child: SizedBox(
                width: 24,
                height: 24,
                child: CircularProgressIndicator(strokeWidth: 2),
              ),
            ),
          );
        },
      ),
    );
  }

  Widget _buildProposalCard(_ProposalDisplayItem item) {
    final statusColor = _statusColor(item.status);
    final statusLabel = _statusLabel(item.status);

    return PressableCard(
      child: Card(
        elevation: 0,
        margin: EdgeInsets.zero,
        shape: RoundedRectangleBorder(
          borderRadius: BorderRadius.circular(12),
          side: BorderSide(color: statusColor.withValues(alpha: 0.2)),
        ),
        child: InkWell(
          onTap: () => _openProposalDetail(item),
          borderRadius: BorderRadius.circular(12),
          child: Padding(
            padding: const EdgeInsets.symmetric(horizontal: 14, vertical: 12),
            child: Row(
              children: [
                // 左侧图标
                Container(
                  width: 36,
                  height: 36,
                  decoration: BoxDecoration(
                    color: statusColor.withValues(alpha: 0.10),
                    borderRadius: BorderRadius.circular(10),
                  ),
                  child:
                      Icon(_proposalIcon(item), size: 18, color: statusColor),
                ),
                const SizedBox(width: 12),
                // 中间信息
                Expanded(
                  child: Column(
                    crossAxisAlignment: CrossAxisAlignment.start,
                    children: [
                      Row(
                        children: [
                          Text(
                            item.displayId,
                            style: const TextStyle(
                              fontSize: 15,
                              fontWeight: FontWeight.w600,
                              color: AppTheme.primaryDark,
                            ),
                          ),
                          const SizedBox(width: 8),
                          if (item.institutionName != null)
                            Container(
                              padding: const EdgeInsets.symmetric(
                                  horizontal: 6, vertical: 1),
                              decoration: BoxDecoration(
                                color: AppTheme.primaryDark
                                    .withValues(alpha: 0.08),
                                borderRadius: BorderRadius.circular(8),
                              ),
                              child: Text(
                                item.institutionName!,
                                style: const TextStyle(
                                    fontSize: 10, color: AppTheme.primaryDark),
                                overflow: TextOverflow.ellipsis,
                              ),
                            ),
                        ],
                      ),
                      const SizedBox(height: 2),
                      Text(
                        item.summary.listSubtitle,
                        style: const TextStyle(
                            fontSize: 12, color: AppTheme.textTertiary),
                      ),
                    ],
                  ),
                ),
                // 右侧状态 + 红点
                Column(
                  crossAxisAlignment: CrossAxisAlignment.end,
                  children: [
                    Container(
                      padding: const EdgeInsets.symmetric(
                          horizontal: 8, vertical: 2),
                      decoration: BoxDecoration(
                        color: statusColor.withValues(alpha: 0.1),
                        borderRadius: BorderRadius.circular(10),
                      ),
                      child: Text(
                        statusLabel,
                        style: TextStyle(
                            fontSize: 11,
                            fontWeight: FontWeight.w600,
                            color: statusColor),
                      ),
                    ),
                    if (item.needsVote) ...[
                      const SizedBox(height: 4),
                      Container(
                        width: 8,
                        height: 8,
                        decoration: const BoxDecoration(
                          color: AppTheme.danger,
                          shape: BoxShape.circle,
                        ),
                      ),
                    ],
                  ],
                ),
                const SizedBox(width: 4),
                const Icon(Icons.chevron_right,
                    size: 20, color: AppTheme.textTertiary),
              ],
            ),
          ),
        ),
      ),
    );
  }

  String _statusLabel(int status) {
    switch (status) {
      case 0:
        return '投票中';
      case 1:
        return '已通过';
      case 2:
        return '已拒绝';
      case 3:
        return '已执行';
      case 4:
        return '执行失败';
      default:
        return '未知';
    }
  }

  Color _statusColor(int status) => AppTheme.proposalStatusColor(status);

  /// 根据提案类型返回图标。
  IconData _proposalIcon(
    _ProposalDisplayItem item,
  ) {
    return switch (item.summary.iconKind) {
      'transfer' => Icons.send_outlined,
      'safety_fund' => Icons.health_and_safety_outlined,
      'sweep' => Icons.account_balance_wallet_outlined,
      'create_duoqian' => Icons.group_add,
      'close_duoqian' => Icons.group_remove,
      'runtime_upgrade' => Icons.arrow_upward,
      'resolution_issuance' => Icons.add_circle_outline,
      'resolution_destroy' => Icons.remove_circle_outline,
      'joint' => Icons.groups_outlined,
      _ => Icons.description_outlined,
    };
  }

  Future<void> _openProposalDetail(_ProposalDisplayItem item) async {
    final resolved = await _resolveProposalDetail(item);
    if (!mounted) return;
    if (resolved == null) {
      ScaffoldMessenger.of(context).showSnackBar(
        const SnackBar(content: Text('提案详情读取失败，请稍后重试')),
      );
      return;
    }
    final (:proposal, :proposalContext) = resolved;
    final inst = proposalContext.institution;
    final proposalId = proposal.meta.proposalId;

    // 协议升级提案（联合投票，kind=1）
    if (proposal.runtimeUpgradeDetail != null) {
      await Navigator.of(context).push(
        MaterialPageRoute(
          builder: (_) => RuntimeUpgradeDetailPage(
            proposalId: proposalId,
            proposalContext: proposalContext,
          ),
        ),
      );
    } else if (DuoqianTransferProposalAdapter.matches(proposal)) {
      await DuoqianTransferProposalAdapter.openDetail(
        context,
        proposal: proposal,
        institution: inst,
        proposalContext: proposalContext,
      );
    } else if ((proposal.createDuoqianDetail != null ||
            proposal.closeDuoqianDetail != null) &&
        inst != null) {
      // 多签管理提案
      await Navigator.of(context).push(
        MaterialPageRoute(
          builder: (_) => InstitutionManageDetailPage(
            institution: inst,
            proposalId: proposalId,
            proposalContext: proposalContext,
          ),
        ),
      );
    } else {
      // 其他未知类型
      ScaffoldMessenger.of(context).showSnackBar(
        const SnackBar(content: Text('该提案类型的详情页面正在开发中')),
      );
      return;
    }

    // 返回后刷新
    if (mounted) {
      _adminService.clearCache();
      ProposalCache.clear();
      DuoqianTransferProposalAdapter.clearCache();
      _loadFirstPage(force: true);
    }
  }

  Future<({ProposalWithDetail proposal, ProposalContext proposalContext})?>
      _resolveProposalDetail(_ProposalDisplayItem item) async {
    final existingProposal = item.proposal;
    final existingContext = item.context;
    if (existingProposal != null && existingContext != null) {
      return (proposal: existingProposal, proposalContext: existingContext);
    }

    try {
      final proposals =
          await _duoqianTransferFeed.fetchProposalsByIds([item.proposalId]);
      if (proposals.isEmpty) return null;
      final proposal = proposals.first;
      final contexts = await _contextResolver.resolveBatch(
        [proposal.meta.institutionBytes?.toList()],
        internalOrgList: [proposal.meta.internalOrg],
      );
      final proposalContext =
          contexts.isEmpty ? const ProposalContext() : contexts.first;
      final resolvedItem = _ProposalDisplayItem.fromProposal(
        proposal: proposal,
        context: proposalContext,
        needsVote: item.needsVote,
      );
      await ProposalLocalStore.instance.upsertSummaries([
        resolvedItem.summary,
      ]);
      if (mounted) {
        setState(() {
          _items = [
            for (final current in _items)
              if (current.proposalId == item.proposalId)
                resolvedItem
              else
                current,
          ];
        });
      }
      return (proposal: proposal, proposalContext: proposalContext);
    } catch (_) {
      return null;
    }
  }
}

class _ProposalDisplayItem {
  const _ProposalDisplayItem({
    required this.summary,
    this.proposal,
    this.context,
    this.needsVote = false,
  });

  factory _ProposalDisplayItem.fromProposal({
    required ProposalWithDetail proposal,
    required ProposalContext context,
    bool needsVote = false,
  }) {
    return _ProposalDisplayItem(
      proposal: proposal,
      context: context,
      summary: LocalProposalSummary.fromProposal(
        proposal,
        institution: context.institution,
      ),
      needsVote: needsVote,
    );
  }

  factory _ProposalDisplayItem.fromLocalSummary(
    LocalProposalSummary summary,
  ) {
    return _ProposalDisplayItem(summary: summary);
  }

  final LocalProposalSummary summary;
  final ProposalWithDetail? proposal;
  final ProposalContext? context;
  final bool needsVote;

  int get proposalId => summary.proposalId;
  int get status => summary.status;
  String get displayId => summary.displayId;
  String? get institutionName =>
      context?.institution?.name ?? summary.institutionName;
}
