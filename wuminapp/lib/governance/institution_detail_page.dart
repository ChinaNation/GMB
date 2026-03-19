import 'package:flutter/material.dart';

import '../wallet/core/wallet_manager.dart';
import 'admin_list_page.dart';
import 'institution_admin_service.dart';
import 'institution_data.dart';
import 'proposal_types_page.dart';
import 'transfer_proposal_detail_page.dart';
import 'transfer_proposal_service.dart';

/// 机构详情页。
class InstitutionDetailPage extends StatefulWidget {
  const InstitutionDetailPage({
    super.key,
    required this.institution,
    required this.icon,
    required this.badgeColor,
  });

  final InstitutionInfo institution;
  final IconData icon;
  final Color badgeColor;

  @override
  State<InstitutionDetailPage> createState() => _InstitutionDetailPageState();
}

class _InstitutionDetailPageState extends State<InstitutionDetailPage> {
  static const Color _inkGreen = Color(0xFF0B3D2E);

  final InstitutionAdminService _adminService = InstitutionAdminService();
  final WalletManager _walletManager = WalletManager();
  final TransferProposalService _transferService = TransferProposalService();

  List<String> _admins = const [];
  bool _isCurrentUserAdmin = false;
  bool _loading = true;
  String? _error;
  /// 当前用户导入的所有管理员钱包（pubkeyHex → WalletProfile）。
  List<WalletProfile> _adminWallets = const [];
  /// 所有匹配的管理员公钥（小写 hex，不含 0x）。
  Set<String> _adminPubkeys = const {};
  /// 该机构的所有转账提案（按 ID 倒序）。
  List<TransferProposalInfo> _transferProposals = const [];

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
      final results = await Future.wait([
        _adminService.fetchAdmins(widget.institution.shenfenId),
        _walletManager.getWallets(),
        _transferService
            .fetchAllInstitutionProposals(widget.institution.shenfenId),
      ]);
      final admins = results[0] as List<String>;
      final wallets = results[1] as List<WalletProfile>;
      final proposals = results[2] as List<TransferProposalInfo>;

      // 收集所有匹配的管理员钱包
      final matchedWallets = <WalletProfile>[];
      final matchedPubkeys = <String>{};
      for (final wallet in wallets) {
        var pubkey = wallet.pubkeyHex.toLowerCase();
        if (pubkey.startsWith('0x')) pubkey = pubkey.substring(2);
        if (admins.contains(pubkey)) {
          matchedWallets.add(wallet);
          matchedPubkeys.add(pubkey);
        }
      }

