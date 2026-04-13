import 'package:flutter/material.dart';
import 'package:smoldot/smoldot.dart' show LightClientStatusSnapshot;
import '../ui/app_theme.dart';
import '../ui/widgets/chain_progress_banner.dart';

import 'duoqian_close_proposal_page.dart';
import 'duoqian_create_proposal_page.dart';
import 'institution_data.dart';
import 'runtime_upgrade_page.dart';
import 'transfer_proposal_page.dart';
import 'transfer_proposal_service.dart';
import '../rpc/smoldot_client.dart';
import '../wallet/core/wallet_manager.dart';

/// 提案类型选择页。
///
/// 根据机构类型条件显示可发起的提案类型：
/// - 所有机构：转账、换管理员、决议销毁
/// - 国储会 + 省储会（NRC/PRC）：决议发行、状态升级、验证密钥
/// - 仅国储会（NRC）：安全基金转账
class ProposalTypesPage extends StatefulWidget {
  const ProposalTypesPage({
    super.key,
    required this.institution,
    required this.icon,
    required this.badgeColor,
    required this.adminWallets,
    required this.isActivated,
  });

  final InstitutionInfo institution;
  final IconData icon;
  final Color badgeColor;

  /// 当前用户已激活的管理员钱包列表。
  final List<WalletProfile> adminWallets;

  /// 用户是否已激活管理员身份。
  final bool isActivated;

  @override
  State<ProposalTypesPage> createState() => _ProposalTypesPageState();
}

class _ProposalTypesPageState extends State<ProposalTypesPage> {
  LightClientStatusSnapshot? _chainProgress;
  String? _chainProgressError;

