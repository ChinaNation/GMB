import 'package:flutter/material.dart';

import 'package:wuminapp_mobile/citizen/public/data/public_institution_accounts.dart';
import 'package:wuminapp_mobile/citizen/public/data/public_institution_chain_data.dart';
import 'package:wuminapp_mobile/citizen/public/data/public_institution_repository.dart';
import 'package:wuminapp_mobile/citizen/public/data/public_provinces.dart';
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
    setState(() {
      _inst = inst;
      _activePubkey = pubkey;
      _subscribed = subscribed;
      _accounts = deriveAccountRows(inst);
      _loading = false;
    });
    _loadDynamics(inst);
  }

  Future<void> _loadDynamics(PublicInstitutionEntity inst) async {
    // 管理员。
    try {
      final mainHex = _accounts.isNotEmpty ? _accounts.first.addressHex : '';
      final admins = await _chainData.admins(
        mainAccountHex: mainHex,
        displayName: inst.institutionName,
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
              : (inst.shortName?.isNotEmpty == true
                  ? inst.shortName!
                  : inst.institutionName),
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

  // ──── ① 机构信息(身份ID / 法定代表人 / 所属地,行间分隔线)────

  Widget _infoCard(PublicInstitutionEntity inst) {
    return _card(
      child: Column(
        crossAxisAlignment: CrossAxisAlignment.start,
        children: [
          Text(inst.institutionName,
              style: const TextStyle(
                  fontSize: 17,
                  fontWeight: FontWeight.w700,
                  color: AppTheme.textPrimary)),
          const SizedBox(height: 10),
          _row('身份 ID', inst.sfidNumber),
          const Divider(height: 18),
          // 法定代表人来自 SFID subjects.legal_rep_name;未录入则留空。
          _row('法定代表人', inst.legalRepName ?? ''),
          const Divider(height: 18),
          _row('所属地', '${provinceDisplayName(inst.province)} · ${inst.city}'),
        ],
      ),
    );
  }

  // ──── ② 机构账户入口(单行 + 箭头 → 全部账户页)────

  Widget _accountsEntry(PublicInstitutionEntity inst) {
    return _entryRow(
      icon: Icons.account_balance_wallet_outlined,
      title: '机构账户(${_accounts.length})',
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

  // ──── ③ 提案发起入口(本期占位)────

  /// 公权机构可发起 转账 / 费用划转 / 更换管理员 提案,但发起流程需与 SFID 管理员
  /// 来源对接,本期**只占位**:展示入口、点击给开发中反馈,不接真实发起页。
  Widget _proposalEntry() {
    return _entryRow(
      icon: Icons.how_to_vote_outlined,
      title: '提案',
      subtitle: '发起提案',
      onTap: () {
        ScaffoldMessenger.of(context).showSnackBar(
          const SnackBar(
            content: Text('发起提案功能开发中（转账 / 费用划转 / 更换管理员）'),
          ),
        );
      },
    );
  }

  // ──── ④ 管理员入口(单行 + 箭头 → 管理员列表)────

  Widget _adminsEntry(PublicInstitutionEntity inst) {
    final name = inst.shortName?.isNotEmpty == true
        ? inst.shortName!
        : inst.institutionName;
    return _entryRow(
      icon: Icons.group_outlined,
      title: '管理员(${_admins.length})',
      onTap: () => Navigator.of(context).push(
        MaterialPageRoute<void>(
          builder: (_) => PublicInstitutionAdminListPage(
            institutionName: name,
            admins: _admins,
          ),
        ),
      ),
    );
  }

  // ──── ⑤ 提案列表(对齐治理机构卡片样式)────

  Widget _proposalList() {
    return Column(
      crossAxisAlignment: CrossAxisAlignment.start,
      children: [
        const Padding(
          padding: EdgeInsets.only(left: 2, bottom: 10),
          child: Text('提案列表',
              style: TextStyle(
                  fontSize: 15,
                  fontWeight: FontWeight.w700,
                  color: AppTheme.textPrimary)),
        ),
        if (_proposals.isEmpty)
          _card(
            child: const Center(
              child: Padding(
                padding: EdgeInsets.symmetric(vertical: 10),
                child: Text('暂无提案',
                    style: TextStyle(
                        fontSize: 12.5, color: AppTheme.textTertiary)),
              ),
            ),
          )
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
                    fontSize: 14,
                    fontWeight: FontWeight.w600,
                    color: AppTheme.textPrimary)),
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

  // ──── 公用零件 ────

  /// 单行入口卡(图标 + 标题 + 可选副标题 + 右箭头),行高紧凑。
  /// 机构账户、提案发起、管理员三个入口共用。
  Widget _entryRow({
    required IconData icon,
    required String title,
    String? subtitle,
    required VoidCallback onTap,
  }) {
    return Container(
      decoration: BoxDecoration(
        color: AppTheme.surfaceCard,
        borderRadius: BorderRadius.circular(14),
        border: Border.all(color: AppTheme.border),
      ),
      child: InkWell(
        onTap: onTap,
        borderRadius: BorderRadius.circular(14),
        child: Padding(
          padding: const EdgeInsets.symmetric(horizontal: 14, vertical: 11),
          child: Row(
            children: [
              Container(
                width: 32,
                height: 32,
                decoration: BoxDecoration(
                  color: AppTheme.primary.withValues(alpha: 0.10),
                  borderRadius: BorderRadius.circular(9),
                ),
                child: Icon(icon, size: 18, color: AppTheme.primary),
              ),
              const SizedBox(width: 12),
              Expanded(
                child: Column(
                  crossAxisAlignment: CrossAxisAlignment.start,
                  mainAxisSize: MainAxisSize.min,
                  children: [
                    Text(title,
                        style: const TextStyle(
                            fontSize: 14,
                            fontWeight: FontWeight.w600,
                            color: AppTheme.textPrimary)),
                    if (subtitle != null) ...[
                      const SizedBox(height: 2),
                      Text(subtitle,
                          style: const TextStyle(
                              fontSize: 12, color: AppTheme.textTertiary)),
                    ],
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

  Widget _card({required Widget child}) {
    return Container(
      padding: const EdgeInsets.all(16),
      decoration: BoxDecoration(
        color: AppTheme.surfaceCard,
        borderRadius: BorderRadius.circular(14),
        border: Border.all(color: AppTheme.border),
      ),
      child: child,
    );
  }

  Widget _row(String label, String value) {
    return Row(
      crossAxisAlignment: CrossAxisAlignment.start,
      children: [
        SizedBox(
          width: 80,
          child: Text(label,
              style:
                  const TextStyle(fontSize: 13, color: AppTheme.textTertiary)),
        ),
        Expanded(
          child: Text(value,
              style:
                  const TextStyle(fontSize: 13.5, color: AppTheme.textPrimary)),
        ),
      ],
    );
  }
}
