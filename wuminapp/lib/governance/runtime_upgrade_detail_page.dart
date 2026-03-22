import 'package:flutter/material.dart';
import 'package:flutter/services.dart';
import 'package:polkadart_keyring/polkadart_keyring.dart' show Keyring;

import 'institution_admin_service.dart';
import 'institution_data.dart';
import 'runtime_upgrade_service.dart';
import 'transfer_proposal_service.dart' show ProposalMeta;
import '../qr/pages/qr_sign_session_page.dart';
import '../signer/qr_signer.dart';
import '../wallet/core/wallet_manager.dart';

/// Runtime 升级提案详情页。
///
/// 从全链提案页进入时为只读模式；
/// 从机构详情页进入时，当前机构管理员可直接提交联合投票。
class RuntimeUpgradeDetailPage extends StatefulWidget {
  const RuntimeUpgradeDetailPage({
    super.key,
    required this.proposalId,
    this.institution,
    this.adminWallets = const [],
  });

  final int proposalId;
  final InstitutionInfo? institution;

  /// 当前用户导入的、属于该机构的管理员钱包列表。
  final List<WalletProfile> adminWallets;

  @override
  State<RuntimeUpgradeDetailPage> createState() =>
      _RuntimeUpgradeDetailPageState();
}

class _RuntimeUpgradeDetailPageState extends State<RuntimeUpgradeDetailPage> {
  static const Color _inkGreen = Color(0xFF0B3D2E);

  final RuntimeUpgradeService _service = RuntimeUpgradeService();
  final InstitutionAdminService _adminService = InstitutionAdminService();

  bool _loading = true;
  bool _submitting = false;
  String? _error;

  RuntimeUpgradeProposalInfo? _proposalInfo;
  ProposalMeta? _meta;
  ({int yes, int no}) _jointTally = (yes: 0, no: 0);
  ({int yes, int no}) _citizenTally = (yes: 0, no: 0);
  bool _reasonExpanded = false;

  bool? _institutionVote;
  List<String> _admins = const [];
  ({int yes, int no}) _institutionAdminTally = (yes: 0, no: 0);
  Map<String, bool?> _adminVotes = const {};
  List<WalletProfile> _votableWallets = const [];
  WalletProfile? _selectedVoteWallet;

  @override
  void initState() {
    super.initState();
    _load();
  }

  bool get _hasInstitutionContext => widget.institution != null;

  int get _requiredAdminThreshold => widget.institution?.internalThreshold ?? 0;

  bool get _jointVoteOpen =>
      (_meta?.status == 0) && (_meta?.stage == 1) && _resolvedStatusCode() == 0;

  bool get _canSubmitVote =>
      _hasInstitutionContext &&
      _jointVoteOpen &&
      _institutionVote == null &&
      _selectedVoteWallet != null &&
      !_submitting;

  bool get _hasImportedAdminWallets => widget.adminWallets.isNotEmpty;

  bool get _allImportedAdminsVoted {
    if (!_hasImportedAdminWallets) return false;
    for (final wallet in widget.adminWallets) {
      final vote = _adminVotes[_normalizeHex(wallet.pubkeyHex)];
      if (vote == null) return false;
    }
    return true;
  }

  String? get _voteDisabledReason {
    if (!_hasInstitutionContext) return null;
    if (!_jointVoteOpen) return '当前提案不在联合投票阶段';
    if (_institutionVote != null) return '本机构已形成最终投票结果';
    if (!_hasImportedAdminWallets) return '当前未导入本机构管理员钱包';
    if (_votableWallets.isEmpty && _allImportedAdminsVoted) {
      return '已导入的管理员钱包都已完成投票';
    }
    if (_votableWallets.isEmpty) return '当前没有可用的管理员钱包';
    if (_selectedVoteWallet == null) return '请选择用于投票的管理员钱包';
    return null;
  }

