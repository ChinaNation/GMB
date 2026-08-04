import 'package:flutter/material.dart';
import 'package:flutter/services.dart';

import 'package:citizenapp/8964/profile/models/citizen_profile.dart';
import 'package:citizenapp/8964/profile/models/profile_presentation.dart';
import 'package:citizenapp/8964/profile/widgets/profile_avatar.dart';
import 'package:citizenapp/citizen/shared/account_derivation.dart';
import 'package:citizenapp/ui/app_theme.dart';

/// 推特式资料卡：头图下方白底，圆角方形头像跨压头图下缘 + 认证勾 +
/// 展示名/SS58/CID/签名/计数 + 右上三图标。
///
/// 头像用 [Positioned] 上移半个身位跨到头图上；文字为深色（落在白底）；
/// 数据来自已加载的 [profile]（可空 → 占位）。
class ProfileHeaderCard extends StatelessWidget {
  const ProfileHeaderCard({
    super.key,
    required this.cidNumber,
    required this.profile,
    required this.actions,
    this.avatarUrl,
    this.avatarHeaders,
    this.onFollowing,
    this.onFollowers,
    this.onPosts,
    this.creatorSubscribeButton,
  });

  /// 主页身份主键 cid_number（默认昵称/头像的稳定派生种子）。展示用的当前绑定
  /// 钱包账户从 [profile].accountId 取。
  final String cidNumber;
  final CitizenProfile? profile;

  /// 头像图片 URL（object_key 解析后的公开媒体地址）；为空显示占位。
  final String? avatarUrl;
  final Map<String, String>? avatarHeaders;

  /// 右上三图标（[ProfileActionIcons]）。
  final Widget actions;

  /// 他人主页「订阅 TA / 取消」按钮（[CreatorSubscribeButton]）；本人主页传 null 不显示。
  final Widget? creatorSubscribeButton;

  final VoidCallback? onFollowing;
  final VoidCallback? onFollowers;
  final VoidCallback? onPosts;

  /// 头像尺寸；上移半个身位跨压头图。
  static const double _avatarSize = 80;
  static const double _avatarOverlap = 40;

  /// 链上身份档位；无有效会员时徽章据此分色（访客金/投票蓝/竞选红）。
  String? get _identityLevel => profile?.identityLevel;

  /// 会员信号；有效态与合法档位共同决定会员徽章的档位色和对勾。
  String? get _membershipLevel => profile?.membershipLevel;
  bool get _membershipActive => profile?.membershipActive ?? false;

  String get _name {
    return ProfilePresentation.forIdentityKey(cidNumber).resolveDisplayName(
      publicName: profile?.displayName,
    );
  }

  @override
  Widget build(BuildContext context) {
    final bio = profile?.bio.trim() ?? '';
    return ColoredBox(
      color: AppTheme.surfaceCard,
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
                _IdentityDetails(
                  accountId: profile?.accountId ?? '',
                  cidNumber: cidNumber,
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
                if (creatorSubscribeButton != null) ...[
                  const SizedBox(height: 12),
                  creatorSubscribeButton!,
                ],
              ],
            ),
          ),
          Positioned(
            left: 16,
            top: -_avatarOverlap,
            child: ProfileAvatar(
              size: _avatarSize,
              identityLevel: _identityLevel,
              membershipLevel: _membershipLevel,
              membershipActive: _membershipActive,
              imageUrl: avatarUrl,
              imageHeaders: avatarHeaders,
              seed: cidNumber,
              borderColor: AppTheme.surfaceCard,
              borderWidth: 4,
              borderRadius: 14,
            ),
          ),
        ],
      ),
    );
  }
}

class _IdentityDetails extends StatelessWidget {
  const _IdentityDetails({
    required this.accountId,
    required this.cidNumber,
  });

  final String accountId;
  final String cidNumber;

  /// SS58 只由规范 AccountId 即时派生，不从资料响应读取第二份地址。
  ///
  /// 公开资料尚未加载或服务端返回非法 AccountId 时从严降级，不显示复制入口，
  /// 更不能把原始 AccountId 冒充为 SS58。
  String? get _ss58Address {
    final normalized = accountId.trim();
    if (normalized.isEmpty) return null;
    try {
      return ss58FromAccountIdText(normalized);
    } on FormatException {
      return null;
    }
  }

  @override
  Widget build(BuildContext context) {
    final ss58Address = _ss58Address;
    final cid = cidNumber.trim();
    return Column(
      crossAxisAlignment: CrossAxisAlignment.start,
      children: [
        Row(
          children: [
            Expanded(
              child: Text(
                ss58Address == null ? 'SS58：暂不可用' : 'SS58：$ss58Address',
                maxLines: 1,
                overflow: TextOverflow.ellipsis,
                style: const TextStyle(
                  color: AppTheme.textTertiary,
                  fontSize: 12,
                ),
              ),
            ),
            if (ss58Address != null) ...[
              const SizedBox(width: 4),
              IconButton(
                tooltip: '复制 SS58 地址',
                visualDensity: VisualDensity.compact,
                constraints: const BoxConstraints.tightFor(
                  width: 28,
                  height: 28,
                ),
                padding: EdgeInsets.zero,
                onPressed: () {
                  Clipboard.setData(ClipboardData(text: ss58Address));
                  ScaffoldMessenger.of(context).showSnackBar(
                    const SnackBar(content: Text('SS58 地址已复制')),
                  );
                },
                icon: const Icon(
                  Icons.copy,
                  size: 14,
                  color: AppTheme.textTertiary,
                ),
              ),
            ],
          ],
        ),
        Text(
          cid.isEmpty ? 'CID：暂不可用' : 'CID：$cid',
          maxLines: 1,
          overflow: TextOverflow.ellipsis,
          style: const TextStyle(
            color: AppTheme.textTertiary,
            fontSize: 12,
          ),
        ),
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
