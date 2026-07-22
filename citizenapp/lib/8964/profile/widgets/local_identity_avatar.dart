import 'dart:io';

import 'package:flutter/material.dart';

import 'package:citizenapp/8964/profile/models/profile_presentation.dart';
import 'package:citizenapp/ui/app_theme.dart';
import 'package:citizenapp/ui/identity_badge.dart';

/// 本地头像 + 右下角扇贝身份徽章的统一头像组件。
///
/// 头像图取本地文件 [path]（用户在「我的」里设置的头像，与用户主页同源）；未设或文件
/// 失效时按 [seed]（默认钱包地址）稳定选默认头像资源。徽章信号：颜色=链上身份档、
/// 勾=会员匹配身份档。「我的」tab 与广场顶部共用本组件，避免同一用户展示成两套视觉身份。
class LocalIdentityAvatar extends StatelessWidget {
  const LocalIdentityAvatar({
    super.key,
    required this.path,
    required this.size,
    required this.seed,
    this.identityLevel,
    this.membershipLevel,
    this.membershipActive = false,
    this.badgeSize = 24,
    this.circular = false,
  });

  final String? path;
  final double size;

  /// 未设头像时按账号稳定选默认头像的种子（默认钱包地址，与用户主页同源）。
  final String seed;

  /// 徽章信号：颜色=链上身份档、勾=会员匹配身份档。
  final String? identityLevel;
  final String? membershipLevel;
  final bool membershipActive;

  /// 扇贝徽章尺寸；默认 24 与「我的」tab 一致，广场顶部可传更小值。
  final double badgeSize;

  /// true=正圆头像（广场顶部用），false=圆角方形（默认，「我的」tab / feed 同款身份视觉）。
  final bool circular;

  @override
  Widget build(BuildContext context) {
    final hasImage = path != null && path!.trim().isNotEmpty;
    final file = hasImage ? File(path!) : null;
    final validImage = file != null && file.existsSync();
    final badgeStyle = identityBadgeStyle(
      identityLevel: identityLevel,
      membershipLevel: membershipLevel,
      membershipActive: membershipActive,
    );
    // 圆形时角半径=半边长；徽章由 -4（方形贴角外）收到 0（贴圆下缘）。
    final radius = circular ? size / 2 : 10.0;
    final badgeInset = circular ? 0.0 : -4.0;

    return SizedBox(
      width: size,
      height: size,
      child: Stack(
        clipBehavior: Clip.none,
        children: [
          Container(
            width: size,
            height: size,
            decoration: BoxDecoration(
              color: AppTheme.primary.withAlpha(20),
              borderRadius: BorderRadius.circular(radius),
            ),
            child: ClipRRect(
              borderRadius: BorderRadius.circular(radius),
              child: validImage
                  ? Image.file(file, fit: BoxFit.cover)
                  : Image.asset(
                      ProfilePresentation.forAccount(seed).avatarAsset,
                      width: size,
                      height: size,
                      fit: BoxFit.cover,
                    ),
            ),
          ),
          if (badgeStyle != null)
            Positioned(
              right: badgeInset,
              bottom: badgeInset,
              child: IdentityBadge(
                style: badgeStyle,
                size: badgeSize,
                tooltip: identityBadgeLabel(
                  identityLevel: identityLevel,
                  checked: badgeStyle.checked,
                ),
              ),
            ),
        ],
      ),
    );
  }
}
