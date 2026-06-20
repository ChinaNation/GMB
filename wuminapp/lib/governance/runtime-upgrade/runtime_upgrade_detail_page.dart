import 'dart:async';

import 'package:flutter/material.dart';
import 'package:wuminapp_mobile/ui/app_theme.dart';
import 'package:flutter/services.dart';
import 'package:polkadart_keyring/polkadart_keyring.dart' show Keyring;

import 'package:wuminapp_mobile/governance/admins-change/models/admin_account.dart';
import 'package:wuminapp_mobile/governance/admins-change/services/institution_admin_service.dart';
import 'package:wuminapp_mobile/governance/shared/institution_info.dart';
import 'package:wuminapp_mobile/governance/organization-manage/institution_registry.dart';
import 'package:wuminapp_mobile/votingengine/internal-vote/pending_vote_store.dart';
import 'package:wuminapp_mobile/governance/shared/proposal/proposal_context.dart';
import 'package:wuminapp_mobile/governance/shared/proposal/proposal_detail_local_store.dart';
import 'package:wuminapp_mobile/governance/runtime-upgrade/runtime_upgrade_service.dart';
import 'package:wuminapp_mobile/governance/shared/proposal/proposal_models.dart';
import 'package:wuminapp_mobile/qr/pages/qr_sign_session_page.dart';
import 'package:wuminapp_mobile/rpc/onchain.dart';
import 'package:wuminapp_mobile/rpc/smoldot_client.dart';
import 'package:wuminapp_mobile/qr/bodies/sign_request_body.dart';
import 'package:wuminapp_mobile/signer/qr_signer.dart';
import 'package:wuminapp_mobile/wallet/core/wallet_manager.dart';
import 'package:wuminapp_mobile/votingengine/internal-vote/proposal_vote_widgets.dart';

/// 协议升级提案详情页。
///
/// 从全链提案页进入时为只读模式；
/// 从机构详情页进入时，当前机构管理员可直接提交联合投票。
class RuntimeUpgradeDetailPage extends StatefulWidget {
  const RuntimeUpgradeDetailPage({
    super.key,
    required this.proposalId,
    required this.proposalContext,
  });

  final int proposalId;

  /// 统一的提案上下文（包含机构信息和管理员钱包）。
  final ProposalContext proposalContext;

  /// 便捷访问。
  InstitutionInfo? get institution => proposalContext.institution;
  List<WalletProfile> get adminWallets => proposalContext.adminWallets;

  @override
  State<RuntimeUpgradeDetailPage> createState() =>
      _RuntimeUpgradeDetailPageState();
}

class _RuntimeUpgradeDetailPageState extends State<RuntimeUpgradeDetailPage> {
  final RuntimeUpgradeService _service = RuntimeUpgradeService();
  final InstitutionAdminService _adminService = InstitutionAdminService();
  final ProposalDetailLocalStore _detailStore =
      ProposalDetailLocalStore.instance;

  bool _loading = true;
  bool _submitting = false;
  String? _error;

  RuntimeUpgradeProposalInfo? _proposalInfo;
  ProposalMeta? _meta;
  ({int yes, int no}) _jointTally = (yes: 0, no: 0);
  ({int yes, int no}) _referendumTally = (yes: 0, no: 0);
  bool _reasonExpanded = false;

  bool? _institutionVote;
  List<String> _admins = const [];
  ({int yes, int no}) _institutionAdminTally = (yes: 0, no: 0);
  Map<String, bool?> _adminVotes = const {};
  List<WalletProfile> _votableWallets = const [];
  WalletProfile? _selectedVoteWallet;

  // 已提交投票但尚未上链确认的管理员公钥集合
  Set<String> _pendingPubkeys = const {};

  @override
  void initState() {
    super.initState();
    _load();
  }

  bool get _isAdmin => widget.proposalContext.isAdmin;

  int get _requiredAdminThreshold => widget.institution?.internalThreshold ?? 0;

  bool get _jointVoteOpen =>
      (_meta?.status == 0) && (_meta?.stage == 1) && _resolvedStatusCode() == 0;

  bool get _canSubmitVote =>
      _isAdmin &&
      _jointVoteOpen &&
      _institutionVote == null &&
      _selectedVoteWallet != null &&
      !_submitting;

  bool get _allImportedAdminsVoted {
    if (!_isAdmin) return false;
    for (final wallet in widget.adminWallets) {
      final pk = _normalizeHex(wallet.pubkeyHex);
      final vote = _adminVotes[pk];
      if (vote == null && !_pendingPubkeys.contains(pk)) return false;
    }
    return true;
  }

