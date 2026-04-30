import 'dart:async';

import 'package:flutter/material.dart';
import 'package:wuminapp_mobile/duoqian/shared/duoqian_manage_detail_page.dart';
import 'package:wuminapp_mobile/duoqian/shared/duoqian_manage_models.dart';

import 'package:wuminapp_mobile/ui/app_theme.dart';
import 'package:wuminapp_mobile/ui/widgets/pressable_card.dart';
import 'package:wuminapp_mobile/ui/widgets/shimmer_loading.dart';
import 'package:wuminapp_mobile/util/amount_format.dart';
import 'package:wuminapp_mobile/rpc/chain_event_subscription.dart';
import 'package:wuminapp_mobile/rpc/smoldot_client.dart';
import 'package:wuminapp_mobile/wallet/core/wallet_manager.dart';
import 'package:wuminapp_mobile/citizen/institution/institution_admin_service.dart';
import 'package:wuminapp_mobile/citizen/institution/institution_data.dart';
import 'package:wuminapp_mobile/citizen/governance/proposal_cache.dart';
import 'package:wuminapp_mobile/citizen/shared/proposal_context.dart';
import 'package:wuminapp_mobile/citizen/proposal/runtime_upgrade/runtime_upgrade_detail_page.dart';
import 'package:wuminapp_mobile/citizen/proposal/shared/proposal_models.dart';
import 'package:wuminapp_mobile/citizen/proposal/transfer/transfer_proposal_detail_page.dart';
import 'package:wuminapp_mobile/citizen/proposal/transfer/transfer_proposal_service.dart';

/// 全局提案列表：展示全链所有提案，按 ID 倒序，标注投票状态和红点。
///
/// 采用四层优化架构：
/// 1. 本地内存缓存（ProposalCache）
/// 2. 批量查询（fetchStorageBatch）
/// 3. 分页加载（ScrollController 滚动触发）
/// 4. 轻节点新区块订阅（新区块自动检测新提案）
class AllProposalsView extends StatefulWidget {
  const AllProposalsView({
    super.key,
    this.onPendingVoteCountChanged,
  });

  /// 待投票数变化时的回调（用于底部 tab 红点数字）。
  final ValueChanged<int>? onPendingVoteCountChanged;

  @override
  State<AllProposalsView> createState() => _AllProposalsViewState();
}

class _AllProposalsViewState extends State<AllProposalsView> {
  static const int _pageSize = 10;

  final TransferProposalService _proposalService = TransferProposalService();
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
  bool _hasMore = true;
  String? _error;
  List<_ProposalDisplayItem> _items = [];

  /// 已知的 nextProposalId（用于检测新提案）。
  int _knownNextId = 0;

  /// 已加载到的最小提案 ID（不含）。
  int _loadedUpTo = -1;

  /// 待投票计数。
  int _pendingVoteCount = 0;