      if (!mounted) return;
      setState(() {
        _admins = admins;
        _adminWallets = matchedWallets;
        _adminPubkeys = matchedPubkeys;
        _isCurrentUserAdmin = matchedWallets.isNotEmpty;
        _transferProposals = proposals;
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

  @override
  Widget build(BuildContext context) {
    return Scaffold(
      backgroundColor: Colors.white,
      appBar: AppBar(
        title: Text(
          widget.institution.name,
          style: const TextStyle(fontSize: 17, fontWeight: FontWeight.w700),
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
      onRefresh: () async {
        _adminService.clearCache(widget.institution.shenfenId);
        await _load();
      },
      child: ListView(
        padding: const EdgeInsets.fromLTRB(16, 8, 16, 32),
        children: [
          _buildHeader(),
          const SizedBox(height: 12),
          if (_isCurrentUserAdmin) ...[
            _buildAdminBadge(),
            const SizedBox(height: 12),
          ],
          _buildAdminEntry(),
          const SizedBox(height: 12),
          _buildVotingEvents(),
        ],
      ),
    );
  }

  // ──── 顶部机构卡片（横向布局 + 右箭头进入提案页） ────

  Widget _buildHeader() {
    final inst = widget.institution;
    return Card(
      elevation: 0,
      margin: EdgeInsets.zero,
      shape: RoundedRectangleBorder(
        borderRadius: BorderRadius.circular(12),
        side: BorderSide(color: widget.badgeColor.withValues(alpha: 0.18)),
      ),
      child: InkWell(
        onTap: _isCurrentUserAdmin ? _openProposalTypes : null,
        borderRadius: BorderRadius.circular(12),
        child: Padding(
          padding: const EdgeInsets.symmetric(horizontal: 14, vertical: 12),
          child: Row(
            children: [
              // 左侧图标
              Container(
                width: 44,
                height: 44,
                decoration: BoxDecoration(
                  color: widget.badgeColor.withValues(alpha: 0.12),
                  borderRadius: BorderRadius.circular(12),
                ),
                child:
                    Icon(widget.icon, size: 22, color: widget.badgeColor),
              ),
              const SizedBox(width: 12),
              // 中间：简称标签 / 管理员信息
              Expanded(
                child: Column(
                  crossAxisAlignment: CrossAxisAlignment.start,
                  children: [
                    Container(
                      padding: const EdgeInsets.symmetric(
                          horizontal: 6, vertical: 1),
                      decoration: BoxDecoration(
                        color: widget.badgeColor.withValues(alpha: 0.10),
                        borderRadius: BorderRadius.circular(10),
                      ),
                      child: Text(
                        '${OrgType.label(inst.orgType)}　提案',
                        style: TextStyle(
                          fontSize: 11,
                          color: widget.badgeColor,
                          fontWeight: FontWeight.w600,
                        ),
                      ),
                    ),
                    const SizedBox(height: 4),
                    Text(
                      '管理员 ${_admins.length} 人　通过阈值 ${inst.internalThreshold}',
                      style: TextStyle(fontSize: 12, color: Colors.grey[500]),
                    ),
                  ],
                ),
              ),
              // 右侧箭头（仅管理员显示）
              if (_isCurrentUserAdmin)
                Icon(Icons.chevron_right, size: 20, color: Colors.grey[400]),
            ],
          ),
        ),
      ),
    );
  }

  // ──── 管理员身份标识 ────

  Widget _buildAdminBadge() {
    return Container(
      padding: const EdgeInsets.symmetric(horizontal: 10, vertical: 6),
      decoration: BoxDecoration(
        color: Colors.green.withValues(alpha: 0.06),
        borderRadius: BorderRadius.circular(10),
        border: Border.all(color: Colors.green.withValues(alpha: 0.2)),
      ),
      child: const Row(
        mainAxisSize: MainAxisSize.min,
        children: [
          Icon(Icons.verified_user, size: 14, color: Colors.green),
          SizedBox(width: 4),
          Text(
            '你是本机构管理员，点击上方卡片可发起提案',
            style: TextStyle(
              fontSize: 12,
              color: Colors.green,
              fontWeight: FontWeight.w500,
            ),
          ),
        ],
      ),
    );
  }

  // ──── 管理员列表入口 ────

  Widget _buildAdminEntry() {
    return Card(
      elevation: 0,
      margin: EdgeInsets.zero,
      shape: RoundedRectangleBorder(
        borderRadius: BorderRadius.circular(12),
        side: BorderSide(color: Colors.grey[200]!),
      ),
      child: InkWell(
        onTap: _openAdminList,
        borderRadius: BorderRadius.circular(12),
        child: Padding(
          padding: const EdgeInsets.symmetric(horizontal: 14, vertical: 12),
          child: Row(
            children: [
              Container(
                width: 36,
                height: 36,
                decoration: BoxDecoration(
                  color: _inkGreen.withValues(alpha: 0.08),
                  borderRadius: BorderRadius.circular(10),
                ),
                child: const Icon(Icons.people_outline,
                    size: 18, color: _inkGreen),
              ),
              const SizedBox(width: 12),
              Expanded(
                child: Column(
                  crossAxisAlignment: CrossAxisAlignment.start,
                  children: [
                    const Text(
                      '管理员列表',
                      style: TextStyle(
                        fontSize: 15,
                        fontWeight: FontWeight.w600,
                        color: _inkGreen,
                      ),
                    ),
                    const SizedBox(height: 2),
                    Text(
                      '共 ${_admins.length} 位管理员',
                      style:
                          TextStyle(fontSize: 12, color: Colors.grey[500]),
                    ),
                  ],
                ),
              ),
              Icon(Icons.chevron_right, size: 20, color: Colors.grey[400]),
            ],
          ),
        ),
      ),
    );
  }

  // ──── 投票事件列表 ────

  Widget _buildVotingEvents() {
    return Column(
      crossAxisAlignment: CrossAxisAlignment.start,
      children: [
        const Text(
          '投票事件',
          style: TextStyle(
            fontSize: 16,
            fontWeight: FontWeight.w700,
            color: _inkGreen,
          ),
        ),
        const SizedBox(height: 12),
        if (_transferProposals.isEmpty)
          Container(
            width: double.infinity,
            padding: const EdgeInsets.all(24),
            decoration: BoxDecoration(
              color: Colors.grey[50],
              borderRadius: BorderRadius.circular(12),
              border: Border.all(color: Colors.grey[200]!),
            ),
            child: Column(
              children: [
                Icon(Icons.ballot_outlined, size: 40, color: Colors.grey[400]),
                const SizedBox(height: 8),
                Text(
                  '暂无投票事件',
                  style: TextStyle(fontSize: 14, color: Colors.grey[500]),
                ),
                const SizedBox(height: 4),
                Text(
                  '本机构的提案投票事件将在此显示',
                  style: TextStyle(fontSize: 12, color: Colors.grey[400]),
                ),
              ],
            ),
          )
        else
          ...List.generate(_transferProposals.length, (index) {
            final proposal = _transferProposals[index];
            return Padding(
              padding: EdgeInsets.only(bottom: index < _transferProposals.length - 1 ? 8 : 0),
              child: _buildTransferProposalCard(proposal),
            );
          }),
      ],
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
      default:
        return Colors.grey;
    }
  }

  Widget _buildTransferProposalCard(TransferProposalInfo proposal) {
    final statusColor = _statusColor(proposal.status);
    return Card(
      elevation: 0,
      margin: EdgeInsets.zero,
      shape: RoundedRectangleBorder(
        borderRadius: BorderRadius.circular(12),
        side: BorderSide(color: statusColor.withValues(alpha: 0.2)),
      ),
      child: InkWell(
        onTap: () => _openTransferProposalDetail(proposal.proposalId),
        borderRadius: BorderRadius.circular(12),
        child: Padding(
          padding: const EdgeInsets.symmetric(horizontal: 14, vertical: 12),
          child: Row(
            children: [
              Container(
                width: 36,
                height: 36,
                decoration: BoxDecoration(
                  color: statusColor.withValues(alpha: 0.10),
                  borderRadius: BorderRadius.circular(10),
                ),
                child: Icon(Icons.send_outlined, size: 18, color: statusColor),
              ),
              const SizedBox(width: 12),
              Expanded(
                child: Column(
                  crossAxisAlignment: CrossAxisAlignment.start,
                  children: [
                    Text(
                      '转账提案 ${formatProposalId(proposal.proposalId)}',
                      style: const TextStyle(
                        fontSize: 15,
                        fontWeight: FontWeight.w600,
                        color: _inkGreen,
                      ),
                    ),
                    const SizedBox(height: 2),
                    Text(
                      '${proposal.amountYuan.toStringAsFixed(2)} 元 · ${_statusLabel(proposal.status)}',
                      style: TextStyle(fontSize: 12, color: Colors.grey[500]),
                    ),
                  ],
                ),
              ),
              Container(
                padding:
                    const EdgeInsets.symmetric(horizontal: 8, vertical: 2),
                decoration: BoxDecoration(
                  color: statusColor.withValues(alpha: 0.1),
                  borderRadius: BorderRadius.circular(10),
                ),
                child: Text(
                  _statusLabel(proposal.status),
                  style: TextStyle(
                    fontSize: 11,
                    fontWeight: FontWeight.w600,
                    color: statusColor,
                  ),
                ),
              ),
              const SizedBox(width: 4),
              Icon(Icons.chevron_right, size: 20, color: Colors.grey[400]),
            ],
          ),
        ),
      ),
    );
  }

  // ──── 导航 ────

  Future<void> _openProposalTypes() async {
    await Navigator.of(context).push(
      MaterialPageRoute(
        builder: (_) => ProposalTypesPage(
          institution: widget.institution,
          icon: widget.icon,
          badgeColor: widget.badgeColor,
          adminWallets: _adminWallets,
        ),
      ),
    );
    // 返回后刷新（可能新建了提案）
    if (mounted) {
      _adminService.clearCache(widget.institution.shenfenId);
      _load();
    }
  }

  Future<void> _openTransferProposalDetail(int proposalId) async {
    await Navigator.of(context).push(
      MaterialPageRoute(
        builder: (_) => TransferProposalDetailPage(
          institution: widget.institution,
          proposalId: proposalId,
          adminWallets: _adminWallets,
        ),
      ),
    );
    // 返回后刷新（投票状态可能变化）
    if (mounted) {
      _adminService.clearCache(widget.institution.shenfenId);
      _load();
    }
  }

  void _openAdminList() {
    Navigator.of(context).push(
      MaterialPageRoute(
        builder: (_) => AdminListPage(
          institution: widget.institution,
          admins: _admins,
          adminPubkeys: _adminPubkeys,
          badgeColor: widget.badgeColor,
        ),
      ),
    );
  }
}
