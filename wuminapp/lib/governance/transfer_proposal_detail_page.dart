import 'package:flutter/material.dart';
import 'package:flutter/services.dart';
import '../ui/app_theme.dart';
import '../util/amount_format.dart';
import 'institution_data.dart';
import 'institution_admin_service.dart';
import 'pending_vote_store.dart';
import 'proposal_context.dart';
import 'internal_vote_service.dart';
import 'transfer_proposal_service.dart';
import '../qr/pages/qr_sign_session_page.dart';
import '../rpc/chain_rpc.dart';
import '../rpc/onchain.dart';
import '../rpc/smoldot_client.dart';
import '../qr/bodies/sign_request_body.dart';
import '../signer/qr_signer.dart';
import '../wallet/core/wallet_manager.dart';
import 'proposal_vote_widgets.dart';

/// 详情页展示/投票的三种提案类型。
///
/// 决定：读哪个 storage map 与 QR 签名如何展示。
/// Phase 3(2026-04-22)起,投票动作统一走 `VotingEngine::internal_vote`
/// (9.0),不再按 kind 区分 call_index;`kind` 仅影响"创建提案"路径与
/// 详情展示逻辑。
enum TransferProposalKind {
  /// 机构转账提案（propose_transfer, pallet=19 call=0）。
  transfer,

  /// 安全基金转账提案（propose_safety_fund_transfer, pallet=19 call=1）。
  safetyFund,

  /// 手续费划转提案（propose_sweep_to_main, pallet=19 call=2）。
  sweep,
}

/// 转账提案详情页：展示提案信息、投票进度、管理员投票明细及投票操作。
///
/// 通过 [kind] 区分三种转账类提案；不同 kind 读不同 storage、提交不同 extrinsic、
/// QR 签名显示不同文案。
class TransferProposalDetailPage extends StatefulWidget {
  const TransferProposalDetailPage({
    super.key,
    required this.institution,
    required this.proposalId,
    required this.proposalContext,
    this.kind = TransferProposalKind.transfer,
  });

  final InstitutionInfo institution;
  final int proposalId;
  final TransferProposalKind kind;

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

  final TransferProposalService _proposalService = TransferProposalService();
  final InstitutionAdminService _adminService = InstitutionAdminService();
  bool _loading = true;
  String? _error;
  bool _submitting = false;

  // 提案状态
  int? _status;

  // 提案详情（从链上读取）— 按 kind 使用对应字段，其余字段为 null。
  TransferProposalInfo? _transferInfo;
  SafetyFundProposalInfo? _safetyFundInfo;
  SweepProposalInfo? _sweepInfo;
  bool _remarkExpanded = false;

  // ──── kind 相关常量 ────

  /// 本详情页绑定的提案类型标签（供 PendingVoteStore 区分 key）。
  String get _proposalTypeKey {
    switch (widget.kind) {
      case TransferProposalKind.transfer:
        return 'transfer';
      case TransferProposalKind.safetyFund:
        return 'safety_fund';
      case TransferProposalKind.sweep:
        return 'sweep';
    }
  }

  /// 签名显示用的人类可读类型名。
  String get _kindLabel {
    switch (widget.kind) {
      case TransferProposalKind.transfer:
        return '转账提案';
      case TransferProposalKind.safetyFund:
        return '安全基金转账提案';
      case TransferProposalKind.sweep:
        return '手续费划转提案';
    }
  }

  /// QR 签名 action 字段。
  ///
  /// Phase 3 起所有管理员投票统一走 `internal_vote`,冷钱包按 action
  /// 识别并解码同一套 call 格式；业务类型仅通过 summary/fields 文案体现。
  String get _signAction => 'internal_vote';

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
      // 根据 kind 选择对应的详情查询方法。
      // 不同 kind 写在不同的 storage map：
      //   transfer   → VotingEngine.ProposalData（带 dq-xfer tag）
      //   safetyFund → DuoqianTransfer.SafetyFundProposalActions
      //   sweep      → DuoqianTransfer.SweepProposalActions
      final Future<dynamic> detailFuture;
      switch (widget.kind) {
        case TransferProposalKind.transfer:
          detailFuture =
              _proposalService.fetchProposalAction(widget.proposalId);
          break;
        case TransferProposalKind.safetyFund:
          detailFuture =
              _proposalService.fetchSafetyFundAction(widget.proposalId);
          break;
        case TransferProposalKind.sweep:
          detailFuture = _proposalService.fetchSweepAction(widget.proposalId);
          break;
      }

