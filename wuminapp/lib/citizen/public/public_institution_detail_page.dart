import 'package:flutter/material.dart';

import 'package:wuminapp_mobile/citizen/public/data/public_institution_accounts.dart';
import 'package:wuminapp_mobile/citizen/public/data/public_institution_chain_data.dart';
import 'package:wuminapp_mobile/citizen/public/data/public_institution_repository.dart';
import 'package:wuminapp_mobile/citizen/public/public_institution_accounts_page.dart';
import 'package:wuminapp_mobile/citizen/public/public_institution_admin_list_page.dart';
import 'package:wuminapp_mobile/governance/shared/account_derivation.dart';
import 'package:wuminapp_mobile/isar/wallet_isar.dart';
import 'package:wuminapp_mobile/ui/app_theme.dart';
import 'package:wuminapp_mobile/wallet/core/wallet_manager.dart';

/// 公权机构详情页(ADR-018 §九 卡C)。
///
/// 中文注释:v1=浏览 + 订阅 + 动态展示,版式对齐治理机构详情页。
/// 自上而下五段:机构信息(身份ID/法定代表人/所属地) → 机构账户入口 →
/// 提案发起入口(本期占位) → 管理员入口 → 提案列表。账户全本地派生(卡0),
/// 余额只在「全部账户页」展示;管理员/提案走链读(卡①⑤)。右上角订阅写本地 store(卡A)。
class PublicInstitutionDetailPage extends StatefulWidget {
  const PublicInstitutionDetailPage({
    super.key,
    required this.sfidNumber,
    required this.repository,
    this.chainData,
    this.walletPubkeyProvider,
  });

  final String sfidNumber;
  final PublicInstitutionRepository repository;

  /// 链上数据源(余额/管理员/提案);测试注入,默认 Live。
  final PublicInstitutionChainData? chainData;

  /// 活动钱包公钥(订阅 + 是否管理员);测试注入,默认 WalletManager。
  final Future<String?> Function()? walletPubkeyProvider;

  @override
  State<PublicInstitutionDetailPage> createState() =>
      _PublicInstitutionDetailPageState();
}

