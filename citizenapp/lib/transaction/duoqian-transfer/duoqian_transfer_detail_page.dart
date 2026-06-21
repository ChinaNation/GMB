import 'dart:async';

import 'package:flutter/material.dart';
import 'package:flutter/services.dart';
import 'package:citizenapp/ui/app_theme.dart';
import 'package:citizenapp/my/util/amount_format.dart';
import 'package:citizenapp/governance/shared/institution_info.dart';
import 'package:citizenapp/governance/admins-change/models/admin_account.dart';
import 'package:citizenapp/governance/admins-change/services/institution_admin_service.dart';
import 'package:citizenapp/votingengine/internal-vote/pending_vote_store.dart';
import 'package:citizenapp/governance/shared/proposal/proposal_context.dart';
import 'package:citizenapp/rpc/chain_event_subscription.dart';
import 'package:citizenapp/governance/shared/proposal/proposal_detail_local_store.dart';
import 'package:citizenapp/votingengine/internal-vote/internal_vote_service.dart';
import 'package:citizenapp/transaction/duoqian-transfer/duoqian_transfer_balance_guard.dart';
import 'package:citizenapp/transaction/duoqian-transfer/duoqian_transfer_models.dart';
import 'package:citizenapp/transaction/duoqian-transfer/duoqian_transfer_service.dart';
import 'package:citizenapp/qr/pages/qr_sign_session_page.dart';
import 'package:citizenapp/rpc/onchain.dart';
import 'package:citizenapp/rpc/smoldot_client.dart';
import 'package:citizenapp/qr/bodies/sign_request_body.dart';
import 'package:citizenapp/signer/qr_signer.dart';
import 'package:citizenapp/wallet/core/wallet_manager.dart';
import 'package:citizenapp/votingengine/internal-vote/proposal_vote_widgets.dart';

/// 详情页展示/投票的三种提案类型。
///
/// 决定：读哪个 storage map 与 QR 签名如何展示。
/// Phase 3(2026-04-22)起,投票动作统一走 `InternalVote::cast`
/// (9.0),不再按 kind 区分 call_index;`kind` 仅影响"创建提案"路径与
/// 详情展示逻辑。
enum DuoqianTransferKind {
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
class DuoqianTransferDetailPage extends StatefulWidget {
  const DuoqianTransferDetailPage({
    super.key,
    required this.institution,
    required this.proposalId,
    required this.proposalContext,
    this.kind = DuoqianTransferKind.transfer,
  });

  final InstitutionInfo institution;
  final int proposalId;
  final DuoqianTransferKind kind;

  /// 统一的提案上下文。
  final ProposalContext proposalContext;

  /// 便捷访问。
  List<WalletProfile> get adminWallets => proposalContext.adminWallets;

  @override
  State<DuoqianTransferDetailPage> createState() =>
      _DuoqianTransferDetailPageState();
}

class _DuoqianTransferDetailPageState extends State<DuoqianTransferDetailPage> {
  static const int _statusVoting = 0;

