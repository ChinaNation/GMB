import 'package:flutter/material.dart';
import 'package:flutter/services.dart';

import 'institution_data.dart' show formatProposalId;
import 'runtime_upgrade_service.dart';
import 'transfer_proposal_service.dart' show ProposalMeta;

/// Runtime 升级提案详情页：展示提案信息和联合投票进度（只读，无投票按钮）。
class RuntimeUpgradeDetailPage extends StatefulWidget {
  const RuntimeUpgradeDetailPage({super.key, required this.proposalId});

  final int proposalId;

  @override
  State<RuntimeUpgradeDetailPage> createState() =>
      _RuntimeUpgradeDetailPageState();
}

class _RuntimeUpgradeDetailPageState extends State<RuntimeUpgradeDetailPage> {
  static const Color _inkGreen = Color(0xFF0B3D2E);

  bool _loading = true;
  String? _error;

  RuntimeUpgradeProposalInfo? _proposalInfo;
  ProposalMeta? _meta;
  ({int yes, int no}) _jointTally = (yes: 0, no: 0);
  ({int yes, int no}) _citizenTally = (yes: 0, no: 0);
  bool _reasonExpanded = false;

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
      final service = RuntimeUpgradeService();
      final results = await Future.wait([
        service.fetchProposalMeta(widget.proposalId),
        service.fetchRuntimeUpgradeProposal(widget.proposalId),
        service.fetchJointTally(widget.proposalId),
        service.fetchCitizenTally(widget.proposalId),
      ]);

      final meta = results[0] as ProposalMeta?;
      final proposalInfo = results[1] as RuntimeUpgradeProposalInfo?;
      final jointTally = results[2] as ({int yes, int no});
      final citizenTally = results[3] as ({int yes, int no});

      if (!mounted) return;
      setState(() {
        _meta = meta;
        _proposalInfo = proposalInfo;
        _jointTally = jointTally;
        _citizenTally = citizenTally;
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

  // ──── 工具函数 ────

  String _truncateAddress(String address) {
    if (address.length <= 14) return address;
    return '${address.substring(0, 6)}...${address.substring(address.length - 6)}';
  }

  // ──── 状态相关 ────

  String _statusLabel(int? status) {
    switch (status) {
      case 0:
        return '投票中';
      case 1:
        return '已通过';
      case 2:
        return '已拒绝';
      case 3:
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
        return Icons.error;
      default:
        return Icons.help_outline;
    }
  }

  // ──── 构建 UI ────

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
      onRefresh: _load,
      child: ListView(
        padding: const EdgeInsets.fromLTRB(16, 8, 16, 32),
        children: [
          _buildStatusBadge(),
          const SizedBox(height: 16),
          _buildProposalInfoCard(),
          const SizedBox(height: 16),
          _buildJointVotingProgress(),
          if (_meta?.stage == 2) ...[
            const SizedBox(height: 16),
            _buildCitizenVotingProgress(),
          ],
        ],
      ),
    );
  }

  // ──── 提案状态标签 ────

  Widget _buildStatusBadge() {
    final status = _meta?.status;
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

  // ──── 提案信息卡片 ────

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
              _buildInfoRow(
                '类型',
                '联合投票 · Runtime 升级',
              ),
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
    return Column(
      crossAxisAlignment: CrossAxisAlignment.start,
      children: [
        Row(
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
                onTap: () =>
                    setState(() => _reasonExpanded = !_reasonExpanded),
                child: Icon(
                  _reasonExpanded
                      ? Icons.keyboard_arrow_up
                      : Icons.keyboard_arrow_down,
                  size: 20,
                  color: Colors.grey[400],
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

  // ──── 联合投票进度 ────

  Widget _buildJointVotingProgress() {
    const threshold = 3; // NRC + PRC + PRB
    final progress =
        threshold > 0 ? (_jointTally.yes / threshold).clamp(0.0, 1.0) : 0.0;

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
                  '赞成 ${_jointTally.yes} / 阈值 $threshold',
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
          ],
        ),
      ),
    );
  }

  // ──── 公民投票进度 ────

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
}