  @override
  void initState() {
    super.initState();
    _scrollController.addListener(_onScroll);
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
      if (event == ChainEvent.newBlock) {
        _checkForNewProposals();
      }
    });
  }

  Future<void> _checkForNewProposals() async {
    try {
      final newNextId = await _proposalService.fetchNextProposalId();
      if (newNextId > _knownNextId && _knownNextId > 0) {
        // 有新提案，在顶部插入
        final newItems = await _loadProposalRange(newNextId - 1, _knownNextId);
        if (newItems.isNotEmpty && mounted) {
          setState(() {
            _items = [...newItems, ..._items];
            _knownNextId = newNextId;
          });
          _updatePendingVoteCount();
        }
      }
    } catch (_) {
      // 静默忽略，不阻塞 UI
    }
  }

  // ──── 分页加载 ────

  void _onScroll() {
    if (_scrollController.position.pixels >=
        _scrollController.position.maxScrollExtent - 200) {
      _loadNextPage();
    }
  }

  Future<void> _loadFirstPage() async {
    setState(() {
      _loading = true;
      _error = null;
      _items = [];
      _hasMore = true;
      _loadingMore = false;
    });

    try {
      final nextId = await _proposalService.fetchNextProposalId();
      _knownNextId = nextId;

      if (nextId == 0) {
        if (!mounted) return;
        setState(() {
          _loading = false;
          _hasMore = false;
        });
        widget.onPendingVoteCountChanged?.call(0);
        return;
      }

      // 计算当前年份的起始 ID
      final year = nextId ~/ 1000000;
      final yearStartId = year * 1000000;

      final startId = nextId - 1; // 最新提案 ID
      final items = await _loadProposalRange(
          startId, (startId - _pageSize + 1).clamp(yearStartId, startId + 1));

      if (!mounted) return;
      setState(() {
        _items = items;
        _loadedUpTo = items.isNotEmpty
            ? items.last.proposal.meta.proposalId
            : yearStartId;
        _hasMore = _loadedUpTo > yearStartId;
        _loading = false;
      });

      _updatePendingVoteCount();
    } catch (e) {
      if (!mounted) return;
      setState(() {
        _error = SmoldotClientManager.instance.buildUserFacingError(e);
        _loading = false;
      });
      widget.onPendingVoteCountChanged?.call(0);
    }
  }

  Future<void> _loadNextPage() async {
    if (_loadingMore || !_hasMore) return;

    setState(() => _loadingMore = true);

    try {
      final year = _knownNextId ~/ 1000000;
      final yearStartId = year * 1000000;
      final startId = _loadedUpTo - 1;

      if (startId < yearStartId) {
        if (mounted) {
          setState(() {
            _hasMore = false;
            _loadingMore = false;
          });
        }
        return;
      }

      final endId = (startId - _pageSize + 1).clamp(yearStartId, startId + 1);
      final newItems = await _loadProposalRange(startId, endId);

      if (!mounted) return;
      setState(() {
        _items = [..._items, ...newItems];
        _loadedUpTo = newItems.isNotEmpty
            ? newItems.last.proposal.meta.proposalId
            : yearStartId;
        _hasMore = _loadedUpTo > yearStartId;
        _loadingMore = false;
      });

      _updatePendingVoteCount();
    } catch (e) {
      if (mounted) {
        setState(() => _loadingMore = false);
      }
    }
  }

  /// 加载 [startId] 到 [endId]（含 startId，不含 endId）的提案并生成 display items。
  Future<List<_ProposalDisplayItem>> _loadProposalRange(
      int startId, int endId) async {
    final count = startId - endId + 1;
    if (count <= 0) return const [];

    final proposals = await _proposalService.fetchProposalPage(startId, count);

    // 批量解析提案上下文
    final contexts = await _contextResolver.resolveBatch(
      proposals.map((p) => p.meta.institutionBytes?.toList()).toList(),
    );

    final items = <_ProposalDisplayItem>[];

    for (var i = 0; i < proposals.length; i++) {
      final p = proposals[i];
      final ctx = contexts[i];

      // 检查是否有未投票的钱包（统一使用 VoteChecker）
      bool needsVote = false;
      if (ctx.hasAdminWallets && p.meta.status == 0) {
        needsVote = await _voteChecker.hasUnvotedWallet(
          proposalId: p.meta.proposalId,
          kind: p.meta.kind,
          adminWallets: ctx.adminWallets,
          institution: ctx.institution,
        );
      }

      items.add(_ProposalDisplayItem(
        proposal: p,
        context: ctx,
        needsVote: needsVote,
      ));
    }

    return items;
  }

  void _updatePendingVoteCount() {
    _pendingVoteCount = _items.where((i) => i.needsVote).length;
    widget.onPendingVoteCountChanged?.call(_pendingVoteCount);
  }

  // ──── UI ────

  @override
  Widget build(BuildContext context) {
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
      return const Center(
        child: Column(
          mainAxisSize: MainAxisSize.min,
          children: [
            Icon(Icons.ballot_outlined, size: 48, color: AppTheme.textTertiary),
            SizedBox(height: 12),
            Text('暂无提案',
                style: TextStyle(fontSize: 16, color: AppTheme.textSecondary)),
            SizedBox(height: 4),
            Text('全链提案将在此显示',
                style: TextStyle(fontSize: 13, color: AppTheme.textTertiary)),
          ],
        ),
      );
    }

    return RefreshIndicator(
      onRefresh: () async {
        _adminService.clearCache();
        _contextResolver.clearWalletCache();
        ProposalCache.clear();
        await _loadFirstPage();
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
    final meta = item.proposal.meta;
    final inst = item.institution;
    final statusColor = _statusColor(meta.status);
    final statusLabel = _statusLabel(meta.status);
    final detail = item.proposal.transferDetail;
    final upgradeDetail = item.proposal.runtimeUpgradeDetail;
    final createDqDetail = item.proposal.createDuoqianDetail;
    final closeDqDetail = item.proposal.closeDuoqianDetail;
    final resIssuance = item.proposal.resolutionIssuanceSummary;
    final resDestroy = item.proposal.resolutionDestroySummary;

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
                  child: Icon(
                      _proposalIcon(detail, upgradeDetail, createDqDetail,
                          closeDqDetail, resIssuance, resDestroy),
                      size: 18,
                      color: statusColor),
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
                            formatProposalId(meta.proposalId),
                            style: const TextStyle(
                              fontSize: 15,
                              fontWeight: FontWeight.w600,
                              color: AppTheme.primaryDark,
                            ),
                          ),
                          const SizedBox(width: 8),
                          if (inst != null)
                            Container(
                              padding: const EdgeInsets.symmetric(
                                  horizontal: 6, vertical: 1),
                              decoration: BoxDecoration(
                                color: AppTheme.primaryDark
                                    .withValues(alpha: 0.08),
                                borderRadius: BorderRadius.circular(8),
                              ),
                              child: Text(
                                inst.name,
                                style: const TextStyle(
                                    fontSize: 10, color: AppTheme.primaryDark),
                                overflow: TextOverflow.ellipsis,
                              ),
                            ),
                        ],
                      ),
                      const SizedBox(height: 2),
                      Text(
                        detail != null
                            ? '转账 ${AmountFormat.format(detail.amountYuan, symbol: '')} 元'
                            : upgradeDetail != null
                                ? 'Runtime 升级'
                                : createDqDetail != null
                                    ? '创建多签 · ${createDqDetail.adminCount} 管理员'
                                    : closeDqDetail != null
                                        ? '关闭多签'
                                        : resIssuance != null
                                            ? '决议发行'
                                            : resDestroy != null
                                                ? '决议销毁'
                                                : meta.kind == 1
                                                    ? '联合投票提案'
                                                    : '提案 ${_kindLabel(meta.kind)}',
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
    TransferProposalInfo? detail,
    RuntimeUpgradeProposalInfo? upgradeDetail, [
    CreateDuoqianProposalInfo? createDqDetail,
    CloseDuoqianProposalInfo? closeDqDetail,
    String? resIssuance,
    String? resDestroy,
  ]) {
    if (detail != null) return Icons.send_outlined; // 转账
    if (upgradeDetail != null) return Icons.arrow_upward; // Runtime 升级
    if (createDqDetail != null) return Icons.group_add; // 创建多签
    if (closeDqDetail != null) return Icons.group_remove; // 关闭多签
    if (resIssuance != null) return Icons.add_circle_outline; // 决议发行
    if (resDestroy != null) return Icons.remove_circle_outline; // 决议销毁
    return Icons.description_outlined; // 其他/未知
  }

  String _kindLabel(int kind) {
    switch (kind) {
      case 0:
        return '内部投票';
      case 1:
        return '联合投票';
      default:
        return '';
    }
  }

  Future<void> _openProposalDetail(_ProposalDisplayItem item) async {
    final inst = item.institution;
    final proposalId = item.proposal.meta.proposalId;

    // Runtime 升级提案（联合投票，kind=1）
    if (item.proposal.runtimeUpgradeDetail != null) {
      await Navigator.of(context).push(
        MaterialPageRoute(
          builder: (_) => RuntimeUpgradeDetailPage(
            proposalId: proposalId,
            proposalContext: item.context,
          ),
        ),
      );
    } else if (item.proposal.transferDetail != null && inst != null) {
      // 转账提案
      await Navigator.of(context).push(
        MaterialPageRoute(
          builder: (_) => TransferProposalDetailPage(
            institution: inst,
            proposalId: proposalId,
            proposalContext: item.context,
          ),
        ),
      );
    } else if ((item.proposal.createDuoqianDetail != null ||
            item.proposal.closeDuoqianDetail != null) &&
        inst != null) {
      // 多签管理提案
      await Navigator.of(context).push(
        MaterialPageRoute(
          builder: (_) => DuoqianManageDetailPage(
            institution: inst,
            proposalId: proposalId,
            proposalContext: item.context,
          ),
        ),
      );
    } else if (item.proposal.safetyFundDetail != null && inst != null) {
      // 安全基金转账提案：复用 TransferProposalDetailPage，传 kind=safetyFund。
      // Phase 3 后管理员投票统一走 VotingEngine::internal_vote(9.0)。
      await Navigator.of(context).push(
        MaterialPageRoute(
          builder: (_) => TransferProposalDetailPage(
            institution: inst,
            proposalId: proposalId,
            proposalContext: item.context,
            kind: TransferProposalKind.safetyFund,
          ),
        ),
      );
    } else if (item.proposal.sweepDetail != null && inst != null) {
      // 手续费划转提案：kind=sweep。
      // Phase 3 后管理员投票统一走 VotingEngine::internal_vote(9.0)。
      await Navigator.of(context).push(
        MaterialPageRoute(
          builder: (_) => TransferProposalDetailPage(
            institution: inst,
            proposalId: proposalId,
            proposalContext: item.context,
            kind: TransferProposalKind.sweep,
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
      _loadFirstPage();
    }
  }
}

class _ProposalDisplayItem {
  const _ProposalDisplayItem({
    required this.proposal,
    required this.context,
    this.needsVote = false,
  });

  final ProposalWithDetail proposal;
  final ProposalContext context;
  final bool needsVote;

  InstitutionInfo? get institution => context.institution;
  List<WalletProfile> get adminWallets => context.adminWallets;
}