  @override
  Widget build(BuildContext context) {
    final proposalActionsEnabled =
        widget.isActivated && _proposalBlockedReason == null;

    return Scaffold(
      backgroundColor: Colors.white,
      appBar: AppBar(
        title: const Text(
          '发起提案',
          style: TextStyle(fontSize: 17, fontWeight: FontWeight.w700),
        ),
        centerTitle: true,
        backgroundColor: Colors.white,
        foregroundColor: AppTheme.primaryDark,
        elevation: 0,
        scrolledUnderElevation: 0.5,
      ),
      body: ListView(
        padding: const EdgeInsets.fromLTRB(16, 8, 16, 32),
        children: [
          ChainProgressBanner(
            onProgressChanged: _handleChainProgressChanged,
            onErrorChanged: _handleChainProgressErrorChanged,
          ),
          // 机构信息
          Padding(
            padding: const EdgeInsets.only(bottom: 16),
            child: Row(
              children: [
                Container(
                  width: 36,
                  height: 36,
                  decoration: BoxDecoration(
                    color: widget.badgeColor.withValues(alpha: 0.12),
                    borderRadius: BorderRadius.circular(10),
                  ),
                  child: Icon(widget.icon, size: 18, color: widget.badgeColor),
                ),
                const SizedBox(width: 10),
                Expanded(
                  child: Text(
                    widget.institution.name,
                    style: const TextStyle(
                      fontSize: 15,
                      fontWeight: FontWeight.w600,
                      color: AppTheme.primaryDark,
                    ),
                  ),
                ),
                Container(
                  padding:
                      const EdgeInsets.symmetric(horizontal: 8, vertical: 2),
                  decoration: BoxDecoration(
                    color: widget.badgeColor.withValues(alpha: 0.10),
                    borderRadius: BorderRadius.circular(10),
                  ),
                  child: Text(
                    OrgType.label(widget.institution.orgType),
                    style: TextStyle(
                      fontSize: 11,
                      color: widget.badgeColor,
                      fontWeight: FontWeight.w600,
                    ),
                  ),
                ),
              ],
            ),
          ),

          // ──── 非管理员提示 ────
          if (!widget.isActivated)
            Padding(
              padding: const EdgeInsets.only(bottom: 16),
              child: Container(
                padding:
                    const EdgeInsets.symmetric(horizontal: 12, vertical: 8),
                decoration: BoxDecoration(
                  color: AppTheme.textTertiary.withValues(alpha: 0.08),
                  borderRadius: BorderRadius.circular(10),
                  border: Border.all(
                    color: AppTheme.textTertiary.withValues(alpha: 0.2),
                  ),
                ),
                child: const Row(
                  children: [
                    Icon(Icons.info_outline,
                        size: 16, color: AppTheme.textTertiary),
                    SizedBox(width: 8),
                    Expanded(
                      child: Text(
                        '仅管理员可发起提案，请先在管理员列表中激活身份',
                        style: TextStyle(
                          fontSize: 12,
                          color: AppTheme.textTertiary,
                        ),
                      ),
                    ),
                  ],
                ),
              ),
            ),
          if (widget.isActivated && _proposalBlockedReason != null)
            Padding(
              padding: const EdgeInsets.only(bottom: 16),
              child: Container(
                padding:
                    const EdgeInsets.symmetric(horizontal: 12, vertical: 8),
                decoration: BoxDecoration(
                  color: AppTheme.warning.withValues(alpha: 0.08),
                  borderRadius: BorderRadius.circular(10),
                  border: Border.all(
                    color: AppTheme.warning.withValues(alpha: 0.2),
                  ),
                ),
                child: Row(
                  children: [
                    const Icon(Icons.sync_problem,
                        size: 16, color: AppTheme.warning),
                    const SizedBox(width: 8),
                    Expanded(
                      child: Text(
                        _proposalBlockedReason!,
                        style: const TextStyle(
                          fontSize: 12,
                          color: AppTheme.textSecondary,
                          height: 1.4,
                        ),
                      ),
                    ),
                  ],
                ),
              ),
            ),

          // ──── 通用提案类型（所有机构） ────
          _buildSectionTitle('通用提案'),
          const SizedBox(height: 8),
          _ProposalTypeCard(
            icon: Icons.send_outlined,
            title: '转账',
            subtitle: '从机构多签账户发起转账',
            color: AppTheme.primary,
            enabled: proposalActionsEnabled,
            onTap: () => _checkAndOpenProposal(
                context,
                () => TransferProposalPage(
                      institution: widget.institution,
                      icon: widget.icon,
                      badgeColor: widget.badgeColor,
                      adminWallets: widget.adminWallets,
                    )),
          ),
          const SizedBox(height: 8),
          _ProposalTypeCard(
            icon: Icons.swap_horiz,
            title: '换管理员',
            subtitle: '提议更换本机构管理员',
            color: AppTheme.accent,
            enabled: proposalActionsEnabled,
            onTap: () => _checkAndOpenProposal(context, null, name: '换管理员'),
          ),
          const SizedBox(height: 8),
          _ProposalTypeCard(
            icon: Icons.delete_outline,
            title: '决议销毁',
            subtitle: '提议销毁机构持有的资产',
            color: AppTheme.danger,
            enabled: proposalActionsEnabled,
            onTap: () => _checkAndOpenProposal(context, null, name: '决议销毁'),
          ),

          // ──── 注册多签机构专属提案类型 ────
          if (widget.institution.orgType == OrgType.duoqian) ...[
            const SizedBox(height: 20),
            _buildSectionTitle('多签管理'),
            const SizedBox(height: 8),
            _ProposalTypeCard(
              icon: Icons.group_add,
              title: '创建多签',
              subtitle: '发起创建多签账户提案',
              color: AppTheme.info,
              enabled: proposalActionsEnabled,
              onTap: () => _checkAndOpenProposal(
                context,
                () => DuoqianCreateProposalPage(
                  institution: widget.institution,
                  adminWallets: widget.adminWallets,
                ),
              ),
            ),
            const SizedBox(height: 8),
            _ProposalTypeCard(
              icon: Icons.group_remove,
              title: '关闭多签',
              subtitle: '发起关闭多签账户提案，资金转入指定受益人',
              color: AppTheme.danger,
              enabled: proposalActionsEnabled,
              onTap: () => _checkAndOpenProposal(
                context,
                () => DuoqianCloseProposalPage(
                  institution: widget.institution,
                  adminWallets: widget.adminWallets,
                ),
              ),
            ),
          ],

          // ──── 联合投票提案（国储会 + 省储会可发起）────
          if (widget.institution.orgType == OrgType.nrc ||
              widget.institution.orgType == OrgType.prc) ...[
            const SizedBox(height: 20),
            _buildSectionTitle('联合投票提案'),
            const SizedBox(height: 8),
            _ProposalTypeCard(
              icon: Icons.account_balance,
              title: '决议发行',
              subtitle: '发起公民币发行决议，需联合投票+公民投票',
              color: AppTheme.primaryDark,
              enabled: proposalActionsEnabled,
              onTap: () => _checkAndOpenProposal(context, null, name: '决议发行'),
            ),
            const SizedBox(height: 8),
            _ProposalTypeCard(
              icon: Icons.arrow_upward,
              title: '状态升级',
              subtitle: 'Runtime 升级，需联合投票+公民投票',
              color: AppTheme.info,
              enabled: proposalActionsEnabled,
              onTap: () => _checkAndOpenProposal(
                context,
                () => RuntimeUpgradePage(adminWallets: widget.adminWallets),
              ),
            ),
            const SizedBox(height: 8),
            _ProposalTypeCard(
              icon: Icons.vpn_key_outlined,
              title: '验证密钥',
              subtitle: '更换 GRANDPA 共识验证密钥（本机构内部投票）',
              color: const Color(0xFF4527A0),
              enabled: proposalActionsEnabled,
              onTap: () => _checkAndOpenProposal(context, null, name: '验证密钥'),
            ),
          ],

          // ──── 国储会专属提案 ────
          if (widget.institution.orgType == OrgType.nrc) ...[
            const SizedBox(height: 20),
            _buildSectionTitle('国储会专属提案'),
            const SizedBox(height: 8),
            _ProposalTypeCard(
              icon: Icons.shield_outlined,
              title: '安全基金转账',
              subtitle: '从安全基金账户向指定地址转账',
              color: AppTheme.info,
              enabled: proposalActionsEnabled,
              onTap: () => _checkAndOpenProposal(context, null, name: '安全基金转账'),
            ),
          ],
        ],
      ),
    );
  }

