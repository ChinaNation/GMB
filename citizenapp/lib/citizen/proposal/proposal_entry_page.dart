import 'package:flutter/material.dart';
import 'package:smoldot/smoldot.dart' show LightClientStatusSnapshot;

import 'package:citizenapp/citizen/proposal/election/election_proposal_page.dart';
import 'package:citizenapp/citizen/proposal/grandpa-key/grandpa_key_page.dart';
import 'package:citizenapp/citizen/proposal/legislation-yuan/legislation_intro_page.dart';
import 'package:citizenapp/citizen/proposal/proposal_registry.dart';
import 'package:citizenapp/citizen/proposal/resolution-destroy/resolution_destroy_page.dart';
import 'package:citizenapp/citizen/proposal/resolution-issuance/resolution_issuance_page.dart';
import 'package:citizenapp/ui/app_theme.dart';
import 'package:citizenapp/ui/widgets/chain_progress_banner.dart';

import 'package:citizenapp/citizen/shared/institution_info.dart';
import 'package:citizenapp/citizen/proposal/admins-change/models/admin_account.dart';
import 'package:citizenapp/citizen/proposal/admins-change/pages/admin_set_change_page.dart';
import 'package:citizenapp/citizen/shared/proposal/proposal_limit_service.dart';
import 'package:citizenapp/citizen/proposal/runtime-upgrade/runtime_upgrade_page.dart';
import 'package:citizenapp/transaction/multisig-transfer/multisig_transfer_page.dart';
import 'package:citizenapp/transaction/multisig-transfer/safety_fund_transfer_page.dart';
import 'package:citizenapp/transaction/multisig-transfer/sweep_to_main_page.dart';
import 'package:citizenapp/rpc/smoldot_client.dart';
import 'package:citizenapp/wallet/core/wallet_manager.dart';

/// 提案类型选择页(个人多签/创世治理机构/注册机构账户统一入口)。
///
/// 页面只把当前机构包装成 `ProposalSubject`,具体能发起哪些提案统一交给
/// `ProposalCapabilityRegistry`。机构码仍参与判断,但只在能力规则层集中使用。
class ProposalEntryPage extends StatefulWidget {
  const ProposalEntryPage({
    super.key,
    required this.institution,
    required this.institutionCode,
    required this.icon,
    required this.badgeColor,
    required this.adminWallets,
    required this.isActivated,
  });

  final InstitutionInfo institution;

  /// 机构码(registry 主键:NRC/PRC/PRB/NRP/CLEG/CGOV…)。
  final String institutionCode;
  final IconData icon;
  final Color badgeColor;

  /// 当前用户已激活的管理员钱包列表。
  final List<WalletProfile> adminWallets;

  /// 用户是否已激活管理员身份。
  final bool isActivated;

  @override
  State<ProposalEntryPage> createState() => _ProposalEntryPageState();
}

class _ProposalEntryPageState extends State<ProposalEntryPage> {
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
                    widget.institution.cidShortName,
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

          // ──── 可发起提案(按主体能力 registry 渲染,单一真源) ────
          _buildSectionTitle('可发起提案'),
          ..._buildProposalCards(proposalActionsEnabled),
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

  ProposalSubject get _subject => ProposalSubject.fromInstitution(
        institution: widget.institution,
        institutionCode: widget.institutionCode,
      );

  /// 按主体能力 registry 取可发起提案,逐项渲染卡片。
  List<Widget> _buildProposalCards(bool enabled) {
    final capabilities =
        ProposalCapabilityRegistry.capabilitiesForSubject(_subject);
    final out = <Widget>[];
    for (final capability in capabilities) {
      out.add(const SizedBox(height: 8));
      out.add(_cardFor(capability.kind, enabled));
    }
    if (out.isEmpty) {
      out.add(const Padding(
        padding: EdgeInsets.only(top: 8),
        child: Text('本机构暂无可发起的提案',
            style: TextStyle(fontSize: 13, color: AppTheme.textTertiary)),
      ));
    }
    return out;
  }

