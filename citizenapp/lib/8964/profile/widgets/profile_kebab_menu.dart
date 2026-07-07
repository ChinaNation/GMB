import 'package:flutter/material.dart';

import 'package:citizenapp/ui/app_theme.dart';

enum ProfileMenuAction { qrCode, editProfile, report }

/// 右上角竖三点菜单（决策 4）：二维码常驻；编辑资料仅本人；举报仅他人。
class ProfileKebabMenu extends StatelessWidget {
  const ProfileKebabMenu({
    super.key,
    required this.isSelf,
    this.onQrCode,
    this.onEditProfile,
    this.onReport,
  });

  final bool isSelf;
  final VoidCallback? onQrCode;
  final VoidCallback? onEditProfile;
  final VoidCallback? onReport;

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
      ],
    );
  }
}

class _MenuRow extends StatelessWidget {
  const _MenuRow({required this.icon, required this.label});

  final IconData icon;
  final String label;

  @override
  Widget build(BuildContext context) {
    return Row(
      children: [
        Icon(icon, size: 20, color: AppTheme.textSecondary),
        const SizedBox(width: 12),
        Text(label),
      ],
    );
  }
}
