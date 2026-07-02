import 'dart:async';

import 'package:flutter/material.dart';

import 'package:citizenapp/citizen/institution/institution.dart';
import 'package:citizenapp/citizen/institution/institution_accounts.dart';
import 'package:citizenapp/citizen/institution/institution_accounts_page.dart';
import 'package:citizenapp/citizen/institution/institution_chain_state.dart';
import 'package:citizenapp/citizen/institution/institution_classification.dart';
import 'package:citizenapp/citizen/institution/institution_repository.dart';
import 'package:citizenapp/citizen/public/public_institution_admin_list_page.dart';
import 'package:citizenapp/citizen/legislation/data/law_models.dart';
import 'package:citizenapp/citizen/legislation/law_list_page.dart';
import 'package:citizenapp/citizen/proposal/admins-change/models/admin_account.dart';
import 'package:citizenapp/citizen/proposal/admins-change/services/admin_activation_service.dart';
import 'package:citizenapp/citizen/proposal/admins-change/services/institution_admin_service.dart';
import 'package:citizenapp/citizen/proposal/proposal_entry_page.dart';
import 'package:citizenapp/citizen/shared/institution_manage_detail_page.dart';
import 'package:citizenapp/citizen/institution/institution_admin_list_page.dart';
import 'package:citizenapp/citizen/proposal/runtime-upgrade/runtime_upgrade_detail_page.dart';
import 'package:citizenapp/citizen/shared/admin_profile.dart';
import 'package:citizenapp/citizen/shared/institution_info.dart';
import 'package:citizenapp/citizen/shared/proposal/proposal_context.dart';
import 'package:citizenapp/citizen/shared/proposal/proposal_local_store.dart';
import 'package:citizenapp/citizen/shared/proposal/proposal_models.dart';
import 'package:citizenapp/transaction/multisig-transfer/multisig_transfer_proposal_adapter.dart';
import 'package:citizenapp/ui/app_theme.dart';
import 'package:citizenapp/wallet/core/wallet_manager.dart';

/// 统一机构详情页(ADR-028 决策 2/6)——替代公权 `PublicInstitutionDetailPage`
/// 与治理 `InstitutionDetailPage` 两套。
///
/// 公共壳(信息卡/账户/管理员/提案列表/关注)对全部机构统一;按机构类型
/// dispatch 重型流:
/// - 储备治理三档(NRC/PRC/PRB):提案列表可点→`_openProposalDetail`;
/// - 其余注册机构:提案列表仍走只读摘要,但发起提案/管理员激活共用统一入口。
/// 提案能力由 `ProposalCapabilityRegistry` 判断,详情页不再散落机构码 if。
class InstitutionDetailPage extends StatefulWidget {
  const InstitutionDetailPage({
    super.key,
    required this.cidNumber,
    required this.repository,
    this.chainState,
    this.walletPubkeyProvider,
  });

  final String cidNumber;
  final InstitutionRepository repository;

  /// 链态读服务(余额/管理员/提案);测试注入,默认 Live。
  final InstitutionChainState? chainState;

  /// 活动钱包公钥(订阅 + 是否管理员);测试注入,默认 WalletManager。
  final Future<String?> Function()? walletPubkeyProvider;

  @override
  State<InstitutionDetailPage> createState() => _InstitutionDetailPageState();
}

class _InstitutionDetailPageState extends State<InstitutionDetailPage> {
  late final InstitutionChainState _chainState =
      widget.chainState ?? LiveInstitutionChainState();

  final InstitutionAdminService _adminService = InstitutionAdminService();
  final WalletManager _walletManager = WalletManager();
  final MultisigTransferProposalFeed _multisigTransferFeed =
      MultisigTransferProposalFeed();
  final ActivationService _activationService = ActivationService();
  late final ProposalContextResolver _contextResolver = ProposalContextResolver(
    adminService: _adminService,
    walletManager: _walletManager,
    activationService: _activationService,
  );

  Institution? _inst;

  /// 提案/管理员入口使用的链上主体信息。固定治理档来自静态注册表,注册机构账户
  /// 则由目录机构派生出 `institution-account:<mainAccount>` identity。
  InstitutionInfo? _govInfo;
  bool get _isGovernance =>
      InstitutionClassification.isGovernance(_inst?.institutionCode ?? '');