class _PublicInstitutionDetailPageState
    extends State<PublicInstitutionDetailPage> {
  late final PublicInstitutionChainData _chainData =
      widget.chainData ?? LivePublicInstitutionChainData();

  PublicInstitutionEntity? _inst;
  bool _loading = true;

  String? _activePubkey;
  bool _subscribed = false;

  /// 账户行本地派生(卡0,零网络);本页只用条数,余额在「全部账户页」补。
  List<PublicAccountRow> _accounts = const [];

  List<String> _admins = const [];
  List<PublicProposalSummary> _proposals = const [];

  /// 主账户余额(元);对齐治理详情在机构信息卡展示。null=未激活/未读到。
  double? _mainBalanceYuan;
  bool _mainBalanceLoading = true;

  /// 所属地预 join 显示路径(省名·市名[·镇名]);字典缺失回退 code(repo 兜底)。
  /// 在 [_load] 里预 join,不在 build 里 await(ADR-021)。
  String _areaPath = '';

  @override
  void initState() {
    super.initState();
    _load();
  }

  Future<String?> _resolvePubkey() async {
    final provider = widget.walletPubkeyProvider;
    if (provider != null) return provider();
    return (await WalletManager().getWallet())?.pubkeyHex;
  }

  Future<void> _load() async {
    final inst = await widget.repository.getBySfid(widget.sfidNumber);
    final pubkey = await _resolvePubkey();
    if (!mounted) return;
    if (inst == null) {
      setState(() => _loading = false);
      return;
    }
    final subscribed = pubkey == null
        ? false
        : await widget.repository.isSubscribed(pubkey, inst.sfidNumber);
    // 所属地预 join(省名·市名[·镇名]),不在 build 里 await。
    final areaPath = await widget.repository.institutionAreaPath(inst);
    if (!mounted) return;
    setState(() {
      _inst = inst;
      _activePubkey = pubkey;
      _subscribed = subscribed;
      _accounts = deriveAccountRows(inst);
      _areaPath = areaPath;
      _loading = false;
    });
    _loadDynamics(inst);
  }

  Future<void> _loadDynamics(PublicInstitutionEntity inst) async {
    final mainHex = _accounts.isNotEmpty ? _accounts.first.addressHex : '';
    // 主账户余额(对齐治理详情;批量接口只查主账户一条)。
    try {
      final balances = await _chainData.balances([mainHex]);
      if (mounted) {
        setState(() {
          _mainBalanceYuan = balances[mainHex];
          _mainBalanceLoading = false;
        });
      }
    } on Exception {
      if (mounted) setState(() => _mainBalanceLoading = false);
    }
    // 管理员。
    try {
      final admins = await _chainData.admins(
        mainAccountHex: mainHex,
        displayName: inst.sfidFullName,
      );
      if (mounted) setState(() => _admins = admins);
    } on Exception {
      // 联网失败保持空,不崩。
    }
    // 提案(按主账户 id 过滤年缓存)。
    try {
      final mainId = deriveInstitutionMainAccountId(inst.sfidNumber);
      final proposals = await _chainData.proposals(mainId);
      if (mounted) setState(() => _proposals = proposals);
    } on Exception {
      // 同上。
    }
  }

  Future<void> _toggleSubscribe() async {
    final inst = _inst;
    final pubkey = _activePubkey;
    if (inst == null || pubkey == null) return;
    if (_subscribed) {
      await widget.repository.unsubscribe(pubkey, inst.sfidNumber);
    } else {
      await widget.repository.subscribe(pubkey, inst.sfidNumber);
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
        title: Text(
          inst == null
              ? '公权机构'
              : (inst.sfidShortName?.isNotEmpty == true
                  ? inst.sfidShortName!
                  : inst.sfidFullName),
        ),
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

  Widget _buildBody(PublicInstitutionEntity? inst) {
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
        const SizedBox(height: 12),
        _adminsEntry(inst),
        const SizedBox(height: 12),
        _proposalList(),
      ],
    );
  }

  // ──── ① 机构信息(身份ID / 主账户 / 主账户余额 / 法定代表人 / 所属地)────
  // 与治理机构详情统一:无机构名标题(名称只在 AppBar);每行 32×32 图标 tile
  // (上标签下数值),Divider(height:18) 分隔。

  Widget _infoCard(PublicInstitutionEntity inst) {
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
              icon: Icons.badge_outlined,
              label: '身份ID',
              value: inst.sfidNumber,
            ),
            const Divider(height: 18),
            // 主账户:卡0 本地派生的主账户 SS58(完整,不截断)。
            _infoTile(
              icon: Icons.account_balance_wallet_outlined,
              label: '主账户',
              value: mainSs58,
            ),
            const Divider(height: 18),
            _infoTile(
              icon: Icons.payments_outlined,
              label: '主账户余额',
              value: _mainBalanceLabel(),
            ),
            const Divider(height: 18),
            // 法定代表人来自 SFID subjects.legal_rep_name;未录入则留空。
            _infoTile(
              icon: Icons.person_outline,
              label: '法定代表人',
              value: inst.legalRepName ?? '',
            ),
            const Divider(height: 18),
            _infoTile(
              icon: Icons.place_outlined,
              label: '所属地',
              value: _areaPath,
            ),
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

  // ──── ② 机构账户入口(治理同款 36px Card → 全部账户页)────

  Widget _accountsEntry(PublicInstitutionEntity inst) {
    return _entryCard(
      icon: Icons.account_balance_wallet_outlined,
      title: '机构账户',
      subtitle: '共 ${_accounts.length} 个账户',
      onTap: () => Navigator.of(context).push(
        MaterialPageRoute<void>(
          builder: (_) => PublicInstitutionAccountsPage(
            institution: inst,
            chainData: _chainData,
          ),
        ),
      ),
    );
  }

  // ──── ③ 提案发起入口(行高/图标与其他行齐,本期占位)────

  /// 「提案」徽章 + 副文 + 右箭头;图标 36×36(与机构账户/管理员行齐高)。
  /// 公权机构可发起 转账 / 费用划转 / 更换管理员 提案,发起流程需与 SFID 管理员
  /// 来源对接,本期**只占位**:点击给开发中反馈,不接真实发起页。
  Widget _proposalEntry() {
    return Container(
      decoration: BoxDecoration(
        color: AppTheme.surfaceCard,
        borderRadius: BorderRadius.circular(12),
        border: Border.all(color: AppTheme.primaryDark.withValues(alpha: 0.18)),
      ),
      child: InkWell(
        onTap: () {
          ScaffoldMessenger.of(context).showSnackBar(
            const SnackBar(
              content: Text('发起提案功能开发中（转账 / 费用划转 / 更换管理员）'),
            ),
          );
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
                child: const Icon(Icons.how_to_vote_outlined,
                    size: 18, color: AppTheme.primaryDark),
              ),
              const SizedBox(width: 12),
              Expanded(
                child: Column(
                  crossAxisAlignment: CrossAxisAlignment.start,
                  children: [
                    Container(
                      padding: const EdgeInsets.symmetric(
                          horizontal: 6, vertical: 1),
                      decoration: BoxDecoration(
                        color: AppTheme.primaryDark.withValues(alpha: 0.10),
                        borderRadius: BorderRadius.circular(10),
                      ),
                      child: const Text('提案',
                          style: TextStyle(
                              fontSize: 11,
                              color: AppTheme.primaryDark,
                              fontWeight: FontWeight.w600)),
                    ),
                    const SizedBox(height: 4),
                    const Text('发起提案',
                        style: TextStyle(
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

  // ──── ④ 管理员入口(治理同款 36px Card → 管理员列表)────

  Widget _adminsEntry(PublicInstitutionEntity inst) {
    final name = inst.sfidShortName?.isNotEmpty == true
        ? inst.sfidShortName!
        : inst.sfidFullName;
    return _entryCard(
      icon: Icons.people_outline,
      title: '管理员',
      subtitle: '共 ${_admins.length} 位管理员',
      onTap: () => Navigator.of(context).push(
        MaterialPageRoute<void>(
          builder: (_) => PublicInstitutionAdminListPage(
            sfidFullName: name,
            admins: _admins,
          ),
        ),
      ),
    );
  }

  // ──── ⑤ 提案列表(对齐治理机构 _buildProposalList)────

  Widget _proposalList() {
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
        if (_proposals.isEmpty)
          _emptyProposalState()
        else
          ...List.generate(_proposals.length, (i) {
            return Padding(
              padding:
                  EdgeInsets.only(bottom: i < _proposals.length - 1 ? 10 : 0),
              child: _proposalCard(_proposals[i]),
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

  Widget _proposalCard(PublicProposalSummary p) {
    final statusColor = AppTheme.proposalStatusColor(p.status);
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
            child: Text(p.idLabel,
                style: const TextStyle(
                    fontSize: 15,
                    fontWeight: FontWeight.w600,
                    color: AppTheme.primaryDark)),
          ),
          Container(
            padding: const EdgeInsets.symmetric(horizontal: 8, vertical: 2),
            decoration: BoxDecoration(
              color: statusColor.withValues(alpha: 0.1),
              borderRadius: BorderRadius.circular(10),
            ),
            child: Text(p.statusLabel,
                style: TextStyle(
                    fontSize: 11,
                    fontWeight: FontWeight.w600,
                    color: statusColor)),
          ),
        ],
      ),
    );
  }

  // ──── 公用零件(尺寸对齐治理机构)────

  /// 机构信息图标 tile(对齐治理 _buildAccountInfoTile):
  /// 32×32 图标 + 上标签(11)下数值(13)。
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
              // value 可能是完整身份ID,允许换行,不截断。
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

  /// 入口卡(对齐治理 _buildAdminEntry):36×36 图标 + 标题(15)+ 副标题(12) + 右箭头。
  /// 机构账户、管理员入口共用。
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
