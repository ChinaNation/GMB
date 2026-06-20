// 个人多签详情页底部"提案列表"组件(req 5)。
//
// 数据双轨制:
// - 链上活跃提案(STATUS_VOTING):由 [PersonalProposalHistoryService.fetchAll] 实时拉
//   并同步到 Isar,保证其他设备发起的提案也被记录。
// - 历史提案(REJECTED / EXECUTED / EXECUTION_FAILED):链上 90 天后清理,
//   wuminapp 通过 Isar 永久保留。两者合并以 Isar 为准。
//
// **样式**(2026-05-03 bug 3 整改):每条提案是**独立方块 Card**(对齐治理机构详情页
// `_buildProposalCard:520`),带 statusColor 边框 + 36×36 状态图标 + 标题/子标题 +
// 右侧状态徽章。Card 之间 8px 间距。

import 'package:flutter/material.dart';

import 'package:wuminapp_mobile/governance/shared/institution_info.dart';
import 'package:wuminapp_mobile/governance/shared/proposal/proposal_context.dart';
import 'package:wuminapp_mobile/governance/institution_manage_detail_page.dart';
import 'package:wuminapp_mobile/transaction/duoqian-transfer/duoqian_transfer_detail_page.dart';
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
    final list = await _service.fetchAll(widget.institution.duoqianAccount);
    if (!mounted) return;
    setState(() {
      _loading = false;
      _items = list;
    });
  }

  Future<void> _openProposal(PersonalDuoqianProposalView view) async {
    // 历史(已终态)提案在链上可能已被 90 天清理,详情页以链上为准,
    // 终态后可能拉不到完整数据;但仍允许进入展示已知信息。
    //
    // 按 view.action 分流到对应详情页:
    // - transfer → DuoqianTransferDetailPage(转账提案专用页)
    // - create / close → InstitutionManageDetailPage(多签管理提案,只懂 create/close)
    final ctx = ProposalContext(
      institution: widget.institution,
      adminWallets: widget.adminWallets,
      role: widget.adminWallets.isEmpty
          ? ProposalRole.viewer
          : ProposalRole.admin,
    );
    final Widget page;
    if (view.action == PersonalProposalAction.transfer) {
      page = DuoqianTransferDetailPage(
        institution: widget.institution,
        proposalId: view.proposalId,
        proposalContext: ctx,
      );
    } else {
      page = InstitutionManageDetailPage(
        institution: widget.institution,
        proposalId: view.proposalId,
        proposalContext: ctx,
      );
    }
    final pushed = await Navigator.push<bool>(
      context,
      MaterialPageRoute(builder: (_) => page),
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

    return Column(
      crossAxisAlignment: CrossAxisAlignment.start,
      children: [
        const Text(
          '提案列表',
          style: TextStyle(
            fontSize: 16,
            fontWeight: FontWeight.w700,
            color: AppTheme.primaryDark,
          ),
        ),
        const SizedBox(height: 12),
        if (_loading)
          Container(
            width: double.infinity,
            padding: const EdgeInsets.all(24),
            decoration: BoxDecoration(
              color: AppTheme.surfaceMuted,
              borderRadius: BorderRadius.circular(AppTheme.radiusMd),
              border: Border.all(color: AppTheme.border),
            ),
            child: const Center(
              child: CircularProgressIndicator(strokeWidth: 2),
            ),
          )
        else if (activeItems.isEmpty && historyItems.isEmpty)
          Container(
            width: double.infinity,
            padding: const EdgeInsets.all(24),
            decoration: BoxDecoration(
              color: AppTheme.surfaceMuted,
              borderRadius: BorderRadius.circular(AppTheme.radiusMd),
              border: Border.all(color: AppTheme.border),
            ),
            child: const Column(
              children: [
                Icon(Icons.ballot_outlined,
                    size: 40, color: AppTheme.textTertiary),
                SizedBox(height: 8),
                Text(
                  '暂无提案',
                  style: TextStyle(fontSize: 14, color: AppTheme.textSecondary),
                ),
              ],
            ),
          )
        else ...[
          if (activeItems.isNotEmpty) ...[
            _buildSubheader('进行中'),
            ...activeItems.map((v) => Padding(
                  padding: const EdgeInsets.only(bottom: 8),
                  child: _buildProposalCard(v),
                )),
          ],
          if (historyItems.isNotEmpty) ...[
            _buildSubheader('历史'),
            ...historyItems.map((v) => Padding(
                  padding: const EdgeInsets.only(bottom: 8),
                  child: _buildProposalCard(v),
                )),
          ],
        ],
      ],
    );
  }

  Widget _buildSubheader(String label) {
    return Padding(
      padding: const EdgeInsets.only(bottom: 8),
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

  /// 提案方块卡片(对齐治理机构详情页 `institution_detail_page._buildProposalCard:520`):
  /// Card with statusColor border(alpha 0.2) + 36×36 statusColor icon container +
  /// 标题/子标题 + 右侧状态徽章。
  Widget _buildProposalCard(PersonalDuoqianProposalView view) {
    final statusColor = _statusColor(view.status);
    return Card(
      elevation: 0,
      margin: EdgeInsets.zero,
      shape: RoundedRectangleBorder(
        borderRadius: BorderRadius.circular(12),
        side: BorderSide(color: statusColor.withValues(alpha: 0.2)),
      ),
      child: InkWell(
        onTap: () => _openProposal(view),
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
                child: Icon(_actionIcon(view.action),
                    size: 18, color: statusColor),
              ),
              const SizedBox(width: 12),
              Expanded(
                child: Column(
                  crossAxisAlignment: CrossAxisAlignment.start,
                  children: [
                    Text(
                      '${_actionLabel(view.action)} · #${view.proposalId}',
                      style: const TextStyle(
                        fontSize: 15,
                        fontWeight: FontWeight.w600,
                        color: AppTheme.primaryDark,
                      ),
                    ),
                    const SizedBox(height: 2),
                    Text(
                      '赞成 ${view.yesVotes} · 反对 ${view.noVotes}',
                      style: const TextStyle(
                          fontSize: 12, color: AppTheme.textTertiary),
                    ),
                  ],
                ),
              ),
              Container(
                padding: const EdgeInsets.symmetric(horizontal: 8, vertical: 2),
                decoration: BoxDecoration(
                  color: statusColor.withValues(alpha: 0.1),
                  borderRadius: BorderRadius.circular(10),
                ),
                child: Text(
                  _statusLabel(view.status),
                  style: TextStyle(
                    fontSize: 11,
                    fontWeight: FontWeight.w600,
                    color: statusColor,
                  ),
                ),
              ),
            ],
          ),
        ),
      ),
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
