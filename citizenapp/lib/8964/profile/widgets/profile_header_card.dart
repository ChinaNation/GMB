import 'package:flutter/material.dart';
import 'package:flutter/services.dart';

import 'package:citizenapp/8964/profile/models/citizen_profile.dart';
import 'package:citizenapp/ui/app_theme.dart';

/// 推特式资料卡：头图下方白底，圆角方形头像跨压头图下缘 + 认证勾 +
/// 展示名/地址·CID/签名/计数 + 右上三图标。
///
/// 头像用 [Positioned] 上移半个身位跨到头图上；文字为深色（落在白底）；
/// 数据来自已加载的 [profile]（可空 → 占位）。
class ProfileHeaderCard extends StatelessWidget {
  const ProfileHeaderCard({
    super.key,
    required this.ownerAccount,
    required this.profile,
    required this.actions,
    this.avatarUrl,
    this.onFollowing,
    this.onFollowers,
    this.onPosts,
  });

  final String ownerAccount;
  final CitizenProfile? profile;

  /// 头像图片 URL（object_key 解析后的公开媒体地址）；为空显示占位。
  final String? avatarUrl;

  /// 右上三图标（[ProfileActionIcons]）。
  final Widget actions;

  final VoidCallback? onFollowing;
  final VoidCallback? onFollowers;
  final VoidCallback? onPosts;

  /// 头像尺寸；上移半个身位跨压头图。
  static const double _avatarSize = 80;
  static const double _avatarOverlap = 40;

  bool get _isCertified => profile?.isCertified ?? false;

  String get _name =>
      profile?.resolvedDisplayName('') ?? _shortenAccount(ownerAccount);

  @override
  Widget build(BuildContext context) {
    final bio = profile?.bio.trim() ?? '';
    return ColoredBox(
      color: AppTheme.surfaceWhite,
      child: Stack(
        clipBehavior: Clip.none,
        children: [
          Padding(
            padding: const EdgeInsets.fromLTRB(16, 0, 16, 16),
            child: Column(
              mainAxisSize: MainAxisSize.min,
              crossAxisAlignment: CrossAxisAlignment.start,
              children: [
                // 头像下半部与右上三图标同处一带；头像另由 Positioned 跨压头图。
                SizedBox(
                  height: _avatarOverlap + 4,
                  child: Align(
                    alignment: Alignment.centerRight,
                    child: Padding(
                      padding: const EdgeInsets.only(top: 10),
                      child: actions,
                    ),
                  ),
                ),
                const SizedBox(height: 8),
                Text(
                  _name,
                  maxLines: 1,
                  overflow: TextOverflow.ellipsis,
                  style: const TextStyle(
                    color: AppTheme.textPrimary,
                    fontSize: 18,
                    fontWeight: FontWeight.w600,
                  ),
                ),
                const SizedBox(height: 3),
                _AddressRow(
                  ownerAccount: ownerAccount,
                  cidNumber: profile?.cidNumber,
                ),
                if (bio.isNotEmpty) ...[
                  const SizedBox(height: 8),
                  Text(
                    bio,
                    maxLines: 2,
                    overflow: TextOverflow.ellipsis,
                    style: const TextStyle(
                      color: AppTheme.textSecondary,
                      fontSize: 13,
                      height: 1.45,
                    ),
                  ),
                ],
                const SizedBox(height: 12),
                Row(
                  children: [
                    _Stat(
                      value: profile?.following ?? 0,
                      label: '关注',
                      onTap: onFollowing,
                    ),
                    const SizedBox(width: 18),
                    _Stat(
                      value: profile?.followers ?? 0,
                      label: '关注者',
                      onTap: onFollowers,
                    ),
                    const SizedBox(width: 18),
                    _Stat(
                      value: profile?.posts ?? 0,
                      label: '帖子',
                      onTap: onPosts,
                    ),
                  ],
                ),
              ],
            ),
          ),
          Positioned(
            left: 16,
            top: -_avatarOverlap,
            child: _Avatar(isCertified: _isCertified, imageUrl: avatarUrl),
          ),
        ],
      ),
    );
  }
}

