import 'package:flutter/material.dart';

import 'package:citizenapp/8964/profile/models/profile_presentation.dart';
import 'package:citizenapp/ui/app_theme.dart';
import 'package:citizenapp/ui/identity_badge.dart';

/// 用户公开资料统一头像。用户主页与通讯录共用同一套圆角、默认头像和身份徽章，
/// 避免不同入口把同一个用户展示成两套视觉身份。
class ProfileAvatar extends StatelessWidget {
  const ProfileAvatar({
    super.key,
    required this.seed,
    required this.size,
    this.imageUrl,
    this.imageHeaders,
    this.identityLevel,
    this.membershipLevel,
    this.membershipActive = false,
    this.borderColor,
    this.borderWidth = 0,
    this.borderRadius,
  });

  final String seed;
  final double size;
  final String? imageUrl;
  final Map<String, String>? imageHeaders;
  final String? identityLevel;
  final String? membershipLevel;
  final bool membershipActive;
  final Color? borderColor;
  final double borderWidth;
  final double? borderRadius;

  @override
  Widget build(BuildContext context) {
    final radius = borderRadius ?? size * 0.22;
    final badge = identityBadgeStyle(
      identityLevel: identityLevel,
      membershipLevel: membershipLevel,
      membershipActive: membershipActive,
    );
    final url = imageUrl?.trim();
    return Stack(
      clipBehavior: Clip.none,
      children: [
        Container(
          width: size,
          height: size,
          decoration: BoxDecoration(
            borderRadius: BorderRadius.circular(radius + borderWidth),
            border: borderWidth <= 0
                ? null
                : Border.all(
                    color: borderColor ?? AppTheme.surfaceCard,
                    width: borderWidth,
                  ),
          ),
          child: ClipRRect(
            borderRadius: BorderRadius.circular(radius),
            child: url == null || url.isEmpty
                ? _StableDefaultAvatar(seed: seed, size: size)
                : Image.network(
                    url,
                    headers: imageHeaders,
                    fit: BoxFit.cover,
                    width: size,
                    height: size,
                    errorBuilder: (_, __, ___) =>
                        _StableDefaultAvatar(seed: seed, size: size),
                  ),
          ),
        ),
        if (badge != null)
          Positioned(
            right: -2,
            bottom: -2,
            child: IdentityBadge(
              style: badge,
              size: (size * 0.34).clamp(18, 28),
              tooltip: identityBadgeLabel(
                identityLevel: identityLevel,
                checked: badge.checked,
              ),
            ),
          ),
      ],
    );
  }
}

class _StableDefaultAvatar extends StatelessWidget {
  const _StableDefaultAvatar({required this.seed, required this.size});

  final String seed;
  final double size;

  @override
  Widget build(BuildContext context) {
    return Image.asset(
      ProfilePresentation.forAccount(seed).avatarAsset,
      width: size,
      height: size,
      fit: BoxFit.cover,
    );
  }
}
