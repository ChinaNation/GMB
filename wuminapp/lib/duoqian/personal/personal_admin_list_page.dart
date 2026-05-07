// 个人多签管理员列表页(req 1+2)。
//
// 从详情页"管理员列表"卡片折叠入口跳转进来,展示该多签的所有管理员,并按
// admin 投票状态渲染三态激活按钮:
//
//   - 创建者:无按钮(创建即同意)
//   - 非本钱包成员:无按钮(仅展示)
//   - 本钱包未投 + 多签 Pending:蓝色"激活"(可点)→ 跳 DuoqianManageDetailPage
//   - 本钱包已投赞成:灰色"已激活"(禁用)
//   - 本钱包已投反对:灰色"已拒绝"(禁用)
//   - 多签 Active:无按钮(创建已完成)
//
// "激活"行为本质是 votingengine `internal_vote(proposal_id, approve=true)`,
// 沿用现有 [DuoqianManageDetailPage] 的 QrSigner 签名 + InternalVoteService 投票流程,
// 不引入新的签名逻辑。

import 'dart:typed_data';

import 'package:flutter/material.dart';
import 'package:polkadart_keyring/polkadart_keyring.dart' show Keyring;

import 'package:wuminapp_mobile/institution/institution_data.dart';
import 'package:wuminapp_mobile/proposal/transfer/transfer_proposal_service.dart';
import 'package:wuminapp_mobile/proposal/shared/proposal_context.dart';
import 'package:wuminapp_mobile/duoqian/shared/duoqian_manage_detail_page.dart';
import 'package:wuminapp_mobile/duoqian/shared/duoqian_manage_models.dart';
import 'package:wuminapp_mobile/ui/app_theme.dart';
import 'package:wuminapp_mobile/wallet/core/wallet_manager.dart';

import 'personal_pending_create_lookup.dart';

/// 管理员行的激活按钮渲染状态。
enum _ActivateButtonState {
  /// 不显示按钮(创建者 / 非本钱包成员 / 多签已激活)。
  hidden,

  /// 蓝色"激活"可点(本钱包成员且未投票且多签待激活)。
  ready,

  /// 灰色"已激活"禁用(本钱包成员已投赞成)。
  alreadyApproved,

  /// 灰色"已拒绝"禁用(本钱包成员已投反对)。
  alreadyRejected,
}

class PersonalAdminListPage extends StatefulWidget {
  const PersonalAdminListPage({
    super.key,
    required this.institution,
    required this.duoqianStatus,
    required this.adminPubkeys,
    required this.adminWallets,
    this.creatorPubkeyHex,
  });

  /// 多签元信息(名称 / 多签地址 / sfidNumber 等)。
  final InstitutionInfo institution;

  /// 多签当前状态(Pending / Active)。
  final DuoqianStatus duoqianStatus;

  /// 管理员公钥列表(小写 hex,无 0x 前缀)。
  final List<String> adminPubkeys;

  /// 用户本地能签名的 admin 钱包子集(由调用方过滤好)。
  final List<WalletProfile> adminWallets;

  /// 创建人公钥(小写 hex,无 0x 前缀)。req 3 未实现时只有创建者本机已知。
  final String? creatorPubkeyHex;

  @override
  State<PersonalAdminListPage> createState() => _PersonalAdminListPageState();
}

class _PersonalAdminListPageState extends State<PersonalAdminListPage> {
  static final _keyring = Keyring();

  final TransferProposalService _proposalService = TransferProposalService();
  final PersonalPendingCreateLookup _lookup = PersonalPendingCreateLookup();

  bool _loading = true;
  String? _error;
  int? _proposalId;
  Map<String, bool?> _votes = const {};

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
      // 多签已激活时无需查投票:激活按钮整体不显示。
      if (widget.duoqianStatus != DuoqianStatus.pending) {
        if (!mounted) return;
        setState(() {
          _loading = false;
          _proposalId = null;
          _votes = const {};
        });
        return;
      }

      final pid = await _lookup.findActiveCreate(
        widget.institution.duoqianAddress,
      );

      // 仅查本钱包持有的 admin 投票状态(其他人投票状态对 UI 无意义,节省 RPC)。
      final votes = <String, bool?>{};
      if (pid != null) {
        for (final wallet in widget.adminWallets) {
          var pk = wallet.pubkeyHex.toLowerCase();
          if (pk.startsWith('0x')) pk = pk.substring(2);
          votes[pk] = await _proposalService.fetchAdminVote(pid, pk);
        }
      }

