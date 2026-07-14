import 'dart:async';

import 'package:flutter/material.dart';

import 'package:citizenapp/citizen/proposal/admins-change/models/admin_account.dart';
import 'package:citizenapp/citizen/proposal/admins-change/services/admin_activation_service.dart';
import 'package:citizenapp/citizen/institution/institution_assignment_card.dart';
import 'package:citizenapp/citizen/institution/institution_role_models.dart';
import 'package:citizenapp/citizen/shared/institution_info.dart';
import 'package:citizenapp/qr/pages/qr_sign_session_page.dart';
import 'package:citizenapp/rpc/chain_rpc.dart';
import 'package:citizenapp/signer/qr_signer.dart';
import 'package:citizenapp/ui/app_theme.dart';
import 'package:citizenapp/wallet/core/wallet_manager.dart';

/// 管理员列表页面。
///
/// 展示机构所有管理员的完整 SS58 地址，支持 QR 扫码激活。
class AdminListPage extends StatefulWidget {
  const AdminListPage({
    super.key,
    required this.institution,
    required this.accountIdentity,
    required this.admins,
    required this.importedColdPubkeys,
    required this.activatedPubkeys,
    required this.badgeColor,
    this.onActivated,
  });

  final InstitutionInfo institution;
  final AdminAccountIdentity accountIdentity;

  /// 机构管理员岗位任职；同一钱包可出现多条不同岗位记录。
  final List<InstitutionAdminAssignment> admins;

  /// 用户已导入的冷钱包公钥集合（小写 hex，不含 0x）。
  final Set<String> importedColdPubkeys;

  /// 已激活的管理员公钥集合（小写 hex）。
  final Set<String> activatedPubkeys;
  final Color badgeColor;

  /// 激活成功后的回调（通知父页面刷新）。
  final VoidCallback? onActivated;

  @override
  State<AdminListPage> createState() => _AdminListPageState();
}

class _AdminListPageState extends State<AdminListPage> {
  late Set<String> _activatedPubkeys;
  Map<String, double> _balanceByAccount = const {};

  @override
  void initState() {
    super.initState();
    _activatedPubkeys = Set.of(widget.activatedPubkeys);
    unawaited(_loadBalances());
  }

  @override
  void didUpdateWidget(covariant AdminListPage oldWidget) {
    super.didUpdateWidget(oldWidget);
    if (oldWidget.admins != widget.admins) {
      unawaited(_loadBalances());
    }
  }

  static String _balanceKey(String account) {
    final trimmed = account.trim();
    return (trimmed.startsWith('0x') || trimmed.startsWith('0X')
            ? trimmed.substring(2)
            : trimmed)
        .toLowerCase();
  }

  Future<void> _loadBalances() async {
    final accounts = {
      for (final assignment in widget.admins)
        _balanceKey(assignment.adminAccount),
    }.where((account) => account.isNotEmpty).toList(growable: false);
    if (accounts.isEmpty) {
      if (mounted) setState(() => _balanceByAccount = const {});
      return;
    }
    try {
      final balances = await ChainRpc().fetchFinalizedBalances(accounts);
      if (!mounted) return;
      setState(() => _balanceByAccount = balances);
    } catch (_) {
      // 余额展示失败不影响管理员激活流程,卡片保留“余额”标签且值为空。
      if (mounted) setState(() => _balanceByAccount = const {});
    }
  }

  void _onAdminActivated(String pubkeyHex) {
    setState(() {
      _activatedPubkeys.add(pubkeyHex);
    });
    widget.onActivated?.call();
  }

  @override
  Widget build(BuildContext context) {
    return Scaffold(
      backgroundColor: Colors.white,
      appBar: AppBar(
        title: const Text(
          '管理员列表',
          style: TextStyle(fontSize: 17, fontWeight: FontWeight.w700),
        ),
        centerTitle: true,
        backgroundColor: Colors.white,
        foregroundColor: AppTheme.primaryDark,
        elevation: 0,
        scrolledUnderElevation: 0.5,
      ),
      body: ListView(
        padding: const EdgeInsets.fromLTRB(16, 8, 16, 32),
        children: [
          // 机构信息
          _buildInstitutionHeader(),
          const SizedBox(height: 16),
          // 管理员总数
          Text(
            '共 ${widget.admins.length} 位管理员　通过阈值 ${widget.institution.internalThreshold}',
            style: const TextStyle(fontSize: 13, color: AppTheme.textTertiary),
          ),
          const SizedBox(height: 12),
          // 管理员列表
          if (widget.admins.isEmpty)
            const Padding(
              padding: EdgeInsets.symmetric(vertical: 24),
              child: Center(
                child: Text(
                  '暂无管理员信息',
                  style: TextStyle(fontSize: 14, color: AppTheme.textTertiary),
                ),
              ),
            )
          else
            ...List.generate(widget.admins.length, (index) {
              final assignment = widget.admins[index];
              final pubkey = assignment.adminAccount;
              final isImported = widget.importedColdPubkeys.contains(pubkey);
              final isActivated = _activatedPubkeys.contains(pubkey);
              return _AdminTile(
                index: index + 1,
                assignment: assignment,
                isImported: isImported,
                isActivated: isActivated,
                institution: widget.institution,
                accountIdentity: widget.accountIdentity,
                onActivated: () => _onAdminActivated(pubkey),
                balanceYuan: _balanceByAccount[_balanceKey(pubkey)],
              );
            }),
        ],
      ),
    );
  }