  bool _loading = true;

  String? _activePubkey;
  bool _subscribed = false;

  List<InstitutionAccountRow> _accounts = const [];
  String _areaPath = '';

  double? _mainBalanceYuan;
  bool _mainBalanceLoading = true;

  List<String> _admins = const [];
  List<AdminProfile> _adminProfiles = const [];

  // 治理路径专用(管理员角色 / 激活 / 富提案列表)。
  List<WalletProfile> _adminWallets = const [];
  bool _isCurrentUserAdmin = false;
  Set<String> _importedColdPubkeys = const {};
  Set<String> _activatedPubkeys = const {};
  List<LocalProposalSummary> _govProposals = const [];
  Map<int, ProposalWithDetail> _govProposalDetailsById = const {};

  // 公权路径专用(只读提案摘要)。
  List<InstitutionProposalSummary> _publicProposals = const [];

  AdminAccountIdentity? get _accountIdentity {
    final info = _govInfo;
    if (info == null) return null;
    try {
      return AdminAccountIdentity.fromInstitution(info);
    } on ArgumentError {
      // 非治理且非注册账户身份暂无法解析 → 优雅降级(提案入口仍开,但需激活后才能发起)。
      return null;
    }
  }

  @override
  void initState() {
    super.initState();
    _load();
  }

  Future<String?> _resolvePubkey() async {
    final provider = widget.walletPubkeyProvider;
    if (provider != null) return provider();
    return (await _walletManager.getWallet())?.pubkeyHex;
  }

  Future<void> _load() async {
    final inst = await widget.repository.getByCid(widget.cidNumber);
    final pubkey = await _resolvePubkey();
    if (!mounted) return;
    if (inst == null) {
      setState(() => _loading = false);
      return;
    }
    // 全机构统一开提案入口:治理三类用静态档(含安全基金等专户),其余从机构派生
    // 注册机构账户 identity。是否能发起某类提案交给 ProposalCapabilityRegistry。
    final govInfo = widget.repository.governanceInfo(inst.cidNumber) ??
        _infoFromInstitution(inst);
    final subscribed = pubkey == null
        ? false
        : await widget.repository.isSubscribed(pubkey, inst.cidNumber);
    final areaPath = await widget.repository.institutionAreaPath(inst);
    if (!mounted) return;
    setState(() {
      _inst = inst;
      _govInfo = govInfo;
      _activePubkey = pubkey;
      _subscribed = subscribed;
      _accounts = institutionAccountRows(inst);
      _areaPath = areaPath;
      _loading = false;
    });
    unawaited(_loadDynamics());
  }

  /// 为非治理注册机构从 Institution 派生 InstitutionInfo(主/费账户 + 机构码)。
  ///
  /// 这里的 `cidNumber` 故意使用 `institution-account:<mainAccount>`,
  /// 因为转账、管理员更换等链上 call 需要的是被管理账户 identity;真实 CID 仍保留在
  /// Institution 页面模型中用于展示和目录查询。
  InstitutionInfo _infoFromInstitution(Institution inst) {
    final rows = institutionAccountRows(inst);
    final main = rows.isNotEmpty ? rows.first.accountHex : '';
    final fee = rows.length > 1 ? rows[1].accountHex : null;
    return InstitutionInfo(
      cidFullName: inst.cidFullName,
      cidShortName: inst.displayName,
      cidFullNameEn: inst.cidFullName, // 普通公权机构暂无英文名,中文兜底
      cidShortNameEn: inst.displayName,
      cidNumber: registeredAccountIdentity(main),
      orgType: inst.orgType,
      accounts: InstitutionAccounts(mainAccount: main, feeAccount: fee),
      adminAccountCode: inst.institutionCode,
    );
  }

  Future<void> _loadDynamics({bool force = false}) async {
    final inst = _inst;
    if (inst == null) return;

    // 主账户余额(批量接口查一条)。
    final mainHex = _accounts.isNotEmpty ? _accounts.first.accountHex : '';
    try {
      final balances = await _chainState.balances([mainHex]);
      if (mounted) {
        setState(() {
          _mainBalanceYuan = balances[mainHex];
          _mainBalanceLoading = false;
        });
      }
    } on Exception {
      if (mounted) setState(() => _mainBalanceLoading = false);
    }

    if (_isGovernance) {
      await _loadGovernanceAdminsAndRole(force: force);
      await _loadGovernanceProposals(force: force);
    } else {
      unawaited(_loadGovernanceAdminsAndRole(force: force));
      await _loadPublicDynamics(inst);
    }
  }