  Future<void> _load() async {
    setState(() {
      _loading = true;
      _error = null;
    });

    try {
      final futures = <Future<dynamic>>[
        _service.fetchProposalMeta(widget.proposalId),
        _service.fetchRuntimeUpgradeProposal(widget.proposalId),
        _service.fetchJointTally(widget.proposalId),
        _service.fetchCitizenTally(widget.proposalId),
      ];

      final institution = widget.institution;
      if (institution != null) {
        futures.add(_adminService.fetchAdmins(institution.shenfenId));
        futures.add(_service.fetchJointVoteByInstitution(
            widget.proposalId, _shenfenIdToFixed48(institution.shenfenId)));
        futures.add(_service.fetchJointInstitutionTally(
            widget.proposalId, _shenfenIdToFixed48(institution.shenfenId)));
      }

      final results = await Future.wait(futures);
      final meta = results[0] as ProposalMeta?;
      final proposalInfo = results[1] as RuntimeUpgradeProposalInfo?;
      final jointTally = results[2] as ({int yes, int no});
      final citizenTally = results[3] as ({int yes, int no});

      List<String> admins = const [];
      bool? institutionVote;
      ({int yes, int no}) institutionAdminTally = (yes: 0, no: 0);
      Map<String, bool?> adminVotes = const {};
      List<WalletProfile> votableWallets = const [];
      WalletProfile? selectedVoteWallet = _selectedVoteWallet;

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

        final institutionBytes = _shenfenIdToFixed48(institution.shenfenId);
        final voteResults = await Future.wait(
          admins.map((pubkey) async => MapEntry(
                pubkey,
                await _service.fetchJointAdminVote(
                  widget.proposalId,
                  institutionBytes,
                  pubkey,
                ),
              )),
        );
        adminVotes = {
          for (final entry in voteResults) entry.key: entry.value,
        };

        votableWallets = matchedAdminWallets.where((wallet) {
          return adminVotes[_normalizeHex(wallet.pubkeyHex)] == null;
        }).toList(growable: false)
          ..sort((a, b) => a.walletIndex.compareTo(b.walletIndex));

        if (selectedVoteWallet == null ||
            !votableWallets.any((wallet) =>
                wallet.walletIndex == selectedVoteWallet!.walletIndex)) {
          selectedVoteWallet =
              votableWallets.isNotEmpty ? votableWallets.first : null;
        }
      }

      if (!mounted) return;
      setState(() {
        _meta = meta;
        _proposalInfo = proposalInfo;
        _jointTally = jointTally;
        _citizenTally = citizenTally;
        _admins = admins;
        _institutionVote = institutionVote;
        _institutionAdminTally = institutionAdminTally;
        _adminVotes = adminVotes;
        _votableWallets = votableWallets;
        _selectedVoteWallet = selectedVoteWallet;
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

  String _normalizeHex(String hex) {
    return hex.startsWith('0x')
        ? hex.substring(2).toLowerCase()
        : hex.toLowerCase();
  }

  Uint8List _shenfenIdToFixed48(String shenfenId) {
    final bytes = Uint8List(48);
    final raw = shenfenId.codeUnits;
    for (var i = 0; i < raw.length && i < 48; i++) {
      bytes[i] = raw[i];
    }
    return bytes;
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
    required Map<String, dynamic> display,
  }) async {
    // 管理员投票统一通过 QR 码签名（wumin 冷钱包）
    final qrSigner = QrSigner();
    final request = qrSigner.buildRequest(
      requestId:
          '$requestPrefix-${wallet.walletIndex}-${DateTime.now().millisecondsSinceEpoch}',
      account: wallet.address,
      pubkey: '0x${wallet.pubkeyHex}',
      payloadHex: '0x${_toHex(payload)}',
      display: display,
    );
    final requestJson = qrSigner.encodeRequest(request);
    final response = await Navigator.push<QrSignResponse>(
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
    return _hexDecode(response.signature);
  }

  Future<void> _submitJointVote(bool approve) async {
    final institution = widget.institution;
    final voteWallet = _selectedVoteWallet;
    if (institution == null || voteWallet == null) return;

    setState(() => _submitting = true);

    try {
      final institutionBytes = _shenfenIdToFixed48(institution.shenfenId);
      final txHash = await _service.submitJointVote(
        proposalId: widget.proposalId,
        institutionId48: institutionBytes,
        approve: approve,
        fromAddress: voteWallet.address,
        signerPubkey: _hexDecode(voteWallet.pubkeyHex),
        sign: (payload) {
          final voteText = approve ? '赞成' : '反对';
          return _signPayloadWithWallet(
            wallet: voteWallet,
            payload: payload,
            requestPrefix: approve ? 'runtime-joint-yes' : 'runtime-joint-no',
            display: {
              'action': 'joint_vote',
              'summary': '联合投票 提案 #${widget.proposalId}：$voteText',
              'fields': {
                'proposal_id': widget.proposalId.toString(),
                'approve': approve.toString(),
              },
            },
          );
        },
      );

      if (!mounted) return;
      ScaffoldMessenger.of(context).showSnackBar(
        SnackBar(
          content: Text('联合投票已提交：${_truncateAddress(txHash)}'),
          backgroundColor: _inkGreen,
        ),
      );

      _adminService.clearCache(institution.shenfenId);
      await _load();
    } on WalletAuthException catch (e) {
      if (!mounted) return;
      ScaffoldMessenger.of(context).showSnackBar(
        SnackBar(content: Text(e.message), backgroundColor: Colors.red),
      );
    } catch (e) {
      if (!mounted) return;
      ScaffoldMessenger.of(context).showSnackBar(
        SnackBar(content: Text('投票失败：$e'), backgroundColor: Colors.red),
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

  String _statusLabel(int? status) {
    switch (status) {
      case 0:
        return '投票中';
      case 1:
        return '已通过';
      case 2:
        return '已拒绝';
      case 3:
        return '已执行';
      case 4:
        return '执行失败';
      default:
        return '未知';
    }
  }

  Color _statusColor(int? status) {
    switch (status) {
      case 0:
        return Colors.blue;
      case 1:
        return Colors.green;
      case 2:
        return Colors.red;
      case 3:
        return Colors.green;
      case 4:
        return Colors.orange;
      default:
        return Colors.grey;
    }
  }

  IconData _statusIcon(int? status) {
    switch (status) {
      case 0:
        return Icons.how_to_vote;
      case 1:
        return Icons.check_circle;
      case 2:
        return Icons.cancel;
      case 3:
        return Icons.task_alt;
      case 4:
        return Icons.error;
      default:
        return Icons.help_outline;
    }
  }

  int? _resolvedStatusCode() {
    // 中文注释：业务提案自己的 3 表示“执行失败”，投票引擎元数据的 3 表示“已执行”；
    // 详情页优先展示业务状态，避免把已执行误标成失败。
    if (_proposalInfo?.status == 3) {
      return 4;
    }
    if (_meta?.status == 3) {
      return 3;
    }
    return _proposalInfo?.status ?? _meta?.status;
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
          '升级提案详情',
          style: TextStyle(fontSize: 17, fontWeight: FontWeight.w700),
        ),
        centerTitle: true,
        backgroundColor: Colors.white,
        foregroundColor: _inkGreen,
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
            const Icon(Icons.error_outline, size: 48, color: Colors.red),
            const SizedBox(height: 12),
            Text('加载失败',
                style: TextStyle(fontSize: 16, color: Colors.grey[700])),
            const SizedBox(height: 6),
            Text(
              _error!,
              style: TextStyle(fontSize: 12, color: Colors.grey[500]),
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
          _adminService.clearCache(institution.shenfenId);
        }
        await _load();
      },
      child: ListView(
        padding: const EdgeInsets.fromLTRB(16, 8, 16, 32),
        children: [
          _buildStatusBadge(),
          const SizedBox(height: 16),
          _buildProposalInfoCard(),
          const SizedBox(height: 16),
          _buildJointVotingProgress(),
          if (_hasInstitutionContext) ...[
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

  Widget _buildStatusBadge() {
    final status = _resolvedStatusCode();
    final color = _statusColor(status);
    final label = _statusLabel(status);
    final icon = _statusIcon(status);
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
              Icon(icon, size: 16, color: color),
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
          style: TextStyle(fontSize: 13, color: Colors.grey[500]),
        ),
      ],
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
        side: BorderSide(color: Colors.grey[200]!),
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
                color: _inkGreen,
              ),
            ),
            const SizedBox(height: 12),
            _buildInfoRow(
              '提案 ID',
              formatProposalId(widget.proposalId),
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
              _buildRemarkRow('升级理由', reason),
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
              const Divider(height: 20),
              _buildInfoRow('Code 状态', _codeStatusLabel(info)),
            ],
          ],
        ),
      ),
    );
  }

