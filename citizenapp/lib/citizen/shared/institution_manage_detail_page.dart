import 'dart:async';
import 'dart:typed_data';

import 'package:flutter/material.dart';
import 'package:flutter/services.dart';
import 'package:polkadart/polkadart.dart' show Hasher;
import 'package:polkadart_keyring/polkadart_keyring.dart' show Keyring;
import 'package:citizenapp/citizen/proposal/admins-change/models/admin_account.dart';
import 'package:citizenapp/citizen/proposal/admins-change/services/institution_admin_service.dart';
import 'package:citizenapp/citizen/shared/institution_info.dart';
import 'package:citizenapp/votingengine/internal-vote/internal_vote_service.dart';
import 'package:citizenapp/citizen/shared/proposal/proposal_context.dart';
import 'package:citizenapp/citizen/shared/proposal/proposal_detail_local_store.dart';
import 'package:citizenapp/votingengine/internal-vote/proposal_vote_widgets.dart';
import 'package:citizenapp/citizen/shared/proposal/proposal_query_service.dart';
import 'package:citizenapp/citizen/shared/proposal/proposal_models.dart';
import 'package:citizenapp/qr/pages/qr_sign_session_page.dart';
import 'package:citizenapp/qr/qr_protocols.dart';
import 'package:citizenapp/rpc/chain_rpc.dart';
import 'package:citizenapp/rpc/smoldot_client.dart';
import 'package:citizenapp/signer/qr_signer.dart';
import 'package:citizenapp/ui/app_theme.dart';
import 'package:citizenapp/my/util/amount_format.dart';
import 'package:citizenapp/wallet/core/wallet_manager.dart';
import 'package:citizenapp/transaction/personal-manage/personal_manage_models.dart'
    as personal_models;
import 'package:citizenapp/transaction/personal-manage/personal_manage_service.dart';
import 'package:citizenapp/citizen/institution/institution_models.dart'
    as institution_models;
import 'package:citizenapp/citizen/institution/institution_chain_service.dart';