  /// 单种提案 → 卡片(图标/标题/副标题/跳转)。类B(协议升级/发起立法)直接进展示页,
  /// 其余走 `_checkAndOpenProposal`(校验活跃提案上限)。
  Widget _cardFor(ProposalKind kind, bool enabled) {
    switch (kind) {
      case ProposalKind.transfer:
        return _typeCard(
            Icons.send_outlined,
            '转账',
            '从机构主账户向指定地址发起转账提案',
            AppTheme.primary,
            enabled,
            () => _checkAndOpenProposal(
                context,
                () => MultisigTransferPage(
                    institution: widget.institution,
                    icon: widget.icon,
                    badgeColor: widget.badgeColor,
                    adminWallets: widget.adminWallets)));
      case ProposalKind.feeTransfer:
        return _typeCard(
            Icons.account_balance_wallet_outlined,
            '手续费划转',
            '从机构费用账户向本机构主账户划转手续费',
            AppTheme.info,
            enabled,
            () => _checkAndOpenProposal(
                context,
                () => SweepToMainPage(
                    institution: widget.institution,
                    icon: widget.icon,
                    badgeColor: widget.badgeColor,
                    adminWallets: widget.adminWallets)));
      case ProposalKind.adminsChange:
        return _typeCard(
            Icons.swap_horiz,
            '换管理员',
            '提议更换本机构管理员',
            AppTheme.accent,
            enabled,
            () => _checkAndOpenProposal(
                context,
                () => AdminsChangePage(
                    institution: widget.institution,
                    accountIdentity: AdminAccountIdentity.fromInstitution(
                        widget.institution),
                    adminWallets: widget.adminWallets)));
      case ProposalKind.safetyFundTransfer:
        return _typeCard(
            Icons.shield_outlined,
            '安全基金转账',
            '从国家储委会安全基金账户向指定地址发起转账提案',
            AppTheme.warning,
            enabled,
            () => _checkAndOpenProposal(
                context,
                () => SafetyFundTransferPage(
                    institution: widget.institution,
                    icon: widget.icon,
                    badgeColor: widget.badgeColor,
                    adminWallets: widget.adminWallets)));
      case ProposalKind.resolutionIssuance:
        return _typeCard(
            Icons.account_balance,
            '决议发行',
            '发起公民币发行决议,需联合投票:内部投票阶段+联合公投阶段',
            AppTheme.primaryDark,
            enabled,
            () => _checkAndOpenProposal(
                context, () => const ResolutionIssuancePage()));
      case ProposalKind.resolutionDestroy:
        return _typeCard(
            Icons.delete_outline,
            '决议销毁',
            '提议销毁机构持有的资产',
            AppTheme.danger,
            enabled,
            () => _checkAndOpenProposal(
                context, () => const ResolutionDestroyPage()));
      case ProposalKind.runtimeUpgrade:
        return _typeCard(
            Icons.arrow_upward,
            '协议升级',
            '查看协议升级说明及流程',
            AppTheme.info,
            enabled,
            () => Navigator.of(context).push(MaterialPageRoute(
                builder: (_) =>
                    RuntimeUpgradePage(adminWallets: widget.adminWallets))));
      case ProposalKind.grandpaKey:
        return _typeCard(
            Icons.vpn_key_outlined,
            '验证密钥',
            '更换 GRANDPA 共识验证密钥(本机构内部投票)',
            const Color(0xFF4527A0),
            enabled,
            () => _checkAndOpenProposal(context, () => const GrandpaKeyPage()));
      case ProposalKind.legislation:
        return _typeCard(
            Icons.gavel_outlined,
            '发起立法',
            '立法 / 修法 / 废法在电脑节点端发起,本端查看 + 投票',
            AppTheme.primaryDark,
            enabled,
            () => Navigator.of(context).push(MaterialPageRoute(
                builder: (_) => const LegislationIntroPage())));
      case ProposalKind.election:
        return _typeCard(
            Icons.how_to_vote_outlined,
            '发起选举',
            '发起选举提案',
            AppTheme.accent,
            enabled,
            () => _checkAndOpenProposal(
                context, () => const ElectionProposalPage()));
    }
  }

  Widget _typeCard(IconData icon, String title, String subtitle, Color color,
      bool enabled, VoidCallback onTap) {
    return _ProposalTypeCard(
      icon: icon,
      title: title,
      subtitle: subtitle,
      color: color,
      enabled: enabled,
      onTap: onTap,
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
      final service = ProposalLimitService();
      final activeIds =
          await service.fetchActiveProposalIds(widget.institution);
      if (!context.mounted) return;

      if (activeIds.length >=
          ProposalLimitService.maxActiveProposalsPerInstitution) {
        showDialog(
          context: context,
          builder: (ctx) => AlertDialog(
            title: const Text('提案数量已达上限'),
            content: Text(
              '本机构当前有 ${activeIds.length} 个活跃提案，'
              '已达上限 ${ProposalLimitService.maxActiveProposalsPerInstitution} 个。'
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
      return '轻节点仍在验证或同步链状态，完成后才能发起提案';
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