  String _codeStatusLabel(RuntimeUpgradeProposalInfo info) {
    if (!info.hasCode) return '已清理';
    if (info.status == 0) return '待执行';
    return '已归档';
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
            style: TextStyle(fontSize: 13, color: Colors.grey[600]),
          ),
        ),
        Expanded(
          child: Text(
            text,
            style: const TextStyle(fontSize: 13, color: Color(0xFF333333)),
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
              color: Colors.grey[400],
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
            style: TextStyle(fontSize: 13, color: Colors.grey[600]),
          ),
        ),
        Expanded(
          child: Text(
            value,
            style: const TextStyle(fontSize: 13, color: Color(0xFF333333)),
          ),
        ),
        if (onCopy != null)
          GestureDetector(
            onTap: onCopy,
            child: Icon(Icons.copy, size: 16, color: Colors.grey[400]),
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
        side: BorderSide(color: Colors.grey[200]!),
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
                color: _inkGreen,
              ),
            ),
            const SizedBox(height: 12),
            ClipRRect(
              borderRadius: BorderRadius.circular(6),
              child: LinearProgressIndicator(
                value: progress,
                minHeight: 10,
                backgroundColor: Colors.grey[200],
                valueColor: const AlwaysStoppedAnimation<Color>(_inkGreen),
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
                    color: _inkGreen,
                  ),
                ),
                Text(
                  '反对 ${_jointTally.no}',
                  style: TextStyle(
                    fontSize: 13,
                    color: Colors.red[400],
                  ),
                ),
              ],
            ),
            const SizedBox(height: 6),
            Text(
              '联合投票总权重 $jointVoteTotal，国储会权重 19，省储会/省储行各权重 1',
              style: TextStyle(fontSize: 12, color: Colors.grey[500]),
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
        side: BorderSide(color: Colors.grey[200]!),
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
                color: _inkGreen,
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
                color: _inkGreen,
              ),
            ),
            ClipRRect(
              borderRadius: BorderRadius.circular(6),
              child: LinearProgressIndicator(
                value: progress,
                minHeight: 8,
                backgroundColor: Colors.grey[200],
                valueColor: AlwaysStoppedAnimation<Color>(
                    _institutionVote == true ? _inkGreen : Colors.orange),
              ),
            ),
            const SizedBox(height: 12),
            Row(
              mainAxisAlignment: MainAxisAlignment.spaceBetween,
              children: [
                Text(
                  '管理员反对 ${_institutionAdminTally.no}',
                  style: TextStyle(fontSize: 13, color: Colors.red[400]),
                ),
                Text(
                  '链上当前管理员 ${_admins.length} 人',
                  style: TextStyle(fontSize: 12, color: Colors.grey[500]),
                ),
              ],
            ),
            const Divider(height: 20),
            _buildVoteWalletSelector(),
            if (_admins.isNotEmpty) ...[
              const SizedBox(height: 8),
              Text(
                '本机构管理员直接上链投票，赞成达到阈值会自动形成机构赞成结果；若剩余管理员已不足以达到阈值，链上会自动形成机构反对结果。',
                style: TextStyle(fontSize: 12, color: Colors.grey[500]),
              ),
            ],
          ],
        ),
      ),
    );
  }

  Widget _buildVoteWalletSelector() {
    if (!_hasImportedAdminWallets) {
      return Container(
        width: double.infinity,
        padding: const EdgeInsets.all(12),
        decoration: BoxDecoration(
          color: Colors.orange.withValues(alpha: 0.08),
          borderRadius: BorderRadius.circular(8),
        ),
        child: const Text(
          '当前未导入属于本机构的管理员钱包',
          style: TextStyle(fontSize: 13, color: Colors.orange),
        ),
      );
    }

    if (_votableWallets.isEmpty) {
      return Container(
        width: double.infinity,
        padding: const EdgeInsets.all(12),
        decoration: BoxDecoration(
          color: Colors.grey[100],
          borderRadius: BorderRadius.circular(8),
        ),
        child: Text(
          _allImportedAdminsVoted ? '已导入管理员钱包均已完成投票' : '当前没有可用的管理员钱包',
          style: TextStyle(fontSize: 13, color: Colors.grey[600]),
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
          style: TextStyle(fontSize: 11, color: Colors.grey[500]),
        ),
        trailing: const Icon(Icons.shield_outlined, size: 18, color: Colors.orange),
      );
    }

    return Container(
      padding: const EdgeInsets.symmetric(horizontal: 12),
      decoration: BoxDecoration(
        color: Colors.green.withValues(alpha: 0.05),
        borderRadius: BorderRadius.circular(8),
        border: Border.all(color: Colors.green.withValues(alpha: 0.2)),
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
                  const Icon(Icons.shield_outlined, size: 18, color: Colors.orange),
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
        side: BorderSide(color: Colors.grey[200]!),
      ),
      child: Padding(
        padding: const EdgeInsets.all(16),
        child: Column(
          crossAxisAlignment: CrossAxisAlignment.start,
          children: [
            const Text(
              '公民投票进度',
              style: TextStyle(
                fontSize: 16,
                fontWeight: FontWeight.w700,
                color: _inkGreen,
              ),
            ),
            const SizedBox(height: 12),
            Row(
              mainAxisAlignment: MainAxisAlignment.spaceBetween,
              children: [
                Text(
                  '赞成 ${_citizenTally.yes}',
                  style: const TextStyle(
                    fontSize: 14,
                    fontWeight: FontWeight.w600,
                    color: _inkGreen,
                  ),
                ),
                Text(
                  '反对 ${_citizenTally.no}',
                  style: TextStyle(
                    fontSize: 13,
                    color: Colors.red[400],
                  ),
                ),
              ],
            ),
          ],
        ),
      ),
    );
  }

  /// 公民投票阶段判断
  bool get _citizenVoteOpen =>
      (_meta?.status == 0) && (_meta?.stage == 2) && _resolvedStatusCode() == 0;

  Widget? _buildBottomBar() {
    if (_loading || _error != null) return null;
    // 联合投票阶段：仅管理员显示投票按钮
    if (_hasInstitutionContext && _jointVoteOpen) {
      return _buildVoteButtons();
    }
    // 公民投票阶段：所有用户显示投票按钮（SFID 绑定校验后续完善）
    if (_citizenVoteOpen) {
      return _buildCitizenVoteButtons();
    }
    // 非投票阶段但有机构上下文：显示禁用状态的投票按钮
    if (_hasInstitutionContext) {
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
            color: Colors.black.withValues(alpha: 0.06),
            blurRadius: 8,
            offset: const Offset(0, -2),
          ),
        ],
      ),
      child: Column(
        mainAxisSize: MainAxisSize.min,
        children: [
          Padding(
            padding: const EdgeInsets.only(bottom: 10),
            child: Text(
              '公民投票',
              style: TextStyle(fontSize: 13, color: Colors.grey[600]),
              textAlign: TextAlign.center,
            ),
          ),
          Row(
            children: [
              Expanded(
                child: ElevatedButton(
                  onPressed: _submitting
                      ? null
                      : () => _confirmCitizenVote(true),
                  style: ElevatedButton.styleFrom(
                    backgroundColor: Colors.green,
                    foregroundColor: Colors.white,
                    disabledBackgroundColor:
                        Colors.green.withValues(alpha: 0.25),
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
              const SizedBox(width: 12),
              Expanded(
                child: ElevatedButton(
                  onPressed: _submitting
                      ? null
                      : () => _confirmCitizenVote(false),
                  style: ElevatedButton.styleFrom(
                    backgroundColor: Colors.red,
                    foregroundColor: Colors.white,
                    disabledBackgroundColor: Colors.red.withValues(alpha: 0.25),
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
        title: Text('确认公民投票$label'),
        content: Text(
          '将对此升级提案投"$label"票。投票后不可修改。',
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
    // TODO: 公民投票提交逻辑（需要链上 citizen_vote extrinsic）
    // 暂时提示功能开发中
    if (!mounted) return;
    ScaffoldMessenger.of(context).showSnackBar(
      const SnackBar(
        content: Text('公民投票功能开发中'),
        backgroundColor: Colors.orange,
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
            color: Colors.black.withValues(alpha: 0.06),
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
                style: TextStyle(fontSize: 13, color: Colors.grey[500]),
                textAlign: TextAlign.center,
              ),
            ),
          Row(
            children: [
              Expanded(
                child: ElevatedButton(
                  onPressed: _canSubmitVote ? () => _confirmVote(true) : null,
                  style: ElevatedButton.styleFrom(
                    backgroundColor: Colors.green,
                    foregroundColor: Colors.white,
                    disabledBackgroundColor:
                        Colors.green.withValues(alpha: 0.25),
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
              const SizedBox(width: 12),
              Expanded(
                child: ElevatedButton(
                  onPressed: _canSubmitVote ? () => _confirmVote(false) : null,
                  style: ElevatedButton.styleFrom(
                    backgroundColor: Colors.red,
                    foregroundColor: Colors.white,
                    disabledBackgroundColor: Colors.red.withValues(alpha: 0.25),
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
            ],
          ),
        ],
      ),
    );
  }
}