  final DuoqianTransferService _proposalService = DuoqianTransferService();
  final ProposalDetailLocalStore _detailStore =
      ProposalDetailLocalStore.instance;
  final InstitutionAdminService _adminService = InstitutionAdminService();
  AdminAccountIdentity get _accountIdentity =>
      AdminAccountIdentity.fromInstitution(widget.institution);
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
      case DuoqianTransferKind.transfer:
        return 'transfer';
      case DuoqianTransferKind.safetyFund:
        return 'safety_fund';
      case DuoqianTransferKind.sweep:
        return 'sweep';
    }
  }

  /// 签名显示用的人类可读类型名。
  String get _kindLabel {
    switch (widget.kind) {
      case DuoqianTransferKind.transfer:
        return '转账提案';
      case DuoqianTransferKind.safetyFund:
        return '安全基金转账提案';
      case DuoqianTransferKind.sweep:
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
  int? _thresholdSnapshot;

  // 管理员列表与投票记录
  List<String> _admins = const [];
  // pubkeyHex → true(赞成) / false(反对) / null(未投票)
  Map<String, bool?> _adminVotes = {};

  // 当前用户可投票的管理员钱包
  List<WalletProfile> _votableWallets = const [];
  WalletProfile? _selectedVoteWallet;

  // 已提交投票但尚未上链确认的管理员公钥集合
  Set<String> _pendingPubkeys = const {};
  String? _voteNotice;
  bool _voteNoticeIsError = false;
  // ADR-018 R2:待投票确认期改用 finalized 头订阅驱动刷新,空闲链不再 20s 盲轮询。
  ChainEventSubscription? _pendingSub;
  StreamSubscription<ChainEvent>? _pendingEventSub;

  @override
  void initState() {
    super.initState();
    _load();
  }

  @override
  void dispose() {
    _pendingEventSub?.cancel();
    _pendingSub?.disconnect();
    super.dispose();
  }

  Future<void> _load({bool showSpinner = true}) async {
    ProposalDetailSnapshot? localSnapshot;
    if (showSpinner) {
      localSnapshot = await _applyLocalSnapshot();
    }
    if (showSpinner &&
        localSnapshot?.isFresh(ProposalDetailLocalStore.activeTtl) == true) {
      return;
    }

    if (showSpinner && localSnapshot == null) {
      setState(() {
        _loading = true;
        _error = null;
      });
    } else if (mounted) {
      setState(() => _error = null);
    }

    try {
      // 根据 kind 选择对应的详情查询方法。
      // 不同 kind 写在不同的 storage map：
      //   transfer   → VotingEngine.ProposalData（带 dq-xfer tag）
      //   safetyFund → DuoqianTransfer.SafetyFundProposalActions
      //   sweep      → DuoqianTransfer.SweepProposalActions
      final Future<dynamic> detailFuture;
      switch (widget.kind) {
        case DuoqianTransferKind.transfer:
          detailFuture =
              _proposalService.fetchProposalAction(widget.proposalId);
          break;
        case DuoqianTransferKind.safetyFund:
          detailFuture =
              _proposalService.fetchSafetyFundAction(widget.proposalId);
          break;
        case DuoqianTransferKind.sweep:
          detailFuture = _proposalService.fetchSweepAction(widget.proposalId);
          break;
      }

      // 并行加载提案快照、提案状态、投票计数、提案详情。
      // 中文注释：多签转账投票资格和进度必须以提案创建时的快照为准，
      // 不能使用机构当前管理员列表或当前阈值，否则管理员变更后旧提案会显示错误。
      final results = await Future.wait([
        _proposalService.fetchAdminSnapshot(
          widget.proposalId,
          widget.institution,
        ),
        _proposalService.fetchInternalThresholdSnapshot(widget.proposalId),
        _proposalService.fetchProposalStatus(widget.proposalId),
        _proposalService.fetchVoteTally(widget.proposalId),
        detailFuture,
      ]);

      var admins = results[0] as List<String>;
      final thresholdSnapshot = results[1] as int?;
      final status = results[2] as int?;
      final tally = results[3] as ({int yes, int no});
      final detail = results[4];
      if (admins.isEmpty) {
        admins = await _adminService.fetchAdmins(_accountIdentity);
      }

      // 中文注释：管理员投票记录批量读取，避免 43 个管理员产生 43 次 RPC。
      final votes = await _proposalService.fetchAdminVotesBatch(
        widget.proposalId,
        admins,
      );

      // 检查待确认投票：先批量确认，再获取仍在等待的记录。
      // 用 kind 对应的 type key，避免跨类型提案 ID 误判（虽然 ID 全局递增，
      // 但分开归档便于后续清理/迁移）。
      //
      // 中文注释：nonce 只由 runtime frame_system 管理；这里仅根据投票引擎
      // storage 清理 pending 记录，不再重置或回滚客户端本地 nonce。
      final pendingSummary = await PendingVoteStore.instance.confirmAllDetailed(
        _proposalTypeKey,
        widget.proposalId,
        OnchainRpc(),
      );
      for (final confirmed in pendingSummary.confirmed) {
        votes[confirmed.walletPubkey] = confirmed.approve;
      }
      final pendingPks =
          pendingSummary.stillPending.map((r) => r.walletPubkey).toSet();
      final pendingNotice = _pendingSummaryNotice(pendingSummary);

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
      try {
        await _detailStore.put(_snapshotFromChain(
          status: status,
          tally: tally,
          thresholdSnapshot: thresholdSnapshot,
          admins: admins,
          votes: votes,
          pendingPks: pendingPks,
          detail: detail,
        ));
      } catch (e) {
        // 中文注释：详情快照写入失败不能影响链上最新结果展示；仅留痕便于排查。
        debugPrint('[DuoqianDetail] 详情快照写入失败: $e');
      }
      if (!mounted) return;
      setState(() {
        _admins = admins;
        _status = status;
        _yesCount = tally.yes;
        _noCount = tally.no;
        _thresholdSnapshot = thresholdSnapshot;
        _adminVotes = votes;
        _pendingPubkeys = pendingPks;
        _votableWallets = votable;
        _selectedVoteWallet = votable.isNotEmpty ? votable.first : null;
        if (pendingNotice != null) {
          _voteNotice = pendingNotice.$1;
          _voteNoticeIsError = pendingNotice.$2;
        }
        _transferInfo = null;
        _safetyFundInfo = null;
        _sweepInfo = null;
        switch (widget.kind) {
          case DuoqianTransferKind.transfer:
            _transferInfo = detail as TransferProposalInfo?;
            break;
          case DuoqianTransferKind.safetyFund:
            _safetyFundInfo = detail as SafetyFundProposalInfo?;
            break;
          case DuoqianTransferKind.sweep:
            _sweepInfo = detail as SweepProposalInfo?;
            break;
        }
        _loading = false;
      });
      _syncPendingPoll(pendingPks.isNotEmpty && status == _statusVoting);
    } catch (e) {
      if (!mounted) return;
      if (localSnapshot != null) {
        setState(() => _loading = false);
        return;
      }
      setState(() {
        _error = SmoldotClientManager.instance.buildUserFacingError(e);
        _loading = false;
      });
    }
  }

  Future<ProposalDetailSnapshot?> _applyLocalSnapshot() async {
    try {
      final snapshot =
          await _detailStore.read(_proposalTypeKey, widget.proposalId);
      if (snapshot == null || !mounted) return snapshot;
      final admins = snapshot.admins;
      final pendingPks = snapshot.pendingPubkeys.toSet();
      final votable = <WalletProfile>[];
      for (final w in widget.adminWallets) {
        final pk = _normalizePubkey(w.pubkeyHex);
        if (admins.contains(pk) &&
            snapshot.adminVotes[pk] == null &&
            !pendingPks.contains(pk)) {
          votable.add(w);
        }
      }
      final detail = _detailFromSnapshot(snapshot);
      setState(() {
        _admins = admins;
        _status = snapshot.status;
        _yesCount = snapshot.yesCount;
        _noCount = snapshot.noCount;
        _thresholdSnapshot = snapshot.threshold;
        _adminVotes = snapshot.adminVotes;
        _pendingPubkeys = pendingPks;
        _votableWallets = votable;
        _selectedVoteWallet = votable.isNotEmpty ? votable.first : null;
        _transferInfo = null;
        _safetyFundInfo = null;
        _sweepInfo = null;
        switch (widget.kind) {
          case DuoqianTransferKind.transfer:
            _transferInfo = detail as TransferProposalInfo?;
            break;
          case DuoqianTransferKind.safetyFund:
            _safetyFundInfo = detail as SafetyFundProposalInfo?;
            break;
          case DuoqianTransferKind.sweep:
            _sweepInfo = detail as SweepProposalInfo?;
            break;
        }
        _loading = false;
        _error = null;
      });
      _syncPendingPoll(
          pendingPks.isNotEmpty && snapshot.status == _statusVoting);
      return snapshot;
    } catch (e) {
      debugPrint('[DuoqianDetail] 加载多签详情快照失败: $e');
      return null;
    }
  }

  ProposalDetailSnapshot _snapshotFromChain({
    required int? status,
    required ({int yes, int no}) tally,
    required int? thresholdSnapshot,
    required List<String> admins,
    required Map<String, bool?> votes,
    required Set<String> pendingPks,
    required Object? detail,
  }) {
    return ProposalDetailSnapshot(
      proposalId: widget.proposalId,
      typeKey: _proposalTypeKey,
      updatedAtMillis: DateTime.now().millisecondsSinceEpoch,
      status: status,
      yesCount: tally.yes,
      noCount: tally.no,
      threshold: thresholdSnapshot,
      admins: admins.map(_normalizePubkey).toList(growable: false),
      adminVotes: votes.map(
        (key, value) => MapEntry(_normalizePubkey(key), value),
      ),
      pendingPubkeys: pendingPks.map(_normalizePubkey).toList(growable: false),
      detail: _detailToJson(detail),
    );
  }

  Map<String, Object?> _detailToJson(Object? detail) {
    if (detail is TransferProposalInfo) {
      return {
        'kind': 'transfer',
        'institution_bytes_hex': _toHex(detail.institutionBytes),
        'beneficiary': detail.beneficiary,
        'amount_fen': detail.amountFen.toString(),
        'remark': detail.remark,
        'proposer': detail.proposer,
        'status': detail.status,
      };
    }
    if (detail is SafetyFundProposalInfo) {
      return {
        'kind': 'safety_fund',
        'beneficiary': detail.beneficiary,
        'amount_fen': detail.amountFen.toString(),
        'remark': detail.remark,
        'proposer': detail.proposer,
        'status': detail.status,
      };
    }
    if (detail is SweepProposalInfo) {
      return {
        'kind': 'sweep',
        'institution_bytes_hex': _toHex(detail.institutionBytes),
        'amount_fen': detail.amountFen.toString(),
        'status': detail.status,
      };
    }
    return const {};
  }

  Object? _detailFromSnapshot(ProposalDetailSnapshot snapshot) {
    final detail = snapshot.detail;
    final kind = detail['kind']?.toString();
    if (kind == 'transfer') {
      final amountFen = BigInt.tryParse(detail['amount_fen']?.toString() ?? '');
      final institutionBytesHex = detail['institution_bytes_hex']?.toString();
      if (amountFen == null || institutionBytesHex == null) return null;
      return TransferProposalInfo(
        proposalId: snapshot.proposalId,
        institutionBytes: _hexDecode(institutionBytesHex),
        beneficiary: detail['beneficiary']?.toString() ?? '',
        amountFen: amountFen,
        remark: detail['remark']?.toString() ?? '',
        proposer: detail['proposer']?.toString() ?? '',
        status: snapshot.status,
      );
    }
    if (kind == 'safety_fund') {
      final amountFen = BigInt.tryParse(detail['amount_fen']?.toString() ?? '');
      if (amountFen == null) return null;
      return SafetyFundProposalInfo(
        proposalId: snapshot.proposalId,
        beneficiary: detail['beneficiary']?.toString() ?? '',
        amountFen: amountFen,
        remark: detail['remark']?.toString() ?? '',
        proposer: detail['proposer']?.toString() ?? '',
        status: snapshot.status,
      );
    }
    if (kind == 'sweep') {
      final amountFen = BigInt.tryParse(detail['amount_fen']?.toString() ?? '');
      final institutionBytesHex = detail['institution_bytes_hex']?.toString();
      if (amountFen == null || institutionBytesHex == null) return null;
      return SweepProposalInfo(
        proposalId: snapshot.proposalId,
        institutionBytes: _hexDecode(institutionBytesHex),
        amountFen: amountFen,
        status: snapshot.status,
      );
    }
    return null;
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

  String _normalizePubkey(String pubkeyHex) {
    var pubkey = pubkeyHex.toLowerCase();
    if (pubkey.startsWith('0x')) pubkey = pubkey.substring(2);
    return pubkey;
  }

  (String, bool)? _pendingSummaryNotice(PendingVoteConfirmSummary summary) {
    if (summary.lost.isNotEmpty) {
      return ('${summary.lost.length} 笔投票未写入链上投票记录，已清除等待状态，可重新提交。', true);
    }
    if (summary.confirmed.isNotEmpty) {
      return ('${summary.confirmed.length} 笔投票已由链上投票记录确认。', false);
    }
    return null;
  }

  void _syncPendingPoll(bool enabled) {
    if (!enabled) {
      _pendingEventSub?.cancel();
      _pendingEventSub = null;
      _pendingSub?.disconnect();
      _pendingSub = null;
      return;
    }
    if (_pendingSub != null) return;
    // 待投票确认期:订阅 finalized 头,有新最终块(即有新交易上链)才刷新,
    // 空闲链零查询;不再每 20 秒盲查一次。
    final sub = ChainEventSubscription();
    if (!sub.connect()) {
      sub.disconnect();
      return;
    }
    _pendingSub = sub;
    _pendingEventSub = sub.events.listen((event) {
      if (event.type != ChainEventType.newFinalizedBlock) return;
      if (!mounted || _loading) return;
      unawaited(_load(showSpinner: false));
    });
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

    final balanceBlockedReason =
        await DuoqianTransferBalanceGuard.checkAdminWalletBalance(
      wallet: wallet,
      requiredFeeYuan: DuoqianTransferBalanceGuard.voteFeeYuan,
      actionLabel: '提交多签转账投票',
    );
    if (balanceBlockedReason != null) {
      if (!mounted) return;
      ScaffoldMessenger.of(context).showSnackBar(
        SnackBar(
          content: Text(balanceBlockedReason),
          backgroundColor: AppTheme.danger,
        ),
      );
      return;
    }

    setState(() => _submitting = true);

    try {
      final pubkeyBytes = _hexDecode(wallet.pubkeyHex);
      final pubkey = _normalizePubkey(wallet.pubkeyHex);

      // 热钱包：先认证，后续 signCallback 优先走本地签名;冷钱包：fallback QR 签名。
      WalletManager? hotWalletManager;
      if (wallet.isHotWallet) {
        hotWalletManager = WalletManager();
        await hotWalletManager.authenticateForSigning();
      }

      Future<Uint8List> signCallback(Uint8List payload) async {
        if (hotWalletManager != null) {
          return await hotWalletManager.signWithWalletNoAuth(
              wallet.walletIndex, payload);
        }
        // 冷钱包 QR 签名
        final qrSigner = QrSigner();
        final voteText = approve ? '赞成' : '反对';
        final request = qrSigner.buildRequest(
          requestId: QrSigner.generateRequestId(prefix: 'vote-'),
          address: wallet.address,
          pubkey: '0x${wallet.pubkeyHex}',
          payloadHex: '0x${_toHex(payload)}',
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

      // Phase 3: 所有管理员投票统一走 InternalVote::cast(22.0)。
      // 业务 kind 仅用于 QR 展示的文案与 storage 读取,不再分派 call_index。
      final result = await InternalVoteService().submit(
        proposalId: widget.proposalId,
        approve: approve,
        fromAddress: wallet.address,
        signerPubkey: Uint8List.fromList(pubkeyBytes),
        sign: signCallback,
        onWatchEvent: (event) {
          if (event.isIncluded) {
            unawaited(_load(showSpinner: false));
          }
        },
      );
      debugPrint(
          '[DuoqianTransferVote] submit 已入块 txHash=${result.txHash} nonce=${result.usedNonce} block=${result.blockHashHex}');

      // 中文注释：服务层已经确认 runtime 投票记录，新流程不再写 pending。
      // 这里只清除旧版本可能残留的同管理员 pending 记录。
      await PendingVoteStore.instance.remove(
        _proposalTypeKey,
        widget.proposalId,
        pubkey,
      );

      if (!mounted) return;
      setState(() {
        _adminVotes[pubkey] = approve;
        _pendingPubkeys = _pendingPubkeys.difference({pubkey});
        _votableWallets = _votableWallets
            .where((w) => _normalizePubkey(w.pubkeyHex) != pubkey)
            .toList(growable: false);
        _selectedVoteWallet =
            _votableWallets.isNotEmpty ? _votableWallets.first : null;
        _voteNotice = '链上已确认该管理员投票。';
        _voteNoticeIsError = false;
      });
      ScaffoldMessenger.of(context).showSnackBar(
        SnackBar(
          content: Text('投票已由 runtime 确认：${_truncateAddress(result.txHash)}'),
          backgroundColor: AppTheme.primaryDark,
        ),
      );

      // 刷新数据
      _adminService.clearCache(_accountIdentity);
      // 中文注释：服务层已经等待入块并回读 InternalVote storage；这里
      // 只后台刷新展示状态，不能再把 txHash 当作投票成功依据。
      unawaited(_load(showSpinner: false));
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
        _adminService.clearCache(_accountIdentity);
        await _load();
      },
      child: ListView(
        padding: const EdgeInsets.fromLTRB(16, 8, 16, 32),
        children: [
          ProposalStatusBadge(status: _status, proposalId: widget.proposalId),
          if (_voteNotice != null) ...[
            const SizedBox(height: 12),
            _buildVoteNotice(),
          ],
          const SizedBox(height: 16),
          _buildProposalInfoCard(),
          const SizedBox(height: 16),
          ProposalVoteProgress(
            yesCount: _yesCount,
            noCount: _noCount,
            threshold:
                _thresholdSnapshot ?? widget.institution.internalThreshold,
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

  Widget _buildVoteNotice() {
    final color = _voteNoticeIsError ? AppTheme.danger : AppTheme.info;
    return Container(
      padding: const EdgeInsets.all(12),
      decoration: BoxDecoration(
        color: color.withValues(alpha: 0.08),
        borderRadius: BorderRadius.circular(8),
        border: Border.all(color: color.withValues(alpha: 0.18)),
      ),
      child: Row(
        crossAxisAlignment: CrossAxisAlignment.start,
        children: [
          Icon(
            _voteNoticeIsError ? Icons.error_outline : Icons.info_outline,
            size: 18,
            color: color,
          ),
          const SizedBox(width: 8),
          Expanded(
            child: Text(
              _voteNotice!,
              style: TextStyle(
                fontSize: 13,
                color: color,
                fontWeight: FontWeight.w600,
              ),
            ),
          ),
        ],
      ),
    );
  }

  // ──── 提案信息卡片 ────

  /// 提案创建者公钥（仅 transfer / safetyFund 有）。
  String? get _proposerPubkey {
    switch (widget.kind) {
      case DuoqianTransferKind.transfer:
        return _transferInfo?.proposer;
      case DuoqianTransferKind.safetyFund:
        return _safetyFundInfo?.proposer;
      case DuoqianTransferKind.sweep:
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
      case DuoqianTransferKind.transfer:
        return _buildTransferRows();
      case DuoqianTransferKind.safetyFund:
        return _buildSafetyFundRows();
      case DuoqianTransferKind.sweep:
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
