import 'dart:async';
import 'dart:typed_data';

import 'package:flutter/material.dart';
import 'package:flutter/services.dart';
import 'package:polkadart/polkadart.dart' show Hasher;
import 'package:polkadart_keyring/polkadart_keyring.dart' show Keyring;
import 'package:wuminapp_mobile/governance/admins-change/models/admin_account.dart';
import 'package:wuminapp_mobile/governance/admins-change/services/institution_admin_service.dart';
import 'package:wuminapp_mobile/governance/shared/institution_info.dart';
import 'package:wuminapp_mobile/votingengine/internal-vote/internal_vote_service.dart';
import 'package:wuminapp_mobile/votingengine/internal-vote/pending_vote_store.dart';
import 'package:wuminapp_mobile/governance/shared/proposal/proposal_context.dart';
import 'package:wuminapp_mobile/governance/shared/proposal/proposal_detail_local_store.dart';
import 'package:wuminapp_mobile/votingengine/internal-vote/proposal_vote_widgets.dart';
import 'package:wuminapp_mobile/governance/shared/proposal/proposal_query_service.dart';
import 'package:wuminapp_mobile/qr/bodies/sign_request_body.dart';
import 'package:wuminapp_mobile/qr/pages/qr_sign_session_page.dart';
import 'package:wuminapp_mobile/rpc/chain_rpc.dart';
import 'package:wuminapp_mobile/rpc/onchain.dart';
import 'package:wuminapp_mobile/rpc/smoldot_client.dart';
import 'package:wuminapp_mobile/signer/qr_signer.dart';
import 'package:wuminapp_mobile/ui/app_theme.dart';
import 'package:wuminapp_mobile/my/util/amount_format.dart';
import 'package:wuminapp_mobile/wallet/core/wallet_manager.dart';
import 'package:wuminapp_mobile/governance/personal-manage/personal_manage_models.dart'
    as personal_models;
import 'package:wuminapp_mobile/governance/personal-manage/personal_manage_service.dart';
import 'package:wuminapp_mobile/governance/organization-manage/institution_manage_models.dart'
    as org_models;
import 'package:wuminapp_mobile/governance/organization-manage/institution_manage_service.dart';

/// 多签管理提案详情页：展示创建/关闭提案信息、投票进度及投票操作。
class InstitutionManageDetailPage extends StatefulWidget {
  const InstitutionManageDetailPage({
    super.key,
    required this.institution,
    required this.proposalId,
    required this.proposalContext,
  });

  final InstitutionInfo institution;
  final int proposalId;
  final ProposalContext proposalContext;

  List<WalletProfile> get adminWallets => proposalContext.adminWallets;

  @override
  State<InstitutionManageDetailPage> createState() =>
      _InstitutionManageDetailPageState();
}

class _InstitutionManageDetailPageState extends State<InstitutionManageDetailPage> {
  static const int _statusVoting = 0;

  final ProposalQueryService _proposalService = ProposalQueryService();
  final ProposalDetailLocalStore _detailStore =
      ProposalDetailLocalStore.instance;
  final InstitutionManageService _manageService = InstitutionManageService();
  final PersonalManageService _personalManageService = PersonalManageService();
  final InstitutionAdminService _adminService = InstitutionAdminService();
  AdminAccountIdentity get _accountIdentity =>
      AdminAccountIdentity.fromInstitution(widget.institution);
  bool _loading = true;
  String? _error;
  bool _submitting = false;

  int? _status;

  // 提案详情（二选一）
  personal_models.CreateDuoqianProposalInfo? _createInfo;
  personal_models.CloseDuoqianProposalInfo? _closeInfo;

  bool get _isCreateProposal => _createInfo != null;

  // 投票计数
  int _yesCount = 0;
  int _noCount = 0;
  int _threshold = 0;

  // 管理员列表与投票记录
  List<String> _admins = const [];
  Map<String, bool?> _adminVotes = {};

