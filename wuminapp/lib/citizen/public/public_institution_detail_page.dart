import 'package:flutter/material.dart';

import 'package:wuminapp_mobile/citizen/public/data/public_institution_accounts.dart';
import 'package:wuminapp_mobile/citizen/public/data/public_institution_chain_data.dart';
import 'package:wuminapp_mobile/citizen/public/data/public_institution_repository.dart';
import 'package:wuminapp_mobile/citizen/public/data/public_provinces.dart';
import 'package:wuminapp_mobile/citizen/public/public_institution_accounts_page.dart';
import 'package:wuminapp_mobile/governance/shared/account_derivation.dart';
import 'package:wuminapp_mobile/isar/wallet_isar.dart';
import 'package:wuminapp_mobile/ui/app_theme.dart';
import 'package:wuminapp_mobile/wallet/core/wallet_manager.dart';

/// 公权机构详情页(ADR-018 §九 卡C)。
///
/// 中文注释:v1=浏览 + 订阅 + 动态展示。账户全本地派生(卡0),余额批量走
/// ChainReadCache(卡⑤),提案走年缓存过滤(卡①),管理员走 AdminsChange。
/// 右上角订阅按钮写本地 store(卡A)。发起提案/换管理员本期门控隐藏(非管理员)。
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

  List<PublicAccountRow> _accounts = const [];
  bool _balanceLoading = true;

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
    // 余额批量(主+费+自定义)。
    try {
      final balances = await _chainData
          .balances(_accounts.map((a) => a.addressHex).toList());
      if (mounted) {
        setState(() {
          _accounts = _accounts
              .map((a) => a.withBalance(balances[a.addressHex]))
              .toList(growable: false);
          _balanceLoading = false;
        });
      }
    } on Exception {
      if (mounted) setState(() => _balanceLoading = false);
    }
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
        const SizedBox(height: 16),
        _accountsCard(inst),
        const SizedBox(height: 12),
        _proposalsCard(),
        const SizedBox(height: 12),
        _adminsCard(),
      ],
    );
  }

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
          _row('所属', '${provinceDisplayName(inst.province)} · ${inst.city}'),
          _row('账户数', '${inst.accountCount}'),
        ],
      ),
    );
  }

  Widget _accountsCard(PublicInstitutionEntity inst) {
    final main = _accounts.isNotEmpty ? _accounts[0] : null;
    final fee = _accounts.length > 1 ? _accounts[1] : null;
    return _card(
      child: Column(
        crossAxisAlignment: CrossAxisAlignment.start,
        children: [
          _cardHeader(Icons.account_balance_wallet_outlined, '账户与余额'),
          const SizedBox(height: 10),
          if (main != null) _accountLine('主账户', main),
          if (fee != null) ...[
            const SizedBox(height: 8),
            _accountLine('费用账户', fee),
          ],
          const SizedBox(height: 12),
          InkWell(
            onTap: () => Navigator.of(context).push(
              MaterialPageRoute<void>(
                builder: (_) => PublicInstitutionAccountsPage(
                  institution: inst,
                  chainData: _chainData,
                ),
              ),
            ),
            child: Row(
              mainAxisAlignment: MainAxisAlignment.spaceBetween,
              children: [
                Text('更多账户(${_accounts.length})',
                    style: const TextStyle(
                        fontSize: 13.5,
                        fontWeight: FontWeight.w600,
                        color: AppTheme.primary)),
                const Icon(Icons.chevron_right,
                    size: 18, color: AppTheme.textTertiary),
              ],
            ),
          ),
        ],
      ),
    );
  }

  Widget _accountLine(String label, PublicAccountRow row) {
    return Row(
      mainAxisAlignment: MainAxisAlignment.spaceBetween,
      children: [
        Text(label,
            style:
                const TextStyle(fontSize: 13, color: AppTheme.textSecondary)),
        Text(
          _balanceLoading
              ? '—'
              : (row.balanceYuan == null
                  ? '未激活'
                  : '${row.balanceYuan!.toStringAsFixed(2)} 元'),
          style: const TextStyle(
              fontSize: 13.5,
              fontWeight: FontWeight.w600,
              color: AppTheme.primary),
        ),
      ],
    );
  }

  Widget _proposalsCard() {
    return _card(
      child: Column(
        crossAxisAlignment: CrossAxisAlignment.start,
        children: [
          _cardHeader(Icons.how_to_vote_outlined, '提案'),
          const SizedBox(height: 8),
          if (_proposals.isEmpty)
            const Text('暂无提案',
                style: TextStyle(fontSize: 12.5, color: AppTheme.textTertiary))
          else
            ..._proposals.map((p) => Padding(
                  padding: const EdgeInsets.symmetric(vertical: 4),
                  child: Row(
                    mainAxisAlignment: MainAxisAlignment.spaceBetween,
                    children: [
                      Text(p.idLabel,
                          style: const TextStyle(
                              fontSize: 13, color: AppTheme.textPrimary)),
                      Text(p.statusLabel,
                          style: const TextStyle(
                              fontSize: 12, color: AppTheme.textSecondary)),
                    ],
                  ),
                )),
        ],
      ),
    );
  }

  Widget _adminsCard() {
    return _card(
      child: Column(
        crossAxisAlignment: CrossAxisAlignment.start,
        children: [
          _cardHeader(Icons.group_outlined, '管理员(${_admins.length})'),
          const SizedBox(height: 8),
          if (_admins.isEmpty)
            const Text('暂无管理员数据',
                style: TextStyle(fontSize: 12.5, color: AppTheme.textTertiary))
          else
            ..._admins.map((a) => Padding(
                  padding: const EdgeInsets.symmetric(vertical: 3),
                  child: Text(
                    a,
                    maxLines: 1,
                    overflow: TextOverflow.ellipsis,
                    style: const TextStyle(
                        fontSize: 12, color: AppTheme.textSecondary),
                  ),
                )),
        ],
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

  Widget _cardHeader(IconData icon, String title) {
    return Row(
      children: [
        Icon(icon, size: 20, color: AppTheme.primary),
        const SizedBox(width: 8),
        Text(title,
            style: const TextStyle(
                fontSize: 14,
                fontWeight: FontWeight.w700,
                color: AppTheme.textPrimary)),
      ],
    );
  }

  Widget _row(String label, String value) {
    return Padding(
      padding: const EdgeInsets.symmetric(vertical: 4),
      child: Row(
        crossAxisAlignment: CrossAxisAlignment.start,
        children: [
          SizedBox(
            width: 64,
            child: Text(label,
                style: const TextStyle(
                    fontSize: 13, color: AppTheme.textTertiary)),
          ),
          Expanded(
            child: Text(value,
                style: const TextStyle(
                    fontSize: 13.5, color: AppTheme.textPrimary)),
          ),
        ],
      ),
    );
  }
}