      // 并行加载管理员列表、提案状态、投票计数、提案详情
      final results = await Future.wait([
        _adminService.fetchAdmins(widget.institution.shenfenId),
        _proposalService.fetchProposalStatus(widget.proposalId),
        _proposalService.fetchVoteTally(widget.proposalId),
        detailFuture,
      ]);

      final admins = results[0] as List<String>;
      final status = results[1] as int?;
      final tally = results[2] as ({int yes, int no});
      final detail = results[3];

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

      // 检查待确认投票：先批量确认，再获取仍在等待的记录。
      // 用 kind 对应的 type key，避免跨类型提案 ID 误判（虽然 ID 全局递增，
      // 但分开归档便于后续清理/迁移）。
      final pendingRecords = await PendingVoteStore.instance.confirmAll(
        _proposalTypeKey,
        widget.proposalId,
        OnchainRpc(),
      );
      final pendingPks = pendingRecords.map((r) => r.walletPubkey).toSet();

      // 筛选出可投票的管理员钱包（未投票且无待确认投票的）
      final votable = <WalletProfile>[];
      for (final w in widget.adminWallets) {
        var pk = w.pubkeyHex.toLowerCase();
        if (pk.startsWith('0x')) pk = pk.substring(2);
        if (admins.contains(pk) &&
            votes[pk] == null &&
            !pendingPks.contains(pk)) {
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
        _transferInfo = null;
        _safetyFundInfo = null;
        _sweepInfo = null;
        switch (widget.kind) {
          case TransferProposalKind.transfer:
            _transferInfo = detail as TransferProposalInfo?;
            break;
          case TransferProposalKind.safetyFund:
            _safetyFundInfo = detail as SafetyFundProposalInfo?;
            break;
          case TransferProposalKind.sweep:
            _sweepInfo = detail as SweepProposalInfo?;
            break;
        }
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

  // ──── SS58 编码工具 ────

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
          address: wallet.address,
          pubkey: '0x${wallet.pubkeyHex}',
          payloadHex: '0x${_toHex(payload)}',
          specVersion: rv.specVersion,
          display: SignDisplay(
            action: _signAction,
            summary: '$_kindLabel #${widget.proposalId} 投票：$voteText',
            fields: [
              // Phase 3: 转账提案管理员投票统一走 internal_vote,
              // fields 按 Registry 恒为 (proposal_id, approve)。
              // 类型特有字段(收款账户/金额/备注/机构/划转金额/目标)属辅助展示,
              // 页面已独立显示,不塞 display.fields 避免对齐失败
              // (2026-04-22 两色识别整改)。
              SignDisplayField(
                  key: 'proposal_id',
                  label: '提案编号',
                  value: widget.proposalId.toString()),
              SignDisplayField(
                  key: 'approve', label: '投票', value: approve.toString()),
            ],
          ),
        );
        final requestJson = qrSigner.encodeRequest(request);
        if (!mounted) throw Exception('页面已关闭');
        final response = await Navigator.push<SignResponseEnvelope>(
          context,
          MaterialPageRoute(
            builder: (_) => QrSignSessionPage(
                request: request,
                requestJson: requestJson,
                expectedPubkey: '0x${wallet.pubkeyHex}'),
          ),
        );
        if (response == null) throw Exception('签名已取消');
        return Uint8List.fromList(_hexDecode(response.body.signature));
      }