      if (!mounted) return;
      setState(() {
        _loading = false;
        _proposalId = pid;
        _votes = votes;
      });
    } catch (e) {
      if (!mounted) return;
      setState(() {
        _loading = false;
        _error = '$e';
      });
    }
  }

  _ActivateButtonState _resolveButtonState(String adminPubkeyHex) {
    // 多签已激活 → 全部隐藏
    if (widget.duoqianStatus != DuoqianStatus.pending) {
      return _ActivateButtonState.hidden;
    }
    // 创建者 → 隐藏(创建即同意)
    if (widget.creatorPubkeyHex != null &&
        widget.creatorPubkeyHex!.toLowerCase() == adminPubkeyHex) {
      return _ActivateButtonState.hidden;
    }
    // 非本钱包持有的 admin → 隐藏(本机不能代签)
    final isLocalWallet = widget.adminWallets.any((w) {
      var pk = w.pubkeyHex.toLowerCase();
      if (pk.startsWith('0x')) pk = pk.substring(2);
      return pk == adminPubkeyHex;
    });
    if (!isLocalWallet) return _ActivateButtonState.hidden;
    // 找不到活跃创建提案(异常态)→ 不显示按钮
    if (_proposalId == null) return _ActivateButtonState.hidden;
    final vote = _votes[adminPubkeyHex];
    if (vote == null) return _ActivateButtonState.ready;
    return vote
        ? _ActivateButtonState.alreadyApproved
        : _ActivateButtonState.alreadyRejected;
  }

  Future<void> _onActivatePressed() async {
    final pid = _proposalId;
    if (pid == null) return;
    final pushed = await Navigator.push(
      context,
      MaterialPageRoute(
        builder: (_) => DuoqianManageDetailPage(
          institution: widget.institution,
          proposalId: pid,
          proposalContext: ProposalContext(
            institution: widget.institution,
            adminWallets: widget.adminWallets,
            role: ProposalRole.admin,
          ),
        ),
      ),
    );
    if (pushed == true && mounted) {
      // 投票完成 → 重读投票状态(可能本行变成"已激活")
      await _load();
    }
  }

  // ──── UI ────

  @override
  Widget build(BuildContext context) {
    return Scaffold(
      backgroundColor: Colors.white,
      appBar: AppBar(
        title: const Text(
          '管理员列表',
          style: TextStyle(fontSize: 17, fontWeight: FontWeight.w700),
        ),
        centerTitle: true,
        backgroundColor: Colors.white,
        foregroundColor: AppTheme.primaryDark,
        elevation: 0,
        scrolledUnderElevation: 0.5,
      ),
      body: _loading
          ? const Center(child: CircularProgressIndicator())
          : _error != null
              ? _buildError()
              : _buildContent(),
    );
  }

  Widget _buildError() {
    return Padding(
      padding: const EdgeInsets.all(24),
      child: Column(
        mainAxisAlignment: MainAxisAlignment.center,
        children: [
          const Icon(Icons.error_outline,
              size: 36, color: AppTheme.textTertiary),
          const SizedBox(height: 12),
          Text(_error!,
              style: const TextStyle(color: AppTheme.textSecondary),
              textAlign: TextAlign.center),
          const SizedBox(height: 16),
          OutlinedButton(onPressed: _load, child: const Text('重试')),
        ],
      ),
    );
  }

  Widget _buildContent() {
    return RefreshIndicator(
      onRefresh: _load,
      child: ListView(
        padding: const EdgeInsets.fromLTRB(16, 8, 16, 32),
        children: [
          _buildHeaderCard(),
          const SizedBox(height: 16),
          _buildAdminListCard(),
        ],
      ),
    );
  }

  Widget _buildHeaderCard() {
    return Card(
      elevation: 0,
      margin: EdgeInsets.zero,
      shape: RoundedRectangleBorder(
        borderRadius: BorderRadius.circular(12),
        side: const BorderSide(color: AppTheme.border),
      ),
      child: Padding(
        padding: const EdgeInsets.all(16),
        child: Row(
          children: [
            const Icon(Icons.person, color: AppTheme.accent),
            const SizedBox(width: 10),
            Expanded(
              child: Column(
                crossAxisAlignment: CrossAxisAlignment.start,
                children: [
                  Text(
                    widget.institution.name,
                    style: const TextStyle(
                      fontSize: 15,
                      fontWeight: FontWeight.w700,
                      color: AppTheme.primaryDark,
                    ),
                  ),
                  const SizedBox(height: 2),
                  Text(
                    widget.duoqianStatus == DuoqianStatus.active
                        ? '已激活 · ${widget.adminPubkeys.length} 位管理员'
                        : '待激活 · ${widget.adminPubkeys.length} 位管理员需逐一签名',
                    style: const TextStyle(
                      fontSize: 12,
                      color: AppTheme.textTertiary,
                    ),
                  ),
                ],
              ),
            ),
          ],
        ),
      ),
    );
  }

  Widget _buildAdminListCard() {
    return Card(
      elevation: 0,
      margin: EdgeInsets.zero,
      shape: RoundedRectangleBorder(
        borderRadius: BorderRadius.circular(12),
        side: const BorderSide(color: AppTheme.border),
      ),
      child: Column(
        children: List.generate(widget.adminPubkeys.length, (index) {
          final pubkey = widget.adminPubkeys[index].toLowerCase();
          final ss58 = _pubkeyToSS58(pubkey);
          final isCreator = widget.creatorPubkeyHex != null &&
              widget.creatorPubkeyHex!.toLowerCase() == pubkey;
          final state = _resolveButtonState(pubkey);
          return Column(
            children: [
              if (index > 0) const Divider(height: 1),
              ListTile(
                leading: CircleAvatar(
                  radius: 16,
                  backgroundColor:
                      AppTheme.primaryDark.withValues(alpha: 0.08),
                  child: Text(
                    '${index + 1}',
                    style: const TextStyle(
                      fontSize: 12,
                      fontWeight: FontWeight.w600,
                      color: AppTheme.primaryDark,
                    ),
                  ),
                ),
                title: Text(
                  ss58,
                  style: const TextStyle(
                    fontSize: 11,
                    fontFamily: 'monospace',
                  ),
                ),
                subtitle: isCreator
                    ? const Text('创建者',
                        style:
                            TextStyle(fontSize: 11, color: AppTheme.accent))
                    : null,
                trailing: _buildActivateButton(state),
              ),
            ],
          );
        }),
      ),
    );
  }

  Widget? _buildActivateButton(_ActivateButtonState state) {
    switch (state) {
      case _ActivateButtonState.hidden:
        return null;
      case _ActivateButtonState.ready:
        return TextButton(
          onPressed: _onActivatePressed,
          style: TextButton.styleFrom(
            foregroundColor: AppTheme.primaryDark,
            backgroundColor: AppTheme.primaryDark.withValues(alpha: 0.08),
            padding: const EdgeInsets.symmetric(horizontal: 14, vertical: 6),
            minimumSize: const Size(0, 32),
            tapTargetSize: MaterialTapTargetSize.shrinkWrap,
          ),
          child: const Text('激活', style: TextStyle(fontSize: 13)),
        );
      case _ActivateButtonState.alreadyApproved:
        return const _DisabledTag(label: '已激活');
      case _ActivateButtonState.alreadyRejected:
        return const _DisabledTag(label: '已拒绝');
    }
  }

  /// 把 32 字节 pubkey hex 编码为 GMB SS58 地址(prefix=2027),并做两端截断
  /// 以适配 monospace 11 字号的 ListTile title 行宽。
  ///
  /// 编码失败(理论上不会发生,数据来自链上 storage)兜底返回原始 hex,避免崩溃。
  String _pubkeyToSS58(String pubkeyHex) {
    try {
      final hex = pubkeyHex.startsWith('0x')
          ? pubkeyHex.substring(2)
          : pubkeyHex;
      final bytes = Uint8List(hex.length ~/ 2);
      for (var i = 0; i < bytes.length; i++) {
        bytes[i] = int.parse(hex.substring(i * 2, i * 2 + 2), radix: 16);
      }
      final ss58 = _keyring.encodeAddress(bytes, 2027);
      if (ss58.length <= 24) return ss58;
      return '${ss58.substring(0, 12)}…${ss58.substring(ss58.length - 8)}';
    } catch (_) {
      return pubkeyHex;
    }
  }
}

class _DisabledTag extends StatelessWidget {
  const _DisabledTag({required this.label});

  final String label;

  @override
  Widget build(BuildContext context) {
    return Container(
      padding: const EdgeInsets.symmetric(horizontal: 10, vertical: 4),
      decoration: BoxDecoration(
        color: AppTheme.textTertiary.withValues(alpha: 0.08),
        borderRadius: BorderRadius.circular(6),
      ),
      child: Text(
        label,
        style: const TextStyle(
          fontSize: 12,
          color: AppTheme.textTertiary,
          fontWeight: FontWeight.w500,
        ),
      ),
    );
  }
}
