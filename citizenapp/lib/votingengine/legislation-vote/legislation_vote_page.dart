import 'dart:async';
import 'dart:typed_data';

import 'package:flutter/material.dart';

import 'package:citizenapp/qr/pages/qr_sign_session_page.dart';
import 'package:citizenapp/qr/qr_protocols.dart';
import 'package:citizenapp/rpc/chain_rpc.dart';
import 'package:citizenapp/signer/qr_signer.dart';
import 'package:citizenapp/ui/app_theme.dart';
import 'package:citizenapp/votingengine/internal-vote/proposal_vote_widgets.dart';
import 'package:citizenapp/votingengine/legislation-vote/legislation_vote_query_service.dart';
import 'package:citizenapp/votingengine/legislation-vote/legislation_vote_service.dart';
import 'package:citizenapp/wallet/core/wallet_manager.dart';

/// 立法提案表决页(LegislationVote sub-pallet)。
///
/// 按提案当前阶段渲染对应动作——代表机构表决 / 行政签署 / 三人会签 /
/// 护宪终审,均为纯 extrinsic(signer=origin),走标准交易签名 + 冷钱包 QR。
/// 特别案公投(referendum 阶段)走公民 CID 凭证流程(citizen 投票入口),本页只提示。
/// 发起立法不在手机端(见 legislation_intro_page,类B)。
class LegislationVotePage extends StatefulWidget {
  const LegislationVotePage({
    super.key,
    required this.proposalId,
    required this.adminWallets,
    this.voteService,
    this.queryService,
  });

  final int proposalId;

  /// 当前公民登录态下可用于签名的管理员钱包(由上层注入)。
  final List<WalletProfile> adminWallets;

  final LegislationVoteService? voteService;
  final LegislationVoteQueryService? queryService;

  @override
  State<LegislationVotePage> createState() => _LegislationVotePageState();
}

class _LegislationVotePageState extends State<LegislationVotePage> {
  late final LegislationVoteService _vote =
      widget.voteService ?? LegislationVoteService();
  late final LegislationVoteQueryService _query =
      widget.queryService ?? LegislationVoteQueryService();

  LegProposalState? _state;
  LegRepresentativeMeta? _representativeMeta;
  LegislationMeta? _legislationMeta;
  ({int yes, int no}) _representativeTally = (yes: 0, no: 0);
  ({int yes, int no}) _referendumTally = (yes: 0, no: 0);

  List<WalletProfile> _votableWallets = const [];
  WalletProfile? _selectedWallet;
  bool _loading = true;
  bool _submitting = false;
  String? _error;

  @override
  void initState() {
    super.initState();
    _load();
  }

  Future<void> _load({bool showSpinner = true}) async {
    if (showSpinner && mounted) setState(() => _loading = true);
    try {
      final state = await _query.fetchProposalState(widget.proposalId);
      final representativeMeta =
          await _query.fetchRepresentativeMeta(widget.proposalId);
      final legislationMeta =
          await _query.fetchLegislationMeta(widget.proposalId);
      final representativeTally = representativeMeta == null
          ? (yes: 0, no: 0)
          : await _query.fetchRepresentativeTally(
              widget.proposalId, representativeMeta.currentBody);
      final refTally = representativeMeta?.rule == 2
          ? await _query.fetchReferendumTally(widget.proposalId)
          : (yes: 0, no: 0);

      // 代表机构阶段按 body_index 检查席位票据，其余阶段交由链端校验身份。
      final votable = <WalletProfile>[];
      if (state?.stage == LegStage.representative &&
          representativeMeta != null) {
        for (final w in widget.adminWallets) {
          final voted = await _query.fetchRepresentativeVote(
            widget.proposalId,
            representativeMeta.currentBody,
            _normalize(w.pubkeyHex),
          );
          if (voted == null) votable.add(w);
        }
      } else {
        votable.addAll(widget.adminWallets);
      }

      if (!mounted) return;
      setState(() {
        _state = state;
        _representativeMeta = representativeMeta;
        _legislationMeta = legislationMeta;
        _representativeTally = representativeTally;
        _referendumTally = refTally;
        _votableWallets = votable;
        _selectedWallet = votable.isNotEmpty ? votable.first : null;
        _loading = false;
        _error = null;
      });
    } on Object catch (e) {
      if (mounted) {
        setState(() {
          _loading = false;
          _error = '提案读取失败：$e';
        });
      }
    }
  }

  bool get _isVotingStage =>
      _state?.status == LegProposalStatus.voting &&
      _state?.stage != LegStage.referendum;

  bool get _canAct => _isVotingStage && !_submitting && _selectedWallet != null;

