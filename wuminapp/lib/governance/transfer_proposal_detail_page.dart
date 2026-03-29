import 'package:flutter/material.dart';
import 'package:flutter/services.dart';
import 'package:polkadart_keyring/polkadart_keyring.dart' show Keyring;

import '../ui/app_theme.dart';
import '../util/amount_format.dart';
import 'institution_data.dart';
import 'institution_admin_service.dart';
import 'pending_vote_store.dart';
import 'proposal_context.dart';
import 'transfer_proposal_service.dart';
import '../qr/pages/qr_sign_session_page.dart';
import '../rpc/chain_rpc.dart';
import '../rpc/onchain.dart';
import '../signer/qr_signer.dart';
import '../wallet/core/wallet_manager.dart';

/// 转账提案详情页：展示提案信息、投票进度、管理员投票明细及投票操作。
class TransferProposalDetailPage extends StatefulWidget {
  const TransferProposalDetailPage({
    super.key,
    required this.institution,
    required this.proposalId,
    required this.proposalContext,
  });

  final InstitutionInfo institution;
  final int proposalId;

  /// 统一的提案上下文。
  final ProposalContext proposalContext;

  /// 便捷访问。
  List<WalletProfile> get adminWallets => proposalContext.adminWallets;

  @override
  State<TransferProposalDetailPage> createState() =>
      _TransferProposalDetailPageState();
}