  // ──── 管理员角色加载(固定治理与注册机构账户共用)────

  Future<void> _loadGovernanceAdminsAndRole({bool force = false}) async {
    final identity = _accountIdentity;
    final govInfo = _govInfo;
    if (identity == null || govInfo == null) return;
    if (force) {
      _adminService.clearCache(identity);
      _contextResolver.clearWalletCache();
    }
    try {
      final results = await Future.wait<Object>([
        _adminService.fetchAdmins(identity),
        _contextResolver.resolve(knownInstitution: govInfo),
        _activationService
            .getActivatedAdmins(identity)
            .catchError((_) => <ActivatedAdmin>[]),
        _adminService.fetchAdminProfiles(identity),
      ]);
      final admins = results[0] as List<String>;
      final ctx = results[1] as ProposalContext;
      final adminProfiles = results[3] as List<AdminProfile>;
      final activated = results[2] as List<ActivatedAdmin>;
      final coldPubkeys = await _loadImportedColdPubkeys(admins);
      if (ctx.isAdmin) {
        ProposalContextResolver.markInstitutionAdmin(
          _inst?.cidNumber ?? govInfo.cidNumber,
        );
      }
      if (!mounted) return;
      final shouldUpdateAdmins =
          _isGovernance || admins.isNotEmpty || _admins.isEmpty;
      setState(() {
        if (shouldUpdateAdmins) {
          _admins = admins;
          _adminProfiles = adminProfiles;
        }
        _govInfo = ctx.institution ?? govInfo;
        _adminWallets = ctx.adminWallets;
        _importedColdPubkeys = coldPubkeys;
        _activatedPubkeys = activated.map((a) => a.pubkeyHex).toSet();
        _isCurrentUserAdmin = ctx.isAdmin;
      });
    } catch (_) {
      // 联网失败保持空,不崩(治理角色仅影响发起入口可用态)。
    }
  }

  Future<Set<String>> _loadImportedColdPubkeys(List<String> admins) async {
    final coldPubkeys = <String>{};
    try {
      final allWallets = await _walletManager.getWallets();
      for (final w in allWallets) {
        if (w.isColdWallet) {
          var pk = w.pubkeyHex.toLowerCase();
          if (pk.startsWith('0x')) pk = pk.substring(2);
          if (admins.contains(pk)) coldPubkeys.add(pk);
        }
      }
    } on Exception {
      // 本地钱包库异常不影响展示。
    }
    return coldPubkeys;
  }

  Future<void> _loadGovernanceProposals({bool force = false}) async {
    final govInfo = _govInfo;
    if (govInfo == null) return;
    try {
      final proposals = await _multisigTransferFeed
          .fetchInstitutionVisibleProposals(govInfo, forceRefresh: force);
      final summaries = proposals
          .map(
              (p) => LocalProposalSummary.fromProposal(p, institution: govInfo))
          .toList(growable: false);
      await ProposalLocalStore.instance.upsertSummaries(summaries);
      await ProposalLocalStore.instance.putInstitutionIndex(
        govInfo.cidNumber,
        summaries.map((s) => s.proposalId).toList(growable: false),
      );
      if (!mounted) return;
      setState(() {
        _govProposals = summaries;
        _govProposalDetailsById = {
          for (final p in proposals) p.meta.proposalId: p,
        };
      });
    } catch (_) {
      // 同上,保持空。
    }
  }

  // ──── 注册机构路径加载 ────

  Future<void> _loadPublicDynamics(Institution inst) async {
    try {
      final profiles = await _chainState.adminProfiles(inst);
      if (mounted) {
        setState(() {
          _adminProfiles = profiles;
          _admins = profiles.map((p) => p.account).toList(growable: false);
        });
      }
    } on Exception {
      // 保持空。
    }
    try {
      final proposals = await _chainState.proposals(inst);
      if (mounted) setState(() => _publicProposals = proposals);
    } on Exception {
      // 保持空。
    }
  }