  List<WalletProfile> _votableWallets = const [];
  WalletProfile? _selectedVoteWallet;
  Set<String> _pendingPubkeys = const {};
  String? _voteNotice;
  bool _voteNoticeIsError = false;
  Timer? _pendingPollTimer;

  @override
  void initState() {
    super.initState();
    _load();
  }

  @override
  void dispose() {
    _pendingPollTimer?.cancel();
    super.dispose();
  }

  Future<void> _load({bool showSpinner = true}) async {
    debugPrint('[VoteDetail._load] 开始 proposalId=${widget.proposalId}');
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
      final rpc = ChainRpc();

      // step1:并行加载管理员快照、提案状态、投票计数、阈值快照。
      debugPrint(
          '[VoteDetail._load] step1: 并行 fetchAdmins/Status/Tally/Threshold...');
      final thresholdFuture = _proposalService
          .fetchInternalThresholdSnapshot(widget.proposalId)
          .catchError((_) => null);
      final adminSnapshotFuture = _proposalService
          .fetchAdminSnapshot(widget.proposalId, widget.institution)
          .catchError((_) => const <String>[]);
      final results = await Future.wait([
        _adminService.fetchAdmins(_accountIdentity),
        _proposalService.fetchProposalStatus(widget.proposalId),
        _proposalService.fetchVoteTally(widget.proposalId),
        thresholdFuture,
        adminSnapshotFuture,
      ]);

      final activeAdmins = results[0] as List<String>;
      final status = results[1] as int?;
      final tally = results[2] as ({int yes, int no});
      final thresholdSnapshot = results[3] as int?;
      final snapshotAdmins = results[4] as List<String>;
      final admins = snapshotAdmins.isNotEmpty ? snapshotAdmins : activeAdmins;
      final threshold = _resolveVoteThreshold(thresholdSnapshot, admins.length);
      debugPrint(
          '[VoteDetail._load] step1 完成 admins.len=${admins.length} status=$status yes=${tally.yes} no=${tally.no} threshold=$threshold');

      // step2:加载提案业务数据（从 ProposalData 解码）
      debugPrint('[VoteDetail._load] step2: fetchProposalData');
      final key = _buildProposalDataStorageKey(widget.proposalId);
      final raw = await rpc.fetchStorage('0x${_hexEncode(key)}');
      debugPrint('[VoteDetail._load] step2 完成 raw.len=${raw?.length ?? 0}');
      personal_models.CreateDuoqianProposalInfo? createInfo;
      personal_models.CloseDuoqianProposalInfo? closeInfo;
      if (raw != null && raw.isNotEmpty) {
        final personalDetail = _personalManageService
            .decodePersonalProposalData(widget.proposalId, raw);
        if (personalDetail is personal_models.CreateDuoqianProposalInfo) {
          createInfo = personalDetail;
        } else if (personalDetail is personal_models.CloseDuoqianProposalInfo) {
          closeInfo = personalDetail;
        } else {
          final orgDetail =
              _manageService.decodeManageProposalData(widget.proposalId, raw);
          if (orgDetail is org_models.CloseDuoqianProposalInfo) {
            closeInfo = personal_models.CloseDuoqianProposalInfo(
              proposalId: orgDetail.proposalId,
              duoqianAddress: orgDetail.duoqianAddress,
              beneficiary: orgDetail.beneficiary,
              proposer: orgDetail.proposer,
              status: orgDetail.status,
            );
          }
        }
      }

      // step3:检查待确认投票。nonce 只由 runtime frame_system 管理，
      // 客户端只清理 pending 记录，不再重置或回滚本地 nonce。
      debugPrint(
          '[VoteDetail._load] step3: PendingVoteStore.confirmAllDetailed');
      final pendingSummary = await PendingVoteStore.instance.confirmAllDetailed(
        'duoqian_manage',
        widget.proposalId,
        OnchainRpc(),
      );
      final pendingPks =
          pendingSummary.stillPending.map((r) => r.walletPubkey).toSet();
      final pendingNotice = _pendingSummaryNotice(pendingSummary);
      debugPrint(
          '[VoteDetail._load] step3 完成 stillPending.len=${pendingSummary.stillPending.length}');

      // step4:批量查询每位管理员的投票记录，避免按管理员逐条 RPC。
      debugPrint('[VoteDetail._load] step4: 批量查 admin 投票 (${admins.length} 个)');
      final votes = await _proposalService.fetchAdminVotesBatch(
        widget.proposalId,
        admins,
      );
      debugPrint('[VoteDetail._load] step4 完成');

      // 筛选可投票钱包
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

      if (!mounted) {
        debugPrint('[VoteDetail._load] !mounted 提前返回');
        return;
      }
      try {
        await _detailStore.put(_snapshotFromChain(
          status: status,
          tally: tally,
          threshold: threshold,
          admins: admins,
          votes: votes,
          pendingPks: pendingPks,
          createInfo: createInfo,
          closeInfo: closeInfo,
        ));
      } catch (_) {
        // 中文注释：详情快照只是首屏加速，写入失败不能影响链上结果展示。
      }
      if (!mounted) return;
      debugPrint('[VoteDetail._load] step5: setState');
      setState(() {
        _admins = admins;
        _status = status;
        _yesCount = tally.yes;
        _noCount = tally.no;
        _threshold = threshold;
        _adminVotes = votes;
        _pendingPubkeys = pendingPks;
        _votableWallets = votable;
        _selectedVoteWallet = votable.isNotEmpty ? votable.first : null;
        _createInfo = createInfo;
        _closeInfo = closeInfo;
        if (pendingNotice != null) {
          _voteNotice = pendingNotice.$1;
          _voteNoticeIsError = pendingNotice.$2;
        }
        _loading = false;
      });
      _syncPendingPoll(pendingPks.isNotEmpty && status == _statusVoting);
      debugPrint('[VoteDetail._load] 结束');
    } catch (e, st) {
      debugPrint('[VoteDetail._load] catch 异常: $e\n$st');
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
          await _detailStore.read('duoqian_manage', widget.proposalId);
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
      final createInfo = _createInfoFromSnapshot(snapshot);
      final closeInfo = _closeInfoFromSnapshot(snapshot);
      setState(() {
        _admins = admins;
        _status = snapshot.status;
        _yesCount = snapshot.yesCount;
        _noCount = snapshot.noCount;
        _threshold = snapshot.threshold ?? 0;
        _adminVotes = snapshot.adminVotes;
        _pendingPubkeys = pendingPks;
        _votableWallets = votable;
        _selectedVoteWallet = votable.isNotEmpty ? votable.first : null;
        _createInfo = createInfo;
        _closeInfo = closeInfo;
        _loading = false;
        _error = null;
      });
      _syncPendingPoll(
          pendingPks.isNotEmpty && snapshot.status == _statusVoting);
      return snapshot;
    } catch (_) {
      return null;
    }
  }

  ProposalDetailSnapshot _snapshotFromChain({
    required int? status,
    required ({int yes, int no}) tally,
    required int threshold,
    required List<String> admins,
    required Map<String, bool?> votes,
    required Set<String> pendingPks,
    required personal_models.CreateDuoqianProposalInfo? createInfo,
    required personal_models.CloseDuoqianProposalInfo? closeInfo,
  }) {
    return ProposalDetailSnapshot(
      proposalId: widget.proposalId,
      typeKey: 'duoqian_manage',
      updatedAtMillis: DateTime.now().millisecondsSinceEpoch,
      status: status,
      yesCount: tally.yes,
      noCount: tally.no,
      threshold: threshold,
      admins: admins.map(_normalizePubkey).toList(growable: false),
      adminVotes: votes.map(
        (key, value) => MapEntry(_normalizePubkey(key), value),
      ),
      pendingPubkeys: pendingPks.map(_normalizePubkey).toList(growable: false),
      detail: createInfo != null
          ? _createInfoToJson(createInfo)
          : closeInfo != null
              ? _closeInfoToJson(closeInfo)
              : const {},
    );
  }

  Map<String, Object?> _createInfoToJson(
    personal_models.CreateDuoqianProposalInfo info,
  ) {
    return {
      'kind': 'create',
      'duoqian_address': info.duoqianAddress,
      'proposer': info.proposer,
      'amount_fen': info.amountFen.toString(),
      'fee_fen': info.feeFen.toString(),
      'status': info.status,
    };
  }

  Map<String, Object?> _closeInfoToJson(
    personal_models.CloseDuoqianProposalInfo info,
  ) {
    return {
      'kind': 'close',
      'duoqian_address': info.duoqianAddress,
      'beneficiary': info.beneficiary,
      'proposer': info.proposer,
      'status': info.status,
    };
  }

  personal_models.CreateDuoqianProposalInfo? _createInfoFromSnapshot(
    ProposalDetailSnapshot snapshot,
  ) {
    final detail = snapshot.detail;
    if (detail['kind'] != 'create') return null;
    final amountFen = BigInt.tryParse(detail['amount_fen']?.toString() ?? '');
    final feeFen = BigInt.tryParse(detail['fee_fen']?.toString() ?? '');
    final duoqianAddress = detail['duoqian_address']?.toString();
    if (amountFen == null || feeFen == null || duoqianAddress == null) {
      return null;
    }
    return personal_models.CreateDuoqianProposalInfo(
      proposalId: snapshot.proposalId,
      duoqianAddress: duoqianAddress,
      proposer: detail['proposer']?.toString() ?? '',
      amountFen: amountFen,
      feeFen: feeFen,
      status: snapshot.status,
    );
  }

  personal_models.CloseDuoqianProposalInfo? _closeInfoFromSnapshot(
    ProposalDetailSnapshot snapshot,
  ) {
    final detail = snapshot.detail;
    if (detail['kind'] != 'close') return null;
    final duoqianAddress = detail['duoqian_address']?.toString();
    if (duoqianAddress == null) return null;
    return personal_models.CloseDuoqianProposalInfo(
      proposalId: snapshot.proposalId,
      duoqianAddress: duoqianAddress,
      beneficiary: detail['beneficiary']?.toString() ?? '',
      proposer: detail['proposer']?.toString() ?? '',
      status: snapshot.status,
    );
  }

  // ──── 工具方法 ────

  Uint8List _buildProposalDataStorageKey(int proposalId) {
    final palletHash = Hasher.twoxx128.hashString('VotingEngine');
    final storageHash = Hasher.twoxx128.hashString('ProposalData');
    final idBytes = _u64ToLeBytes(proposalId);
    final keyHash = _blake2128Concat(idBytes);
    final result =
        Uint8List(palletHash.length + storageHash.length + keyHash.length);
    var offset = 0;
    result.setAll(offset, palletHash);
    offset += palletHash.length;
    result.setAll(offset, storageHash);
    offset += storageHash.length;
    result.setAll(offset, keyHash);
    return result;
  }

  String _truncateAddress(String address) {
    if (address.length <= 14) return address;
    return '${address.substring(0, 6)}...${address.substring(address.length - 6)}';
  }

  int _resolveVoteThreshold(int? snapshotThreshold, int adminCount) {
    if (snapshotThreshold != null && snapshotThreshold > 0) {
      return snapshotThreshold;
    }
    final institutionThreshold = widget.institution.internalThreshold;
    if (institutionThreshold > 0) return institutionThreshold;
    return adminCount;
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
      _pendingPollTimer?.cancel();
      _pendingPollTimer = null;
      return;
    }
    if (_pendingPollTimer != null) return;
    _pendingPollTimer = Timer.periodic(const Duration(seconds: 20), (_) {
      if (!mounted || _loading) return;
      unawaited(_load(showSpinner: false));
    });
  }

  // ──── 投票提交 ────

  bool get _isCurrentUserAdmin => widget.proposalContext.isAdmin;

  bool get _canVote {
    if (_selectedVoteWallet == null) return false;
    if (_status != _statusVoting) return false;
    return _votableWallets.isNotEmpty;
  }

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
    debugPrint(
        '[VoteDetail] _submitVote 开始 approve=$approve proposalId=${widget.proposalId}');
    final wallet = _selectedVoteWallet;
    if (wallet == null) {
      debugPrint('[VoteDetail] _submitVote 无可投钱包,直接 return');
      return;
    }
    debugPrint(
        '[VoteDetail] 选中钱包 ${wallet.address} pubkey=${wallet.pubkeyHex} isHot=${wallet.isHotWallet}');

    setState(() => _submitting = true);

    try {
      final pubkeyBytes = _hexDecode(wallet.pubkeyHex);
      final pubkey = _normalizePubkey(wallet.pubkeyHex);
      if (!_admins.contains(pubkey)) {
        throw StateError('当前钱包不在该提案的管理员快照中，不能投票');
      }
      if (_adminVotes[pubkey] != null) {
        throw StateError('当前管理员已经投过票');
      }
      if (_pendingPubkeys.contains(pubkey)) {
        throw StateError('当前管理员已有待确认投票，请稍后刷新');
      }
      final balance = await ChainRpc().fetchFinalizedBalance(pubkey);
      if (balance <= 0) {
        throw StateError('当前管理员钱包余额不足，无法支付链上投票手续费');
      }

      // 热钱包：先认证，后续用本地签名；冷钱包：走 QR 签名。
      WalletManager? hotWalletManager;
      if (wallet.isHotWallet) {
        debugPrint('[VoteDetail] 热钱包 authenticate 开始');
        hotWalletManager = WalletManager();
        await hotWalletManager.authenticateForSigning();
        debugPrint('[VoteDetail] 热钱包 authenticate 完成');
      }

      Future<Uint8List> signCallback(Uint8List payload) async {
        if (hotWalletManager != null) {
          return await hotWalletManager.signWithWalletNoAuth(
              wallet.walletIndex, payload);
        }
        // 冷钱包 QR 签名
        final qrSigner = QrSigner();
        final voteText = approve ? '赞成' : '反对';
        final summaryType = _isCreateProposal ? '创建多签' : '关闭多签';
        final request = qrSigner.buildRequest(
          requestId: QrSigner.generateRequestId(prefix: 'vote-'),
          address: wallet.address,
          pubkey: '0x${wallet.pubkeyHex}',
          payloadHex: '0x${_toHex(payload)}',
          display: SignDisplay(
            action: 'internal_vote',
            summary: '$summaryType提案 #${widget.proposalId} 投票：$voteText',
            fields: [
              // internal_vote 链端 fields 按 Registry = (proposal_id, approve)。
              // _createInfo / _closeInfo 属辅助上下文,页面已独立展示,
              // 不塞 display.fields 避免对齐失败(2026-04-22 两色识别整改)。
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

      // Phase 3: 创建/关闭多签的投票都走 InternalVote::cast(22.0),
      // 由 runtime 的 InternalVoteExecutor 按 MODULE_TAG+ACTION 分派。
      debugPrint('[VoteDetail] 调 InternalVoteService.submit');
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
          '[VoteDetail] submit 已入块 txHash=${result.txHash} nonce=${result.usedNonce} block=${result.blockHashHex}');

      // 中文注释：服务层已经确认 runtime 投票记录，新流程不再写 pending。
      // 这里只清除旧版本可能残留的同管理员 pending 记录。
      await PendingVoteStore.instance.remove(
        'duoqian_manage',
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

      _adminService.clearCache(_accountIdentity);
      // 中文注释：服务层已经等待入块并回读 InternalVote storage；这里
      // 只后台刷新展示状态，不能再把 txHash 当作投票成功依据。
      debugPrint('[VoteDetail] fire-and-forget 调 _load 后台刷新');
      unawaited(_load());
    } catch (e, st) {
      debugPrint('[VoteDetail] _submitVote catch 异常: $e\n$st');
      if (!mounted) return;
      ScaffoldMessenger.of(context).showSnackBar(
        SnackBar(
          content: Text('投票失败：$e'),
          backgroundColor: AppTheme.danger,
        ),
      );
    } finally {
      debugPrint('[VoteDetail] finally setState(_submitting=false)');
      if (mounted) setState(() => _submitting = false);
    }
  }

  String _normalizePubkey(String pubkeyHex) {
    var pubkey = pubkeyHex.toLowerCase();
    if (pubkey.startsWith('0x')) pubkey = pubkey.substring(2);
    return pubkey;
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
      backgroundColor: Colors.white,
      appBar: AppBar(
        title: const Text(
          '提案详情',
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
            threshold: _threshold,
          ),
          const SizedBox(height: 16),
          ProposalAdminVoteList(
            admins: _admins,
            adminVotes: _adminVotes,
            pendingPubkeys: _pendingPubkeys,
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
            Text(
              _isCreateProposal ? '创建多签提案信息' : '关闭多签提案信息',
              style: const TextStyle(
                fontSize: 16,
                fontWeight: FontWeight.w700,
                color: AppTheme.primaryDark,
              ),
            ),
            const SizedBox(height: 12),
            if (_createInfo != null) ..._buildCreateInfoRows(),
            if (_closeInfo != null) ..._buildCloseInfoRows(),
          ],
        ),
      ),
    );
  }

  List<Widget> _buildCreateInfoRows() {
    final info = _createInfo!;
    final duoqianSs58 =
        Keyring().encodeAddress(_hexDecode(info.duoqianAddress), 2027);
    return [
      _buildInfoRow('多签地址', _truncateAddress(duoqianSs58), onCopy: () {
        Clipboard.setData(ClipboardData(text: duoqianSs58));
        ScaffoldMessenger.of(context).showSnackBar(
          const SnackBar(
              content: Text('地址已复制'), duration: Duration(seconds: 1)),
        );
      }),
      const Divider(height: 20),
      _buildInfoRow('发起人', _truncateAddress(info.proposer)),
      const Divider(height: 20),
      _buildInfoRow(
        '初始资金',
        '${AmountFormat.format(info.amountYuan, symbol: '')} 元',
      ),
      const Divider(height: 20),
      _buildInfoRow(
        '创建手续费',
        '${AmountFormat.format(info.feeYuan, symbol: '')} 元',
      ),
    ];
  }

  List<Widget> _buildCloseInfoRows() {
    final info = _closeInfo!;
    final duoqianSs58 =
        Keyring().encodeAddress(_hexDecode(info.duoqianAddress), 2027);
    return [
      _buildInfoRow('多签地址', _truncateAddress(duoqianSs58), onCopy: () {
        Clipboard.setData(ClipboardData(text: duoqianSs58));
        ScaffoldMessenger.of(context).showSnackBar(
          const SnackBar(
              content: Text('地址已复制'), duration: Duration(seconds: 1)),
        );
      }),
      const Divider(height: 20),
      _buildInfoRow('受益人', _truncateAddress(info.beneficiary), onCopy: () {
        Clipboard.setData(ClipboardData(text: info.beneficiary));
        ScaffoldMessenger.of(context).showSnackBar(
          const SnackBar(
              content: Text('地址已复制'), duration: Duration(seconds: 1)),
        );
      }),
      const Divider(height: 20),
      _buildInfoRow('发起人', _truncateAddress(info.proposer)),
    ];
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

  // ──── 工具 ────

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

  static String _hexEncode(Uint8List bytes) {
    return bytes.map((b) => b.toRadixString(16).padLeft(2, '0')).join();
  }

  Uint8List _u64ToLeBytes(int value) {
    final bytes = Uint8List(8);
    final bd = ByteData.sublistView(bytes);
    bd.setUint64(0, value, Endian.little);
    return bytes;
  }

  Uint8List _blake2128Concat(Uint8List data) {
    final hash = Hasher.blake2b128.hash(data);
    final result = Uint8List(hash.length + data.length);
    result.setAll(0, hash);
    result.setAll(hash.length, data);
    return result;
  }
}
