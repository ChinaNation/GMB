import 'package:flutter/material.dart';
import 'package:isar/isar.dart';
import 'package:wuminapp_mobile/ui/app_theme.dart';

import '../../Isar/wallet_isar.dart';
import '../../governance/duoqian_institution_list_page.dart';
import '../../governance/duoqian_manage_detail_page.dart';
import '../../governance/institution_admin_service.dart';
import '../../governance/institution_data.dart';
import '../../governance/proposal_context.dart';
import '../../governance/transfer_proposal_detail_page.dart';
import '../../governance/transfer_proposal_page.dart';
import '../../governance/transfer_proposal_service.dart';
import '../../util/amount_format.dart';
import '../../wallet/core/wallet_manager.dart';

/// 多签交易主页。
///
/// 显示用户所有多签机构的提案记录列表，右上角 "+" 选择机构发起转账。
class DuoqianTradePage extends StatefulWidget {
  const DuoqianTradePage({super.key});

  @override
  State<DuoqianTradePage> createState() => _DuoqianTradePageState();
}

class _DuoqianTradePageState extends State<DuoqianTradePage> {
  final TransferProposalService _proposalService = TransferProposalService();

  bool _loading = true;
  String? _error;
  List<_ProposalItem> _proposals = [];

  @override
  void initState() {
    super.initState();
    _load();
  }

  Future<void> _load() async {
    setState(() {
      _loading = true;
      _error = null;
    });

    try {
      // 1. 读取本地所有多签账户（机构 + 个人）
      final isar = await WalletIsar.instance.db();
      final institutions = await isar.duoqianInstitutionEntitys
          .where()
          .findAll();
      final personals = await isar.personalDuoqianEntitys
          .where()
          .findAll();

      if (institutions.isEmpty && personals.isEmpty) {
        if (!mounted) return;
        setState(() {
          _proposals = [];
          _loading = false;
        });
        return;
      }

      // 2. 对每个多签账户查询提案
      final allProposals = <_ProposalItem>[];
      final seen = <int>{};

      // 机构多签
      for (final entity in institutions) {
        final inst = InstitutionInfo(
          name: entity.name,
          shenfenId: registeredDuoqianIdentity(entity.duoqianAddress),
          orgType: OrgType.duoqian,
          duoqianAddress: entity.duoqianAddress,
        );
        await _queryProposals(inst, allProposals, seen);
      }

      // 个人多签
      for (final entity in personals) {
        final inst = InstitutionInfo(
          name: entity.name,
          shenfenId: 'personal:${entity.duoqianAddress}',
          orgType: OrgType.duoqian,
          duoqianAddress: entity.duoqianAddress,
        );
        await _queryProposals(inst, allProposals, seen);
      }

      // 3. 按 proposalId 倒序
      allProposals.sort(
          (a, b) => b.proposal.meta.proposalId.compareTo(a.proposal.meta.proposalId));

      if (!mounted) return;
      setState(() {
        _proposals = allProposals;
        _loading = false;
      });
    } catch (e) {
      if (!mounted) return;
      setState(() {
        _error = e.toString();
        _loading = false;
      });
    }
  }

  // ──── 选择机构发起交易 ────

  Future<void> _queryProposals(
    InstitutionInfo inst,
    List<_ProposalItem> allProposals,
    Set<int> seen,
  ) async {
    try {
      final proposals = await _proposalService
          .fetchInstitutionVisibleProposals(inst.shenfenId);
      for (final p in proposals) {
        if (seen.add(p.meta.proposalId)) {
          allProposals.add(_ProposalItem(proposal: p, institution: inst));
        }
      }
    } catch (_) {
      // 单个账户查询失败不影响整体
    }
  }

  Future<void> _selectInstitutionAndTrade() async {
    final selected = await Navigator.push<InstitutionInfo>(
      context,
      MaterialPageRoute(
        builder: (_) =>
            const DuoqianInstitutionListPage(mode: InstitutionListMode.select),
      ),
    );
    if (selected == null || !mounted) return;

    // 获取管理员钱包
    final wallets = await _getAdminWallets(selected);
    if (!mounted || wallets.isEmpty) {
      if (mounted) {
        ScaffoldMessenger.of(context).showSnackBar(
          const SnackBar(content: Text('未找到此机构的管理员钱包')),
        );
      }
      return;
    }

    final created = await Navigator.push<bool>(
      context,
      MaterialPageRoute(
        builder: (_) => TransferProposalPage(
          institution: selected,
          icon: Icons.groups,
          badgeColor: AppTheme.primaryDark,
          adminWallets: wallets,
        ),
      ),
    );
    if (created == true && mounted) {
      _load();
    }
  }

  Future<List<WalletProfile>> _getAdminWallets(InstitutionInfo inst) async {
    final adminService = InstitutionAdminService();
    final admins = await adminService.fetchAdmins(inst.shenfenId);
    final adminSet = admins.toSet();
    final wm = WalletManager();
    final wallets = await wm.getWallets();
    return wallets.where((w) {
      var pk = w.pubkeyHex.toLowerCase();
      if (pk.startsWith('0x')) pk = pk.substring(2);
      return adminSet.contains(pk);
    }).toList();
  }

  // ──── 提案点击 ────