  Future<void> _toggleSubscribe() async {
    final inst = _inst;
    final pubkey = _activePubkey;
    if (inst == null || pubkey == null) return;
    if (_subscribed) {
      await widget.repository.unsubscribe(pubkey, inst.cidNumber);
    } else {
      await widget.repository.subscribe(pubkey, inst.cidNumber);
    }
    if (!mounted) return;
    setState(() => _subscribed = !_subscribed);
  }

  @override
  Widget build(BuildContext context) {
    final inst = _inst;
    return Scaffold(
      backgroundColor: AppTheme.scaffoldBg,
      appBar: AppBar(
        title: Text(inst?.displayName ?? '机构'),
        backgroundColor: AppTheme.surfaceWhite,
        foregroundColor: AppTheme.textPrimary,
        elevation: 0,
        actions: [
          if (inst != null && _activePubkey != null)
            IconButton(
              tooltip: _subscribed ? '取消关注' : '订阅关注',
              icon: Icon(
                _subscribed ? Icons.bookmark : Icons.bookmark_border,
                color: _subscribed ? AppTheme.primary : AppTheme.textSecondary,
              ),
              onPressed: _toggleSubscribe,
            ),
        ],
      ),
      body: _buildBody(inst),
    );
  }

  Widget _buildBody(Institution? inst) {
    if (_loading) {
      return const Center(child: CircularProgressIndicator(strokeWidth: 2));
    }
    if (inst == null) {
      return const Center(
        child: Text('未找到该机构', style: TextStyle(color: AppTheme.textTertiary)),
      );
    }
    return ListView(
      padding: const EdgeInsets.all(16),
      children: [
        _infoCard(inst),
        const SizedBox(height: 12),
        _accountsEntry(inst),
        const SizedBox(height: 12),
        _proposalEntry(),
        // 法律原文(仅立法机构):查看该机构全部法律。发起立法=类B,归口提案入口
        // (proposal_entry_page,按 registry 立法机构→发起立法),不在详情页另设入口。
        if (_lawTarget(inst) != null) ...[
          const SizedBox(height: 12),
          _lawOriginalEntry(inst),
        ],
        const SizedBox(height: 12),
        _adminsEntry(),
        const SizedBox(height: 12),
        _proposalList(),
      ],
    );
  }

  // ──── ① 机构信息卡(全称/身份ID/主账户/余额/法代/所属地;非法人 +所属上级法人)────

  Widget _infoCard(Institution inst) {
    final mainSs58 = _accounts.isNotEmpty ? _accounts.first.addressSs58 : '—';
    return Container(
      decoration: BoxDecoration(
        color: AppTheme.surfaceCard,
        borderRadius: BorderRadius.circular(12),
        border: Border.all(color: AppTheme.primary.withValues(alpha: 0.18)),
      ),
      child: Padding(
        padding: const EdgeInsets.symmetric(horizontal: 14, vertical: 12),
        child: Column(
          crossAxisAlignment: CrossAxisAlignment.start,
          children: [
            _infoTile(
                icon: Icons.account_balance_outlined,
                label: '全称',
                value: inst.cidFullName),
            const Divider(height: 18),
            _infoTile(
                icon: Icons.badge_outlined,
                label: '身份ID',
                value: inst.cidNumber),
            const Divider(height: 18),
            _infoTile(
                icon: Icons.account_balance_wallet_outlined,
                label: '主账户',
                value: mainSs58),
            const Divider(height: 18),
            _infoTile(
                icon: Icons.payments_outlined,
                label: '主账户余额',
                value: _mainBalanceLabel()),
            const Divider(height: 18),
            _infoTile(
                icon: Icons.person_outline,
                label: '法定代表人',
                value: inst.legalRepName ?? ''),
            const Divider(height: 18),
            _infoTile(
                icon: Icons.place_outlined, label: '所属地', value: _areaPath),
            // 非法人加显「所属上级法人全称」(ADR-028 决策 6)。
            if (inst.isUnincorporated) ...[
              const Divider(height: 18),
              _infoTile(
                icon: Icons.account_tree_outlined,
                label: '所属上级法人',
                value: inst.parentCidNumber ?? '',
              ),
            ],
          ],
        ),
      ),
    );
  }