  Future<void> _act(bool approve) async {
    final wallet = _selectedWallet;
    final stage = _state?.stage;
    if (wallet == null || stage == null) return;
    setState(() => _submitting = true);
    try {
      final pubkeyBytes = _hexDecode(wallet.pubkeyHex);
      final balance = await ChainRpc()
          .fetchFinalizedBalance(_normalize0x(wallet.pubkeyHex));
      if (balance <= 0) {
        throw StateError('当前钱包余额不足，无法支付链上手续费');
      }

      WalletManager? hot;
      if (wallet.isHotWallet) {
        hot = WalletManager();
      }
      Future<Uint8List> signCallback(Uint8List payload) async {
        if (hot != null) {
          return hot.signWithWallet(wallet.walletIndex, payload);
        }
        final qrSigner = QrSigner();
        final request = qrSigner.buildRequest(
          requestId: QrSigner.generateRequestId(prefix: 'leg-'),
          pubkey: '0x${wallet.pubkeyHex}',
          payloadHex: '0x${_toHex(payload)}',
          action: _qrAction(stage),
        );
        final requestJson = qrSigner.encodeRequest(request);
        if (!mounted) throw Exception('页面已关闭');
        final response = await Navigator.push<SignResponseEnvelope>(
          context,
          MaterialPageRoute(
            builder: (_) => QrSignSessionPage(
              request: request,
              requestJson: requestJson,
              expectedPubkey: '0x${wallet.pubkeyHex}',
            ),
          ),
        );
        if (response == null) throw Exception('签名已取消');
        return Uint8List.fromList(_hexDecode(response.body.signatureHex));
      }

      await _dispatch(
        stage: stage,
        approve: approve,
        wallet: wallet,
        pubkeyBytes: pubkeyBytes,
        sign: signCallback,
      );

      if (!mounted) return;
      ScaffoldMessenger.of(context).showSnackBar(
        const SnackBar(content: Text('提交成功')),
      );
      await _load(showSpinner: false);
    } on Object catch (e) {
      if (mounted) {
        ScaffoldMessenger.of(context).showSnackBar(
          SnackBar(content: Text('提交失败：$e')),
        );
      }
    } finally {
      if (mounted) setState(() => _submitting = false);
    }
  }

  Future<void> _dispatch({
    required int stage,
    required bool approve,
    required WalletProfile wallet,
    required Uint8List pubkeyBytes,
    required Future<Uint8List> Function(Uint8List) sign,
  }) {
    final common = (
      proposalId: widget.proposalId,
      approve: approve,
      fromAddress: wallet.address,
      signerPubkey: Uint8List.fromList(pubkeyBytes),
      sign: sign,
    );
    switch (stage) {
      case LegStage.representative:
        return _vote.castRepresentativeVote(
          proposalId: common.proposalId,
          approve: common.approve,
          fromAddress: common.fromAddress,
          signerPubkey: common.signerPubkey,
          sign: common.sign,
        );
      case LegStage.sign:
        return _vote.executiveSign(
          proposalId: common.proposalId,
          approve: common.approve,
          fromAddress: common.fromAddress,
          signerPubkey: common.signerPubkey,
          sign: common.sign,
        );
      case LegStage.override_:
        return _vote.overrideSign(
          proposalId: common.proposalId,
          approve: common.approve,
          fromAddress: common.fromAddress,
          signerPubkey: common.signerPubkey,
          sign: common.sign,
        );
      case LegStage.guard:
        return _vote.guardVote(
          proposalId: common.proposalId,
          approve: common.approve,
          fromAddress: common.fromAddress,
          signerPubkey: common.signerPubkey,
          sign: common.sign,
        );
      default:
        return Future<void>.error(StateError('当前阶段不支持本端操作'));
    }
  }

  int _qrAction(int stage) => switch (stage) {
        LegStage.representative => QrActions.legislationRepresentativeVote,
        LegStage.sign => QrActions.legislationExecutiveSign,
        LegStage.override_ => QrActions.legislationOverrideSign,
        LegStage.guard => QrActions.legislationGuardVote,
        _ => 0,
      };

  String _stageLabel(int stage) => switch (stage) {
        LegStage.representative => '代表机构表决',
        LegStage.referendum => '特别案公投',
        LegStage.sign => '行政首长签署',
        LegStage.override_ => '三人会签',
        LegStage.guard => '护宪大法官终审',
        _ => '未知阶段',
      };

  @override
  Widget build(BuildContext context) {
    return Scaffold(
      backgroundColor: AppTheme.scaffoldBg,
      appBar: AppBar(
        title: Text('立法提案 #${widget.proposalId}'),
        backgroundColor: AppTheme.surfaceCard,
        foregroundColor: AppTheme.textPrimary,
        elevation: 0,
      ),
      body: _buildBody(),
      bottomNavigationBar: _isVotingStage && _qrAction(_state!.stage) != 0
          ? ProposalVoteActions(
              votableWallets: _votableWallets,
              selectedWallet: _selectedWallet,
              submitting: _submitting,
              canVote: _canAct,
              allVoted: _votableWallets.isEmpty,
              onWalletChanged: (w) => setState(() => _selectedWallet = w),
              onVote: _act,
            )
          : null,
    );
  }