  Widget _buildSectionTitle(String title) {
    return Text(
      title,
      style: const TextStyle(
        fontSize: 14,
        fontWeight: FontWeight.w600,
        color: AppTheme.textTertiary,
      ),
    );
  }

  /// 检查活跃提案数，未达上限则打开页面，达上限则弹窗提示。
  /// [pageBuilder] 为 null 时表示该功能开发中。
  Future<void> _checkAndOpenProposal(
    BuildContext context,
    Widget Function()? pageBuilder, {
    String? name,
  }) async {
    final blockedReason = _proposalBlockedReason;
    if (blockedReason != null) {
      if (context.mounted) {
        ScaffoldMessenger.of(context).showSnackBar(
          SnackBar(content: Text(blockedReason)),
        );
      }
      return;
    }
    try {
      final service = TransferProposalService();
      final activeIds =
          await service.fetchActiveProposalIds(widget.institution.shenfenId);
      if (!context.mounted) return;

      if (activeIds.length >=
          TransferProposalService.maxActiveProposalsPerInstitution) {
        showDialog(
          context: context,
          builder: (ctx) => AlertDialog(
            title: const Text('提案数量已达上限'),
            content: Text(
              '本机构当前有 ${activeIds.length} 个活跃提案，'
              '已达上限 ${TransferProposalService.maxActiveProposalsPerInstitution} 个。'
              '请等待现有提案完成后再发起新提案。',
            ),
            actions: [
              TextButton(
                onPressed: () => Navigator.pop(ctx),
                child: const Text('知道了'),
              ),
            ],
          ),
        );
        return;
      }

      if (pageBuilder != null) {
        final created = await Navigator.push<bool>(
          context,
          MaterialPageRoute(builder: (_) => pageBuilder()),
        );
        if (created == true && context.mounted) {
          Navigator.of(context).pop(true);
        }
      } else {
        ScaffoldMessenger.of(context).showSnackBar(
          SnackBar(content: Text('${name ?? "该"}功能开发中')),
        );
      }
    } catch (e) {
      if (!context.mounted) return;
      ScaffoldMessenger.of(context).showSnackBar(
        SnackBar(
          content: Text(
            SmoldotClientManager.instance.buildUserFacingError(e),
          ),
        ),
      );
    }
  }