  String _mainBalanceLabel() {
    if (_mainBalanceLoading) return '读取中...';
    final yuan = _mainBalanceYuan;
    if (yuan == null) return '未激活';
    return '${yuan.toStringAsFixed(2)} 元';
  }

  // ──── ② 机构账户入口 ────

  Widget _accountsEntry(Institution inst) {
    return _entryCard(
      icon: Icons.account_balance_wallet_outlined,
      title: '机构账户',
      subtitle: '共 ${_accounts.length} 个账户',
      onTap: () => Navigator.of(context).push(
        MaterialPageRoute<void>(
          builder: (_) => InstitutionAccountsPage(
            institution: inst,
            chainState: _chainState,
          ),
        ),
      ),
    );
  }

  // ──── ③ 提案入口(按主体能力统一展示)────

  Widget _proposalEntry() {
    return _entryCard(
      icon: Icons.how_to_vote_outlined,
      title: '发起提案',
      subtitle: _isCurrentUserAdmin ? '转账 / 管理员更换 / …' : '激活管理员后可发起',
      onTap: _openProposalTypes,
    );
  }

  // ──── 法律原文入口(仅立法机构,ADR-028 P3-1)────

  /// 立法机构 → (tier, scope_code);非立法机构返回 null。国家级 scope=0;省/市级
  /// scope 取行政区数字 code(省码为字母时回退 0,待链端有省/市级法律后核验映射;
  /// 当前仅宪法 law_id=0 经顶部卡直达,本入口对其余立法机构暂为空)。
  ({LawTier tier, int scope})? _lawTarget(Institution inst) {
    const national = {'NLG', 'NRP', 'NSN', 'NED'};
    const provincial = {'PLG', 'PRP', 'PSN'};
    const municipal = {'CLEG'};
    final code = inst.institutionCode;
    if (national.contains(code)) return (tier: LawTier.national, scope: 0);
    if (provincial.contains(code)) {
      return (
        tier: LawTier.provincial,
        scope: int.tryParse(inst.provinceCode) ?? 0
      );
    }
    if (municipal.contains(code)) {
      return (tier: LawTier.municipal, scope: int.tryParse(inst.cityCode) ?? 0);
    }
    return null;
  }

  Widget _lawOriginalEntry(Institution inst) {
    final target = _lawTarget(inst)!;
    return _entryCard(
      icon: Icons.menu_book_outlined,
      title: '法律原文',
      subtitle: '该机构制定的全部法律',
      onTap: () => Navigator.of(context).push(
        MaterialPageRoute<void>(
          builder: (_) => LawListPage(
            tier: target.tier,
            scopeCode: target.scope,
            title: '${inst.displayName} · 法律原文',
          ),
        ),
      ),
    );
  }

  Future<void> _openProposalTypes() async {
    final govInfo = _govInfo;
    if (govInfo == null) return;
    await Navigator.of(context).push<bool>(
      MaterialPageRoute(
        builder: (_) => ProposalEntryPage(
          institution: govInfo,
          institutionCode: _inst?.institutionCode ?? '',
          icon: Icons.account_balance,
          badgeColor: AppTheme.primary,
          adminWallets: _adminWallets,
          isActivated: _isCurrentUserAdmin,
        ),
      ),
    );
    if (mounted) unawaited(_loadDynamics(force: true));
  }

  // ──── ④ 管理员入口(治理→AdminListPage 含激活;公权→只读列表)────

  Widget _adminsEntry() {
    return _entryCard(
      icon: Icons.people_outline,
      title: '管理员',
      subtitle: '共 ${_admins.length} 位管理员',
      onTap: _accountIdentity != null
          ? _openGovernanceAdminList
          : _openPublicAdminList,
    );
  }

  void _openGovernanceAdminList() {
    final govInfo = _govInfo;
    final identity = _accountIdentity;
    if (govInfo == null || identity == null) return;
    Navigator.of(context).push(
      MaterialPageRoute<void>(
        builder: (_) => AdminListPage(
          institution: govInfo,
          accountIdentity: identity,
          admins: _adminProfiles,
          importedColdPubkeys: _importedColdPubkeys,
          activatedPubkeys: _activatedPubkeys,
          badgeColor: AppTheme.primary,
          onActivated: () {
            _adminService.clearCache(identity);
            _contextResolver.clearWalletCache();
            unawaited(_loadGovernanceAdminsAndRole());
          },
        ),
      ),
    );
  }