  Future<void> _openProposalDetail(_ProposalItem item) async {
    final proposalId = item.proposal.meta.proposalId;
    final inst = item.institution;

    // 构建提案上下文
    final wallets = await _getAdminWallets(inst);
    final ctx = ProposalContext(
      institution: inst,
      adminWallets: wallets,
      role: wallets.isNotEmpty ? ProposalRole.admin : ProposalRole.viewer,
    );

    if (!mounted) return;

    if (item.proposal.transferDetail != null) {
      await Navigator.push(
        context,
        MaterialPageRoute(
          builder: (_) => TransferProposalDetailPage(
            institution: inst,
            proposalId: proposalId,
            proposalContext: ctx,
          ),
        ),
      );
    } else if (item.proposal.createDuoqianDetail != null ||
        item.proposal.closeDuoqianDetail != null) {
      await Navigator.push(
        context,
        MaterialPageRoute(
          builder: (_) => DuoqianManageDetailPage(
            institution: inst,
            proposalId: proposalId,
            proposalContext: ctx,
          ),
        ),
      );
    } else {
      ScaffoldMessenger.of(context).showSnackBar(
        const SnackBar(content: Text('暂不支持查看此提案类型')),
      );
      return;
    }

    if (mounted) _load();
  }

  // ──── UI ────

  @override
  Widget build(BuildContext context) {
    return Scaffold(
      appBar: AppBar(
        title: const Text(
          '多签交易',
          style: TextStyle(fontSize: 17, fontWeight: FontWeight.w700),
        ),
        actions: [
          IconButton(
            icon: const Icon(Icons.add),
            onPressed: _selectInstitutionAndTrade,
          ),
        ],
      ),
      body: _loading
          ? const Center(child: CircularProgressIndicator())
          : _error != null
              ? _buildError()
              : _proposals.isEmpty
                  ? _buildEmpty()
                  : _buildList(),
    );
  }

  Widget _buildError() {
    return Center(
      child: Padding(
        padding: const EdgeInsets.all(24),
        child: Column(
          mainAxisSize: MainAxisSize.min,
          children: [
            const Icon(Icons.error_outline, size: 48, color: AppTheme.danger),
            const SizedBox(height: 12),
            Text(_error!,
                style: const TextStyle(fontSize: 12, color: AppTheme.textSecondary),
                textAlign: TextAlign.center),
            const SizedBox(height: 16),
            OutlinedButton(onPressed: _load, child: const Text('重试')),
          ],
        ),
      ),
    );
  }

  Widget _buildEmpty() {
    return const Center(
      child: Column(
        mainAxisSize: MainAxisSize.min,
        children: [
          Icon(Icons.receipt_long, size: 64, color: AppTheme.border),
          SizedBox(height: 12),
          Text(
            '暂无多签交易记录',
            style: TextStyle(fontSize: 16, color: AppTheme.textSecondary),
          ),
          SizedBox(height: 6),
          Text(
            '点击右上角 + 发起多签转账',
            style: TextStyle(fontSize: 13, color: AppTheme.textTertiary),
          ),
        ],
      ),
    );
  }

  Widget _buildList() {
    return RefreshIndicator(
      onRefresh: _load,
      child: ListView.separated(
        padding: const EdgeInsets.fromLTRB(16, 8, 16, 32),
        itemCount: _proposals.length,
        separatorBuilder: (_, __) => const SizedBox(height: 8),
        itemBuilder: (_, index) => _buildProposalCard(_proposals[index]),
      ),
    );
  }

  Widget _buildProposalCard(_ProposalItem item) {
    final meta = item.proposal.meta;
    final statusColor = _statusColor(meta.status);
    final statusLabel = _statusLabel(meta.status);

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
              Container(
                width: 36,
                height: 36,
                decoration: BoxDecoration(
                  color: statusColor.withValues(alpha: 0.10),
                  borderRadius: BorderRadius.circular(10),
                ),
                child: Icon(_proposalIcon(item.proposal),
                    size: 18, color: statusColor),
              ),
              const SizedBox(width: 12),
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
                            color: AppTheme.textPrimary,
                          ),
                        ),
                        const SizedBox(width: 8),
                        Container(
                          padding: const EdgeInsets.symmetric(
                              horizontal: 6, vertical: 1),
                          decoration: BoxDecoration(
                            color: AppTheme.primaryDark.withValues(alpha: 0.08),
                            borderRadius: BorderRadius.circular(8),
                          ),
                          child: Text(
                            item.institution.name,
                            style: const TextStyle(
                                fontSize: 10, color: AppTheme.primaryDark),
                            overflow: TextOverflow.ellipsis,
                          ),
                        ),
                      ],
                    ),
                    const SizedBox(height: 2),
                    Text(
                      _proposalSubtitle(item.proposal),
                      style: const TextStyle(fontSize: 12, color: AppTheme.textSecondary),
                    ),
                  ],
                ),
              ),
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
              const SizedBox(width: 4),
              const Icon(Icons.chevron_right, size: 20, color: AppTheme.textTertiary),
            ],
          ),
        ),
      ),
    );
  }

  // ──── 工具 ────

  IconData _proposalIcon(ProposalWithDetail proposal) {
    if (proposal.transferDetail != null) return Icons.send_outlined;
    if (proposal.createDuoqianDetail != null) return Icons.group_add;
    if (proposal.closeDuoqianDetail != null) return Icons.group_remove;
    return Icons.description_outlined;
  }

  String _proposalSubtitle(ProposalWithDetail proposal) {
    final transfer = proposal.transferDetail;
    if (transfer != null) {
      return '转账 ${AmountFormat.format(transfer.amountYuan, symbol: '')} 元';
    }
    final create = proposal.createDuoqianDetail;
    if (create != null) {
      return '创建多签 · ${create.adminCount} 管理员';
    }
    if (proposal.closeDuoqianDetail != null) {
      return '关闭多签';
    }
    return '提案';
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
    return AppTheme.proposalStatusColor(status);
  }
}

class _ProposalItem {
  const _ProposalItem({
    required this.proposal,
    required this.institution,
  });

  final ProposalWithDetail proposal;
  final InstitutionInfo institution;
}