class _TransferProposalDetailPageState
    extends State<TransferProposalDetailPage> {
  static const int _statusVoting = 0;
  static const int _statusPassed = 1;
  static const int _statusRejected = 2;
  static const int _statusExecuted = 3;

  final TransferProposalService _proposalService = TransferProposalService();
  final InstitutionAdminService _adminService = InstitutionAdminService();
  bool _loading = true;
  String? _error;
  bool _submitting = false;

  // 提案状态
  int? _status;

  // 提案详情（从链上读取）
  TransferProposalInfo? _proposalInfo;
  bool _remarkExpanded = false;

  // 投票计数
  int _yesCount = 0;
  int _noCount = 0;

  // 管理员列表与投票记录
  List<String> _admins = const [];
  // pubkeyHex → true(赞成) / false(反对) / null(未投票)
  Map<String, bool?> _adminVotes = {};

  // 当前用户可投票的管理员钱包
  List<WalletProfile> _votableWallets = const [];
  WalletProfile? _selectedVoteWallet;

  // 已提交投票但尚未上链确认的管理员公钥集合
  Set<String> _pendingPubkeys = const {};

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
      // 并行加载管理员列表、提案状态、投票计数、提案详情
      final results = await Future.wait([
        _adminService.fetchAdmins(widget.institution.shenfenId),
        _proposalService.fetchProposalStatus(widget.proposalId),
        _proposalService.fetchVoteTally(widget.proposalId),
        _proposalService.fetchProposalAction(widget.proposalId),
      ]);

      final admins = results[0] as List<String>;
      final status = results[1] as int?;
      final tally = results[2] as ({int yes, int no});
      final proposalInfo = results[3] as TransferProposalInfo?;

      // 逐个查询每位管理员的投票记录
      final votes = <String, bool?>{};
      final voteFutures = admins.map((pubkey) async {
        final vote =
            await _proposalService.fetchAdminVote(widget.proposalId, pubkey);
        return MapEntry(pubkey, vote);
      });
      final voteResults = await Future.wait(voteFutures);
      for (final entry in voteResults) {
        votes[entry.key] = entry.value;
      }

      // 检查待确认投票：先批量确认，再获取仍在等待的记录
      final pendingRecords = await PendingVoteStore.instance.confirmAll(
        'transfer',
        widget.proposalId,
        OnchainRpc(),
      );
      final pendingPks = pendingRecords.map((r) => r.walletPubkey).toSet();

      // 筛选出可投票的管理员钱包（未投票且无待确认投票的）
      final votable = <WalletProfile>[];
      for (final w in widget.adminWallets) {
        var pk = w.pubkeyHex.toLowerCase();
        if (pk.startsWith('0x')) pk = pk.substring(2);
        if (admins.contains(pk) && votes[pk] == null && !pendingPks.contains(pk)) {
          votable.add(w);
        }
      }

      if (!mounted) return;
      setState(() {
        _admins = admins;
        _status = status;
        _yesCount = tally.yes;
        _noCount = tally.no;
        _adminVotes = votes;
        _pendingPubkeys = pendingPks;
        _votableWallets = votable;
        _selectedVoteWallet = votable.isNotEmpty ? votable.first : null;
        _proposalInfo = proposalInfo;
        _loading = false;
      });
    } catch (e) {
      if (!mounted) return;
      setState(() {
        _error = e.toString();
        _loading = false;
      });
    }
  }

  // ──── SS58 编码工具 ────

  String _pubkeyToSS58(String pubkeyHex) {
    final hex = pubkeyHex.startsWith('0x') ? pubkeyHex.substring(2) : pubkeyHex;
    final bytes = _hexDecode(hex);
    return Keyring().encodeAddress(bytes, 2027);
  }

  String _toHex(List<int> bytes) {
    const chars = '0123456789abcdef';
    final buf = StringBuffer();
    for (final b in bytes) {
      buf
        ..write(chars[(b >> 4) & 0x0f])
        ..write(chars[b & 0x0f]);
    }
    return buf.toString();
  }

  Uint8List _hexDecode(String hex) {
    final h = hex.startsWith('0x') ? hex.substring(2) : hex;
    final result = Uint8List(h.length ~/ 2);
    for (var i = 0; i < result.length; i++) {
      result[i] = int.parse(h.substring(i * 2, i * 2 + 2), radix: 16);
    }
    return result;
  }

  String _truncateAddress(String address) {
    if (address.length <= 14) return address;
    return '${address.substring(0, 6)}...${address.substring(address.length - 6)}';
  }

  // ──── 状态相关 ────

  String _statusLabel(int? status) {
    switch (status) {
      case _statusVoting:
        return '投票中';
      case _statusPassed:
        return '已通过';
      case _statusRejected:
        return '已拒绝';
      case _statusExecuted:
        return '已执行';
      default:
        return '未知';
    }
  }

  Color _statusColor(int? status) => AppTheme.proposalStatusColor(status ?? -1);

  // ──── 投票提交 ────

  /// 当前用户是否是此机构的管理员（可能导入了多个管理员钱包）。
  bool get _isCurrentUserAdmin => widget.proposalContext.isAdmin;

  /// 是否还有可投票的钱包（未投票的管理员钱包）。
  bool get _canVote {
    if (_selectedVoteWallet == null) return false;
    if (_status != _statusVoting) return false;
    return _votableWallets.isNotEmpty;
  }

  /// 所有管理员钱包都已投过票或正在投票中。
  bool get _allVoted {
    if (widget.adminWallets.isEmpty) return false;
    for (final w in widget.adminWallets) {
      var pk = w.pubkeyHex.toLowerCase();
      if (pk.startsWith('0x')) pk = pk.substring(2);
      if (_adminVotes[pk] == null && !_pendingPubkeys.contains(pk)) {
        return false;
      }
    }
    return true;
  }

  Future<void> _submitVote(bool approve) async {
    final wallet = _selectedVoteWallet;
    if (wallet == null) return;

    setState(() => _submitting = true);

    try {
      final pubkeyBytes = _hexDecode(wallet.pubkeyHex);

      Future<Uint8List> signCallback(Uint8List payload) async {
        // 管理员投票统一通过 QR 码签名（wumin 冷钱包）
        final qrSigner = QrSigner();
        final voteText = approve ? '赞成' : '反对';
        final rv = await ChainRpc().fetchRuntimeVersion();
        final request = qrSigner.buildRequest(
          requestId: QrSigner.generateRequestId(prefix: 'vote-'),
          account: wallet.address,
          pubkey: '0x${wallet.pubkeyHex}',
          payloadHex: '0x${_toHex(payload)}',
          specVersion: rv.specVersion,
          display: {
            'action': 'vote_transfer',
            'action_label': '转账投票',
            'summary': '转账提案 #${widget.proposalId} 投票：$voteText',
            'fields': [
              {'key': 'proposal_id', 'label': '提案编号', 'value': widget.proposalId.toString()},
              {'key': 'approve', 'label': '投票', 'value': approve.toString()},
              if (_proposalInfo != null) ...{
                {'key': 'beneficiary', 'label': '收款账户', 'value': _proposalInfo!.beneficiary},
                {'key': 'amount_yuan', 'label': '金额', 'value': AmountFormat.format(_proposalInfo!.amountYuan), 'format': 'currency'},
                if (_proposalInfo!.remark.isNotEmpty)
                  {'key': 'remark', 'label': '备注', 'value': _proposalInfo!.remark},
              },
            ],
          },
        );
        final requestJson = qrSigner.encodeRequest(request);
        final response = await Navigator.push<QrSignResponse>(
          context,
          MaterialPageRoute(
            builder: (_) => QrSignSessionPage(
                request: request,
                requestJson: requestJson,
                expectedPubkey: '0x${wallet.pubkeyHex}'),
          ),
        );
        if (response == null) throw Exception('签名已取消');
        return Uint8List.fromList(_hexDecode(response.signature));
      }

      final result = await _proposalService.submitVoteTransfer(
        proposalId: widget.proposalId,
        approve: approve,
        fromAddress: wallet.address,
        signerPubkey: Uint8List.fromList(pubkeyBytes),
        sign: signCallback,
      );

      // 持久化待确认投票记录
      var pubkey = wallet.pubkeyHex.toLowerCase();
      if (pubkey.startsWith('0x')) pubkey = pubkey.substring(2);
      await PendingVoteStore.instance.save(PendingVoteRecord(
        proposalType: 'transfer',
        proposalId: widget.proposalId,
        walletPubkey: pubkey,
        approve: approve,
        txHash: result.txHash,
        usedNonce: result.usedNonce,
        createdAt: DateTime.now(),
      ));

      if (!mounted) return;
      ScaffoldMessenger.of(context).showSnackBar(
        SnackBar(
          content: Text('投票已提交：${_truncateAddress(result.txHash)}'),
          backgroundColor: AppTheme.primaryDark,
        ),
      );

      // 刷新数据
      _adminService.clearCache(widget.institution.shenfenId);
      await _load();
    } catch (e) {
      if (!mounted) return;
      ScaffoldMessenger.of(context).showSnackBar(
        SnackBar(
          content: Text('投票失败：$e'),
          backgroundColor: AppTheme.danger,
        ),
      );
    } finally {
      if (mounted) setState(() => _submitting = false);
    }
  }

  void _confirmVote(bool approve) {
    final label = approve ? '赞成' : '反对';
    showDialog(
      context: context,
      builder: (ctx) => AlertDialog(
        title: Text('确认$label'),
        content: Text('确定要对此提案投"$label"票吗？投票后不可更改。'),
        actions: [
          TextButton(
            onPressed: () => Navigator.pop(ctx),
            child: const Text('取消'),
          ),
          TextButton(
            onPressed: () {
              Navigator.pop(ctx);
              _submitVote(approve);
            },
            child: Text(label),
          ),
        ],
      ),
    );
  }

  // ──── 构建 UI ────

  @override
  Widget build(BuildContext context) {
    return Scaffold(
      backgroundColor: AppTheme.scaffoldBg,
      appBar: AppBar(
        title: const Text(
          '提案详情',
          style: TextStyle(fontSize: 17, fontWeight: FontWeight.w700),
        ),
        centerTitle: true,
        foregroundColor: AppTheme.textPrimary,
      ),
      body: _loading
          ? const Center(child: CircularProgressIndicator())
          : _error != null
              ? _buildError()
              : _buildContent(),
      bottomNavigationBar: (!_loading &&
              _error == null &&
              _status == _statusVoting &&
              _isCurrentUserAdmin)
          ? _buildVoteButtons()
          : null,
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
              style: TextStyle(fontSize: 12, color: AppTheme.textTertiary),
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
        await _load();
      },
      child: ListView(
        padding: const EdgeInsets.fromLTRB(16, 8, 16, 32),
        children: [
          _buildStatusBadge(),
          const SizedBox(height: 16),
          _buildProposalInfoCard(),
          const SizedBox(height: 16),
          _buildVotingProgress(),
          const SizedBox(height: 16),
          _buildAdminVoteList(),
        ],
      ),
    );
  }

  // ──── 提案状态标签 ────

  Widget _buildStatusBadge() {
    final color = _statusColor(_status);
    final label = _statusLabel(_status);
    return Row(
      children: [
        Container(
          padding: const EdgeInsets.symmetric(horizontal: 12, vertical: 6),
          decoration: BoxDecoration(
            color: color.withValues(alpha: 0.1),
            borderRadius: BorderRadius.circular(20),
            border: Border.all(color: color.withValues(alpha: 0.3)),
          ),
          child: Row(
            mainAxisSize: MainAxisSize.min,
            children: [
              Icon(
                _status == _statusVoting
                    ? Icons.how_to_vote
                    : _status == _statusPassed
                        ? Icons.check_circle
                        : _status == _statusRejected
                            ? Icons.cancel
                            : Icons.error,
                size: 16,
                color: color,
              ),
              const SizedBox(width: 4),
              Text(
                label,
                style: TextStyle(
                  fontSize: 14,
                  fontWeight: FontWeight.w600,
                  color: color,
                ),
              ),
            ],
          ),
        ),
        const Spacer(),
        Text(
          '提案 ${formatProposalId(widget.proposalId)}',
          style: TextStyle(fontSize: 13, color: AppTheme.textTertiary),
        ),
      ],
    );
  }

  // ──── 提案信息卡片 ────

  Widget _buildProposalInfoCard() {
    final info = _proposalInfo;
    final remark = info?.remark ?? '';

    return Card(
      elevation: 0,
      margin: EdgeInsets.zero,
      shape: RoundedRectangleBorder(
        borderRadius: BorderRadius.circular(12),
        side: BorderSide(color: AppTheme.border),
      ),
      child: Padding(
        padding: const EdgeInsets.all(16),
        child: Column(
          crossAxisAlignment: CrossAxisAlignment.start,
          children: [
            const Text(
              '提案信息',
              style: TextStyle(
                fontSize: 16,
                fontWeight: FontWeight.w700,
                color: AppTheme.primaryDark,
              ),
            ),
            const SizedBox(height: 12),
            _buildInfoRow(
              '机构名称',
              widget.institution.name,
            ),
            if (info != null) ...[
              const Divider(height: 20),
              _buildInfoRow(
                '转账金额',
                '${AmountFormat.format(info.amountYuan, symbol: '')} 元',
              ),
              const Divider(height: 20),
              _buildInfoRow(
                '收款地址',
                _truncateAddress(info.beneficiary),
                onCopy: () {
                  Clipboard.setData(ClipboardData(text: info.beneficiary));
                  ScaffoldMessenger.of(context).showSnackBar(
                    const SnackBar(
                      content: Text('地址已复制'),
                      duration: Duration(seconds: 1),
                    ),
                  );
                },
              ),
            ],
            const Divider(height: 20),
            // 备注（可折叠）
            _buildRemarkRow(remark),
          ],
        ),
      ),
    );
  }

  Widget _buildRemarkRow(String remark) {
    if (remark.isEmpty) {
      return _buildInfoRow('备注', '无');
    }
    final isLong = remark.length > 30;
    return Column(
      crossAxisAlignment: CrossAxisAlignment.start,
      children: [
        Row(
          crossAxisAlignment: CrossAxisAlignment.start,
          children: [
            SizedBox(
              width: 80,
              child: Text(
                '备注',
                style: TextStyle(fontSize: 13, color: AppTheme.textSecondary),
              ),
            ),
            Expanded(
              child: Text(
                remark,
                style: const TextStyle(fontSize: 13, color: AppTheme.textPrimary),
                maxLines: _remarkExpanded ? null : 1,
                overflow: _remarkExpanded ? null : TextOverflow.ellipsis,
              ),
            ),
            if (isLong)
              GestureDetector(
                onTap: () => setState(() => _remarkExpanded = !_remarkExpanded),
                child: Icon(
                  _remarkExpanded
                      ? Icons.keyboard_arrow_up
                      : Icons.keyboard_arrow_down,
                  size: 20,
                  color: AppTheme.textTertiary,
                ),
              ),
          ],
        ),
      ],
    );
  }

  Widget _buildInfoRow(String label, String value, {VoidCallback? onCopy}) {
    return Row(
      crossAxisAlignment: CrossAxisAlignment.start,
      children: [
        SizedBox(
          width: 80,
          child: Text(
            label,
            style: TextStyle(fontSize: 13, color: AppTheme.textSecondary),
          ),
        ),
        Expanded(
          child: Text(
            value,
            style: const TextStyle(fontSize: 13, color: AppTheme.textPrimary),
          ),
        ),
        if (onCopy != null)
          GestureDetector(
            onTap: onCopy,
            child: Icon(Icons.copy, size: 16, color: AppTheme.textTertiary),
          ),
      ],
    );
  }

  // ──── 投票进度 ────

  Widget _buildVotingProgress() {
    final threshold = widget.institution.internalThreshold;
    final progress =
        threshold > 0 ? (_yesCount / threshold).clamp(0.0, 1.0) : 0.0;

    return Card(
      elevation: 0,
      margin: EdgeInsets.zero,
      shape: RoundedRectangleBorder(
        borderRadius: BorderRadius.circular(12),
        side: BorderSide(color: AppTheme.border),
      ),
      child: Padding(
        padding: const EdgeInsets.all(16),
        child: Column(
          crossAxisAlignment: CrossAxisAlignment.start,
          children: [
            const Text(
              '投票进度',
              style: TextStyle(
                fontSize: 16,
                fontWeight: FontWeight.w700,
                color: AppTheme.primaryDark,
              ),
            ),
            const SizedBox(height: 12),
            ClipRRect(
              borderRadius: BorderRadius.circular(6),
              child: LinearProgressIndicator(
                value: progress,
                minHeight: 10,
                backgroundColor: AppTheme.border,
                valueColor: const AlwaysStoppedAnimation<Color>(AppTheme.primaryDark),
              ),
            ),
            const SizedBox(height: 8),
            Row(
              mainAxisAlignment: MainAxisAlignment.spaceBetween,
              children: [
                Text(
                  '赞成 $_yesCount / 阈值 $threshold',
                  style: const TextStyle(
                    fontSize: 14,
                    fontWeight: FontWeight.w600,
                    color: AppTheme.primaryDark,
                  ),
                ),
                Text(
                  '反对 $_noCount',
                  style: TextStyle(
                    fontSize: 13,
                    color: AppTheme.danger,
                  ),
                ),
              ],
            ),
          ],
        ),
      ),
    );
  }

  // ──── 管理员投票明细 ────

  Widget _buildAdminVoteList() {
    return Card(
      elevation: 0,
      margin: EdgeInsets.zero,
      shape: RoundedRectangleBorder(
        borderRadius: BorderRadius.circular(12),
        side: BorderSide(color: AppTheme.border),
      ),
      child: Padding(
        padding: const EdgeInsets.symmetric(vertical: 8),
        child: Column(
          crossAxisAlignment: CrossAxisAlignment.start,
          children: [
            Padding(
              padding: const EdgeInsets.fromLTRB(16, 8, 16, 4),
              child: Text(
                '管理员投票明细（共 ${_admins.length} 人）',
                style: const TextStyle(
                  fontSize: 16,
                  fontWeight: FontWeight.w700,
                  color: AppTheme.primaryDark,
                ),
              ),
            ),
            const Divider(),
            ...List.generate(_admins.length, (index) {
              final pubkey = _admins[index];
              final vote = _adminVotes[pubkey];
              final ss58 = _pubkeyToSS58(pubkey);
              final isProposer = _proposalInfo?.proposer == ss58;

              return ListTile(
                dense: true,
                leading: CircleAvatar(
                  radius: 16,
                  backgroundColor: AppTheme.primaryDark.withValues(alpha: 0.08),
                  child: Text(
                    '${index + 1}',
                    style: const TextStyle(
                      fontSize: 12,
                      fontWeight: FontWeight.w600,
                      color: AppTheme.primaryDark,
                    ),
                  ),
                ),
                title: Row(
                  children: [
                    Flexible(
                      child: Text(
                        _truncateAddress(ss58),
                        style: const TextStyle(fontSize: 13),
                        overflow: TextOverflow.ellipsis,
                      ),
                    ),
                    if (isProposer) ...[
                      const SizedBox(width: 6),
                      Container(
                        padding: const EdgeInsets.symmetric(
                            horizontal: 6, vertical: 1),
                        decoration: BoxDecoration(
                          color: AppTheme.warning.withValues(alpha: 0.1),
                          borderRadius: BorderRadius.circular(8),
                        ),
                        child: const Text(
                          '发起人',
                          style: TextStyle(
                            fontSize: 10,
                            fontWeight: FontWeight.w600,
                            color: AppTheme.warning,
                          ),
                        ),
                      ),
                    ],
                  ],
                ),
                trailing: _buildVoteStatusChip(vote, pubkey),
              );
            }),
          ],
        ),
      ),
    );
  }

  Widget _buildVoteStatusChip(bool? vote, String pubkey) {
    // 链上已确认的投票结果优先
    if (vote == true) {
      return Container(
        padding: const EdgeInsets.symmetric(horizontal: 8, vertical: 2),
        decoration: BoxDecoration(
          color: AppTheme.success.withValues(alpha: 0.1),
          borderRadius: BorderRadius.circular(10),
        ),
        child: const Text(
          '赞成 \u2713',
          style: TextStyle(
            fontSize: 12,
            fontWeight: FontWeight.w600,
            color: AppTheme.success,
          ),
        ),
      );
    } else if (vote == false) {
      return Container(
        padding: const EdgeInsets.symmetric(horizontal: 8, vertical: 2),
        decoration: BoxDecoration(
          color: AppTheme.danger.withValues(alpha: 0.1),
          borderRadius: BorderRadius.circular(10),
        ),
        child: const Text(
          '反对 \u2717',
          style: TextStyle(
            fontSize: 12,
            fontWeight: FontWeight.w600,
            color: AppTheme.danger,
          ),
        ),
      );
    } else if (_pendingPubkeys.contains(pubkey)) {
      // 已提交但尚未上链确认
      return Container(
        padding: const EdgeInsets.symmetric(horizontal: 8, vertical: 2),
        decoration: BoxDecoration(
          color: AppTheme.warning.withValues(alpha: 0.1),
          borderRadius: BorderRadius.circular(10),
        ),
        child: Row(
          mainAxisSize: MainAxisSize.min,
          children: [
            SizedBox(
              width: 10,
              height: 10,
              child: CircularProgressIndicator(
                strokeWidth: 1.5,
                color: AppTheme.warning,
              ),
            ),
            const SizedBox(width: 4),
            Text(
              '投票中',
              style: TextStyle(
                fontSize: 12,
                fontWeight: FontWeight.w600,
                color: AppTheme.warning,
              ),
            ),
          ],
        ),
      );
    } else {
      return Container(
        padding: const EdgeInsets.symmetric(horizontal: 8, vertical: 2),
        decoration: BoxDecoration(
          color: AppTheme.textTertiary.withValues(alpha: 0.1),
          borderRadius: BorderRadius.circular(10),
        ),
        child: Text(
          '未投票 -',
          style: TextStyle(
            fontSize: 12,
            fontWeight: FontWeight.w500,
            color: AppTheme.textTertiary,
          ),
        ),
      );
    }
  }

  // ──── 底部投票按钮 ────

  String _truncateWalletAddress(String address) {
    if (address.length <= 16) return address;
    return '${address.substring(0, 8)}...${address.substring(address.length - 8)}';
  }

  Widget _buildVoteButtons() {
    return Container(
      padding: EdgeInsets.fromLTRB(
          16, 12, 16, MediaQuery.of(context).padding.bottom + 12),
      decoration: BoxDecoration(
        color: Colors.white,
        boxShadow: [
          BoxShadow(
            color: AppTheme.textPrimary.withValues(alpha: 0.06),
            blurRadius: 8,
            offset: const Offset(0, -2),
          ),
        ],
      ),
      child: Column(
        mainAxisSize: MainAxisSize.min,
        children: [
          // 多管理员时显示钱包选择器
          if (_votableWallets.length > 1)
            Padding(
              padding: const EdgeInsets.only(bottom: 10),
              child: Container(
                padding: const EdgeInsets.symmetric(horizontal: 12),
                decoration: BoxDecoration(
                  color: AppTheme.success.withValues(alpha: 0.05),
                  borderRadius: BorderRadius.circular(8),
                  border:
                      Border.all(color: AppTheme.success.withValues(alpha: 0.2)),
                ),
                child: DropdownButtonHideUnderline(
                  child: DropdownButton<int>(
                    value: _selectedVoteWallet?.walletIndex,
                    isExpanded: true,
                    icon: const Icon(Icons.arrow_drop_down, color: AppTheme.primaryDark),
                    items: _votableWallets.map((w) {
                      return DropdownMenuItem<int>(
                        value: w.walletIndex,
                        child: Row(
                          children: [
                            const Icon(Icons.verified_user,
                                size: 14, color: AppTheme.success),
                            const SizedBox(width: 6),
                            Expanded(
                              child: Text(
                                _truncateWalletAddress(w.address),
                                style: const TextStyle(
                                  fontSize: 13,
                                  fontFamily: 'monospace',
                                ),
                                overflow: TextOverflow.ellipsis,
                              ),
                            ),
                          ],
                        ),
                      );
                    }).toList(),
                    onChanged: (index) {
                      if (index == null) return;
                      setState(() {
                        _selectedVoteWallet = _votableWallets
                            .firstWhere((w) => w.walletIndex == index);
                      });
                    },
                  ),
                ),
              ),
            ),
          if (_allVoted)
            Padding(
              padding: const EdgeInsets.only(bottom: 10),
              child: Text(
                '你的管理员钱包均已投票',
                style: TextStyle(fontSize: 13, color: AppTheme.textTertiary),
                textAlign: TextAlign.center,
              ),
            ),
          Row(
            children: [
              Expanded(
                child: ElevatedButton(
                  onPressed: (_submitting || !_canVote)
                      ? null
                      : () => _confirmVote(false),
                  style: ElevatedButton.styleFrom(
                    backgroundColor: _canVote ? AppTheme.danger : AppTheme.border,
                    foregroundColor: Colors.white,
                    padding: const EdgeInsets.symmetric(vertical: 14),
                    shape: RoundedRectangleBorder(
                      borderRadius: BorderRadius.circular(10),
                    ),
                    elevation: 0,
                  ),
                  child: _submitting
                      ? const SizedBox(
                          width: 20,
                          height: 20,
                          child: CircularProgressIndicator(
                            strokeWidth: 2,
                            color: Colors.white,
                          ),
                        )
                      : const Text(
                          '反对',
                          style: TextStyle(
                              fontSize: 16, fontWeight: FontWeight.w600),
                        ),
                ),
              ),
              const SizedBox(width: 16),
              Expanded(
                child: ElevatedButton(
                  onPressed: (_submitting || !_canVote)
                      ? null
                      : () => _confirmVote(true),
                  style: ElevatedButton.styleFrom(
                    backgroundColor: AppTheme.success,
                    foregroundColor: Colors.white,
                    disabledBackgroundColor:
                        AppTheme.success.withValues(alpha: 0.25),
                    padding: const EdgeInsets.symmetric(vertical: 14),
                    shape: RoundedRectangleBorder(
                      borderRadius: BorderRadius.circular(10),
                    ),
                    elevation: 0,
                  ),
                  child: _submitting
                      ? const SizedBox(
                          width: 20,
                          height: 20,
                          child: CircularProgressIndicator(
                            strokeWidth: 2,
                            color: Colors.white,
                          ),
                        )
                      : const Text(
                          '赞成',
                          style: TextStyle(
                              fontSize: 16, fontWeight: FontWeight.w600),
                        ),
                ),
              ),
            ],
          ),
        ],
      ),
    );
  }
}
