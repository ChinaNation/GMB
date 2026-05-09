import 'dart:typed_data';

import 'package:flutter/material.dart';
import 'package:polkadart_keyring/polkadart_keyring.dart' show Keyring;

import 'package:wuminapp_mobile/admins_change/services/admin_activation_service.dart';
import 'package:wuminapp_mobile/admins_change/services/institution_admin_service.dart';
import 'package:wuminapp_mobile/duoqian/shared/duoqian_manage_detail_page.dart';
import 'package:wuminapp_mobile/ui/app_theme.dart';
import 'package:wuminapp_mobile/util/amount_format.dart';
import 'package:wuminapp_mobile/wallet/core/wallet_manager.dart';
import 'package:wuminapp_mobile/institution/institution_admin_list_page.dart';
import 'package:wuminapp_mobile/institution/institution_data.dart';
import 'package:wuminapp_mobile/proposal/shared/proposal_cache.dart';
import 'package:wuminapp_mobile/proposal/shared/proposal_context.dart';
import 'package:wuminapp_mobile/proposal/proposal_types_page.dart';
import 'package:wuminapp_mobile/proposal/runtime_upgrade/runtime_upgrade_detail_page.dart';
import 'package:wuminapp_mobile/proposal/shared/proposal_models.dart';
import 'package:wuminapp_mobile/proposal/transfer/transfer_proposal_detail_page.dart';
import 'package:wuminapp_mobile/proposal/transfer/transfer_proposal_service.dart';
import 'package:wuminapp_mobile/rpc/chain_rpc.dart';
import 'package:wuminapp_mobile/rpc/smoldot_client.dart';

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
  final ActivationService _activationService = ActivationService();
  final ChainRpc _chainRpc = ChainRpc();
  late final ProposalContextResolver _contextResolver = ProposalContextResolver(
    adminService: _adminService,
    walletManager: _walletManager,
    activationService: _activationService,
  );

  List<String> _admins = const [];
  bool _isCurrentUserAdmin = false;
  bool _loading = true;
  String? _error;

  /// 通过 ProposalContext 解析的管理员钱包。
  List<WalletProfile> _adminWallets = const [];

  /// 用户已导入的冷钱包公钥中，属于本机构链上管理员的集合（小写 hex）。
  Set<String> _importedColdPubkeys = const {};

  /// 已激活的管理员公钥集合（小写 hex）。
  Set<String> _activatedPubkeys = const {};

  /// 机构页可见的提案事件（本机构内部提案 + 全局联合投票提案）。
  List<ProposalWithDetail> _proposalEvents = const [];

  /// 主账户实时可用余额（元）。
  double? _mainBalance;

  /// 更多制度账户是否已在当前页展开。
  bool _extraAccountsExpanded = false;

  /// 更多制度账户余额是否正在读取。
  bool _extraAccountsLoading = false;

  /// 更多制度账户余额是否已至少读取过一次。
  bool _extraAccountsLoaded = false;

  /// 更多制度账户余额读取错误。
  String? _extraAccountsError;

  /// 更多制度账户展示数据。
  List<_InstitutionAccountView> _extraAccounts = const [];

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
        _adminService.fetchAdmins(widget.institution.sfidNumber),
        _contextResolver.resolve(
          knownInstitution: widget.institution,
        ),
        _transferService.fetchInstitutionVisibleProposals(
          widget.institution.sfidNumber,
        ),
        _activationService.getActivatedAdmins(widget.institution.sfidNumber),
        _transferService
            .fetchInstitutionBalance(widget.institution)
            .then<double?>((value) => value)
            .catchError((_) => null),
      ]);
      final admins = results[0] as List<String>;
      final ctx = results[1] as ProposalContext;
      final proposals = results[2] as List<ProposalWithDetail>;
      final activated = results[3] as List<ActivatedAdmin>;
      final mainBalance = results[4] as double?;

      // 已激活公钥集合
      final activatedPks = activated.map((a) => a.pubkeyHex).toSet();

      // 计算用户已导入的冷钱包公钥中，属于本机构链上管理员的集合
      final allWallets = await _walletManager.getWallets();
      final coldPubkeys = <String>{};
      for (final w in allWallets) {
        if (w.isColdWallet) {
          var pk = w.pubkeyHex.toLowerCase();
          if (pk.startsWith('0x')) pk = pk.substring(2);
          if (admins.contains(pk)) {
            coldPubkeys.add(pk);
          }
        }
      }

      // 记录管理员机构状态到公共缓存
      if (ctx.isAdmin) {
        ProposalContextResolver.markAdminInstitution(
          widget.institution.sfidNumber,
        );
      }

      if (!mounted) return;
      setState(() {
        _admins = admins;
        _adminWallets = ctx.adminWallets;
        _importedColdPubkeys = coldPubkeys;
        _activatedPubkeys = activatedPks;
        _isCurrentUserAdmin = ctx.isAdmin;
        _proposalEvents = proposals;
        _mainBalance = mainBalance;
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
        _adminService.clearCache(widget.institution.sfidNumber);
        _contextResolver.clearWalletCache();
        ProposalCache.clear();
        await _load();
        if (_extraAccountsExpanded) {
          await _loadExtraAccounts(force: true);
        }
      },
      child: ListView(
        padding: const EdgeInsets.fromLTRB(16, 8, 16, 32),
        children: [
          _buildInstitutionInfo(),
          const SizedBox(height: 12),
          _buildHeader(),
          const SizedBox(height: 12),
          if (_isCurrentUserAdmin) ...[
            _buildAdminBadge(),
            const SizedBox(height: 12),
          ] else ...[
            _buildNonAdminHint(),
            const SizedBox(height: 12),
          ],
          _buildAdminEntry(),
          const SizedBox(height: 12),
          _buildProposalList(),
        ],
      ),
    );
  }

  // ──── 机构基础信息（身份 ID / 主账户 / 余额 / 更多账户）────

  Widget _buildInstitutionInfo() {
    final inst = widget.institution;
    final extraSources = _extraAccountSources();
    return Card(
      elevation: 0,
      margin: EdgeInsets.zero,
      shape: RoundedRectangleBorder(
        borderRadius: BorderRadius.circular(12),
        side: BorderSide(color: widget.badgeColor.withValues(alpha: 0.18)),
      ),
      child: Padding(
        padding: const EdgeInsets.symmetric(horizontal: 14, vertical: 12),
        child: Column(
          crossAxisAlignment: CrossAxisAlignment.start,
          children: [
            _buildAccountInfoTile(
              icon: Icons.badge_outlined,
              label: '身份ID',
              value: inst.sfidNumber,
            ),
            const Divider(height: 18),
            _buildAccountInfoTile(
              icon: Icons.account_balance_wallet_outlined,
              label: '主账户',
              value: _shortAddress(_accountHexToSs58(inst.mainAddress)),
            ),
            const Divider(height: 18),
            _buildAccountInfoTile(
              icon: Icons.payments_outlined,
              label: '主账户余额',
              value: _mainBalance == null
                  ? '读取失败'
                  : AmountFormat.format(_mainBalance!, symbol: 'GMB'),
              valueColor:
                  _mainBalance == null ? AppTheme.danger : AppTheme.textPrimary,
            ),
            if (extraSources.isNotEmpty) ...[
              const Divider(height: 18),
              _buildMoreAccountsToggle(extraSources),
              ClipRect(
                child: AnimatedSize(
                  duration: const Duration(milliseconds: 180),
                  curve: Curves.easeOutCubic,
                  alignment: Alignment.topCenter,
                  child: _extraAccountsExpanded
                      ? Padding(
                          padding: const EdgeInsets.only(top: 10),
                          child: _buildExpandedAccounts(),
                        )
                      : const SizedBox.shrink(),
                ),
              ),
            ],
          ],
        ),
      ),
    );
  }

  Widget _buildAccountInfoTile({
    required IconData icon,
    required String label,
    required String value,
    Color valueColor = AppTheme.textPrimary,
  }) {
    return Row(
      children: [
        Container(
          width: 32,
          height: 32,
          decoration: BoxDecoration(
            color: AppTheme.surfaceMuted,
            borderRadius: BorderRadius.circular(9),
          ),
          child: Icon(icon, size: 16, color: widget.badgeColor),
        ),
        const SizedBox(width: 10),
        Expanded(
          child: Column(
            crossAxisAlignment: CrossAxisAlignment.start,
            children: [
              Text(
                label,
                style: const TextStyle(
                  fontSize: 11,
                  color: AppTheme.textTertiary,
                  fontWeight: FontWeight.w500,
                ),
              ),
              const SizedBox(height: 2),
              Text(
                value,
                maxLines: 1,
                overflow: TextOverflow.ellipsis,
                style: TextStyle(
                  fontSize: 13,
                  color: valueColor,
                  fontWeight: FontWeight.w600,
                ),
              ),
            ],
          ),
        ),
      ],
    );
  }

  Widget _buildMoreAccountsToggle(
    List<({String name, String address, IconData icon})> sources,
  ) {
    return InkWell(
      onTap: _toggleExtraAccounts,
      borderRadius: BorderRadius.circular(10),
      child: Padding(
        padding: const EdgeInsets.symmetric(vertical: 2),
        child: Row(
          children: [
            Container(
              width: 32,
              height: 32,
              decoration: BoxDecoration(
                color: widget.badgeColor.withValues(alpha: 0.08),
                borderRadius: BorderRadius.circular(9),
              ),
              child: Icon(
                Icons.account_tree_outlined,
                size: 16,
                color: widget.badgeColor,
              ),
            ),
            const SizedBox(width: 10),
            Expanded(
              child: Column(
                crossAxisAlignment: CrossAxisAlignment.start,
                children: [
                  const Text(
                    '更多账户',
                    style: TextStyle(
                      fontSize: 13,
                      color: AppTheme.primaryDark,
                      fontWeight: FontWeight.w700,
                    ),
                  ),
                  const SizedBox(height: 2),
                  Text(
                    sources.map((item) => item.name).join(' / '),
                    maxLines: 1,
                    overflow: TextOverflow.ellipsis,
                    style: const TextStyle(
                      fontSize: 12,
                      color: AppTheme.textTertiary,
                    ),
                  ),
                ],
              ),
            ),
            Icon(
              _extraAccountsExpanded
                  ? Icons.keyboard_arrow_up
                  : Icons.keyboard_arrow_down,
              size: 22,
              color: AppTheme.textTertiary,
            ),
          ],
        ),
      ),
    );
  }

  Widget _buildExpandedAccounts() {
    if (_extraAccountsLoading) {
      return Container(
        width: double.infinity,
        padding: const EdgeInsets.symmetric(horizontal: 12, vertical: 14),
        decoration: BoxDecoration(
          color: AppTheme.surfaceMuted,
          borderRadius: BorderRadius.circular(10),
          border: Border.all(color: AppTheme.borderLight),
        ),
        child: const Row(
          children: [
            SizedBox(
              width: 16,
              height: 16,
              child: CircularProgressIndicator(strokeWidth: 2),
            ),
            SizedBox(width: 10),
            Text(
              '正在读取更多账户余额...',
              style: TextStyle(fontSize: 12, color: AppTheme.textSecondary),
            ),
          ],
        ),
      );
    }

    if (_extraAccountsError != null) {
      return Container(
        width: double.infinity,
        padding: const EdgeInsets.all(12),
        decoration: AppTheme.bannerDecoration(AppTheme.danger),
        child: Row(
          children: [
            const Icon(Icons.error_outline, size: 16, color: AppTheme.danger),
            const SizedBox(width: 8),
            Expanded(
              child: Text(
                _extraAccountsError!,
                maxLines: 2,
                overflow: TextOverflow.ellipsis,
                style: const TextStyle(
                  fontSize: 12,
                  color: AppTheme.danger,
                  fontWeight: FontWeight.w500,
                ),
              ),
            ),
            TextButton(
              onPressed: () => _loadExtraAccounts(force: true),
              child: const Text('重试'),
            ),
          ],
        ),
      );
    }

    if (_extraAccounts.isEmpty) {
      return Container(
        width: double.infinity,
        padding: const EdgeInsets.symmetric(horizontal: 12, vertical: 14),
        decoration: BoxDecoration(
          color: AppTheme.surfaceMuted,
          borderRadius: BorderRadius.circular(10),
          border: Border.all(color: AppTheme.borderLight),
        ),
        child: const Text(
          '暂无更多账户',
          textAlign: TextAlign.center,
          style: TextStyle(fontSize: 12, color: AppTheme.textSecondary),
        ),
      );
    }

    return Column(
      children: [
        for (var i = 0; i < _extraAccounts.length; i++) ...[
          if (i > 0) const SizedBox(height: 8),
          _buildExpandedAccountItem(_extraAccounts[i]),
        ],
      ],
    );
  }

  Widget _buildExpandedAccountItem(_InstitutionAccountView account) {
    return Container(
      width: double.infinity,
      padding: const EdgeInsets.all(10),
      decoration: BoxDecoration(
        color: AppTheme.surfaceMuted,
        borderRadius: BorderRadius.circular(10),
        border: Border.all(color: AppTheme.borderLight),
      ),
      child: Column(
        children: [
          Row(
            children: [
              Container(
                width: 28,
                height: 28,
                decoration: BoxDecoration(
                  color: widget.badgeColor.withValues(alpha: 0.08),
                  borderRadius: BorderRadius.circular(8),
                ),
                child: Icon(account.icon, size: 15, color: widget.badgeColor),
              ),
              const SizedBox(width: 8),
              Expanded(
                child: Text(
                  account.name,
                  style: const TextStyle(
                    fontSize: 13,
                    color: AppTheme.primaryDark,
                    fontWeight: FontWeight.w700,
                  ),
                ),
              ),
              Flexible(
                child: Text(
                  AmountFormat.format(account.balance, symbol: 'GMB'),
                  textAlign: TextAlign.right,
                  maxLines: 1,
                  overflow: TextOverflow.ellipsis,
                  style: const TextStyle(
                    fontSize: 12,
                    color: AppTheme.textPrimary,
                    fontWeight: FontWeight.w700,
                  ),
                ),
              ),
            ],
          ),
          const SizedBox(height: 6),
          Align(
            alignment: Alignment.centerLeft,
            child: Text(
              _shortAddress(account.addressSs58),
              maxLines: 1,
              overflow: TextOverflow.ellipsis,
              style: const TextStyle(
                fontSize: 12,
                color: AppTheme.textTertiary,
                fontWeight: FontWeight.w500,
              ),
            ),
          ),
        ],
      ),
    );
  }

  List<({String name, String address, IconData icon})> _extraAccountSources() {
    final accounts = widget.institution.accounts;
    final items = <({String name, String address, IconData icon})>[];

    final feeAddress = accounts?.feeAddress;
    if (feeAddress != null) {
      items.add((
        name: '费用账户',
        address: feeAddress,
        icon: Icons.receipt_long_outlined,
      ));
    }

    final safetyFundAddress = accounts?.safetyFundAddress;
    if (safetyFundAddress != null) {
      items.add((
        name: '安全基金账户',
        address: safetyFundAddress,
        icon: Icons.health_and_safety_outlined,
      ));
    }

    final stakeAddress = accounts?.stakeAddress;
    if (stakeAddress != null) {
      items.add((
        name: '质押账户',
        address: stakeAddress,
        icon: Icons.lock_outline,
      ));
    }

    return items;
  }

  Future<void> _toggleExtraAccounts() async {
    final shouldExpand = !_extraAccountsExpanded;
    setState(() {
      _extraAccountsExpanded = shouldExpand;
    });
    if (shouldExpand && !_extraAccountsLoaded) {
      await _loadExtraAccounts();
    }
  }

  Future<void> _loadExtraAccounts({bool force = false}) async {
    if (_extraAccountsLoading) return;
    if (_extraAccountsLoaded && !force) return;

    final sources = _extraAccountSources();
    if (sources.isEmpty) {
      setState(() {
        _extraAccounts = const [];
        _extraAccountsLoaded = true;
        _extraAccountsError = null;
      });
      return;
    }

    setState(() {
      _extraAccountsLoading = true;
      _extraAccountsError = null;
    });

    try {
      // 中文注释：更多账户余额只在用户展开时读取，避免机构详情列表初次进入时放大 RPC 压力。
      final views = await Future.wait(
        sources.map((item) async {
          final balance = await _chainRpc.fetchBalance(item.address);
          return _InstitutionAccountView(
            name: item.name,
            addressSs58: _accountHexToSs58(item.address),
            balance: balance,
            icon: item.icon,
          );
        }),
      );
      if (!mounted) return;
      setState(() {
        _extraAccounts = views;
        _extraAccountsLoaded = true;
        _extraAccountsLoading = false;
        _extraAccountsError = null;
      });
    } catch (e) {
      if (!mounted) return;
      setState(() {
        _extraAccountsLoading = false;
        _extraAccountsLoaded = false;
        _extraAccountsError =
            SmoldotClientManager.instance.buildUserFacingError(e);
      });
    }
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
        onTap: _openProposalTypes,
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
                      style: const TextStyle(
                          fontSize: 12, color: AppTheme.textTertiary),
                    ),
                  ],
                ),
              ),
              // 右侧箭头
              const Icon(Icons.chevron_right,
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

  // ──── 非管理员提示 ────

  Widget _buildNonAdminHint() {
    return Container(
      padding: const EdgeInsets.symmetric(horizontal: 10, vertical: 6),
      decoration: AppTheme.bannerDecoration(AppTheme.textTertiary),
      child: const Row(
        mainAxisSize: MainAxisSize.min,
        children: [
          Icon(Icons.info_outline, size: 14, color: AppTheme.textTertiary),
          SizedBox(width: 4),
          Text(
            '仅管理员可发起提案',
            style: TextStyle(
              fontSize: 12,
              color: AppTheme.textTertiary,
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
        side: const BorderSide(color: AppTheme.border),
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
                      style: const TextStyle(
                          fontSize: 12, color: AppTheme.textTertiary),
                    ),
                  ],
                ),
              ),
              const Icon(Icons.chevron_right,
                  size: 20, color: AppTheme.textTertiary),
            ],
          ),
        ),
      ),
    );
  }

  // ──── 提案列表 ────

  Widget _buildProposalList() {
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
        if (_proposalEvents.isEmpty)
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
                SizedBox(height: 4),
                Text(
                  '本机构提案与全局联合投票将在此显示',
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
      case 4:
        return '执行失败';
      default:
        return '未知';
    }
  }

  Color _statusColor(int? status) => AppTheme.proposalStatusColor(status ?? -1);

  String _proposalTitle(ProposalWithDetail proposal) {
    final proposalId = formatProposalId(proposal.meta.displayMeta);
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
      return '创建个人多签账户 · $status';
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
                  _statusLabel(proposal.meta.status),
                  style: TextStyle(
                    fontSize: 11,
                    fontWeight: FontWeight.w600,
                    color: statusColor,
                  ),
                ),
              ),
              const SizedBox(width: 4),
              const Icon(Icons.chevron_right,
                  size: 20, color: AppTheme.textTertiary),
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
          isActivated: _isCurrentUserAdmin,
        ),
      ),
    );
    // 返回后刷新（可能新建了提案）
    if (mounted) {
      _adminService.clearCache(widget.institution.sfidNumber);
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
    } else if (proposal.safetyFundDetail != null) {
      // 安全基金转账提案：传 kind=safetyFund，页面内按 call_index=4 投票。
      await Navigator.of(context).push(
        MaterialPageRoute(
          builder: (_) => TransferProposalDetailPage(
            institution: widget.institution,
            proposalId: proposalId,
            proposalContext: ctx,
            kind: TransferProposalKind.safetyFund,
          ),
        ),
      );
    } else if (proposal.sweepDetail != null) {
      // 手续费划转提案：传 kind=sweep，页面内按 call_index=6 投票。
      await Navigator.of(context).push(
        MaterialPageRoute(
          builder: (_) => TransferProposalDetailPage(
            institution: widget.institution,
            proposalId: proposalId,
            proposalContext: ctx,
            kind: TransferProposalKind.sweep,
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
      _adminService.clearCache(widget.institution.sfidNumber);
      ProposalCache.clear();
      _load();
    }
  }

  String _accountHexToSs58(String hex) {
    return Keyring().encodeAddress(Uint8List.fromList(_hexDecode(hex)), 2027);
  }

  List<int> _hexDecode(String hex) {
    final clean = hex.startsWith('0x') ? hex.substring(2) : hex;
    return List<int>.generate(
      clean.length ~/ 2,
      (index) =>
          int.parse(clean.substring(index * 2, index * 2 + 2), radix: 16),
      growable: false,
    );
  }

  String _shortAddress(String address) {
    if (address.length <= 18) return address;
    return '${address.substring(0, 8)}...${address.substring(address.length - 8)}';
  }

  void _openAdminList() {
    Navigator.of(context).push(
      MaterialPageRoute(
        builder: (_) => AdminListPage(
          institution: widget.institution,
          admins: _admins,
          importedColdPubkeys: _importedColdPubkeys,
          activatedPubkeys: _activatedPubkeys,
          badgeColor: widget.badgeColor,
          onActivated: () {
            // 激活成功后刷新页面
            _adminService.clearCache(widget.institution.sfidNumber);
            _contextResolver.clearWalletCache();
            _load();
          },
        ),
      ),
    );
  }
}

class _InstitutionAccountView {
  const _InstitutionAccountView({
    required this.name,
    required this.addressSs58,
    required this.balance,
    required this.icon,
  });

  final String name;
  final String addressSs58;
  final double balance;
  final IconData icon;
}