      // Phase 3: 所有管理员投票统一走 VotingEngine::internal_vote(9.0)。
      // 业务 kind 仅用于 QR 展示的文案与 storage 读取,不再分派 call_index。
      final result = await InternalVoteService().submit(
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
        proposalType: _proposalTypeKey,
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
        title: Text(
          '$_kindLabel详情',
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
      bottomNavigationBar: (!_loading &&
              _error == null &&
              _status == _statusVoting &&
              _isCurrentUserAdmin)
          ? ProposalVoteActions(
              votableWallets: _votableWallets,
              selectedWallet: _selectedVoteWallet,
              submitting: _submitting,
              canVote: _canVote,
              allVoted: _allVoted,
              onWalletChanged: (w) => setState(() => _selectedVoteWallet = w),
              onVote: _confirmVote,
            )
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
        await _load();
      },
      child: ListView(
        padding: const EdgeInsets.fromLTRB(16, 8, 16, 32),
        children: [
          ProposalStatusBadge(status: _status, proposalId: widget.proposalId),
          const SizedBox(height: 16),
          _buildProposalInfoCard(),
          const SizedBox(height: 16),
          ProposalVoteProgress(
            yesCount: _yesCount,
            noCount: _noCount,
            threshold: widget.institution.internalThreshold,
          ),
          const SizedBox(height: 16),
          ProposalAdminVoteList(
            admins: _admins,
            adminVotes: _adminVotes,
            pendingPubkeys: _pendingPubkeys,
            proposerPubkey: _proposerPubkey,
          ),
        ],
      ),
    );
  }

  // ──── 提案信息卡片 ────

  /// 提案创建者公钥（仅 transfer / safetyFund 有）。
  String? get _proposerPubkey {
    switch (widget.kind) {
      case TransferProposalKind.transfer:
        return _transferInfo?.proposer;
      case TransferProposalKind.safetyFund:
        return _safetyFundInfo?.proposer;
      case TransferProposalKind.sweep:
        return null; // sweep 提案 storage 不记录 proposer
    }
  }

  Widget _buildProposalInfoCard() {
    return Card(
      elevation: 0,
      margin: EdgeInsets.zero,
      shape: RoundedRectangleBorder(
        borderRadius: BorderRadius.circular(12),
        side: const BorderSide(color: AppTheme.border),
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
            ..._buildInfoRowsByKind(),
          ],
        ),
      ),
    );
  }

  /// 按 kind 生成提案信息卡的内容行（含 Divider）。
  List<Widget> _buildInfoRowsByKind() {
    switch (widget.kind) {
      case TransferProposalKind.transfer:
        return _buildTransferRows();
      case TransferProposalKind.safetyFund:
        return _buildSafetyFundRows();
      case TransferProposalKind.sweep:
        return _buildSweepRows();
    }
  }

  /// 普通机构转账：机构名称 + 金额 + 收款地址 + 备注。
  List<Widget> _buildTransferRows() {
    final info = _transferInfo;
    final rows = <Widget>[
      _buildInfoRow('机构名称', widget.institution.name),
    ];
    if (info != null) {
      rows
        ..add(const Divider(height: 20))
        ..add(_buildInfoRow(
          '转账金额',
          '${AmountFormat.format(info.amountYuan, symbol: '')} 元',
        ))
        ..add(const Divider(height: 20))
        ..add(_buildInfoRow(
          '收款地址',
          _truncateAddress(info.beneficiary),
          onCopy: () => _copyToClipboard(info.beneficiary),
        ));
    }
    rows
      ..add(const Divider(height: 20))
      ..add(_buildRemarkRow(info?.remark ?? ''));
    return rows;
  }

  /// 安全基金转账：金额 + 收款地址 + 备注（无机构维度，安全基金是全链级账户）。
  List<Widget> _buildSafetyFundRows() {
    final info = _safetyFundInfo;
    final rows = <Widget>[
      _buildInfoRow('付款账户', '安全基金账户'),
    ];
    if (info != null) {
      rows
        ..add(const Divider(height: 20))
        ..add(_buildInfoRow(
          '转账金额',
          '${AmountFormat.format(info.amountYuan, symbol: '')} 元',
        ))
        ..add(const Divider(height: 20))
        ..add(_buildInfoRow(
          '收款地址',
          _truncateAddress(info.beneficiary),
          onCopy: () => _copyToClipboard(info.beneficiary),
        ));
    }
    rows
      ..add(const Divider(height: 20))
      ..add(_buildRemarkRow(info?.remark ?? ''));
    return rows;
  }

  /// 手续费划转：机构名称 + 划转金额 + 目标（机构主账户），无备注、无收款地址。
  List<Widget> _buildSweepRows() {
    final info = _sweepInfo;
    final rows = <Widget>[
      _buildInfoRow('机构名称', widget.institution.name),
    ];
    if (info != null) {
      rows
        ..add(const Divider(height: 20))
        ..add(_buildInfoRow(
          '划转金额',
          '${AmountFormat.format(info.amountYuan, symbol: '')} 元',
        ))
        ..add(const Divider(height: 20))
        ..add(_buildInfoRow('划转路径', '手续费账户 → 机构主账户'));
    }
    return rows;
  }

  void _copyToClipboard(String value) {
    Clipboard.setData(ClipboardData(text: value));
    ScaffoldMessenger.of(context).showSnackBar(
      const SnackBar(
        content: Text('地址已复制'),
        duration: Duration(seconds: 1),
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
            const SizedBox(
              width: 80,
              child: Text(
                '备注',
                style: TextStyle(fontSize: 13, color: AppTheme.textSecondary),
              ),
            ),
            Expanded(
              child: Text(
                remark,
                style:
                    const TextStyle(fontSize: 13, color: AppTheme.textPrimary),
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
            style: const TextStyle(fontSize: 13, color: AppTheme.textSecondary),
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
            child:
                const Icon(Icons.copy, size: 16, color: AppTheme.textTertiary),
          ),
      ],
    );
  }
}
