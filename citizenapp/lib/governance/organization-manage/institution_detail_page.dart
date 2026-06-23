import 'dart:async';
import 'dart:typed_data';

import 'package:flutter/material.dart';
import 'package:polkadart_keyring/polkadart_keyring.dart' show Keyring;

import 'package:citizenapp/governance/admins-change/models/admin_account.dart';
import 'package:citizenapp/governance/admins-change/services/admin_activation_service.dart';
import 'package:citizenapp/governance/admins-change/services/institution_admin_service.dart';
import 'package:citizenapp/transaction/duoqian-transfer/duoqian_transfer_proposal_adapter.dart';
import 'package:citizenapp/governance/institution_manage_detail_page.dart';
import 'package:citizenapp/ui/app_theme.dart';
import 'package:citizenapp/my/util/amount_format.dart';
import 'package:citizenapp/wallet/core/wallet_manager.dart';
import 'package:citizenapp/governance/organization-manage/institution_admin_list_page.dart';
import 'package:citizenapp/governance/shared/institution_info.dart';
import 'package:citizenapp/governance/shared/proposal/proposal_cache.dart';
import 'package:citizenapp/governance/shared/proposal/proposal_context.dart';
import 'package:citizenapp/governance/shared/proposal/proposal_local_store.dart';
import 'package:citizenapp/isar/wallet_isar.dart';
import 'package:citizenapp/governance/governance_proposals_page.dart';
import 'package:citizenapp/governance/runtime-upgrade/runtime_upgrade_detail_page.dart';
import 'package:citizenapp/governance/shared/proposal/proposal_models.dart';
import 'package:citizenapp/rpc/smoldot_client.dart';
import 'package:citizenapp/transaction/shared/account_balance_snapshot_store.dart';
import 'package:citizenapp/citizen/public/data/cid_directory_lookup.dart';
import 'package:citizenapp/governance/organization-manage/institution_accounts_page.dart';

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
  final DuoqianTransferProposalFeed _duoqianTransferFeed =
      DuoqianTransferProposalFeed();
  final ActivationService _activationService = ActivationService();
  late final ProposalContextResolver _contextResolver = ProposalContextResolver(
    adminService: _adminService,
    walletManager: _walletManager,
    activationService: _activationService,
  );

  List<String> _admins = const [];
  bool _isCurrentUserAdmin = false;

  /// 管理员和当前用户权限属于链上/本地混合动态数据，不能阻塞机构固定信息首屏。
  bool _adminLoading = true;
  String? _adminError;

  /// 提案列表链上查询较重，单独后台刷新并局部展示状态。
  bool _proposalLoading = true;
  String? _proposalError;

  /// 主账户余额是动态链上数据，首屏先展示地址，再异步补余额。
  bool _mainBalanceLoading = true;
  String? _mainBalanceError;

  /// 通过 ProposalContext 解析的管理员钱包。
  List<WalletProfile> _adminWallets = const [];

  /// 用户已导入的冷钱包公钥中，属于本机构链上管理员的集合（小写 hex）。
  Set<String> _importedColdPubkeys = const {};

  /// 已激活的管理员公钥集合（小写 hex）。
  Set<String> _activatedPubkeys = const {};

  /// 机构页可见的提案展示摘要，优先来自本地持久化读库。
  List<LocalProposalSummary> _proposalSummaries = const [];

  /// 当前会话已从链上取回的提案详情；本地摘要点击时再按需补链上详情。
  Map<int, ProposalWithDetail> _proposalDetailsById = const {};

  /// 主账户实时可用余额（元）。
  double? _mainBalance;

  AdminAccountIdentity get _accountIdentity =>
      AdminAccountIdentity.fromInstitution(widget.institution);

  /// 机构目录信息(法定代表人/所属地):按 cid 反查公权目录本地库,与公权详情统一展示。
  /// 内置治理机构都带真实 CID 号且在确定性目录内;注册机构账户反查不到则留空。
  final CidDirectoryLookup _directoryLookup = CidDirectoryLookup();
  CidDirectoryInfo? _directory;

  @override
  void initState() {
    super.initState();
    unawaited(_refreshAll());
    unawaited(_loadDirectory());
  }

  Future<void> _loadDirectory() async {
    try {
      final info = await _directoryLookup.lookup(widget.institution.cidNumber);
      if (mounted && info != null) setState(() => _directory = info);
    } on Exception {
      // 反查失败留空,不影响主信息展示。
    }
  }

  Future<void> _refreshAll({bool force = false}) async {
    if (force) {
      _adminService.clearCache(_accountIdentity);
      _contextResolver.clearWalletCache();
      ProposalCache.clear();
      DuoqianTransferProposalAdapter.clearCache();
    }

    await Future.wait([
      _loadAdminsAndRole(),
      _loadMainBalance(force: force),
      _loadProposalEvents(force: force),
    ]);
  }

  Future<void> _loadAdminsAndRole() async {
    if (!mounted) return;
    setState(() {
      _adminLoading = true;
      _adminError = null;
    });

    try {
      final accountIdentity = _accountIdentity;
      final results = await Future.wait<Object>([
        _adminService.fetchAdmins(accountIdentity),
        _contextResolver.resolve(
          knownInstitution: widget.institution,
        ),
        _activationService
            .getActivatedAdmins(accountIdentity)
            .catchError((_) => <ActivatedAdmin>[]),
      ]);
      final admins = results[0] as List<String>;
      final ctx = results[1] as ProposalContext;
      final activated = results[2] as List<ActivatedAdmin>;
      final activatedPks = activated.map((a) => a.pubkeyHex).toSet();
      final coldPubkeys = await _loadImportedColdPubkeys(admins);

      // 中文注释：这里只记录已确认的管理员机构，用于列表页本地视觉提示，不改变链上排序或身份。
      if (ctx.isAdmin) {
        ProposalContextResolver.markInstitutionAdmin(
          widget.institution.cidNumber,
        );
      }

      if (!mounted) return;
      setState(() {
        _admins = admins;
        _adminWallets = ctx.adminWallets;
        _importedColdPubkeys = coldPubkeys;
        _activatedPubkeys = activatedPks;
        _isCurrentUserAdmin = ctx.isAdmin;
        _adminLoading = false;
        _adminError = null;
      });
    } catch (e, st) {
      debugPrint('[InstitutionDetail] admin load failed: $e\n$st');
      if (!mounted) return;
      setState(() {
        _adminError = SmoldotClientManager.instance.buildUserFacingError(e);
        _adminLoading = false;
      });
    }
  }

  Future<Set<String>> _loadImportedColdPubkeys(List<String> admins) async {
    final coldPubkeys = <String>{};
    try {
      // 中文注释：本地钱包库只影响管理员身份提示，不能让机构链上信息整体加载失败。
      final allWallets = await _walletManager.getWallets();
      for (final w in allWallets) {
        if (w.isColdWallet) {
          var pk = w.pubkeyHex.toLowerCase();
          if (pk.startsWith('0x')) pk = pk.substring(2);
          if (admins.contains(pk)) {
            coldPubkeys.add(pk);
          }
        }
      }
    } catch (e, st) {
      if (!WalletIsar.instance.isBusyError(e)) {
        debugPrint('[InstitutionDetail] local wallet load failed: $e\n$st');
      }
    }
    return coldPubkeys;
  }

  Future<void> _loadMainBalance({bool force = false}) async {
    if (!mounted) return;
    final balanceStore = AccountBalanceSnapshotStore.instance;
    final local =
        force ? null : await balanceStore.read(widget.institution.mainAccount);
    if (local != null && mounted) {
      setState(() {
        _mainBalance = local.balanceYuan;
        _mainBalanceLoading = false;
        _mainBalanceError = null;
      });
      if (local.isFresh(AccountBalanceSnapshotStore.displayTtl)) return;
    }
    setState(() {
      _mainBalanceLoading = local == null;
      _mainBalanceError = null;
    });

    try {
      final balance = await _duoqianTransferFeed.fetchInstitutionBalance(
        widget.institution,
        forceRefresh: force,
      );
      try {
        await balanceStore.put(
          accountHex: widget.institution.mainAccount,
          balanceYuan: balance,
        );
      } catch (_) {
        // 余额快照写入失败不影响当前链上余额展示。
      }
      if (!mounted) return;
      setState(() {
        _mainBalance = balance;
        _mainBalanceLoading = false;
        _mainBalanceError = null;
      });
    } catch (e, st) {
      debugPrint('[InstitutionDetail] main balance load failed: $e\n$st');
      if (!mounted) return;
      if (local == null) {
        setState(() {
          _mainBalanceError =
              SmoldotClientManager.instance.buildUserFacingError(e);
          _mainBalanceLoading = false;
        });
      }
    }
  }

  Future<void> _loadProposalEvents({bool force = false}) async {
    if (!mounted) return;
    final localLoaded = await _loadLocalProposalSummaries();
    final localFresh = await ProposalLocalStore.instance
        .isInstitutionIndexFresh(widget.institution.cidNumber)
        .catchError((_) => false);
    if (!force && localLoaded && localFresh) {
      if (!mounted) return;
      setState(() {
        _proposalLoading = false;
        _proposalError = null;
      });
      return;
    }

    setState(() {
      _proposalLoading = true;
      _proposalError = null;
    });

    try {
      final proposals =
          await _duoqianTransferFeed.fetchInstitutionVisibleProposals(
        widget.institution,
        forceRefresh: force,
      );
      final summaries = proposals
          .map(
            (proposal) => LocalProposalSummary.fromProposal(
              proposal,
              institution: widget.institution,
            ),
          )
          .toList(growable: false);
      await ProposalLocalStore.instance.upsertSummaries(summaries);
      await ProposalLocalStore.instance.putInstitutionIndex(
        widget.institution.cidNumber,
        summaries.map((summary) => summary.proposalId).toList(growable: false),
      );
      if (!mounted) return;
      setState(() {
        _proposalSummaries = summaries;
        _proposalDetailsById = {
          for (final proposal in proposals) proposal.meta.proposalId: proposal,
        };
        _proposalLoading = false;
        _proposalError = null;
      });
    } catch (e, st) {
      debugPrint('[InstitutionDetail] proposal load failed: $e\n$st');
      if (!mounted) return;
      setState(() {
        _proposalError = SmoldotClientManager.instance.buildUserFacingError(e);
        _proposalLoading = false;
      });
    }
  }

  Future<bool> _loadLocalProposalSummaries() async {
    try {
      final summaries = await ProposalLocalStore.instance
          .readInstitutionSummaries(widget.institution.cidNumber);
      if (!mounted || summaries.isEmpty) return summaries.isNotEmpty;
      setState(() {
        _proposalSummaries = summaries;
      });
      return true;
    } catch (e, st) {
      if (!WalletIsar.instance.isBusyError(e)) {
        debugPrint('[InstitutionDetail] local proposal load failed: $e\n$st');
      }
      return false;
    }
  }

  @override
  Widget build(BuildContext context) {
    return Scaffold(
      backgroundColor: AppTheme.scaffoldBg,
      appBar: AppBar(
        title: Text(
          widget.institution.cidShortName,
          style: const TextStyle(fontSize: 17, fontWeight: FontWeight.w700),
        ),
        centerTitle: true,
        foregroundColor: AppTheme.textPrimary,
      ),
      body: _buildContent(),
    );
  }

  Widget _buildContent() {
    return RefreshIndicator(
      onRefresh: () async {
        await _refreshAll(force: true);
      },
      child: ListView(
        padding: const EdgeInsets.fromLTRB(16, 8, 16, 32),
        children: [
          _buildInstitutionInfo(),
          const SizedBox(height: 12),
          _buildAccountsEntry(),
          const SizedBox(height: 12),
          _buildHeader(),
          const SizedBox(height: 12),
          if (_adminLoading && _admins.isEmpty) ...[
            _buildAdminGroupLoading(),
            const SizedBox(height: 12),
          ] else if (_adminError != null && _admins.isEmpty) ...[
            _buildAdminGroupError(),
            const SizedBox(height: 12),
          ] else if (_isCurrentUserAdmin) ...[
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

  // ──── 机构基础信息（身份ID / 主账户 / 主账户余额 / 法定代表人 / 所属地）────
  // 与公权机构详情统一:更多账户改为下方独立一行入口(_buildAccountsEntry)。

  Widget _buildInstitutionInfo() {
    final inst = widget.institution;
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
              icon: Icons.account_balance_outlined,
              label: '全称',
              value: inst.cidFullName,
            ),
            const Divider(height: 18),
            _buildAccountInfoTile(
              icon: Icons.label_outline,
              label: '简称',
              value: inst.cidShortName,
            ),
            const Divider(height: 18),
            _buildAccountInfoTile(
              icon: Icons.badge_outlined,
              label: '身份ID',
              value: inst.cidNumber,
            ),
            const Divider(height: 18),
            _buildAccountInfoTile(
              icon: Icons.account_balance_wallet_outlined,
              label: '主账户',
              // 完整 SS58 地址,不截断。
              value: _accountHexToSs58(inst.mainAccount),
            ),
            const Divider(height: 18),
            _buildAccountInfoTile(
              icon: Icons.payments_outlined,
              label: '主账户余额',
              value: _mainBalanceLabel(),
              valueColor: _mainBalanceColor(),
            ),
            const Divider(height: 18),
            // 法定代表人/所属地:按 cid 反查公权目录库(与公权详情同源),反查不到留空。
            _buildAccountInfoTile(
              icon: Icons.person_outline,
              label: '法定代表人',
              value: _directory?.legalRepName ?? '',
            ),
            const Divider(height: 18),
            _buildAccountInfoTile(
              icon: Icons.place_outlined,
              label: '所属地',
              value: _locationLabel(),
            ),
          ],
        ),
      ),
    );
  }

  /// 所属地展示:完整省名 + 市(与公权详情一致);反查不到留空。
  String _locationLabel() {
    final dir = _directory;
    if (dir == null) return '';
    final province = dir.provinceName ?? '';
    final city = dir.cityName ?? '';
    if (province.isEmpty && city.isEmpty) return '';
    if (province.isEmpty) return city;
    if (city.isEmpty) return province;
    return '$province · $city';
  }

  // ──── 机构账户入口（独立一行 + 箭头 → 全部账户页,与公权机构详情统一）────

  Widget _buildAccountsEntry() {
    final count = _accountCount();
    return Card(
      elevation: 0,
      margin: EdgeInsets.zero,
      shape: RoundedRectangleBorder(
        borderRadius: BorderRadius.circular(12),
        side: const BorderSide(color: AppTheme.border),
      ),
      child: InkWell(
        onTap: _openAccounts,
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
                child: const Icon(Icons.account_balance_wallet_outlined,
                    size: 18, color: AppTheme.primaryDark),
              ),
              const SizedBox(width: 12),
              Expanded(
                child: Column(
                  crossAxisAlignment: CrossAxisAlignment.start,
                  children: [
                    const Text(
                      '机构账户',
                      style: TextStyle(
                        fontSize: 15,
                        fontWeight: FontWeight.w600,
                        color: AppTheme.primaryDark,
                      ),
                    ),
                    const SizedBox(height: 2),
                    Text(
                      '共 $count 个账户',
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

  /// 账户条数:主账户 + 费用/安全基金/两和基金/永久质押中存在的。
  int _accountCount() {
    final accounts = widget.institution.accounts;
    var count = 1; // 主账户
    if (accounts?.feeAccount != null) count++;
    if (accounts?.anquanAccount != null) count++;
    if (accounts?.heAccount != null) count++;
    if (accounts?.stakeAccount != null) count++;
    return count;
  }

  void _openAccounts() {
    Navigator.of(context).push(
      MaterialPageRoute<void>(
        builder: (_) => GovernanceInstitutionAccountsPage(
          institution: widget.institution,
          badgeColor: widget.badgeColor,
        ),
      ),
    );
  }

  String _mainBalanceLabel() {
    final balance = _mainBalance;
    if (balance != null) return AmountFormat.format(balance, symbol: 'GMB');
    if (_mainBalanceLoading) return '读取中...';
    if (_mainBalanceError != null) return '读取失败';
    return '未读取';
  }

  Color _mainBalanceColor() {
    if (_mainBalance == null && _mainBalanceError != null) {
      return AppTheme.danger;
    }
    return _mainBalance == null ? AppTheme.textTertiary : AppTheme.textPrimary;
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
              // 中文注释:value 可能是完整 SS58 地址,允许换行,不截断。
              Text(
                value,
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
              // 左侧图标(36×36,与机构账户/管理员行齐高)
              Container(
                width: 36,
                height: 36,
                decoration: BoxDecoration(
                  color: widget.badgeColor.withValues(alpha: 0.12),
                  borderRadius: BorderRadius.circular(10),
                ),
                child: Icon(widget.icon, size: 18, color: widget.badgeColor),
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
                      _adminSummaryText(inst),
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

  String _adminSummaryText(InstitutionInfo inst) {
    if (_adminLoading && _admins.isEmpty) {
      return '管理员读取中　通过阈值 ${inst.internalThreshold}';
    }
    if (_adminError != null && _admins.isEmpty) {
      return '管理员读取失败　通过阈值 ${inst.internalThreshold}';
    }
    return '管理员 ${_admins.length} 人　通过阈值 ${inst.internalThreshold}';
  }

  String _adminEntrySubtitle() {
    if (_adminLoading && _admins.isEmpty) return '正在读取管理员列表';
    if (_adminError != null && _admins.isEmpty) return '管理员读取失败，点击重试';
    return '共 ${_admins.length} 位管理员';
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

  Widget _buildAdminGroupLoading() {
    return Container(
      padding: const EdgeInsets.symmetric(horizontal: 10, vertical: 8),
      decoration: AppTheme.bannerDecoration(AppTheme.textTertiary),
      child: const Row(
        mainAxisSize: MainAxisSize.min,
        children: [
          SizedBox(
            width: 14,
            height: 14,
            child: CircularProgressIndicator(strokeWidth: 2),
          ),
          SizedBox(width: 8),
          Text(
            '正在确认管理员身份',
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

  Widget _buildAdminGroupError() {
    return Container(
      padding: const EdgeInsets.symmetric(horizontal: 10, vertical: 6),
      decoration: AppTheme.bannerDecoration(AppTheme.danger),
      child: Row(
        children: [
          const Icon(Icons.error_outline, size: 14, color: AppTheme.danger),
          const SizedBox(width: 6),
          Expanded(
            child: Text(
              _adminError ?? '管理员身份读取失败',
              maxLines: 1,
              overflow: TextOverflow.ellipsis,
              style: const TextStyle(
                fontSize: 12,
                color: AppTheme.danger,
                fontWeight: FontWeight.w500,
              ),
            ),
          ),
          TextButton(
            onPressed: () => unawaited(_loadAdminsAndRole()),
            child: const Text('重试'),
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
        onTap: _adminLoading && _admins.isEmpty
            ? null
            : () {
                if (_adminError != null && _admins.isEmpty) {
                  unawaited(_loadAdminsAndRole());
                } else {
                  _openAdminList();
                }
              },
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
                      _adminEntrySubtitle(),
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
        if (_proposalLoading && _proposalSummaries.isEmpty)
          _buildProposalLoading()
        else if (_proposalError != null && _proposalSummaries.isEmpty)
          _buildProposalError()
        else if (_proposalSummaries.isEmpty)
          _buildEmptyProposalState()
        else if (_proposalLoading) ...[
          _buildProposalRefreshingHint(),
          const SizedBox(height: 8),
        ],
        ...List.generate(_proposalSummaries.length, (index) {
          final summary = _proposalSummaries[index];
          return Padding(
            padding: EdgeInsets.only(
                bottom: index < _proposalSummaries.length - 1 ? 8 : 0),
            child: _buildProposalCard(summary),
          );
        }),
      ],
    );
  }

  Widget _buildProposalLoading() {
    return Container(
      width: double.infinity,
      padding: const EdgeInsets.all(18),
      decoration: BoxDecoration(
        color: AppTheme.surfaceMuted,
        borderRadius: BorderRadius.circular(AppTheme.radiusMd),
        border: Border.all(color: AppTheme.border),
      ),
      child: const Row(
        mainAxisAlignment: MainAxisAlignment.center,
        children: [
          SizedBox(
            width: 16,
            height: 16,
            child: CircularProgressIndicator(strokeWidth: 2),
          ),
          SizedBox(width: 10),
          Text(
            '正在读取链上提案...',
            style: TextStyle(fontSize: 12, color: AppTheme.textSecondary),
          ),
        ],
      ),
    );
  }

  Widget _buildProposalError() {
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
              _proposalError ?? '提案读取失败',
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
            onPressed: () => unawaited(_loadProposalEvents(force: true)),
            child: const Text('重试'),
          ),
        ],
      ),
    );
  }

  Widget _buildProposalRefreshingHint() {
    return Container(
      width: double.infinity,
      padding: const EdgeInsets.symmetric(horizontal: 10, vertical: 6),
      decoration: AppTheme.bannerDecoration(AppTheme.textTertiary),
      child: const Row(
        children: [
          SizedBox(
            width: 12,
            height: 12,
            child: CircularProgressIndicator(strokeWidth: 2),
          ),
          SizedBox(width: 8),
          Text(
            '正在刷新提案状态',
            style: TextStyle(fontSize: 12, color: AppTheme.textTertiary),
          ),
        ],
      ),
    );
  }

  Widget _buildEmptyProposalState() {
    return Container(
      width: double.infinity,
      padding: const EdgeInsets.all(24),
      decoration: BoxDecoration(
        color: AppTheme.surfaceMuted,
        borderRadius: BorderRadius.circular(AppTheme.radiusMd),
        border: Border.all(color: AppTheme.border),
      ),
      child: const Column(
        children: [
          Icon(Icons.ballot_outlined, size: 40, color: AppTheme.textTertiary),
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

  IconData _proposalIcon(LocalProposalSummary summary) {
    return switch (summary.iconKind) {
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

  Widget _buildProposalCard(LocalProposalSummary summary) {
    final statusColor = _statusColor(summary.status);
    return Card(
      elevation: 0,
      margin: EdgeInsets.zero,
      shape: RoundedRectangleBorder(
        borderRadius: BorderRadius.circular(12),
        side: BorderSide(color: statusColor.withValues(alpha: 0.2)),
      ),
      child: InkWell(
        onTap: () => _openProposalDetail(summary),
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
                    Icon(_proposalIcon(summary), size: 18, color: statusColor),
              ),
              const SizedBox(width: 12),
              Expanded(
                child: Column(
                  crossAxisAlignment: CrossAxisAlignment.start,
                  children: [
                    Text(
                      summary.title,
                      style: const TextStyle(
                        fontSize: 15,
                        fontWeight: FontWeight.w600,
                        color: AppTheme.primaryDark,
                      ),
                    ),
                    const SizedBox(height: 2),
                    Text(
                      summary.subtitle,
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
                  _statusLabel(summary.status),
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
        builder: (_) => GovernanceProposalsPage(
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
      unawaited(_refreshAll(force: true));
    }
  }

  Future<void> _openProposalDetail(LocalProposalSummary summary) async {
    final proposal = await _resolveProposalDetail(summary);
    if (!mounted) return;
    if (proposal == null) {
      ScaffoldMessenger.of(context).showSnackBar(
        const SnackBar(content: Text('提案详情读取失败，请稍后重试')),
      );
      return;
    }
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
    } else if (DuoqianTransferProposalAdapter.matches(proposal)) {
      await DuoqianTransferProposalAdapter.openDetail(
        context,
        proposal: proposal,
        institution: widget.institution,
        proposalContext: ctx,
      );
    } else if (proposal.createDuoqianDetail != null ||
        proposal.closeDuoqianDetail != null) {
      await Navigator.of(context).push(
        MaterialPageRoute(
          builder: (_) => InstitutionManageDetailPage(
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
      unawaited(_refreshAll(force: true));
    }
  }

  Future<ProposalWithDetail?> _resolveProposalDetail(
    LocalProposalSummary summary,
  ) async {
    final cached = _proposalDetailsById[summary.proposalId];
    if (cached != null) return cached;
    try {
      final fresh =
          await _duoqianTransferFeed.fetchProposalsByIds([summary.proposalId]);
      if (fresh.isEmpty) return null;
      final proposal = fresh.first;
      final refreshedSummary = LocalProposalSummary.fromProposal(
        proposal,
        institution: widget.institution,
      );
      await ProposalLocalStore.instance.upsertSummaries([refreshedSummary]);
      if (!mounted) return proposal;
      setState(() {
        _proposalDetailsById = {
          ..._proposalDetailsById,
          proposal.meta.proposalId: proposal,
        };
        _proposalSummaries = [
          for (final item in _proposalSummaries)
            if (item.proposalId == refreshedSummary.proposalId)
              refreshedSummary
            else
              item,
        ];
      });
      return proposal;
    } catch (e, st) {
      debugPrint('[InstitutionDetail] proposal detail resolve failed: $e\n$st');
      return null;
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

  void _openAdminList() {
    Navigator.of(context).push(
      MaterialPageRoute(
        builder: (_) => AdminListPage(
          institution: widget.institution,
          accountIdentity: _accountIdentity,
          admins: _admins,
          importedColdPubkeys: _importedColdPubkeys,
          activatedPubkeys: _activatedPubkeys,
          badgeColor: widget.badgeColor,
          onActivated: () {
            // 中文注释：激活只影响当前用户管理员身份和管理员列表展示，局部刷新即可。
            _adminService.clearCache(_accountIdentity);
            _contextResolver.clearWalletCache();
            unawaited(_loadAdminsAndRole());
          },
        ),
      ),
    );
  }
}
