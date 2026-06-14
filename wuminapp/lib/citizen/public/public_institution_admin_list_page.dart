import 'dart:typed_data';

import 'package:flutter/material.dart';

import 'package:wuminapp_mobile/governance/shared/account_derivation.dart';
import 'package:wuminapp_mobile/ui/app_theme.dart';

/// 公权机构管理员列表页(只读)。
///
/// 中文注释:公权机构管理员来自 SFID 系统、尚未对接,本页**只读展示**链上已有的
/// 管理员公钥(转 SS58,prefix=2027),不做冷钱包导入/扫码激活——那是治理机构
/// `AdminListPage` 的能力,公权端本期不引入重型桥接。无管理员时显示占位文案。
class PublicInstitutionAdminListPage extends StatelessWidget {
  const PublicInstitutionAdminListPage({
    super.key,
    required this.institutionName,
    required this.admins,
  });

  /// 机构展示名(简称优先,由调用方决定)。
  final String institutionName;

  /// 管理员公钥列表(hex,可能带 0x);来自链上 AdminsChange::AdminAccounts。
  final List<String> admins;

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
              '管理员数据待与 SFID 系统对接',
              textAlign: TextAlign.center,
              style: TextStyle(fontSize: 12.5, color: AppTheme.textTertiary),
            ),
          ],
        ),
      ),
    );
  }

  Widget _adminCard(int index, String pubkeyHex) {
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
                const Text('管理员',
                    style:
                        TextStyle(fontSize: 11, color: AppTheme.textTertiary)),
                const SizedBox(height: 3),
                // 完整 SS58 地址,允许换行,不截断。
                Text(_formatAddress(pubkeyHex),
                    style: const TextStyle(
                        fontSize: 12.5,
                        color: AppTheme.textPrimary,
                        fontWeight: FontWeight.w600)),
              ],
            ),
          ),
        ],
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
