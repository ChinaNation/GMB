import 'dart:async';

import 'package:flutter/material.dart';

import '../rpc/chain_event_subscription.dart';
import '../wallet/core/wallet_manager.dart';
import 'institution_admin_service.dart';
import 'institution_data.dart';
import 'proposal_cache.dart';
import 'transfer_proposal_detail_page.dart';
import 'transfer_proposal_service.dart';

/// 全局提案列表：展示全链所有提案，按 ID 倒序，标注投票状态和红点。
///
/// 采用四层优化架构：
/// 1. 本地内存缓存（ProposalCache）
/// 2. 批量查询（fetchStorageBatch）
/// 3. 分页加载（ScrollController 滚动触发）
/// 4. WebSocket 订阅（新区块自动检测新提案）
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
  static const Color _inkGreen = Color(0xFF0B3D2E);
  static const int _pageSize = 10;

  final TransferProposalService _proposalService = TransferProposalService();
  final InstitutionAdminService _adminService = InstitutionAdminService();
  final WalletManager _walletManager = WalletManager();
  final ScrollController _scrollController = ScrollController();

  // WebSocket 订阅
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

  /// 缓存的钱包列表（首次加载后缓存）。
  List<WalletProfile>? _wallets;

  /// 待投票计数。
  int _pendingVoteCount = 0;

  @override
  void initState() {
    super.initState();
    _scrollController.addListener(_onScroll);
    _loadFirstPage();
    _startWebSocket();
  }

  @override
  void dispose() {
    _scrollController.dispose();
    _eventSub?.cancel();
    _subscription?.disconnect();
    super.dispose();
  }

  // ──── WebSocket ────

  void _startWebSocket() {
    _subscription = ChainEventSubscription();
    _subscription!.connect(_proposalService.rpcNodeUrl);
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
        final newItems = await _loadProposalRange(
            newNextId - 1, _knownNextId);
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
      final results = await Future.wait([
        _proposalService.fetchNextProposalId(),
        _walletManager.getWallets(),
      ]);
      final nextId = results[0] as int;
      _wallets = results[1] as List<WalletProfile>;
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
      final items = await _loadProposalRange(startId,
          (startId - _pageSize + 1).clamp(yearStartId, startId + 1));

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
        _error = e.toString();
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
        if (mounted) setState(() { _hasMore = false; _loadingMore = false; });
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

    final proposals =
        await _proposalService.fetchProposalPage(startId, count);

    final wallets = _wallets ?? [];
    final items = <_ProposalDisplayItem>[];

    for (final p in proposals) {
      final inst = p.meta.institutionBytes != null
          ? findInstitutionByPalletId(p.meta.institutionBytes!.toList())
          : null;

      // 查管理员列表
      List<String> admins = const [];
      if (inst != null) {
        try {
          admins = await _adminService.fetchAdmins(inst.shenfenId);
        } catch (_) {}
      }

      // 匹配当前用户的管理员钱包
      final matchedWallets = <WalletProfile>[];
      for (final w in wallets) {
        var pk = w.pubkeyHex.toLowerCase();
        if (pk.startsWith('0x')) pk = pk.substring(2);
        if (admins.contains(pk)) {
          matchedWallets.add(w);
        }
      }

      // 检查是否有未投票的钱包
      bool needsVote = false;
      if (matchedWallets.isNotEmpty && p.meta.status == 0) {
        for (final w in matchedWallets) {
          var pk = w.pubkeyHex.toLowerCase();
          if (pk.startsWith('0x')) pk = pk.substring(2);
          final vote = await _proposalService.fetchAdminVote(
              p.meta.proposalId, pk);
          if (vote == null) {
            needsVote = true;
            break;
          }
        }
      }

      items.add(_ProposalDisplayItem(
        proposal: p,
        institution: inst,
        adminWallets: matchedWallets,
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
      return const Center(child: CircularProgressIndicator());
    }
    if (_error != null) {
      return Center(
        child: Padding(
          padding: const EdgeInsets.all(24),
          child: Column(
            mainAxisSize: MainAxisSize.min,
            children: [
              const Icon(Icons.error_outline, size: 48, color: Colors.red),
              const SizedBox(height: 12),
              Text('加载失败',
                  style: TextStyle(fontSize: 16, color: Colors.grey[700])),
              const SizedBox(height: 6),
              Text(_error!,
                  style: TextStyle(fontSize: 12, color: Colors.grey[500]),
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
      return Center(
        child: Column(
          mainAxisSize: MainAxisSize.min,
          children: [
            Icon(Icons.ballot_outlined, size: 48, color: Colors.grey[400]),
            const SizedBox(height: 12),
            Text('暂无提案',
                style: TextStyle(fontSize: 16, color: Colors.grey[500])),
            const SizedBox(height: 4),
            Text('全链提案将在此显示',
                style: TextStyle(fontSize: 13, color: Colors.grey[400])),
          ],
        ),
      );
    }

    return RefreshIndicator(
      onRefresh: () async {
        _adminService.clearCache();
        ProposalCache.clear();
        await _loadFirstPage();
      },
      child: ListView.separated(
        controller: _scrollController,
        padding: const EdgeInsets.fromLTRB(16, 16, 16, 32),
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

    return Card(
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
                    Icon(_proposalIcon(detail), size: 18, color: statusColor),
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
                            color: _inkGreen,
                          ),
                        ),
                        const SizedBox(width: 8),
                        if (inst != null)
                          Container(
                            padding: const EdgeInsets.symmetric(
                                horizontal: 6, vertical: 1),
                            decoration: BoxDecoration(
                              color: _inkGreen.withValues(alpha: 0.08),
                              borderRadius: BorderRadius.circular(8),
                            ),
                            child: Text(
                              inst.name,
                              style: const TextStyle(
                                  fontSize: 10, color: _inkGreen),
                              overflow: TextOverflow.ellipsis,
                            ),
                          ),
                      ],
                    ),
                    const SizedBox(height: 2),
                    Text(
                      detail != null
                          ? '转账 ${detail.amountYuan.toStringAsFixed(2)} 元'
                          : '提案 ${_kindLabel(meta.kind)}',
                      style: TextStyle(fontSize: 12, color: Colors.grey[500]),
                    ),
                  ],
                ),
              ),
              // 右侧状态 + 红点
              Column(
                crossAxisAlignment: CrossAxisAlignment.end,
                children: [
                  Container(
                    padding:
                        const EdgeInsets.symmetric(horizontal: 8, vertical: 2),
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
                        color: Colors.red,
                        shape: BoxShape.circle,
                      ),
                    ),
                  ],
                ],
              ),
              const SizedBox(width: 4),
              Icon(Icons.chevron_right, size: 20, color: Colors.grey[400]),
            ],
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
      default:
        return '未知';
    }
  }

  Color _statusColor(int status) {
    switch (status) {
      case 0:
        return Colors.blue;
      case 1:
        return Colors.green;
      case 2:
        return Colors.red;
      default:
        return Colors.grey;
    }
  }

  /// 根据提案类型返回图标。
  IconData _proposalIcon(TransferProposalInfo? detail) {
    if (detail != null) return Icons.send_outlined; // 转账
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
    if (inst == null || item.proposal.transferDetail == null) {
      // 非转账提案或未知机构，暂不支持详情
      ScaffoldMessenger.of(context).showSnackBar(
        const SnackBar(content: Text('该提案类型的详情页面正在开发中')),
      );
      return;
    }

    await Navigator.of(context).push(
      MaterialPageRoute(
        builder: (_) => TransferProposalDetailPage(
          institution: inst,
          proposalId: item.proposal.meta.proposalId,
          adminWallets: item.adminWallets,
        ),
      ),
    );

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
    this.institution,
    this.adminWallets = const [],
    this.needsVote = false,
  });

  final ProposalWithDetail proposal;
  final InstitutionInfo? institution;
  final List<WalletProfile> adminWallets;
  final bool needsVote;
}
