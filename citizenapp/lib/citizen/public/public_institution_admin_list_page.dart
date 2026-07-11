import 'dart:async';

import 'package:flutter/material.dart';

import 'package:citizenapp/citizen/shared/admin_profile.dart';
import 'package:citizenapp/citizen/shared/admin_profile_card.dart';
import 'package:citizenapp/rpc/chain_rpc.dart';
import 'package:citizenapp/ui/app_theme.dart';

/// 公权机构管理员列表页(只读)。
///
/// **只读展示**链上 PublicAdmins::AdminAccounts 的管理员实名资料(A2:
/// 姓名/职务/任期/来源/实名 CID + 账户 SS58,prefix=2027);不做冷钱包导入/扫码激活
/// ——那是治理机构 `AdminListPage` 的能力,公权端本期不引入重型桥接。无管理员时显示占位。
class PublicInstitutionAdminListPage extends StatefulWidget {
  const PublicInstitutionAdminListPage({
    super.key,
    required this.admins,
  });

  /// 管理员完整资料;来自链上 PublicAdmins::AdminAccounts(A2 AdminProfile)。
  final List<AdminProfile> admins;

  @override
  State<PublicInstitutionAdminListPage> createState() =>
      _PublicInstitutionAdminListPageState();
}

class _PublicInstitutionAdminListPageState
    extends State<PublicInstitutionAdminListPage> {
  Map<String, double> _balanceByAccount = const {};

  @override
  void initState() {
    super.initState();
    unawaited(_loadBalances());
  }

  @override
  void didUpdateWidget(covariant PublicInstitutionAdminListPage oldWidget) {
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
      for (final profile in widget.admins) _balanceKey(profile.account),
    }.where((account) => account.isNotEmpty).toList(growable: false);
    if (accounts.isEmpty) {
      if (mounted) setState(() => _balanceByAccount = const {});
      return;
    }
    try {
      final balances = await ChainRpc().fetchFinalizedBalances(accounts);
      if (mounted) setState(() => _balanceByAccount = balances);
    } catch (_) {
      // 只读管理员列表的余额失败不影响资料展示。
      if (mounted) setState(() => _balanceByAccount = const {});
    }
  }

  @override
  Widget build(BuildContext context) {
    return Scaffold(
      backgroundColor: AppTheme.scaffoldBg,
      appBar: AppBar(
        title: const Text(
          '管理员列表',
          style: TextStyle(fontSize: 17, fontWeight: FontWeight.w700),
        ),
        centerTitle: true,
        backgroundColor: AppTheme.surfaceCard,
        foregroundColor: AppTheme.textPrimary,
        elevation: 0,
      ),
      body: widget.admins.isEmpty
          ? _emptyState()
          : ListView.separated(
              padding: const EdgeInsets.all(16),
              itemCount: widget.admins.length,
              separatorBuilder: (_, __) => const SizedBox(height: 10),
              itemBuilder: (context, i) {
                final profile = widget.admins[i];
                return AdminProfileCard(
                  profile: profile,
                  index: i + 1,
                  balanceYuan: _balanceByAccount[_balanceKey(profile.account)],
                );
              },
            ),
    );
  }

  Widget _emptyState() {
    return const Center(
      child: Padding(
        padding: EdgeInsets.all(32),
        child: Column(
          mainAxisSize: MainAxisSize.min,
          children: [
            Icon(Icons.group_outlined, size: 44, color: AppTheme.textTertiary),
            SizedBox(height: 12),
            Text('暂无管理员',
                style: TextStyle(fontSize: 14, color: AppTheme.textSecondary)),
            SizedBox(height: 6),
            Text(
              '该机构链上暂无管理员资料',
              textAlign: TextAlign.center,
              style: TextStyle(fontSize: 12.5, color: AppTheme.textTertiary),
            ),
          ],
        ),
      ),
    );
  }
}
