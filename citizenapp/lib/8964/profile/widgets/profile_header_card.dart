import 'package:flutter/material.dart';
import 'package:flutter/services.dart';
import 'package:flutter_svg/flutter_svg.dart';

import 'package:citizenapp/8964/profile/models/citizen_profile.dart';
import 'package:citizenapp/ui/app_theme.dart';
import 'package:citizenapp/ui/identity_badge.dart';

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
    this.fallbackName = '',
    this.avatarUrl,
    this.onFollowing,
    this.onFollowers,
    this.onPosts,
  });

  final String ownerAccount;
  final CitizenProfile? profile;

  /// 展示名兜底 = 本机钱包名称（即昵称）。竞选公民认证用户由后端把
  /// `display_name` 置为链上真实姓名故优先；普通用户展示名 = 钱包名 = 昵称，
  /// 只有钱包名也缺失时才最后回落截断地址。
  final String fallbackName;

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

  /// 链上身份档位；徽章据此分色（访客橙/投票蓝/竞选红/纯访客无）。
  String? get _identityLevel => profile?.identityLevel;

  /// 会员信号（决定徽章是否带勾）。
  String? get _membershipLevel => profile?.membershipLevel;
  bool get _membershipActive => profile?.membershipActive ?? false;

  /// 展示名 = 后端 display_name（认证用户 = 链上真实姓名）→ 钱包名（昵称）
  /// → 截断地址（最后兜底）。绝不越过钱包名直接显示地址。
  String get _name {
    final resolved = profile?.resolvedDisplayName(fallbackName);
    if (resolved != null && resolved.isNotEmpty) return resolved;
    final fallback = fallbackName.trim();
    return fallback.isNotEmpty ? fallback : _shortenAccount(ownerAccount);
  }

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
            child: _Avatar(
              identityLevel: _identityLevel,
              membershipLevel: _membershipLevel,
              membershipActive: _membershipActive,
              imageUrl: avatarUrl,
              seed: ownerAccount,
            ),
          ),
        ],
      ),
    );
  }
}

class _Avatar extends StatelessWidget {
  const _Avatar({
    required this.identityLevel,
    required this.membershipLevel,
    required this.membershipActive,
    required this.seed,
    this.imageUrl,
  });

  /// 链上身份档位：颜色来源（访客橙/投票蓝/竞选红/纯访客无）。
  final String? identityLevel;

  /// 会员信号：决定徽章是否带勾（会员档匹配身份档且有效）。
  final String? membershipLevel;
  final bool membershipActive;
  final String? imageUrl;

  /// 用于给未设头像的用户稳定选一张默认头像的种子（钱包地址）。
  final String seed;

  @override
  Widget build(BuildContext context) {
    final url = imageUrl;
    final badgeStyle = identityBadgeStyle(
      identityLevel: identityLevel,
      membershipLevel: membershipLevel,
      membershipActive: membershipActive,
    );
    return Stack(
      clipBehavior: Clip.none,
      children: [
        Container(
          width: ProfileHeaderCard._avatarSize,
          height: ProfileHeaderCard._avatarSize,
          decoration: BoxDecoration(
            borderRadius: BorderRadius.circular(18),
            border: Border.all(color: AppTheme.surfaceWhite, width: 4),
          ),
          child: ClipRRect(
            borderRadius: BorderRadius.circular(14),
            child: url == null
                ? _DefaultAvatar(seed: seed)
                : Image.network(
                    url,
                    fit: BoxFit.cover,
                    width: ProfileHeaderCard._avatarSize,
                    height: ProfileHeaderCard._avatarSize,
                    errorBuilder: (_, __, ___) => _DefaultAvatar(seed: seed),
                  ),
          ),
        ),
        if (badgeStyle != null)
          Positioned(
            right: -2,
            bottom: -2,
            child: CitizenBadge(
              style: badgeStyle,
              tooltip: identityBadgeLabel(
                identityLevel: identityLevel,
                checked: badgeStyle.checked,
              ),
            ),
          ),
      ],
    );
  }
}

/// 未设头像时按账号稳定选一张不透明默认头像（不再透出头图/白底）。
class _DefaultAvatar extends StatelessWidget {
  const _DefaultAvatar({required this.seed});

  /// 可选默认头像张数，与 assets/avatars/default_1..N.svg 一致。
  static const int _count = 6;

  final String seed;

  int get _index {
    // 账号 code unit 求和取模：稳定、确定，同一账号永远同一张默认头像。
    final sum = seed.codeUnits.fold<int>(0, (acc, unit) => acc + unit);
    return sum % _count + 1;
  }

  @override
  Widget build(BuildContext context) {
    return SvgPicture.asset(
      'assets/avatars/default_$_index.svg',
      width: ProfileHeaderCard._avatarSize,
      height: ProfileHeaderCard._avatarSize,
      fit: BoxFit.cover,
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