  void _openPublicAdminList() {
    Navigator.of(context).push(
      MaterialPageRoute<void>(
        builder: (_) => PublicInstitutionAdminListPage(admins: _adminProfiles),
      ),
    );
  }

  // ──── ⑤ 提案列表 ────

  Widget _proposalList() {
    final hasGov = _isGovernance && _govProposals.isNotEmpty;
    final hasPublic = !_isGovernance && _publicProposals.isNotEmpty;
    return Column(
      crossAxisAlignment: CrossAxisAlignment.start,
      children: [
        const Padding(
          padding: EdgeInsets.only(left: 2, bottom: 12),
          child: Text('提案列表',
              style: TextStyle(
                  fontSize: 16,
                  fontWeight: FontWeight.w700,
                  color: AppTheme.primaryDark)),
        ),
        if (!hasGov && !hasPublic)
          _emptyProposalState()
        else if (_isGovernance)
          ...List.generate(_govProposals.length, (i) {
            final s = _govProposals[i];
            return Padding(
              padding: EdgeInsets.only(
                  bottom: i < _govProposals.length - 1 ? 10 : 0),
              child: _govProposalCard(s),
            );
          })
        else
          ...List.generate(_publicProposals.length, (i) {
            return Padding(
              padding: EdgeInsets.only(
                  bottom: i < _publicProposals.length - 1 ? 10 : 0),
              child: _publicProposalCard(_publicProposals[i]),
            );
          }),
      ],
    );
  }

  Widget _emptyProposalState() {
    return Container(
      width: double.infinity,
      padding: const EdgeInsets.all(24),
      decoration: BoxDecoration(
        color: AppTheme.surfaceMuted,
        borderRadius: BorderRadius.circular(12),
        border: Border.all(color: AppTheme.border),
      ),
      child: const Column(
        children: [
          Icon(Icons.ballot_outlined, size: 40, color: AppTheme.textTertiary),
          SizedBox(height: 8),
          Text('暂无提案',
              style: TextStyle(fontSize: 14, color: AppTheme.textSecondary)),
        ],
      ),
    );
  }

  // 治理:可点卡片 → _openProposalDetail。
  Widget _govProposalCard(LocalProposalSummary s) {
    final statusColor = AppTheme.proposalStatusColor(s.status);
    return InkWell(
      borderRadius: BorderRadius.circular(12),
      onTap: () => _openProposalDetail(s),
      child: _proposalCardBody(
        title: s.displayId,
        subtitle: s.listSubtitle,
        statusColor: statusColor,
        statusLabel: _statusLabel(s.status),
        trailingChevron: true,
      ),
    );
  }

  // 公权:只读卡片。
  Widget _publicProposalCard(InstitutionProposalSummary p) {
    final statusColor = AppTheme.proposalStatusColor(p.status);
    return _proposalCardBody(
      title: p.idLabel,
      subtitle: null,
      statusColor: statusColor,
      statusLabel: p.statusLabel,
      trailingChevron: false,
    );
  }

