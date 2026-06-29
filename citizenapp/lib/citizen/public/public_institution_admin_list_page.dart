import 'dart:typed_data';

import 'package:flutter/material.dart';

import 'package:citizenapp/citizen/shared/account_derivation.dart';
import 'package:citizenapp/citizen/shared/admin_profile.dart';
import 'package:citizenapp/ui/app_theme.dart';

/// 公权机构管理员列表页(只读)。
///
/// 中文注释:**只读展示**链上 PublicAdmins::AdminAccounts 的管理员实名资料(A2:
/// 姓名/职务/任期/来源/实名 CID + 账户 SS58,prefix=2027);不做冷钱包导入/扫码激活
/// ——那是治理机构 `AdminListPage` 的能力,公权端本期不引入重型桥接。无管理员时显示占位。
class PublicInstitutionAdminListPage extends StatelessWidget {
  const PublicInstitutionAdminListPage({
    super.key,
    required this.admins,
  });

  /// 管理员完整资料;来自链上 PublicAdmins::AdminAccounts(A2 AdminProfile)。
  final List<AdminProfile> admins;

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
        backgroundColor: AppTheme.surfaceWhite,
        foregroundColor: AppTheme.textPrimary,
        elevation: 0,
      ),
      body: admins.isEmpty
          ? _emptyState()
          : ListView.separated(
              padding: const EdgeInsets.all(16),
              itemCount: admins.length,
              separatorBuilder: (_, __) => const SizedBox(height: 10),
              itemBuilder: (context, i) => _adminCard(i + 1, admins[i]),
            ),
    );
  }

  Widget _adminCard(int index, AdminProfile profile) {
    final hasIdentity = profile.name.isNotEmpty || profile.title.isNotEmpty;
    return Container(
      padding: const EdgeInsets.all(14),
      decoration: BoxDecoration(
        color: AppTheme.surfaceCard,
        borderRadius: BorderRadius.circular(12),
        border: Border.all(color: AppTheme.border),
      ),
      child: Row(
        crossAxisAlignment: CrossAxisAlignment.start,
        children: [
          Container(
            width: 28,
            height: 28,
            alignment: Alignment.center,
            decoration: BoxDecoration(
              color: AppTheme.primary.withValues(alpha: 0.10),
              borderRadius: BorderRadius.circular(8),
            ),
            child: Text('$index',
                style: const TextStyle(
                    fontSize: 13,
                    fontWeight: FontWeight.w700,
                    color: AppTheme.primary)),
          ),
          const SizedBox(width: 10),
          Expanded(
            child: Column(
              crossAxisAlignment: CrossAxisAlignment.start,
              children: [
                // 姓名 + 职务 + 来源(实名资料)。
                if (hasIdentity)
                  Wrap(
                    spacing: 6,
                    crossAxisAlignment: WrapCrossAlignment.center,
                    children: [
                      if (profile.name.isNotEmpty)
                        Text(profile.name,
                            style: const TextStyle(
                                fontSize: 13.5,
                                fontWeight: FontWeight.w700,
                                color: AppTheme.textPrimary)),
                      if (profile.title.isNotEmpty)
                        Text(profile.title,
                            style: const TextStyle(
                                fontSize: 11, color: AppTheme.textTertiary)),
                      if (profile.source != AdminProfileSource.unknown)
                        Text(profile.source.label,
                            style: const TextStyle(
                                fontSize: 11,
                                color: AppTheme.primary,
                                fontWeight: FontWeight.w600)),
                    ],
                  )
                else
                  const Text('管理员',
                      style: TextStyle(
                          fontSize: 11, color: AppTheme.textTertiary)),
                if (profile.cidNumber.isNotEmpty)
                  Padding(
                    padding: const EdgeInsets.only(top: 3),
                    child: Text('实名 ${profile.cidNumber}',
                        style: const TextStyle(
                            fontSize: 11, color: AppTheme.textTertiary)),
                  ),
                if (profile.termLabel.isNotEmpty)
                  Padding(
                    padding: const EdgeInsets.only(top: 3),
                    child: Text('任期 ${profile.termLabel}',
                        style: const TextStyle(
                            fontSize: 11, color: AppTheme.textTertiary)),
                  ),
                const SizedBox(height: 3),
                // 完整 SS58 地址,允许换行,不截断。
                Text(_formatAddress(profile.account),
                    style: const TextStyle(
                        fontSize: 12,
                        fontFamily: 'monospace',
                        color: AppTheme.textSecondary)),
              ],
            ),
          ),
        ],
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

  /// hex 公钥 → SS58(prefix=2027)。非法 hex 兜底原样展示,绝不抛。
  String _formatAddress(String pubkeyHex) {
    final clean =
        pubkeyHex.startsWith('0x') ? pubkeyHex.substring(2) : pubkeyHex;
    if (clean.isEmpty || clean.length.isOdd) return pubkeyHex;
    try {
      final bytes = Uint8List(clean.length ~/ 2);
      for (var i = 0; i < bytes.length; i++) {
        bytes[i] = int.parse(clean.substring(i * 2, i * 2 + 2), radix: 16);
      }
      return ss58FromAccountId(bytes);
    } on FormatException {
      return pubkeyHex;
    }
  }
}