  Widget _buildInstitutionHeader() {
    return Row(
      children: [
        Container(
          width: 36,
          height: 36,
          decoration: BoxDecoration(
            color: widget.badgeColor.withValues(alpha: 0.12),
            borderRadius: BorderRadius.circular(10),
          ),
          child: Icon(Icons.people_outline, size: 18, color: widget.badgeColor),
        ),
        const SizedBox(width: 10),
        Expanded(
          child: Text(
            widget.institution.cidShortName,
            style: const TextStyle(
              fontSize: 15,
              fontWeight: FontWeight.w600,
              color: AppTheme.primaryDark,
            ),
          ),
        ),
        Container(
          padding: const EdgeInsets.symmetric(horizontal: 8, vertical: 2),
          decoration: BoxDecoration(
            color: widget.badgeColor.withValues(alpha: 0.10),
            borderRadius: BorderRadius.circular(10),
          ),
          child: Text(
            OrgType.label(widget.institution.orgType),
            style: TextStyle(
              fontSize: 11,
              color: widget.badgeColor,
              fontWeight: FontWeight.w600,
            ),
          ),
        ),
      ],
    );
  }
}

class _AdminTile extends StatelessWidget {
  const _AdminTile({
    required this.index,
    required this.assignment,
    required this.isImported,
    required this.isActivated,
    required this.institution,
    required this.accountIdentity,
    required this.onActivated,
    required this.balanceYuan,
  });

  final int index;

  final InstitutionAdminAssignment assignment;

  /// 管理员账户(小写 hex);激活/匹配仍按账户。
  String get pubkeyHex => assignment.adminAccount;

  /// 用户是否已导入此公钥的冷钱包。
  final bool isImported;

  /// 此管理员是否已激活。
  final bool isActivated;
  final InstitutionInfo institution;
  final AdminAccountIdentity accountIdentity;
  final VoidCallback onActivated;
  final double? balanceYuan;

  Future<void> _startActivation(BuildContext context) async {
    // 检查是否为冷钱包（热钱包不允许激活）
    final wallets = await WalletManager().getWallets();
    final wallet = wallets.where((w) {
      final pk = w.pubkeyHex.toLowerCase().replaceFirst('0x', '');
      return pk == pubkeyHex;
    }).firstOrNull;

    if (wallet == null) {
      if (!context.mounted) return;
      ScaffoldMessenger.of(context).showSnackBar(
        const SnackBar(content: Text('未找到对应钱包')),
      );
      return;
    }

    if (wallet.isHotWallet) {
      if (!context.mounted) return;
      ScaffoldMessenger.of(context).showSnackBar(
        const SnackBar(content: Text('管理员仅支持冷钱包激活')),
      );
      return;
    }

    // 构建激活签名请求
    final activationService = ActivationService();
    final (:request, :json) = activationService.buildActivationRequest(
      pubkeyHex: pubkeyHex,
      identity: accountIdentity,
    );

    if (!context.mounted) return;

    // 跳转 QR 签名会话页
    final response = await Navigator.of(context).push<SignResponseEnvelope>(
      MaterialPageRoute(
        builder: (_) => QrSignSessionPage(
          request: request,
          requestJson: json,
          expectedPubkey: pubkeyHex,
        ),
      ),
    );

    if (response == null || !context.mounted) return;

    // 验证并存储激活记录
    try {
      await activationService.activateViaQr(
        pubkeyHex: pubkeyHex,
        identity: accountIdentity,
        response: response,
      );
      onActivated();
      if (!context.mounted) return;
      ScaffoldMessenger.of(context).showSnackBar(
        const SnackBar(content: Text('管理员激活成功')),
      );
    } catch (e) {
      if (!context.mounted) return;
      ScaffoldMessenger.of(context).showSnackBar(
        SnackBar(content: Text('激活失败：$e')),
      );
    }
  }

  @override
  Widget build(BuildContext context) {
    return Padding(
      padding: const EdgeInsets.only(bottom: 6),
      child: InstitutionAssignmentCard(
        assignment: assignment,
        index: index,
        balanceYuan: balanceYuan,
        trailing: _buildActivationControl(context),
      ),
    );
  }

  Widget? _buildActivationControl(BuildContext context) {
    // 激活按钮：仅对已导入冷钱包的管理员显示。
    if (!isImported) return null;
    if (isActivated) {
      return Container(
        height: InstitutionAssignmentCard.actionHeight,
        alignment: Alignment.center,
        padding: const EdgeInsets.symmetric(horizontal: 8, vertical: 4),
        decoration: BoxDecoration(
          color: AppTheme.success.withValues(alpha: 0.1),
          borderRadius: BorderRadius.circular(8),
        ),
        child: const Text(
          '已激活',
          style: TextStyle(
            fontSize: 11,
            color: AppTheme.success,
            fontWeight: FontWeight.w600,
          ),
        ),
      );
    }
    return GestureDetector(
      onTap: () => _startActivation(context),
      child: Container(
        height: InstitutionAssignmentCard.actionHeight,
        alignment: Alignment.center,
        padding: const EdgeInsets.symmetric(horizontal: 8, vertical: 4),
        decoration: BoxDecoration(
          color: AppTheme.primary.withValues(alpha: 0.1),
          borderRadius: BorderRadius.circular(8),
        ),
        child: const Text(
          '激活',
          style: TextStyle(
            fontSize: 11,
            color: AppTheme.primary,
            fontWeight: FontWeight.w600,
          ),
        ),
      ),
    );
  }
}