class _Avatar extends StatelessWidget {
  const _Avatar({required this.isCertified, this.imageUrl});

  final bool isCertified;
  final String? imageUrl;

  @override
  Widget build(BuildContext context) {
    final url = imageUrl;
    return Stack(
      clipBehavior: Clip.none,
      children: [
        Container(
          width: ProfileHeaderCard._avatarSize,
          height: ProfileHeaderCard._avatarSize,
          decoration: BoxDecoration(
            color: AppTheme.primary.withAlpha(20),
            borderRadius: BorderRadius.circular(18),
            border: Border.all(color: AppTheme.surfaceWhite, width: 4),
          ),
          child: ClipRRect(
            borderRadius: BorderRadius.circular(14),
            child: url == null
                ? const _AvatarPlaceholder()
                : Image.network(
                    url,
                    fit: BoxFit.cover,
                    width: ProfileHeaderCard._avatarSize,
                    height: ProfileHeaderCard._avatarSize,
                    errorBuilder: (_, __, ___) => const _AvatarPlaceholder(),
                  ),
          ),
        ),
        if (isCertified)
          Positioned(
            right: -2,
            bottom: -2,
            child: Container(
              width: 24,
              height: 24,
              decoration: const BoxDecoration(
                color: AppTheme.surfaceWhite,
                shape: BoxShape.circle,
              ),
              child: const Icon(
                Icons.verified,
                size: 22,
                color: Color(0xFF007A74),
              ),
            ),
          ),
      ],
    );
  }
}

class _AvatarPlaceholder extends StatelessWidget {
  const _AvatarPlaceholder();

  @override
  Widget build(BuildContext context) {
    return const SizedBox(
      width: ProfileHeaderCard._avatarSize,
      height: ProfileHeaderCard._avatarSize,
      child: Icon(Icons.person, size: 36, color: AppTheme.primary),
    );
  }
}

class _AddressRow extends StatelessWidget {
  const _AddressRow({required this.ownerAccount, required this.cidNumber});

  final String ownerAccount;
  final String? cidNumber;

  @override
  Widget build(BuildContext context) {
    final cid = cidNumber?.trim() ?? '';
    return Row(
      children: [
        Flexible(
          child: Text(
            _shortenAccount(ownerAccount),
            maxLines: 1,
            overflow: TextOverflow.ellipsis,
            style: const TextStyle(color: AppTheme.textTertiary, fontSize: 12),
          ),
        ),
        const SizedBox(width: 4),
        InkWell(
          onTap: () {
            Clipboard.setData(ClipboardData(text: ownerAccount));
            ScaffoldMessenger.of(context).showSnackBar(
              const SnackBar(content: Text('地址已复制')),
            );
          },
          child: const Padding(
            padding: EdgeInsets.all(2),
            child: Icon(Icons.copy, size: 13, color: AppTheme.textTertiary),
          ),
        ),
        if (cid.isNotEmpty) ...[
          const SizedBox(width: 6),
          Flexible(
            child: Text(
              '· $cid',
              maxLines: 1,
              overflow: TextOverflow.ellipsis,
              style:
                  const TextStyle(color: AppTheme.textTertiary, fontSize: 11),
            ),
          ),
        ],
      ],
    );
  }
}

class _Stat extends StatelessWidget {
  const _Stat({required this.value, required this.label, this.onTap});

  final int value;
  final String label;
  final VoidCallback? onTap;

  @override
  Widget build(BuildContext context) {
    return InkWell(
      onTap: onTap,
      child: Text.rich(
        TextSpan(
          children: [
            TextSpan(
              text: '$value ',
              style: const TextStyle(
                color: AppTheme.textPrimary,
                fontSize: 13,
                fontWeight: FontWeight.w600,
              ),
            ),
            TextSpan(
              text: label,
              style: const TextStyle(
                color: AppTheme.textSecondary,
                fontSize: 13,
              ),
            ),
          ],
        ),
      ),
    );
  }
}

String _shortenAccount(String account) {
  if (account.length <= 12) return account;
  return '${account.substring(0, 6)}...'
      '${account.substring(account.length - 6)}';
}
