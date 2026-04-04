import 'dart:typed_data';

import 'package:flutter/material.dart';
import '../ui/app_theme.dart';
import 'package:flutter/services.dart';
import 'package:polkadart/polkadart.dart' show Hasher;
import 'package:polkadart_keyring/polkadart_keyring.dart' show Keyring;

import '../util/amount_format.dart';
import 'duoqian_manage_models.dart';
import 'duoqian_manage_service.dart';
import 'institution_data.dart';
import 'institution_admin_service.dart';
import 'pending_vote_store.dart';
import 'proposal_context.dart';
import 'proposal_vote_widgets.dart';
import 'transfer_proposal_service.dart';
import '../qr/pages/qr_sign_session_page.dart';
import '../rpc/chain_rpc.dart';
import '../rpc/onchain.dart';
import '../rpc/smoldot_client.dart';
import '../signer/qr_signer.dart';
import '../wallet/core/wallet_manager.dart';

/// 多签管理提案详情页：展示创建/关闭提案信息、投票进度及投票操作。
class DuoqianManageDetailPage extends StatefulWidget {
  const DuoqianManageDetailPage({
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
  State<DuoqianManageDetailPage> createState() =>
      _DuoqianManageDetailPageState();
}

class _DuoqianManageDetailPageState extends State<DuoqianManageDetailPage> {
  static const int _statusVoting = 0;

  final TransferProposalService _proposalService = TransferProposalService();
  final DuoqianManageService _manageService = DuoqianManageService();
  final InstitutionAdminService _adminService = InstitutionAdminService();
  bool _loading = true;
  String? _error;
  bool _submitting = false;

  int? _status;

  // 提案详情（二选一）
  CreateDuoqianProposalInfo? _createInfo;
  CloseDuoqianProposalInfo? _closeInfo;

  bool get _isCreateProposal => _createInfo != null;

  // 投票计数
  int _yesCount = 0;
  int _noCount = 0;

  // 管理员列表与投票记录
  List<String> _admins = const [];
  Map<String, bool?> _adminVotes = {};

  List<WalletProfile> _votableWallets = const [];
  WalletProfile? _selectedVoteWallet;
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
      final rpc = ChainRpc();

      // 并行加载管理员列表、提案状态、投票计数
      final results = await Future.wait([
        _adminService.fetchAdmins(widget.institution.shenfenId),
        _proposalService.fetchProposalStatus(widget.proposalId),
        _proposalService.fetchVoteTally(widget.proposalId),
      ]);

      final admins = results[0] as List<String>;
      final status = results[1] as int?;
      final tally = results[2] as ({int yes, int no});

      // 加载提案业务数据（从 ProposalData 解码）
      final key = _buildProposalDataStorageKey(widget.proposalId);
      final raw = await rpc.fetchStorage('0x${_hexEncode(key)}');
      CreateDuoqianProposalInfo? createInfo;
      CloseDuoqianProposalInfo? closeInfo;
      if (raw != null && raw.isNotEmpty) {
        final detail =
            _manageService.decodeManageProposalData(widget.proposalId, raw);
        if (detail is CreateDuoqianProposalInfo) {
          createInfo = detail;
        } else if (detail is CloseDuoqianProposalInfo) {
          closeInfo = detail;
        }
      }

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

      // 检查待确认投票
      final pendingRecords = await PendingVoteStore.instance.confirmAll(
        'duoqian_manage',
        widget.proposalId,
        OnchainRpc(),
      );
      final pendingPks = pendingRecords.map((r) => r.walletPubkey).toSet();

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
        _createInfo = createInfo;
        _closeInfo = closeInfo;
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

  // ──── 工具方法 ────

  Uint8List _buildProposalDataStorageKey(int proposalId) {
    final palletHash = Hasher.twoxx128.hashString('VotingEngineSystem');
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
    final wallet = _selectedVoteWallet;
    if (wallet == null) return;

    setState(() => _submitting = true);

    try {
      final pubkeyBytes = _hexDecode(wallet.pubkeyHex);

      Future<Uint8List> signCallback(Uint8List payload) async {
        final qrSigner = QrSigner();
        final voteText = approve ? '赞成' : '反对';
        final rv = await ChainRpc().fetchRuntimeVersion();
        final actionLabel = _isCreateProposal ? '创建多签投票' : '关闭多签投票';
        final summaryType = _isCreateProposal ? '创建多签' : '关闭多签';
        final request = qrSigner.buildRequest(
          requestId: QrSigner.generateRequestId(prefix: 'vote-'),
          account: wallet.address,
          pubkey: '0x${wallet.pubkeyHex}',
          payloadHex: '0x${_toHex(payload)}',
          specVersion: rv.specVersion,
          display: {
            'action': _isCreateProposal ? 'vote_create' : 'vote_close',
            'action_label': actionLabel,
            'summary': '$summaryType提案 #${widget.proposalId} 投票：$voteText',
            'fields': [
              {
                'key': 'proposal_id',
                'label': '提案编号',
                'value': widget.proposalId.toString(),
              },
              {'key': 'approve', 'label': '投票', 'value': approve.toString()},
              if (_createInfo != null) ...[
                {
                  'key': 'amount_yuan',
                  'label': '初始资金',
                  'value':
                      AmountFormat.format(_createInfo!.amountYuan, symbol: ''),
                  'format': 'currency',
                },
                {
                  'key': 'threshold',
                  'label': '阈值',
                  'value':
                      '${_createInfo!.threshold}/${_createInfo!.adminCount}',
                },
              ],
              if (_closeInfo != null)
                {
                  'key': 'beneficiary',
                  'label': '受益人',
                  'value': _closeInfo!.beneficiary,
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

      final ({String txHash, int usedNonce}) result;
      if (_isCreateProposal) {
        result = await _manageService.submitVoteCreate(
          proposalId: widget.proposalId,
          approve: approve,
          fromAddress: wallet.address,
          signerPubkey: Uint8List.fromList(pubkeyBytes),
          sign: signCallback,
        );
      } else {
        result = await _manageService.submitVoteClose(
          proposalId: widget.proposalId,
          approve: approve,
          fromAddress: wallet.address,
          signerPubkey: Uint8List.fromList(pubkeyBytes),
          sign: signCallback,
        );
      }

      // 持久化待确认投票记录
      var pubkey = wallet.pubkeyHex.toLowerCase();
      if (pubkey.startsWith('0x')) pubkey = pubkey.substring(2);
      await PendingVoteStore.instance.save(PendingVoteRecord(
        proposalType: 'duoqian_manage',
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
            Text('加载失败',
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
        side: BorderSide(color: AppTheme.border),
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
      _buildInfoRow('管理员数量', '${info.adminCount}'),
      const Divider(height: 20),
      _buildInfoRow('通过阈值', '${info.threshold} / ${info.adminCount}'),
      const Divider(height: 20),
      _buildInfoRow(
        '初始资金',
        '${AmountFormat.format(info.amountYuan, symbol: '')} 元',
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