  Widget _buildBody() {
    if (_loading) {
      return const Center(child: CircularProgressIndicator(strokeWidth: 2));
    }
    final state = _state;
    if (_error != null || state == null) {
      return Center(
        child: Text(_error ?? '提案不存在',
            style: const TextStyle(color: AppTheme.textTertiary)),
      );
    }
    final representativeMeta = _representativeMeta;
    final legislationMeta = _legislationMeta;
    final isReferendum = state.stage == LegStage.referendum;
    return ListView(
      padding: const EdgeInsets.all(16),
      children: [
        ProposalStatusBadge(
            status: state.status, proposalId: widget.proposalId),
        const SizedBox(height: 16),
        _infoCard(state, representativeMeta, legislationMeta),
        const SizedBox(height: 12),
        _tallyCard(isReferendum),
        if (isReferendum) ...[
          const SizedBox(height: 12),
          _referendumNote(),
        ],
      ],
    );
  }

  Widget _infoCard(
    LegProposalState state,
    LegRepresentativeMeta? representativeMeta,
    LegislationMeta? legislationMeta,
  ) {
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
            _kv('当前阶段', _stageLabel(state.stage)),
            if (representativeMeta != null) ...[
              const SizedBox(height: 6),
              _kv('表决规则', _representativeRuleLabel(representativeMeta.rule)),
              const SizedBox(height: 6),
              _kv('代表机构',
                  representativeMeta.bodies.map((b) => b.code).join(' → ')),
              if (legislationMeta?.needsGuard == true) ...[
                const SizedBox(height: 6),
                _kv('修宪', '通过后需护宪大法官终审'),
              ],
            ],
          ],
        ),
      ),
    );
  }

  Widget _tallyCard(bool isReferendum) {
    final t = isReferendum ? _referendumTally : _representativeTally;
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
          mainAxisAlignment: MainAxisAlignment.spaceBetween,
          children: [
            const Text('当前计票',
                style: TextStyle(
                    fontSize: 15,
                    fontWeight: FontWeight.w700,
                    color: AppTheme.primaryDark)),
            Text('赞成 ${t.yes}　反对 ${t.no}',
                style:
                    const TextStyle(fontSize: 14, fontWeight: FontWeight.w600)),
          ],
        ),
      ),
    );
  }

  Widget _referendumNote() {
    return Container(
      padding: const EdgeInsets.all(14),
      decoration: BoxDecoration(
        color: AppTheme.info.withValues(alpha: 0.08),
        borderRadius: BorderRadius.circular(12),
        border: Border.all(color: AppTheme.info.withValues(alpha: 0.18)),
      ),
      child: const Text(
        '特别案需立法公投表决。请在「立法投票」入口凭 CID 资格参与，本页仅展示进度。',
        style:
            TextStyle(fontSize: 13, height: 1.5, color: AppTheme.textSecondary),
      ),
    );
  }

  Widget _kv(String k, String v) {
    return Row(
      crossAxisAlignment: CrossAxisAlignment.start,
      children: [
        SizedBox(
          width: 72,
          child: Text(k,
              style:
                  const TextStyle(fontSize: 13, color: AppTheme.textTertiary)),
        ),
        Expanded(
          child: Text(v,
              style: const TextStyle(
                  fontSize: 13,
                  fontWeight: FontWeight.w600,
                  color: AppTheme.textPrimary)),
        ),
      ],
    );
  }

  static String _representativeRuleLabel(int rule) => switch (rule) {
        0 => '常规规则',
        1 => '重要规则',
        2 => '特别规则',
        _ => '未知',
      };

  // ──── 工具 ────

  String _normalize(String hex) =>
      hex.startsWith('0x') ? hex.substring(2).toLowerCase() : hex.toLowerCase();

  String _normalize0x(String hex) =>
      hex.startsWith('0x') ? hex.toLowerCase() : '0x${hex.toLowerCase()}';

  Uint8List _hexDecode(String hex) {
    final h = _normalize(hex);
    final out = Uint8List(h.length ~/ 2);
    for (var i = 0; i < out.length; i++) {
      out[i] = int.parse(h.substring(i * 2, i * 2 + 2), radix: 16);
    }
    return out;
  }

  String _toHex(Uint8List b) =>
      b.map((x) => x.toRadixString(16).padLeft(2, '0')).join();
}