  Widget _proposalCardBody({
    required String title,
    required String? subtitle,
    required Color statusColor,
    required String statusLabel,
    required bool trailingChevron,
  }) {
    return Container(
      padding: const EdgeInsets.symmetric(horizontal: 14, vertical: 12),
      decoration: BoxDecoration(
        color: AppTheme.surfaceCard,
        borderRadius: BorderRadius.circular(12),
        border: Border.all(color: statusColor.withValues(alpha: 0.2)),
      ),
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
                Icon(Icons.how_to_vote_outlined, size: 18, color: statusColor),
          ),
          const SizedBox(width: 12),
          Expanded(
            child: Column(
              crossAxisAlignment: CrossAxisAlignment.start,
              children: [
                Text(title,
                    style: const TextStyle(
                        fontSize: 15,
                        fontWeight: FontWeight.w600,
                        color: AppTheme.primaryDark)),
                if (subtitle != null && subtitle.isNotEmpty) ...[
                  const SizedBox(height: 2),
                  Text(subtitle,
                      style: const TextStyle(
                          fontSize: 12, color: AppTheme.textTertiary)),
                ],
              ],
            ),
          ),
          Container(
            padding: const EdgeInsets.symmetric(horizontal: 8, vertical: 2),
            decoration: BoxDecoration(
              color: statusColor.withValues(alpha: 0.1),
              borderRadius: BorderRadius.circular(10),
            ),
            child: Text(statusLabel,
                style: TextStyle(
                    fontSize: 11,
                    fontWeight: FontWeight.w600,
                    color: statusColor)),
          ),
          if (trailingChevron) ...[
            const SizedBox(width: 4),
            const Icon(Icons.chevron_right,
                size: 20, color: AppTheme.textTertiary),
          ],
        ],
      ),
    );
  }

  String _statusLabel(int status) => switch (status) {
        1 => '已通过',
        2 => '已拒绝',
        3 => '已执行',
        4 => '执行失败',
        _ => '投票中',
      };

  // ──── 治理提案详情路由(port 自治理详情页)────

  Future<void> _openProposalDetail(LocalProposalSummary summary) async {
    final govInfo = _govInfo;
    if (govInfo == null) return;
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
      institution: govInfo,
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
    } else if (MultisigTransferProposalAdapter.matches(proposal)) {
      await MultisigTransferProposalAdapter.openDetail(
        context,
        proposal: proposal,
        institution: govInfo,
        proposalContext: ctx,
      );
    } else if (proposal.createMultisigDetail != null ||
        proposal.closeMultisigDetail != null) {
      await Navigator.of(context).push(
        MaterialPageRoute(
          builder: (_) => InstitutionManageDetailPage(
            institution: govInfo,
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
    if (mounted) unawaited(_loadDynamics(force: true));
  }

  Future<ProposalWithDetail?> _resolveProposalDetail(
    LocalProposalSummary summary,
  ) async {
    final cached = _govProposalDetailsById[summary.proposalId];
    if (cached != null) return cached;
    try {
      final fresh =
          await _multisigTransferFeed.fetchProposalsByIds([summary.proposalId]);
      if (fresh.isEmpty) return null;
      final proposal = fresh.first;
      if (mounted) {
        setState(() {
          _govProposalDetailsById = {
            ..._govProposalDetailsById,
            proposal.meta.proposalId: proposal,
          };
        });
      }
      return proposal;
    } catch (_) {
      return null;
    }
  }

  // ──── 公用零件 ────

  Widget _infoTile({
    required IconData icon,
    required String label,
    required String value,
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
          child: Icon(icon, size: 16, color: AppTheme.primary),
        ),
        const SizedBox(width: 10),
        Expanded(
          child: Column(
            crossAxisAlignment: CrossAxisAlignment.start,
            children: [
              Text(label,
                  style: const TextStyle(
                      fontSize: 11,
                      color: AppTheme.textTertiary,
                      fontWeight: FontWeight.w500)),
              const SizedBox(height: 2),
              Text(value,
                  style: const TextStyle(
                      fontSize: 13,
                      color: AppTheme.textPrimary,
                      fontWeight: FontWeight.w600)),
            ],
          ),
        ),
      ],
    );
  }

  Widget _entryCard({
    required IconData icon,
    required String title,
    required String subtitle,
    required VoidCallback onTap,
  }) {
    return Container(
      decoration: BoxDecoration(
        color: AppTheme.surfaceCard,
        borderRadius: BorderRadius.circular(12),
        border: Border.all(color: AppTheme.border),
      ),
      child: InkWell(
        onTap: onTap,
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
                child: Icon(icon, size: 18, color: AppTheme.primaryDark),
              ),
              const SizedBox(width: 12),
              Expanded(
                child: Column(
                  crossAxisAlignment: CrossAxisAlignment.start,
                  children: [
                    Text(title,
                        style: const TextStyle(
                            fontSize: 15,
                            fontWeight: FontWeight.w600,
                            color: AppTheme.primaryDark)),
                    const SizedBox(height: 2),
                    Text(subtitle,
                        style: const TextStyle(
                            fontSize: 12, color: AppTheme.textTertiary)),
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
}