  String? get _voteDisabledReason {
    if (!_isAdmin) return null;
    if (!_jointVoteOpen) return '当前提案不在联合投票阶段';
    if (_institutionVote != null) return '本机构已形成最终投票结果';
    if (_votableWallets.isEmpty && _allImportedAdminsVoted) {
      return '已导入的管理员钱包都已完成投票';
    }
    if (_votableWallets.isEmpty) return '当前没有可用的管理员钱包';
    if (_selectedVoteWallet == null) return '请选择用于投票的管理员钱包';
    return null;
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
      final futures = <Future<dynamic>>[
        _service.fetchProposalMeta(widget.proposalId),
        _service.fetchRuntimeUpgradeProposal(widget.proposalId),
        _service.fetchJointTally(widget.proposalId),
        _service.fetchReferendumTally(widget.proposalId),
      ];

      final institution = widget.institution;
      if (institution != null) {
        final institutionAccountId = _institutionAccountId(institution);
        futures.add(_adminService.fetchAdmins(
          AdminAccountIdentity.fromInstitution(institution),
        ));
        futures.add(_service.fetchJointVoteByInstitution(
            widget.proposalId, institutionAccountId));
        futures.add(_service.fetchJointInstitutionTally(
            widget.proposalId, institutionAccountId));
      }

      final results = await Future.wait(futures);
      final meta = results[0] as ProposalMeta?;
      final proposalInfo = results[1] as RuntimeUpgradeProposalInfo?;
      final jointTally = results[2] as ({int yes, int no});
      final referendumTally = results[3] as ({int yes, int no});

      List<String> admins = const [];
      bool? institutionVote;
      ({int yes, int no}) institutionAdminTally = (yes: 0, no: 0);
      Map<String, bool?> adminVotes = const {};
      List<WalletProfile> votableWallets = const [];
      WalletProfile? selectedVoteWallet = _selectedVoteWallet;
      Set<String> pendingPubkeys = const {};

      if (institution != null) {
        admins = (results[4] as List<String>)
            .map((pubkey) => _normalizeHex(pubkey))
            .toList(growable: false);
        institutionVote = results[5] as bool?;
        institutionAdminTally = results[6] as ({int yes, int no});
        final adminSet = admins.toSet();
        final matchedAdminWallets = widget.adminWallets.where((wallet) {
          return adminSet.contains(_normalizeHex(wallet.pubkeyHex));
        }).toList(growable: false)
          ..sort((a, b) => a.walletIndex.compareTo(b.walletIndex));

        final institutionBytes = _institutionAccountId(institution);
        final voteResults = await _service.fetchJointAdminVotesBatch(
          widget.proposalId,
          institutionBytes,
          admins,
        );
        adminVotes = voteResults;

        // 检查待确认投票。联合投票不读 InternalVote，而是读 JointVote
        // 机构管理员投票记录。
        final pendingSummary =
            await PendingVoteStore.instance.confirmAllDetailed(
          'runtime_upgrade',
          widget.proposalId,
          OnchainRpc(),
          chainVoteLookup: (record) => _service.fetchJointAdminVote(
            record.proposalId,
            institutionBytes,
            record.walletPubkey,
          ),
        );
        for (final confirmed in pendingSummary.confirmed) {
          adminVotes[confirmed.walletPubkey] = confirmed.approve;
        }
        final pendingPks =
            pendingSummary.stillPending.map((r) => r.walletPubkey).toSet();

        votableWallets = matchedAdminWallets.where((wallet) {
          final pk = _normalizeHex(wallet.pubkeyHex);
          return adminVotes[pk] == null && !pendingPks.contains(pk);
        }).toList(growable: false)
          ..sort((a, b) => a.walletIndex.compareTo(b.walletIndex));

        if (selectedVoteWallet == null ||
            !votableWallets.any((wallet) =>
                wallet.walletIndex == selectedVoteWallet!.walletIndex)) {
          selectedVoteWallet =
              votableWallets.isNotEmpty ? votableWallets.first : null;
        }

        pendingPubkeys = pendingPks;
      }

      if (!mounted) return;
      try {
        await _detailStore.put(_snapshotFromChain(
          meta: meta,
          proposalInfo: proposalInfo,
          jointTally: jointTally,
          referendumTally: referendumTally,
          admins: admins,
          adminVotes: adminVotes,
          pendingPubkeys: pendingPubkeys,
          institutionVote: institutionVote,
          institutionAdminTally: institutionAdminTally,
        ));
      } catch (_) {
        // 中文注释：详情快照写入失败不能影响链上最新结果展示。
      }
      if (!mounted) return;
      setState(() {
        _meta = meta;
        _proposalInfo = proposalInfo;
        _jointTally = jointTally;
        _referendumTally = referendumTally;
        _admins = admins;
        _institutionVote = institutionVote;
        _institutionAdminTally = institutionAdminTally;
        _adminVotes = adminVotes;
        _pendingPubkeys = pendingPubkeys;
        _votableWallets = votableWallets;
        _selectedVoteWallet = selectedVoteWallet;
        _loading = false;
      });
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
          await _detailStore.read('runtime_upgrade', widget.proposalId);
      if (snapshot == null || !mounted) return snapshot;
      final admins = snapshot.admins;
      final pendingPks = snapshot.pendingPubkeys.toSet();
      final adminSet = admins.toSet();
      final matchedAdminWallets = widget.adminWallets.where((wallet) {
        return adminSet.contains(_normalizeHex(wallet.pubkeyHex));
      }).toList(growable: false)
        ..sort((a, b) => a.walletIndex.compareTo(b.walletIndex));
      final votableWallets = matchedAdminWallets.where((wallet) {
        final pk = _normalizeHex(wallet.pubkeyHex);
        return snapshot.adminVotes[pk] == null && !pendingPks.contains(pk);
      }).toList(growable: false)
        ..sort((a, b) => a.walletIndex.compareTo(b.walletIndex));
      setState(() {
        _meta = _metaFromSnapshot(snapshot);
        _proposalInfo = _proposalInfoFromSnapshot(snapshot);
        _jointTally = (
          yes: _toInt(snapshot.extra['joint_yes']) ?? snapshot.yesCount,
          no: _toInt(snapshot.extra['joint_no']) ?? snapshot.noCount,
        );
        _referendumTally = (
          yes: _toInt(snapshot.extra['referendum_yes']) ?? 0,
          no: _toInt(snapshot.extra['referendum_no']) ?? 0,
        );
        _admins = admins;
        _institutionVote = _toBool(snapshot.extra['institution_vote']);
        _institutionAdminTally = (
          yes: _toInt(snapshot.extra['institution_yes']) ?? 0,
          no: _toInt(snapshot.extra['institution_no']) ?? 0,
        );
        _adminVotes = snapshot.adminVotes;
        _pendingPubkeys = pendingPks;
        _votableWallets = votableWallets;
        _selectedVoteWallet =
            votableWallets.isNotEmpty ? votableWallets.first : null;
        _loading = false;
        _error = null;
      });
      return snapshot;
    } catch (_) {
      return null;
    }
  }

  ProposalDetailSnapshot _snapshotFromChain({
    required ProposalMeta? meta,
    required RuntimeUpgradeProposalInfo? proposalInfo,
    required ({int yes, int no}) jointTally,
    required ({int yes, int no}) referendumTally,
    required List<String> admins,
    required Map<String, bool?> adminVotes,
    required Set<String> pendingPubkeys,
    required bool? institutionVote,
    required ({int yes, int no}) institutionAdminTally,
  }) {
    return ProposalDetailSnapshot(
      proposalId: widget.proposalId,
      typeKey: 'runtime_upgrade',
      updatedAtMillis: DateTime.now().millisecondsSinceEpoch,
      status: meta?.status,
      yesCount: jointTally.yes,
      noCount: jointTally.no,
      threshold: _requiredAdminThreshold,
      admins: admins.map(_normalizeHex).toList(growable: false),
      adminVotes: adminVotes.map(
        (key, value) => MapEntry(_normalizeHex(key), value),
      ),
      pendingPubkeys: pendingPubkeys.map(_normalizeHex).toList(growable: false),
      detail: _proposalInfoToJson(proposalInfo),
      extra: {
        'meta_kind': meta?.kind,
        'meta_stage': meta?.stage,
        'meta_status': meta?.status,
        'meta_internal_org': meta?.internalOrg,
        'meta_institution_bytes_hex': meta?.institutionBytes == null
            ? null
            : _toHex(meta!.institutionBytes!),
        'joint_yes': jointTally.yes,
        'joint_no': jointTally.no,
        'referendum_yes': referendumTally.yes,
        'referendum_no': referendumTally.no,
        'institution_vote': institutionVote,
        'institution_yes': institutionAdminTally.yes,
        'institution_no': institutionAdminTally.no,
      },
    );
  }

  Map<String, Object?> _proposalInfoToJson(
    RuntimeUpgradeProposalInfo? info,
  ) {
    if (info == null) return const {};
    return {
      'proposer': info.proposer,
      'reason': info.reason,
      'code_hash_hex': info.codeHashHex,
    };
  }

  RuntimeUpgradeProposalInfo? _proposalInfoFromSnapshot(
    ProposalDetailSnapshot snapshot,
  ) {
    final detail = snapshot.detail;
    final codeHash = detail['code_hash_hex']?.toString();
    if (codeHash == null || codeHash.isEmpty) return null;
    return RuntimeUpgradeProposalInfo(
      proposalId: snapshot.proposalId,
      proposer: detail['proposer']?.toString() ?? '',
      reason: detail['reason']?.toString() ?? '',
      codeHashHex: codeHash,
    );
  }

  ProposalMeta? _metaFromSnapshot(ProposalDetailSnapshot snapshot) {
    final kind = _toInt(snapshot.extra['meta_kind']);
    final stage = _toInt(snapshot.extra['meta_stage']);
    final status = _toInt(snapshot.extra['meta_status']);
    if (kind == null || stage == null || status == null) return null;
    final institutionHex =
        snapshot.extra['meta_institution_bytes_hex']?.toString();
    return ProposalMeta(
      proposalId: snapshot.proposalId,
      kind: kind,
      stage: stage,
      status: status,
      internalOrg: _toInt(snapshot.extra['meta_internal_org']),
      institutionBytes: institutionHex == null || institutionHex.isEmpty
          ? null
          : Uint8List.fromList(_hexDecode(institutionHex)),
    );
  }

  int? _toInt(Object? value) {
    if (value == null) return null;
    if (value is int) return value;
    return int.tryParse(value.toString());
  }

  bool? _toBool(Object? value) {
    if (value == null) return null;
    if (value is bool) return value;
    if (value.toString() == 'true') return true;
    if (value.toString() == 'false') return false;
    return null;
  }

  String _normalizeHex(String hex) {
    return hex.startsWith('0x')
        ? hex.substring(2).toLowerCase()
        : hex.toLowerCase();
  }

  Uint8List _institutionAccountId(InstitutionInfo institution) {
    // 中文注释：联合投票 storage 使用机构多签 AccountId，不从 sfid_number 派生主体。
    return Uint8List.fromList(
      institutionIdentityToAccountId(
        institution.sfidNumber,
        mainAccount: institution.mainAccount,
      ),
    );
  }

  Uint8List _hexDecode(String hex) {
    final clean = hex.startsWith('0x') ? hex.substring(2) : hex;
    final out = Uint8List(clean.length ~/ 2);
    for (var i = 0; i < out.length; i++) {
      out[i] = int.parse(clean.substring(i * 2, i * 2 + 2), radix: 16);
    }
    return out;
  }

  String _toHex(List<int> bytes) {
    const chars = '0123456789abcdef';
    final buffer = StringBuffer();
    for (final byte in bytes) {
      buffer
        ..write(chars[(byte >> 4) & 0x0f])
        ..write(chars[byte & 0x0f]);
    }
    return buffer.toString();
  }

  String _truncateAddress(String address) {
    if (address.length <= 14) return address;
    return '${address.substring(0, 6)}...${address.substring(address.length - 6)}';
  }

  String _truncateWalletAddress(String address) {
    if (address.length <= 18) return address;
    return '${address.substring(0, 8)}...${address.substring(address.length - 8)}';
  }

  String _pubkeyToSs58(String pubkeyHex) {
    return Keyring().encodeAddress(_hexDecode(pubkeyHex), 2027);
  }

  Future<Uint8List> _signPayloadWithWallet({
    required WalletProfile wallet,
    required Uint8List payload,
    required String requestPrefix,
    required SignDisplay display,
  }) async {
    // 管理员投票统一通过 QR 码签名（wumin 公民钱包）
    final qrSigner = QrSigner();
    final request = qrSigner.buildRequest(
      requestId: QrSigner.generateRequestId(prefix: '$requestPrefix-'),
      address: wallet.address,
      pubkey: '0x${wallet.pubkeyHex}',
      payloadHex: '0x${_toHex(payload)}',
      display: display,
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
    if (response == null) {
      throw Exception('签名已取消');
    }
    return _hexDecode(response.body.signature);
  }

  Future<void> _submitJointVote(bool approve) async {
    final institution = widget.institution;
    final voteWallet = _selectedVoteWallet;
    if (institution == null || voteWallet == null) return;

    setState(() => _submitting = true);

    try {
      final institutionBytes = _institutionAccountId(institution);
      final result = await _service.submitJointVote(
        proposalId: widget.proposalId,
        institutionAccountId: institutionBytes,
        approve: approve,
        fromAddress: voteWallet.address,
        signerPubkey: _hexDecode(voteWallet.pubkeyHex),
        sign: (payload) {
          final voteText = approve ? '赞成' : '反对';
          return _signPayloadWithWallet(
            wallet: voteWallet,
            payload: payload,
            requestPrefix: approve ? 'runtime-joint-yes' : 'runtime-joint-no',
            display: SignDisplay(
              action: 'joint_vote',
              summary: '联合投票 提案 #${widget.proposalId}：$voteText',
              fields: [
                // joint_vote 当前 decoder 输出 fields = (proposal_id, approve),
                // account_id 在 payload 里但 decoder 跳过不回填 display。
                // _proposalInfo(提案人/理由/代码哈希)属辅助展示,
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
        },
      );

      final pubkey = _normalizeHex(voteWallet.pubkeyHex);
      // 中文注释：服务层已经确认 runtime JointVote 记录，新流程不再写 pending。
      // 这里只清除旧版本可能残留的同管理员 pending 记录。
      await PendingVoteStore.instance.remove(
        'runtime_upgrade',
        widget.proposalId,
        pubkey,
      );

      if (!mounted) return;
      setState(() {
        _adminVotes = {..._adminVotes, pubkey: approve};
        _pendingPubkeys = _pendingPubkeys.difference({pubkey});
        _institutionAdminTally = (
          yes: _institutionAdminTally.yes + (approve ? 1 : 0),
          no: _institutionAdminTally.no + (approve ? 0 : 1),
        );
        _votableWallets = _votableWallets
            .where((w) => _normalizeHex(w.pubkeyHex) != pubkey)
            .toList(growable: false);
        _selectedVoteWallet =
            _votableWallets.isNotEmpty ? _votableWallets.first : null;
      });
      ScaffoldMessenger.of(context).showSnackBar(
        SnackBar(
          content: Text('联合投票已由 runtime 确认：${_truncateAddress(result.txHash)}'),
          backgroundColor: AppTheme.primaryDark,
        ),
      );

      _adminService
          .clearCache(AdminAccountIdentity.fromInstitution(institution));
      // 中文注释：服务层已经等待入块并回读 JointVote storage；这里刷新页面
      // 只负责同步最新展示状态，投票成功与否不再由 txHash 判断。
      unawaited(_load(showSpinner: false));
    } on WalletAuthException catch (e) {
      if (!mounted) return;
      ScaffoldMessenger.of(context).showSnackBar(
        SnackBar(content: Text(e.message), backgroundColor: AppTheme.danger),
      );
    } catch (e) {
      if (!mounted) return;
      ScaffoldMessenger.of(context).showSnackBar(
        SnackBar(content: Text('投票失败：$e'), backgroundColor: AppTheme.danger),
      );
    } finally {
      if (mounted) {
        setState(() => _submitting = false);
      }
    }
  }

  void _confirmVote(bool approve) {
    final label = approve ? '赞成' : '反对';
    showDialog<void>(
      context: context,
      builder: (ctx) => AlertDialog(
        title: Text('确认提交$label票'),
        content: Text(
          '将使用所选管理员钱包直接提交$label票。投票后不可修改。',
        ),
        actions: [
          TextButton(
            onPressed: () => Navigator.of(ctx).pop(),
            child: const Text('取消'),
          ),
          TextButton(
            onPressed: () {
              Navigator.of(ctx).pop();
              _submitJointVote(approve);
            },
            child: Text(label),
          ),
        ],
      ),
    );
  }

  int? _resolvedStatusCode() {
    // 中文注释：协议升级真实状态只以投票引擎元数据为准。
    return _meta?.status;
  }

  String _institutionVoteLabel() {
    if (_institutionVote == null) return '待形成机构结果';
    return _institutionVote! ? '机构已赞成' : '机构已反对';
  }

  @override
  Widget build(BuildContext context) {
    return Scaffold(
      backgroundColor: Colors.white,
      appBar: AppBar(
        title: const Text(
          '协议升级详情',
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
      bottomNavigationBar: _buildBottomBar(),
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
        final institution = widget.institution;
        if (institution != null) {
          _adminService.clearCache(
            AdminAccountIdentity.fromInstitution(institution),
          );
        }
        await _load();
      },
      child: ListView(
        padding: const EdgeInsets.fromLTRB(16, 8, 16, 32),
        children: [
          ProposalStatusBadge(
              status: _meta?.status, proposalId: widget.proposalId),
          const SizedBox(height: 16),
          _buildProposalInfoCard(),
          const SizedBox(height: 16),
          _buildJointVotingProgress(),
          if (_isAdmin) ...[
            const SizedBox(height: 16),
            _buildInstitutionVoteCard(),
          ],
          if (_meta?.stage == 2) ...[
            const SizedBox(height: 16),
            _buildCitizenVotingProgress(),
          ],
        ],
      ),
    );
  }

  Widget _buildProposalInfoCard() {
    final info = _proposalInfo;
    final reason = info?.reason ?? '';

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
            _buildInfoRow(
              '提案 ID',
              formatProposalId(_meta?.displayMeta),
            ),
            if (widget.institution != null) ...[
              const Divider(height: 20),
              _buildInfoRow('当前机构', widget.institution!.name),
            ],
            if (info != null) ...[
              const Divider(height: 20),
              _buildInfoRow(
                '发起人',
                _truncateAddress(info.proposer),
                onCopy: () {
                  Clipboard.setData(ClipboardData(text: info.proposer));
                  ScaffoldMessenger.of(context).showSnackBar(
                    const SnackBar(
                      content: Text('地址已复制'),
                      duration: Duration(seconds: 1),
                    ),
                  );
                },
              ),
              const Divider(height: 20),
              _buildInfoRow(
                'Code Hash',
                _truncateAddress(info.codeHashHex),
                onCopy: () {
                  Clipboard.setData(ClipboardData(text: info.codeHashHex));
                  ScaffoldMessenger.of(context).showSnackBar(
                    const SnackBar(
                      content: Text('Code Hash 已复制'),
                      duration: Duration(seconds: 1),
                    ),
                  );
                },
              ),
              _buildRemarkRow('升级理由', reason),
            ],
          ],
        ),
      ),
    );
  }

  Widget _buildRemarkRow(String label, String text) {
    if (text.isEmpty) {
      return _buildInfoRow(label, '无');
    }
    final isLong = text.length > 30;
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
            text,
            style: const TextStyle(fontSize: 13, color: AppTheme.textPrimary),
            maxLines: _reasonExpanded ? null : 1,
            overflow: _reasonExpanded ? null : TextOverflow.ellipsis,
          ),
        ),
        if (isLong)
          GestureDetector(
            onTap: () => setState(() => _reasonExpanded = !_reasonExpanded),
            child: Icon(
              _reasonExpanded
                  ? Icons.keyboard_arrow_up
                  : Icons.keyboard_arrow_down,
              size: 20,
              color: AppTheme.textTertiary,
            ),
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

  Widget _buildJointVotingProgress() {
    final progress = jointVotePassThreshold > 0
        ? (_jointTally.yes / jointVotePassThreshold).clamp(0.0, 1.0)
        : 0.0;

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
              '联合投票进度',
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
                valueColor:
                    const AlwaysStoppedAnimation<Color>(AppTheme.primaryDark),
              ),
            ),
            const SizedBox(height: 8),
            Row(
              mainAxisAlignment: MainAxisAlignment.spaceBetween,
              children: [
                Text(
                  '赞成 ${_jointTally.yes} / 通过阈值 $jointVotePassThreshold',
                  style: const TextStyle(
                    fontSize: 14,
                    fontWeight: FontWeight.w600,
                    color: AppTheme.primaryDark,
                  ),
                ),
                Text(
                  '反对 ${_jointTally.no}',
                  style: const TextStyle(
                    fontSize: 13,
                    color: AppTheme.danger,
                  ),
                ),
              ],
            ),
            const SizedBox(height: 6),
            Text(
              '联合投票总权重 $jointVoteTotal，国储会权重 19，省储会/省储行各权重 1',
              style:
                  const TextStyle(fontSize: 12, color: AppTheme.textTertiary),
            ),
          ],
        ),
      ),
    );
  }

  Widget _buildInstitutionVoteCard() {
    final institution = widget.institution!;
    final progress = _requiredAdminThreshold > 0
        ? (_institutionAdminTally.yes / _requiredAdminThreshold).clamp(0.0, 1.0)
        : 0.0;
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
              '本机构投票',
              style: TextStyle(
                fontSize: 16,
                fontWeight: FontWeight.w700,
                color: AppTheme.primaryDark,
              ),
            ),
            const SizedBox(height: 12),
            _buildInfoRow('机构名称', institution.name),
            const Divider(height: 20),
            _buildInfoRow('投票状态', _institutionVoteLabel()),
            const Divider(height: 20),
            Text(
              '管理员赞成 ${_institutionAdminTally.yes} / $_requiredAdminThreshold',
              style: const TextStyle(
                fontSize: 14,
                fontWeight: FontWeight.w600,
                color: AppTheme.primaryDark,
              ),
            ),
            ClipRRect(
              borderRadius: BorderRadius.circular(6),
              child: LinearProgressIndicator(
                value: progress,
                minHeight: 8,
                backgroundColor: AppTheme.border,
                valueColor: AlwaysStoppedAnimation<Color>(
                    _institutionVote == true
                        ? AppTheme.primaryDark
                        : AppTheme.warning),
              ),
            ),
            const SizedBox(height: 12),
            Row(
              mainAxisAlignment: MainAxisAlignment.spaceBetween,
              children: [
                Text(
                  '管理员反对 ${_institutionAdminTally.no}',
                  style: const TextStyle(fontSize: 13, color: AppTheme.danger),
                ),
                Text(
                  '链上当前管理员 ${_admins.length} 人',
                  style: const TextStyle(
                      fontSize: 12, color: AppTheme.textTertiary),
                ),
              ],
            ),
            const Divider(height: 20),
            _buildVoteWalletSelector(),
            if (_admins.isNotEmpty) ...[
              const SizedBox(height: 8),
              const Text(
                '本机构管理员直接上链投票，赞成达到阈值会自动形成机构赞成结果；若剩余管理员已不足以达到阈值，链上会自动形成机构反对结果。',
                style: TextStyle(fontSize: 12, color: AppTheme.textTertiary),
              ),
            ],
          ],
        ),
      ),
    );
  }

  Widget _buildVoteWalletSelector() {
    if (!_isAdmin) {
      return Container(
        width: double.infinity,
        padding: const EdgeInsets.all(12),
        decoration: BoxDecoration(
          color: AppTheme.warning.withValues(alpha: 0.08),
          borderRadius: BorderRadius.circular(8),
        ),
        child: const Text(
          '当前未导入属于本机构的管理员钱包',
          style: TextStyle(fontSize: 13, color: AppTheme.warning),
        ),
      );
    }

    if (_votableWallets.isEmpty) {
      return Container(
        width: double.infinity,
        padding: const EdgeInsets.all(12),
        decoration: BoxDecoration(
          color: AppTheme.surfaceMuted,
          borderRadius: BorderRadius.circular(8),
        ),
        child: Text(
          _allImportedAdminsVoted ? '已导入管理员钱包均已完成投票' : '当前没有可用的管理员钱包',
          style: const TextStyle(fontSize: 13, color: AppTheme.textSecondary),
        ),
      );
    }

    if (_votableWallets.length == 1) {
      final wallet = _votableWallets.first;
      return ListTile(
        contentPadding: EdgeInsets.zero,
        title: Text(
          _truncateWalletAddress(wallet.address),
          style: const TextStyle(fontSize: 13),
        ),
        subtitle: Text(
          _pubkeyToSs58(wallet.pubkeyHex),
          maxLines: 1,
          overflow: TextOverflow.ellipsis,
          style: const TextStyle(fontSize: 11, color: AppTheme.textTertiary),
        ),
        trailing: const Icon(Icons.shield_outlined,
            size: 18, color: AppTheme.warning),
      );
    }

    return Container(
      padding: const EdgeInsets.symmetric(horizontal: 12),
      decoration: BoxDecoration(
        color: AppTheme.success.withValues(alpha: 0.05),
        borderRadius: BorderRadius.circular(8),
        border: Border.all(color: AppTheme.success.withValues(alpha: 0.2)),
      ),
      child: DropdownButtonHideUnderline(
        child: DropdownButton<int>(
          value: _selectedVoteWallet?.walletIndex,
          isExpanded: true,
          items: _votableWallets.map((wallet) {
            return DropdownMenuItem<int>(
              value: wallet.walletIndex,
              child: Row(
                children: [
                  Expanded(
                    child: Text(
                      _truncateWalletAddress(wallet.address),
                      style: const TextStyle(fontSize: 13),
                      overflow: TextOverflow.ellipsis,
                    ),
                  ),
                  const SizedBox(width: 8),
                  const Icon(Icons.shield_outlined,
                      size: 18, color: AppTheme.warning),
                ],
              ),
            );
          }).toList(),
          onChanged: (walletIndex) {
            if (walletIndex == null) return;
            setState(() {
              _selectedVoteWallet = _votableWallets
                  .firstWhere((wallet) => wallet.walletIndex == walletIndex);
            });
          },
        ),
      ),
    );
  }

  Widget _buildCitizenVotingProgress() {
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
              '联合公投进度',
              style: TextStyle(
                fontSize: 16,
                fontWeight: FontWeight.w700,
                color: AppTheme.primaryDark,
              ),
            ),
            const SizedBox(height: 12),
            Row(
              mainAxisAlignment: MainAxisAlignment.spaceBetween,
              children: [
                Text(
                  '赞成 ${_referendumTally.yes}',
                  style: const TextStyle(
                    fontSize: 14,
                    fontWeight: FontWeight.w600,
                    color: AppTheme.primaryDark,
                  ),
                ),
                Text(
                  '反对 ${_referendumTally.no}',
                  style: const TextStyle(
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

  /// 联合公投阶段判断
  bool get _citizenVoteOpen =>
      (_meta?.status == 0) && (_meta?.stage == 2) && _resolvedStatusCode() == 0;

  Widget? _buildBottomBar() {
    if (_loading || _error != null) return null;
    // 联合投票阶段：仅管理员显示投票按钮
    if (_isAdmin && _jointVoteOpen) {
      return _buildVoteButtons();
    }
    // 联合公投阶段：所有用户显示投票按钮（SFID 绑定校验后续完善）
    if (_citizenVoteOpen) {
      return _buildCitizenVoteButtons();
    }
    // 非投票阶段但是管理员：显示禁用状态的投票按钮
    if (_isAdmin) {
      return _buildVoteButtons();
    }
    return null;
  }

  Widget _buildCitizenVoteButtons() {
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
          const Padding(
            padding: EdgeInsets.only(bottom: 10),
            child: Text(
              '联合公投',
              style: TextStyle(fontSize: 13, color: AppTheme.textSecondary),
              textAlign: TextAlign.center,
            ),
          ),
          Row(
            children: [
              Expanded(
                child: ElevatedButton(
                  onPressed:
                      _submitting ? null : () => _confirmCitizenVote(false),
                  style: ElevatedButton.styleFrom(
                    backgroundColor: AppTheme.danger,
                    foregroundColor: Colors.white,
                    disabledBackgroundColor:
                        AppTheme.danger.withValues(alpha: 0.25),
                    padding: const EdgeInsets.symmetric(vertical: 14),
                    shape: RoundedRectangleBorder(
                      borderRadius: BorderRadius.circular(10),
                    ),
                  ),
                  child: _submitting
                      ? const SizedBox(
                          width: 18,
                          height: 18,
                          child: CircularProgressIndicator(
                            strokeWidth: 2,
                            color: Colors.white,
                          ),
                        )
                      : const Text('反对'),
                ),
              ),
              const SizedBox(width: 12),
              Expanded(
                child: ElevatedButton(
                  onPressed:
                      _submitting ? null : () => _confirmCitizenVote(true),
                  style: ElevatedButton.styleFrom(
                    backgroundColor: AppTheme.success,
                    foregroundColor: Colors.white,
                    disabledBackgroundColor:
                        AppTheme.success.withValues(alpha: 0.25),
                    padding: const EdgeInsets.symmetric(vertical: 14),
                    shape: RoundedRectangleBorder(
                      borderRadius: BorderRadius.circular(10),
                    ),
                  ),
                  child: _submitting
                      ? const SizedBox(
                          width: 18,
                          height: 18,
                          child: CircularProgressIndicator(
                            strokeWidth: 2,
                            color: Colors.white,
                          ),
                        )
                      : const Text('赞成'),
                ),
              ),
            ],
          ),
        ],
      ),
    );
  }

  void _confirmCitizenVote(bool approve) {
    final label = approve ? '赞成' : '反对';
    showDialog<void>(
      context: context,
      builder: (ctx) => AlertDialog(
        title: Text('确认联合公投$label'),
        content: Text(
          '将对此协议升级提案投"$label"票。投票后不可修改。',
        ),
        actions: [
          TextButton(
            onPressed: () => Navigator.of(ctx).pop(),
            child: const Text('取消'),
          ),
          TextButton(
            onPressed: () {
              Navigator.of(ctx).pop();
              _submitCitizenVote(approve);
            },
            child: Text(label),
          ),
        ],
      ),
    );
  }

  Future<void> _submitCitizenVote(bool approve) async {
    // 中文注释：联合公投提交依赖链上 cast_referendum extrinsic，入口未开放前只提示状态。
    if (!mounted) return;
    ScaffoldMessenger.of(context).showSnackBar(
      const SnackBar(
        content: Text('公民投票功能开发中'),
        backgroundColor: AppTheme.warning,
      ),
    );
  }

  Widget _buildVoteButtons() {
    final disabledReason = _voteDisabledReason;
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
          if (disabledReason != null)
            Padding(
              padding: const EdgeInsets.only(bottom: 10),
              child: Text(
                disabledReason,
                style:
                    const TextStyle(fontSize: 13, color: AppTheme.textTertiary),
                textAlign: TextAlign.center,
              ),
            ),
          Row(
            children: [
              Expanded(
                child: ElevatedButton(
                  onPressed: _canSubmitVote ? () => _confirmVote(false) : null,
                  style: ElevatedButton.styleFrom(
                    backgroundColor: AppTheme.danger,
                    foregroundColor: Colors.white,
                    disabledBackgroundColor:
                        AppTheme.danger.withValues(alpha: 0.25),
                    padding: const EdgeInsets.symmetric(vertical: 14),
                    shape: RoundedRectangleBorder(
                      borderRadius: BorderRadius.circular(10),
                    ),
                  ),
                  child: _submitting
                      ? const SizedBox(
                          width: 18,
                          height: 18,
                          child: CircularProgressIndicator(
                            strokeWidth: 2,
                            color: Colors.white,
                          ),
                        )
                      : const Text('反对'),
                ),
              ),
              const SizedBox(width: 12),
              Expanded(
                child: ElevatedButton(
                  onPressed: _canSubmitVote ? () => _confirmVote(true) : null,
                  style: ElevatedButton.styleFrom(
                    backgroundColor: AppTheme.success,
                    foregroundColor: Colors.white,
                    disabledBackgroundColor:
                        AppTheme.success.withValues(alpha: 0.25),
                    padding: const EdgeInsets.symmetric(vertical: 14),
                    shape: RoundedRectangleBorder(
                      borderRadius: BorderRadius.circular(10),
                    ),
                  ),
                  child: _submitting
                      ? const SizedBox(
                          width: 18,
                          height: 18,
                          child: CircularProgressIndicator(
                            strokeWidth: 2,
                            color: Colors.white,
                          ),
                        )
                      : const Text('赞成'),
                ),
              ),
            ],
          ),
        ],
      ),
    );
  }
}