/// 多签管理提案详情页：展示创建/关闭提案信息、投票进度及投票操作。
class MultisigProposalDetailPage extends StatefulWidget {
  const MultisigProposalDetailPage({
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
  State<MultisigProposalDetailPage> createState() =>
      _MultisigProposalDetailPageState();
}

class _MultisigProposalDetailPageState
    extends State<MultisigProposalDetailPage> {
  static const int _statusVoting = 0;

  final ProposalQueryService _proposalService = ProposalQueryService();
  final ProposalDetailLocalStore _detailStore =
      ProposalDetailLocalStore.instance;
  final InstitutionChainService _manageService = InstitutionChainService();
  final PersonalManageService _personalManageService = PersonalManageService();
  final InstitutionAdminService _adminService = InstitutionAdminService();
  AdminAccountIdentity get _accountIdentity =>
      AdminAccountIdentity.fromInstitution(widget.institution);
  bool _loading = true;
  String? _error;
  bool _submitting = false;

  int? _status;

  // 提案详情（二选一）
  personal_models.CreateProposalInfo? _createInfo;
  personal_models.CloseProposalInfo? _closeInfo;
  institution_models.CloseProposalInfo? _institutionCloseInfo;

  bool get _isCreateProposal => _createInfo != null;

  // 投票计数
  int _yesCount = 0;
  int _noCount = 0;
  int _threshold = 0;

  // 提案创建时冻结的合格选民与投票记录；机构路径来自岗位快照。
  List<String> _admins = const [];
  Map<String, bool?> _adminVotes = {};
  List<EligibleVoterTicket> _voterTickets = const [];

  List<WalletProfile> _votableWallets = const [];
  WalletProfile? _selectedVoteWallet;
  String? _voteNotice;
  bool _voteNoticeIsError = false;

  @override
  void initState() {
    super.initState();
    _load();
  }

  Future<void> _load({bool showSpinner = true}) async {
    debugPrint('[VoteDetail._load] 开始 proposalId=${widget.proposalId}');
    ProposalDetailSnapshot? localSnapshot;
    if (showSpinner) {
      localSnapshot = await _applyLocalSnapshot();
    }
    // 岗位票据包含 CID + 岗位码，旧的账户级本地详情快照不能作为投票资格真源；
    // 即使缓存新鲜也继续读取链上 VotePlan/VoterSnapshot。

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

      // step1:并行加载合格选民快照、提案状态、投票计数、阈值快照。
      // 机构资格只来自岗位有效选民快照，个人资格来自管理员快照；缺失或损坏
      // 必须失败，禁止回落到当前 admins。
      debugPrint(
          '[VoteDetail._load] step1: 并行 fetchSnapshot/Status/Tally/Threshold...');
      final thresholdFuture = _proposalService
          .fetchInternalThresholdSnapshot(widget.proposalId)
          .catchError((_) => null);
      final results = await Future.wait([
        _proposalService.fetchEligibleVoterTickets(
          widget.proposalId,
          widget.institution,
        ),
        _proposalService.fetchProposalStatus(widget.proposalId),
        _proposalService.fetchVoteTally(widget.proposalId),
        thresholdFuture,
      ]);

      final voterTickets = results[0] as List<EligibleVoterTicket>;
      final admins =
          voterTickets.map((ticket) => ticket.pubkeyHex).toSet().toList();
      final status = results[1] as int?;
      final tally = results[2] as ({int yes, int no});
      final thresholdSnapshot = results[3] as int?;
      final threshold =
          _resolveVoteThreshold(thresholdSnapshot, voterTickets.length);
      debugPrint(
          '[VoteDetail._load] step1 完成 admins.len=${admins.length} status=$status yes=${tally.yes} no=${tally.no} threshold=$threshold');

      // step2:加载提案业务数据（从 ProposalData 解码）
      debugPrint('[VoteDetail._load] step2: fetchProposalData');
      final key = _buildProposalDataStorageKey(widget.proposalId);
      final raw = await rpc.fetchStorage('0x${_hexEncode(key)}');
      debugPrint('[VoteDetail._load] step2 完成 raw.len=${raw?.length ?? 0}');
      personal_models.CreateProposalInfo? createInfo;
      personal_models.CloseProposalInfo? closeInfo;
      institution_models.CloseProposalInfo? institutionCloseInfo;
      if (raw != null && raw.isNotEmpty) {
        final personalDetail = _personalManageService
            .decodePersonalProposalData(widget.proposalId, raw);
        if (personalDetail is personal_models.CreateProposalInfo) {
          createInfo = personalDetail;
        } else if (personalDetail is personal_models.CloseProposalInfo) {
          closeInfo = personalDetail;
        } else {
          final orgDetail =
              _manageService.decodeManageProposalData(widget.proposalId, raw);
          if (orgDetail is institution_models.CloseProposalInfo) {
            if (orgDetail.actorCidNumber != widget.institution.cidNumber) {
              throw StateError('机构关闭提案 actor CID 与当前机构不一致');
            }
            institutionCloseInfo = orgDetail;
          }
        }
      }

      // step3:批量查询每张快照票据的投票记录，避免逐条 RPC。
      debugPrint(
          '[VoteDetail._load] step3: 批量查岗位票据 (${voterTickets.length} 张)');
      final votes = await _proposalService.fetchTicketVotesBatch(
        widget.proposalId,
        voterTickets,
      );
      debugPrint('[VoteDetail._load] step3 完成');

      // 筛选可投票钱包
      final votable = <WalletProfile>[];
      for (final w in widget.adminWallets) {
        var pk = w.pubkeyHex.toLowerCase();
        if (pk.startsWith('0x')) pk = pk.substring(2);
        final walletTickets = voterTickets
            .where((ticket) => _normalizePubkey(ticket.pubkeyHex) == pk);
        if (walletTickets.any((ticket) => votes[ticket.ticketKey] == null)) {
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
          createInfo: createInfo,
          closeInfo: closeInfo,
          institutionCloseInfo: institutionCloseInfo,
        ));
      } catch (_) {
        // 详情快照只是首屏加速，写入失败不能影响链上结果展示。
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
        _voterTickets = voterTickets;
        _votableWallets = votable;
        _selectedVoteWallet = votable.isNotEmpty ? votable.first : null;
        _createInfo = createInfo;
        _closeInfo = closeInfo;
        _institutionCloseInfo = institutionCloseInfo;
        _loading = false;
      });
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
          await _detailStore.read('institution_multisig', widget.proposalId);
      if (snapshot == null || !mounted) return snapshot;
      final admins = snapshot.admins;
      final votable = <WalletProfile>[];
      for (final w in widget.adminWallets) {
        final pk = _normalizePubkey(w.pubkeyHex);
        if (admins.contains(pk) && snapshot.adminVotes[pk] == null) {
          votable.add(w);
        }
      }
      final createInfo = _createInfoFromSnapshot(snapshot);
      final closeInfo = _closeInfoFromSnapshot(snapshot);
      final institutionCloseInfo = _institutionCloseInfoFromSnapshot(snapshot);
      if (createInfo == null &&
          closeInfo == null &&
          institutionCloseInfo == null) {
        return null;
      }
      setState(() {
        _admins = admins;
        _status = snapshot.status;
        _yesCount = snapshot.yesCount;
        _noCount = snapshot.noCount;
        _threshold = snapshot.threshold ?? 0;
        _adminVotes = snapshot.adminVotes;
        _votableWallets = votable;
        _selectedVoteWallet = votable.isNotEmpty ? votable.first : null;
        _createInfo = createInfo;
        _closeInfo = closeInfo;
        _institutionCloseInfo = institutionCloseInfo;
        _loading = false;
        _error = null;
      });
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
    required personal_models.CreateProposalInfo? createInfo,
    required personal_models.CloseProposalInfo? closeInfo,
    required institution_models.CloseProposalInfo? institutionCloseInfo,
  }) {
    return ProposalDetailSnapshot(
      proposalId: widget.proposalId,
      typeKey: 'institution_multisig',
      updatedAtMillis: DateTime.now().millisecondsSinceEpoch,
      status: status,
      yesCount: tally.yes,
      noCount: tally.no,
      threshold: threshold,
      admins: admins.map(_normalizePubkey).toList(growable: false),
      adminVotes: votes.map(
        (key, value) => MapEntry(_normalizePubkey(key), value),
      ),
      pendingPubkeys: const [],
      detail: createInfo != null
          ? _createInfoToJson(createInfo)
          : closeInfo != null
              ? _closeInfoToJson(closeInfo)
              : institutionCloseInfo != null
                  ? _institutionCloseInfoToJson(institutionCloseInfo)
                  : const {},
    );
  }

  Map<String, Object?> _createInfoToJson(
    personal_models.CreateProposalInfo info,
  ) {
    return {
      'kind': 'create',
      'account': info.account,
      'proposer': info.proposer,
      'amount_fen': info.amountFen.toString(),
      'fee_fen': info.feeFen.toString(),
      'status': info.status,
    };
  }

  Map<String, Object?> _closeInfoToJson(
    personal_models.CloseProposalInfo info,
  ) {
    return {
      'kind': 'close',
      'account': info.account,
      'beneficiary': info.beneficiary,
      'proposer': info.proposer,
      'status': info.status,
    };
  }

  Map<String, Object?> _institutionCloseInfoToJson(
    institution_models.CloseProposalInfo info,
  ) {
    return {
      'kind': 'institution_close',
      'actor_cid_number': info.actorCidNumber,
      'institution_account': info.institutionAccount,
      'beneficiary': info.beneficiary,
      'proposer': info.proposer,
      'status': info.status,
    };
  }

  personal_models.CreateProposalInfo? _createInfoFromSnapshot(
    ProposalDetailSnapshot snapshot,
  ) {
    if (!isPersonalAccountIdentity(widget.institution.cidNumber)) return null;
    final detail = snapshot.detail;
    if (detail['kind'] != 'create') return null;
    final amountFen = BigInt.tryParse(detail['amount_fen']?.toString() ?? '');
    final feeFen = BigInt.tryParse(detail['fee_fen']?.toString() ?? '');
    final account = detail['account']?.toString();
    if (amountFen == null || feeFen == null || account == null) {
      return null;
    }
    return personal_models.CreateProposalInfo(
      proposalId: snapshot.proposalId,
      account: account,
      proposer: detail['proposer']?.toString() ?? '',
      amountFen: amountFen,
      feeFen: feeFen,
      status: snapshot.status,
    );
  }

  personal_models.CloseProposalInfo? _closeInfoFromSnapshot(
    ProposalDetailSnapshot snapshot,
  ) {
    if (!isPersonalAccountIdentity(widget.institution.cidNumber)) return null;
    final detail = snapshot.detail;
    if (detail['kind'] != 'close') return null;
    final account = detail['account']?.toString();
    if (account == null) return null;
    return personal_models.CloseProposalInfo(
      proposalId: snapshot.proposalId,
      account: account,
      beneficiary: detail['beneficiary']?.toString() ?? '',
      proposer: detail['proposer']?.toString() ?? '',
      status: snapshot.status,
    );
  }

  institution_models.CloseProposalInfo? _institutionCloseInfoFromSnapshot(
    ProposalDetailSnapshot snapshot,
  ) {
    if (isPersonalAccountIdentity(widget.institution.cidNumber)) return null;
    final detail = snapshot.detail;
    if (detail['kind'] != 'institution_close') return null;
    final actorCidNumber = detail['actor_cid_number']?.toString();
    final institutionAccount = detail['institution_account']?.toString();
    final beneficiary = detail['beneficiary']?.toString();
    final proposer = detail['proposer']?.toString();
    if (actorCidNumber != widget.institution.cidNumber ||
        institutionAccount == null ||
        institutionAccount.length != 64 ||
        beneficiary == null ||
        beneficiary.isEmpty ||
        proposer == null ||
        proposer.isEmpty) {
      return null;
    }
    return institution_models.CloseProposalInfo(
      proposalId: snapshot.proposalId,
      actorCidNumber: actorCidNumber!,
      institutionAccount: institutionAccount,
      beneficiary: beneficiary,
      proposer: proposer,
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

  int _resolveVoteThreshold(int? snapshotThreshold, int adminsLen) {
    if (snapshotThreshold != null && snapshotThreshold > 0) {
      return snapshotThreshold;
    }
    final institutionThreshold = widget.institution.internalThreshold;
    if (institutionThreshold > 0) return institutionThreshold;
    return adminsLen;
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
      final tickets = _voterTickets
          .where((ticket) => _normalizePubkey(ticket.pubkeyHex) == pk);
      if (tickets.any((ticket) => _adminVotes[ticket.ticketKey] == null)) {
        return false;
      }
    }
    return true;
  }

  Future<EligibleVoterTicket?> _selectTicket(
    List<EligibleVoterTicket> tickets,
  ) async {
    if (tickets.length == 1) return tickets.single;
    return showDialog<EligibleVoterTicket>(
      context: context,
      builder: (dialogContext) => SimpleDialog(
        title: const Text('选择本次投票岗位'),
        children: tickets
            .map((ticket) => SimpleDialogOption(
                  onPressed: () => Navigator.pop(dialogContext, ticket),
                  child: Text(ticket.voterRoleCode ?? '个人多签管理员'),
                ))
            .toList(growable: false),
      ),
    );
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
      final availableTickets = _voterTickets
          .where((ticket) =>
              _normalizePubkey(ticket.pubkeyHex) == pubkey &&
              _adminVotes[ticket.ticketKey] == null)
          .toList(growable: false);
      if (availableTickets.isEmpty) {
        throw StateError('当前钱包不在该提案的合格选民快照中，不能投票');
      }
      final ticket = await _selectTicket(availableTickets);
      if (ticket == null) throw StateError('已取消选择投票岗位');
      final balance = await ChainRpc().fetchFinalizedBalance(pubkey);
      if (balance <= 0) {
        throw StateError('当前投票钱包余额不足，无法支付链上投票手续费');
      }

      // 热钱包：先认证，后续用本地签名；冷钱包：走 QR 签名。
      WalletManager? hotWalletManager;
      if (wallet.isHotWallet) {
        hotWalletManager = WalletManager();
      }

      Future<Uint8List> signCallback(Uint8List payload) async {
        if (hotWalletManager != null) {
          return await hotWalletManager.signWithWallet(
              wallet.walletIndex, payload);
        }
        // 冷钱包 QR 签名
        final qrSigner = QrSigner();
        final request = qrSigner.buildRequest(
          requestId: QrSigner.generateRequestId(prefix: 'vote-'),
          pubkey: '0x${wallet.pubkeyHex}',
          payloadHex: '0x${_toHex(payload)}',
          action: QrActions.internalVote,
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
        return Uint8List.fromList(_hexDecode(response.body.signatureHex));
      }

      // 创建/关闭多签的投票都走 InternalVote::cast(20.0),
      // 由 runtime 的 InternalVoteExecutor 按 MODULE_TAG+ACTION 分派。
      debugPrint('[VoteDetail] 调 InternalVoteService.submit');
      final result = await InternalVoteService().submit(
        proposalId: widget.proposalId,
        approve: approve,
        actorCidNumber: ticket.cidNumber,
        voterRoleCode: ticket.voterRoleCode,
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

      if (!mounted) return;
      setState(() {
        _adminVotes[ticket.ticketKey] = approve;
        _votableWallets = _votableWallets.where((w) {
          final walletPk = _normalizePubkey(w.pubkeyHex);
          return _voterTickets.any((candidate) =>
              _normalizePubkey(candidate.pubkeyHex) == walletPk &&
              _adminVotes[candidate.ticketKey] == null);
        }).toList(growable: false);
        _selectedVoteWallet =
            _votableWallets.isNotEmpty ? _votableWallets.first : null;
        _voteNotice = '链上已确认该合格选民投票。';
        _voteNoticeIsError = false;
      });
      ScaffoldMessenger.of(context).showSnackBar(
        SnackBar(
          content: Text('投票已由 runtime 确认：${_truncateAddress(result.txHash)}'),
          backgroundColor: AppTheme.primaryDark,
        ),
      );

      _adminService.clearCache(_accountIdentity);
      // 服务层已经等待入块并回读 InternalVote storage；这里
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
            voterTickets: _voterTickets,
            adminVotes: _adminVotes,
            pendingPubkeys: const {},
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
            if (_institutionCloseInfo != null)
              ..._buildInstitutionCloseInfoRows(),
          ],
        ),
      ),
    );
  }

  List<Widget> _buildCreateInfoRows() {
    final info = _createInfo!;
    final accountSs58 = Keyring().encodeAddress(_hexDecode(info.account), 2027);
    return [
      _buildInfoRow('多签账户', _truncateAddress(accountSs58), onCopy: () {
        Clipboard.setData(ClipboardData(text: accountSs58));
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
    final accountSs58 = Keyring().encodeAddress(_hexDecode(info.account), 2027);
    return [
      _buildInfoRow('多签账户', _truncateAddress(accountSs58), onCopy: () {
        Clipboard.setData(ClipboardData(text: accountSs58));
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

  List<Widget> _buildInstitutionCloseInfoRows() {
    final info = _institutionCloseInfo!;
    final accountSs58 =
        Keyring().encodeAddress(_hexDecode(info.institutionAccount), 2027);
    return [
      _buildInfoRow('机构 CID', info.actorCidNumber),
      const Divider(height: 20),
      _buildInfoRow('机构账户', _truncateAddress(accountSs58), onCopy: () {
        Clipboard.setData(ClipboardData(text: accountSs58));
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
      _buildInfoRow('发起管理员', _truncateAddress(info.proposer)),
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