  void _handleChainProgressChanged(LightClientStatusSnapshot? progress) {
    if (!mounted) return;
    setState(() {
      _chainProgress = progress;
    });
  }

  void _handleChainProgressErrorChanged(String? error) {
    if (!mounted) return;
    setState(() {
      _chainProgressError = error;
    });
  }

  String? get _proposalBlockedReason {
    final progress = _chainProgress;
    if (progress == null) {
      return _chainProgressError ?? '正在读取区块链状态，请稍后再试';
    }
    if (!progress.hasPeers) {
      return '轻节点尚未连接到区块链网络，暂不能发起提案';
    }
    if (progress.isSyncing) {
      return '轻节点仍在同步区块头，完成后才能发起提案';
    }
    if (!progress.isUsable) {
      return _chainProgressError ?? '区块链状态尚未就绪，暂不能发起提案';
    }
    return null;
  }
}

class _ProposalTypeCard extends StatelessWidget {
  const _ProposalTypeCard({
    required this.icon,
    required this.title,
    required this.subtitle,
    required this.color,
    required this.onTap,
    this.enabled = true,
  });

  final IconData icon;
  final String title;
  final String subtitle;
  final Color color;
  final VoidCallback onTap;

  /// 是否可点击。未激活管理员时为 false，显示灰色禁用态。
  final bool enabled;

  @override
  Widget build(BuildContext context) {
    final effectiveColor = enabled ? color : AppTheme.textTertiary;

    return Card(
      elevation: 0,
      margin: EdgeInsets.zero,
      shape: RoundedRectangleBorder(
        borderRadius: BorderRadius.circular(12),
        side: BorderSide(color: effectiveColor.withValues(alpha: 0.15)),
      ),
      child: InkWell(
        onTap: enabled ? onTap : null,
        borderRadius: BorderRadius.circular(12),
        child: Opacity(
          opacity: enabled ? 1.0 : 0.5,
          child: Padding(
            padding: const EdgeInsets.symmetric(horizontal: 14, vertical: 12),
            child: Row(
              children: [
                Container(
                  width: 40,
                  height: 40,
                  decoration: BoxDecoration(
                    color: effectiveColor.withValues(alpha: 0.10),
                    borderRadius: BorderRadius.circular(10),
                  ),
                  child: Icon(icon, size: 20, color: effectiveColor),
                ),
                const SizedBox(width: 12),
                Expanded(
                  child: Column(
                    crossAxisAlignment: CrossAxisAlignment.start,
                    children: [
                      Text(
                        title,
                        style: TextStyle(
                          fontSize: 15,
                          fontWeight: FontWeight.w600,
                          color: effectiveColor,
                        ),
                      ),
                      const SizedBox(height: 2),
                      Text(
                        subtitle,
                        style: const TextStyle(
                            fontSize: 12, color: AppTheme.textTertiary),
                        maxLines: 2,
                        overflow: TextOverflow.ellipsis,
                      ),
                    ],
                  ),
                ),
                Icon(Icons.chevron_right,
                    size: 20,
                    color: enabled
                        ? AppTheme.textTertiary
                        : AppTheme.textTertiary.withValues(alpha: 0.3)),
              ],
            ),
          ),
        ),
      ),
    );
  }
}
