import 'package:flutter/material.dart';

import '../ui/app_theme.dart';
import '../util/amount_format.dart';
import '../wallet/core/wallet_manager.dart';
import 'admin_list_page.dart';
import 'duoqian_manage_detail_page.dart';
import 'institution_admin_service.dart';
import 'institution_data.dart';
import 'proposal_cache.dart';
import 'proposal_context.dart';
import 'proposal_types_page.dart';
import 'runtime_upgrade_detail_page.dart';
import 'transfer_proposal_detail_page.dart';
import 'transfer_proposal_service.dart';
import '../rpc/smoldot_client.dart';

/// 机构详情页。
class InstitutionDetailPage extends StatefulWidget {
  const InstitutionDetailPage({
    super.key,
    required this.institution,
    required this.icon,
    required this.badgeColor,
  });

  final InstitutionInfo institution;
  final IconData icon;
  final Color badgeColor;

  @override
  State<InstitutionDetailPage> createState() => _InstitutionDetailPageState();
}

class _InstitutionDetailPageState extends State<InstitutionDetailPage> {
  final InstitutionAdminService _adminService = InstitutionAdminService();
  final WalletManager _walletManager = WalletManager();
  final TransferProposalService _transferService = TransferProposalService();
  late final ProposalContextResolver _contextResolver = ProposalContextResolver(
    adminService: _adminService,
    walletManager: _walletManager,
  );

  List<String> _admins = const [];
  bool _isCurrentUserAdmin = false;
  bool _loading = true;
  String? _error;

  /// 通过 ProposalContext 解析的管理员钱包。
  List<WalletProfile> _adminWallets = const [];

  /// 所有匹配的管理员公钥（小写 hex，不含 0x）。
  Set<String> _adminPubkeys = const {};

  /// 机构页可见的提案事件（本机构内部提案 + 全局联合投票提案）。
  List<ProposalWithDetail> _proposalEvents = const [];

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
      final results = await Future.wait([
        _adminService.fetchAdmins(widget.institution.shenfenId),
        _contextResolver.resolve(
          knownInstitution: widget.institution,
        ),
        _transferService.fetchInstitutionVisibleProposals(
          widget.institution.shenfenId,
        ),
      ]);
      final admins = results[0] as List<String>;
      final ctx = results[1] as ProposalContext;
      final proposals = results[2] as List<ProposalWithDetail>;

      // 从 ProposalContext 获取匹配的管理员冷钱包
      final matchedPubkeys = <String>{};
      for (final wallet in ctx.adminWallets) {
        var pubkey = wallet.pubkeyHex.toLowerCase();
        if (pubkey.startsWith('0x')) pubkey = pubkey.substring(2);
        matchedPubkeys.add(pubkey);
      }

      // 记录管理员机构状态到公共缓存
      if (ctx.isAdmin) {
        ProposalContextResolver.markAdminInstitution(
          widget.institution.shenfenId,
        );
      }

