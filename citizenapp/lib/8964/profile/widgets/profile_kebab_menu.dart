import 'package:flutter/material.dart';

import 'package:citizenapp/ui/app_theme.dart';

enum ProfileMenuAction { qrCode, editProfile, report, deleteAccount }

/// 右上角竖三点菜单（决策 4）：二维码常驻；编辑资料 / 注销用户仅本人；举报仅他人。
class ProfileKebabMenu extends StatelessWidget {
  const ProfileKebabMenu({
    super.key,
    required this.isSelf,
    this.onQrCode,
    this.onEditProfile,
    this.onReport,
    this.onDeleteAccount,
  });

  final bool isSelf;
  final VoidCallback? onQrCode;
  final VoidCallback? onEditProfile;
  final VoidCallback? onReport;

  /// 注销用户（仅本人，破坏性）：硬删除该用户在 Cloudflare 的全部数据。
  final VoidCallback? onDeleteAccount;

  @override
  Widget build(BuildContext context) {
    return PopupMenuButton<ProfileMenuAction>(
      icon: const Icon(Icons.more_vert),
      onSelected: (action) {
        switch (action) {
          case ProfileMenuAction.qrCode:
            onQrCode?.call();
            break;
          case ProfileMenuAction.editProfile:
            onEditProfile?.call();
            break;
          case ProfileMenuAction.report:
            onReport?.call();
            break;
          case ProfileMenuAction.deleteAccount:
            onDeleteAccount?.call();
            break;
        }
      },
      itemBuilder: (context) => [
        const PopupMenuItem(
          value: ProfileMenuAction.qrCode,
          child: _MenuRow(icon: Icons.qr_code_2, label: '二维码'),
        ),
        if (isSelf)
          const PopupMenuItem(
            value: ProfileMenuAction.editProfile,
            child: _MenuRow(icon: Icons.edit_outlined, label: '编辑资料'),
          ),
        if (!isSelf)
          const PopupMenuItem(
            value: ProfileMenuAction.report,
            child: _MenuRow(icon: Icons.flag_outlined, label: '举报'),
          ),
        // 注销放末位（破坏性），仅本人可见，红色区分。
        if (isSelf)
          const PopupMenuItem(
            value: ProfileMenuAction.deleteAccount,
            child: _MenuRow(
              icon: Icons.no_accounts,
              label: '注销用户',
              color: AppTheme.danger,
            ),
          ),
      ],
    );
  }
}

class _MenuRow extends StatelessWidget {
  const _MenuRow({required this.icon, required this.label, this.color});

  final IconData icon;
  final String label;

  /// 行主色；破坏性项传 [AppTheme.danger]，其余用默认次要色。
  final Color? color;

  @override
  Widget build(BuildContext context) {
    final rowColor = color ?? AppTheme.textSecondary;
    return Row(
      children: [
        Icon(icon, size: 20, color: rowColor),
        const SizedBox(width: 12),
        Text(label, style: TextStyle(color: color)),
      ],
    );
  }
}
