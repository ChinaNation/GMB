// 个人多签详情页底部"提案列表"组件(req 5)。
//
// 数据双轨制:
// - 链上活跃提案(STATUS_VOTING):由 [PersonalProposalHistoryService.fetchAll] 实时拉
//   并同步到 Isar,保证其他设备发起的提案也被记录。
// - 历史提案(REJECTED / EXECUTED / EXECUTION_FAILED):链上 90 天后清理,
//   wuminapp 通过 Isar 永久保留。两者合并以 Isar 为准。

import 'package:flutter/material.dart';

import 'package:wuminapp_mobile/citizen/institution/institution_data.dart';
import 'package:wuminapp_mobile/citizen/shared/proposal_context.dart';
import 'package:wuminapp_mobile/duoqian/shared/duoqian_manage_detail_page.dart';
import 'package:wuminapp_mobile/ui/app_theme.dart';
import 'package:wuminapp_mobile/wallet/core/wallet_manager.dart';

import 'personal_proposal_history_service.dart';

class PersonalProposalListSection extends StatefulWidget {
  const PersonalProposalListSection({
    super.key,
    required this.institution,
    required this.adminWallets,
  });

  final InstitutionInfo institution;
  final List<WalletProfile> adminWallets;

  @override
  State<PersonalProposalListSection> createState() =>
      _PersonalProposalListSectionState();
}

class _PersonalProposalListSectionState
    extends State<PersonalProposalListSection> {
  final PersonalProposalHistoryService _service =
      PersonalProposalHistoryService();

  bool _loading = true;
  List<PersonalDuoqianProposalView> _items = const [];

  @override
  void initState() {
    super.initState();
    _load();
  }

  Future<void> _load() async {
    setState(() => _loading = true);
    final list = await _service.fetchAll(widget.institution.duoqianAddress);
    if (!mounted) return;
    setState(() {
      _loading = false;
      _items = list;
    });
  }

  Future<void> _openProposal(PersonalDuoqianProposalView view) async {
    // 历史(已终态)提案在链上可能已被 90 天清理,DuoqianManageDetailPage 以链上为准,
    // 终态后可能拉不到完整数据;但仍允许进入展示已知信息。
    final pushed = await Navigator.push<bool>(
      context,
      MaterialPageRoute(
        builder: (_) => DuoqianManageDetailPage(
          institution: widget.institution,
          proposalId: view.proposalId,
          proposalContext: ProposalContext(
            institution: widget.institution,
            adminWallets: widget.adminWallets,
            role: widget.adminWallets.isEmpty
                ? ProposalRole.viewer
                : ProposalRole.admin,
          ),
        ),
      ),
    );
    if (pushed == true && mounted) {
      // 投票/操作后可能改变了提案状态,重读
      await _load();
    }
  }

  @override
  Widget build(BuildContext context) {
    final activeItems = _items.where((v) => v.isActive).toList();
    final historyItems = _items.where((v) => v.isFinal).toList();

    return Card(
      elevation: 0,
      margin: EdgeInsets.zero,
      shape: RoundedRectangleBorder(
        borderRadius: BorderRadius.circular(12),
        side: const BorderSide(color: AppTheme.border),
      ),
      child: Padding(
        padding: const EdgeInsets.symmetric(vertical: 8),
        child: Column(
          crossAxisAlignment: CrossAxisAlignment.start,
          children: [
            const Padding(
              padding: EdgeInsets.fromLTRB(16, 8, 16, 4),
              child: Text(
                '该多签提案',
                style: TextStyle(
                  fontSize: 16,
                  fontWeight: FontWeight.w700,
                  color: AppTheme.primaryDark,
                ),
              ),
            ),
            const Divider(height: 1),
            if (_loading)
              const Padding(
                padding: EdgeInsets.symmetric(vertical: 24),
                child: Center(child: CircularProgressIndicator(strokeWidth: 2)),
              )
            else if (activeItems.isEmpty && historyItems.isEmpty)
              const Padding(
                padding: EdgeInsets.all(16),
                child: Text(
                  '暂无提案',
                  style: TextStyle(color: AppTheme.textTertiary),
                ),
              )
            else ...[
              if (activeItems.isNotEmpty) ...[
                _buildSubheader('进行中'),
                ...activeItems.map(_buildProposalTile),
              ],
              if (historyItems.isNotEmpty) ...[
                if (activeItems.isNotEmpty) const Divider(height: 1),
                _buildSubheader('历史'),
                ...historyItems.map(_buildProposalTile),
              ],
            ],
          ],
        ),
      ),
    );
  }

  Widget _buildSubheader(String label) {
    return Padding(
      padding: const EdgeInsets.fromLTRB(16, 12, 16, 4),
      child: Text(
        label,
        style: const TextStyle(
          fontSize: 12,
          fontWeight: FontWeight.w600,
          color: AppTheme.textSecondary,
        ),
      ),
    );
  }

  Widget _buildProposalTile(PersonalDuoqianProposalView view) {
    final actionLabel = _actionLabel(view.action);
    final statusLabel = _statusLabel(view.status);
    final statusColor = _statusColor(view.status);

    return ListTile(
      dense: true,
      leading: Container(
        width: 30,
        height: 30,
        alignment: Alignment.center,
        decoration: BoxDecoration(
          color: statusColor.withValues(alpha: 0.08),
          borderRadius: BorderRadius.circular(8),
        ),
        child: Icon(_actionIcon(view.action), size: 16, color: statusColor),
      ),
      title: Text(
        '$actionLabel · #${view.proposalId}',
        style: const TextStyle(fontSize: 13, fontWeight: FontWeight.w600),
      ),
      subtitle: Text(
        '$statusLabel · 赞成 ${view.yesVotes} / 反对 ${view.noVotes}',
        style: const TextStyle(fontSize: 11, color: AppTheme.textTertiary),
      ),
      trailing: const Icon(Icons.chevron_right,
          size: 18, color: AppTheme.textTertiary),
      onTap: () => _openProposal(view),
    );
  }

  String _actionLabel(String action) {
    switch (action) {
      case PersonalProposalAction.create:
        return '创建提案';
      case PersonalProposalAction.transfer:
        return '转账提案';
      case PersonalProposalAction.close:
        return '关闭提案';
      default:
        return '提案';
    }
  }

  IconData _actionIcon(String action) {
    switch (action) {
      case PersonalProposalAction.create:
        return Icons.fiber_new_outlined;
      case PersonalProposalAction.transfer:
        return Icons.swap_horiz;
      case PersonalProposalAction.close:
        return Icons.close;
      default:
        return Icons.description_outlined;
    }
  }

  String _statusLabel(String status) {
    switch (status) {
      case PersonalProposalStatus.voting:
        return '投票中';
      case PersonalProposalStatus.passed:
        return '已通过';
      case PersonalProposalStatus.rejected:
        return '已拒绝';
      case PersonalProposalStatus.executed:
        return '已执行';
      case PersonalProposalStatus.executionFailed:
        return '执行失败';
      default:
        return status;
    }
  }

  Color _statusColor(String status) {
    switch (status) {
      case PersonalProposalStatus.voting:
        return AppTheme.primaryDark;
      case PersonalProposalStatus.passed:
      case PersonalProposalStatus.executed:
        return AppTheme.success;
      case PersonalProposalStatus.rejected:
      case PersonalProposalStatus.executionFailed:
        return AppTheme.danger;
      default:
        return AppTheme.textTertiary;
    }
  }
}