      if (!mounted) return;
      setState(() {
        _admins = admins;
        _adminWallets = ctx.adminWallets;
        _adminPubkeys = matchedPubkeys;
        _isCurrentUserAdmin = ctx.isAdmin;
        _proposalEvents = proposals;
        _loading = false;
      });
    } catch (e) {
      if (!mounted) return;
      setState(() {
        _error = SmoldotClientManager.instance.buildUserFacingError(e);
        _loading = false;
      });
    }
  }

  @override
  Widget build(BuildContext context) {
    return Scaffold(
      backgroundColor: AppTheme.scaffoldBg,
      appBar: AppBar(
        title: Text(
          widget.institution.name,
          style: const TextStyle(fontSize: 17, fontWeight: FontWeight.w700),
        ),
        centerTitle: true,
        foregroundColor: AppTheme.textPrimary,
      ),
      body: _loading
          ? const Center(child: CircularProgressIndicator())
          : _error != null
              ? _buildError()
              : _buildContent(),
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
            const Text('加载失败',
                style: TextStyle(fontSize: 16, color: AppTheme.textSecondary)),
            const SizedBox(height: 6),
            Text(
              _error!,
              style:
                  const TextStyle(fontSize: 12, color: AppTheme.textTertiary),
              textAlign: TextAlign.center,
              maxLines: 4,
              overflow: TextOverflow.ellipsis,
            ),
            const SizedBox(height: 16),
            OutlinedButton(onPressed: _load, child: const Text('重试')),
          ],
        ),
      ),
    );
  }

  Widget _buildContent() {
    return RefreshIndicator(
      onRefresh: () async {
        _adminService.clearCache(widget.institution.shenfenId);
        _contextResolver.clearWalletCache();
        ProposalCache.clear();
        await _load();
      },
      child: ListView(
        padding: const EdgeInsets.fromLTRB(16, 8, 16, 32),
        children: [
          _buildHeader(),
          const SizedBox(height: 12),
          if (_isCurrentUserAdmin) ...[
            _buildAdminBadge(),
            const SizedBox(height: 12),
          ],
          _buildAdminEntry(),
          const SizedBox(height: 12),
          _buildVotingEvents(),
        ],
      ),
    );
  }

  // ──── 顶部机构卡片（横向布局 + 右箭头进入提案页） ────

  Widget _buildHeader() {
    final inst = widget.institution;
    return Card(
      elevation: 0,
      margin: EdgeInsets.zero,
      shape: RoundedRectangleBorder(
        borderRadius: BorderRadius.circular(12),
        side: BorderSide(color: widget.badgeColor.withValues(alpha: 0.18)),
      ),
      child: InkWell(
        onTap: _isCurrentUserAdmin ? _openProposalTypes : null,
        borderRadius: BorderRadius.circular(12),
        child: Padding(
          padding: const EdgeInsets.symmetric(horizontal: 14, vertical: 12),
          child: Row(
            children: [
              // 左侧图标
              Container(
                width: 44,
                height: 44,
                decoration: BoxDecoration(
                  color: widget.badgeColor.withValues(alpha: 0.12),
                  borderRadius: BorderRadius.circular(12),
                ),
                child: Icon(widget.icon, size: 22, color: widget.badgeColor),
              ),
              const SizedBox(width: 12),
              // 中间：简称标签 / 管理员信息
              Expanded(
                child: Column(
                  crossAxisAlignment: CrossAxisAlignment.start,
                  children: [
                    Container(
                      padding: const EdgeInsets.symmetric(
                          horizontal: 6, vertical: 1),
                      decoration: BoxDecoration(
                        color: widget.badgeColor.withValues(alpha: 0.10),
                        borderRadius: BorderRadius.circular(10),
                      ),
                      child: Text(
                        '${OrgType.label(inst.orgType)}　提案',
                        style: TextStyle(
                          fontSize: 11,
                          color: widget.badgeColor,
                          fontWeight: FontWeight.w600,
                        ),
                      ),
                    ),
                    const SizedBox(height: 4),
                    Text(
                      '管理员 ${_admins.length} 人　通过阈值 ${inst.internalThreshold}',
                      style:
                          TextStyle(fontSize: 12, color: AppTheme.textTertiary),
                    ),
                  ],
                ),
              ),
              // 右侧箭头（仅管理员显示）
              if (_isCurrentUserAdmin)
                Icon(Icons.chevron_right,
                    size: 20, color: AppTheme.textTertiary),
            ],
          ),
        ),
      ),
    );
  }

  // ──── 管理员身份标识 ────

  Widget _buildAdminBadge() {
    return Container(
      padding: const EdgeInsets.symmetric(horizontal: 10, vertical: 6),
      decoration: AppTheme.bannerDecoration(AppTheme.success),
      child: const Row(
        mainAxisSize: MainAxisSize.min,
        children: [
          Icon(Icons.verified_user, size: 14, color: AppTheme.success),
          SizedBox(width: 4),
          Text(
            '你是本机构管理员，点击上方卡片可发起提案',
            style: TextStyle(
              fontSize: 12,
              color: AppTheme.success,
              fontWeight: FontWeight.w500,
            ),
          ),
        ],
      ),
    );
  }

  // ──── 管理员列表入口 ────

  Widget _buildAdminEntry() {
    return Card(
      elevation: 0,
      margin: EdgeInsets.zero,
      shape: RoundedRectangleBorder(
        borderRadius: BorderRadius.circular(12),
        side: BorderSide(color: AppTheme.border),
      ),
      child: InkWell(
        onTap: _openAdminList,
        borderRadius: BorderRadius.circular(12),
        child: Padding(
          padding: const EdgeInsets.symmetric(horizontal: 14, vertical: 12),
          child: Row(
            children: [
              Container(
                width: 36,
                height: 36,
                decoration: BoxDecoration(
                  color: AppTheme.primaryDark.withValues(alpha: 0.08),
                  borderRadius: BorderRadius.circular(10),
                ),
                child: const Icon(Icons.people_outline,
                    size: 18, color: AppTheme.primaryDark),
              ),
              const SizedBox(width: 12),
              Expanded(
                child: Column(
                  crossAxisAlignment: CrossAxisAlignment.start,
                  children: [
                    const Text(
                      '管理员列表',
                      style: TextStyle(
                        fontSize: 15,
                        fontWeight: FontWeight.w600,
                        color: AppTheme.primaryDark,
                      ),
                    ),
                    const SizedBox(height: 2),
                    Text(
                      '共 ${_admins.length} 位管理员',
                      style:
                          TextStyle(fontSize: 12, color: AppTheme.textTertiary),
                    ),
                  ],
                ),
              ),
              Icon(Icons.chevron_right, size: 20, color: AppTheme.textTertiary),
            ],
          ),
        ),
      ),
    );
  }

  // ──── 投票事件列表 ────

  Widget _buildVotingEvents() {
    return Column(
      crossAxisAlignment: CrossAxisAlignment.start,
      children: [
        const Text(
          '投票事件',
          style: TextStyle(
            fontSize: 16,
            fontWeight: FontWeight.w700,
            color: AppTheme.primaryDark,
          ),
        ),
        const SizedBox(height: 12),
        if (_proposalEvents.isEmpty)
          Container(
            width: double.infinity,
            padding: const EdgeInsets.all(24),
            decoration: BoxDecoration(
              color: AppTheme.surfaceMuted,
              borderRadius: BorderRadius.circular(AppTheme.radiusMd),
              border: Border.all(color: AppTheme.border),
            ),
            child: Column(
              children: [
                const Icon(Icons.ballot_outlined,
                    size: 40, color: AppTheme.textTertiary),
                const SizedBox(height: 8),
                const Text(
                  '暂无投票事件',
                  style: TextStyle(fontSize: 14, color: AppTheme.textSecondary),
                ),
                const SizedBox(height: 4),
                const Text(
                  '本机构提案和全局联合投票事件将在此显示',
                  style: TextStyle(fontSize: 12, color: AppTheme.textTertiary),
                ),
              ],
            ),
          )
        else
          ...List.generate(_proposalEvents.length, (index) {
            final proposal = _proposalEvents[index];
            return Padding(
              padding: EdgeInsets.only(
                  bottom: index < _proposalEvents.length - 1 ? 8 : 0),
              child: _buildProposalCard(proposal),
            );
          }),
      ],
    );
  }

  String _statusLabel(int? status) {
    switch (status) {
      case 0:
        return '投票中';
      case 1:
        return '已通过';
      case 2:
        return '已拒绝';
      case 3:
        return '已执行';
      default:
        return '未知';
    }
  }

  Color _statusColor(int? status) => AppTheme.proposalStatusColor(status ?? -1);

  String _proposalTitle(ProposalWithDetail proposal) {
    final proposalId = formatProposalId(proposal.meta.proposalId);
    if (proposal.transferDetail != null) {
      return '转账提案 $proposalId';
    }
    if (proposal.createDuoqianDetail != null) {
      return '创建多签 $proposalId';
    }
    if (proposal.closeDuoqianDetail != null) {
      return '关闭多签 $proposalId';
    }
    if (proposal.runtimeUpgradeDetail != null) {
      return 'Runtime 升级 $proposalId';
    }
    if (proposal.meta.kind == 1) {
      return '联合投票提案 $proposalId';
    }
    return '提案 $proposalId';
  }

  String _proposalSubtitle(ProposalWithDetail proposal) {
    final status = _statusLabel(proposal.meta.status);
    final transferDetail = proposal.transferDetail;
    if (transferDetail != null) {
      return '${AmountFormat.format(transferDetail.amountYuan, symbol: '')} 元 · $status';
    }
    final createDetail = proposal.createDuoqianDetail;
    if (createDetail != null) {
      return '${createDetail.adminCount} 管理员 · 阈值 ${createDetail.threshold} · $status';
    }
    if (proposal.closeDuoqianDetail != null) {
      return '关闭多签账户 · $status';
    }
    if (proposal.runtimeUpgradeDetail != null) {
      return 'Runtime 升级 · $status';
    }
    if (proposal.meta.kind == 1) {
      return '联合投票 · $status';
    }
    return '提案事件 · $status';
  }

  IconData _proposalIcon(ProposalWithDetail proposal) {
    if (proposal.transferDetail != null) {
      return Icons.send_outlined;
    }
    if (proposal.createDuoqianDetail != null) {
      return Icons.group_add;
    }
    if (proposal.closeDuoqianDetail != null) {
      return Icons.group_remove;
    }
    if (proposal.runtimeUpgradeDetail != null) {
      return Icons.arrow_upward;
    }
    if (proposal.meta.kind == 1) {
      return Icons.groups_outlined;
    }
    return Icons.description_outlined;
  }

  Widget _buildProposalCard(ProposalWithDetail proposal) {
    final statusColor = _statusColor(proposal.meta.status);
    return Card(
      elevation: 0,
      margin: EdgeInsets.zero,
      shape: RoundedRectangleBorder(
        borderRadius: BorderRadius.circular(12),
        side: BorderSide(color: statusColor.withValues(alpha: 0.2)),
      ),
      child: InkWell(
        onTap: () => _openProposalDetail(proposal),
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
                child:
                    Icon(_proposalIcon(proposal), size: 18, color: statusColor),
              ),
              const SizedBox(width: 12),
              Expanded(
                child: Column(
                  crossAxisAlignment: CrossAxisAlignment.start,
                  children: [
                    Text(
                      _proposalTitle(proposal),
                      style: const TextStyle(
                        fontSize: 15,
                        fontWeight: FontWeight.w600,
                        color: AppTheme.primaryDark,
                      ),
                    ),
                    const SizedBox(height: 2),
                    Text(
                      _proposalSubtitle(proposal),
                      style:
                          TextStyle(fontSize: 12, color: AppTheme.textTertiary),
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
                  _statusLabel(proposal.meta.status),
                  style: TextStyle(
                    fontSize: 11,
                    fontWeight: FontWeight.w600,
                    color: statusColor,
                  ),
                ),
              ),
              const SizedBox(width: 4),
              Icon(Icons.chevron_right, size: 20, color: AppTheme.textTertiary),
            ],
          ),
        ),
      ),
    );
  }

  // ──── 导航 ────

  Future<void> _openProposalTypes() async {
    await Navigator.of(context).push<bool>(
      MaterialPageRoute(
        builder: (_) => ProposalTypesPage(
          institution: widget.institution,
          icon: widget.icon,
          badgeColor: widget.badgeColor,
          adminWallets: _adminWallets,
        ),
      ),
    );
    // 返回后刷新（可能新建了提案）
    if (mounted) {
      _adminService.clearCache(widget.institution.shenfenId);
      ProposalCache.clear();
      _load();
    }
  }

  Future<void> _openProposalDetail(ProposalWithDetail proposal) async {
    final proposalId = proposal.meta.proposalId;
    final ctx = ProposalContext(
      institution: widget.institution,
      adminWallets: _adminWallets,
      role: _isCurrentUserAdmin ? ProposalRole.admin : ProposalRole.viewer,
    );
    if (proposal.runtimeUpgradeDetail != null) {
      await Navigator.of(context).push(
        MaterialPageRoute(
          builder: (_) => RuntimeUpgradeDetailPage(
            proposalId: proposalId,
            proposalContext: ctx,
          ),
        ),
      );
    } else if (proposal.transferDetail != null) {
      await Navigator.of(context).push(
        MaterialPageRoute(
          builder: (_) => TransferProposalDetailPage(
            institution: widget.institution,
            proposalId: proposalId,
            proposalContext: ctx,
          ),
        ),
      );
    } else if (proposal.createDuoqianDetail != null ||
        proposal.closeDuoqianDetail != null) {
      await Navigator.of(context).push(
        MaterialPageRoute(
          builder: (_) => DuoqianManageDetailPage(
            institution: widget.institution,
            proposalId: proposalId,
            proposalContext: ctx,
          ),
        ),
      );
    } else {
      ScaffoldMessenger.of(context).showSnackBar(
        const SnackBar(content: Text('该联合提案详情页正在开发中')),
      );
      return;
    }
    // 返回后刷新（投票状态可能变化）
    if (mounted) {
      _adminService.clearCache(widget.institution.shenfenId);
      ProposalCache.clear();
      _load();
    }
  }

  void _openAdminList() {
    Navigator.of(context).push(
      MaterialPageRoute(
        builder: (_) => AdminListPage(
          institution: widget.institution,
          admins: _admins,
          adminPubkeys: _adminPubkeys,
          badgeColor: widget.badgeColor,
        ),
      ),
    );
  }
}
