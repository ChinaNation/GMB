import 'package:flutter/material.dart';
import 'package:flutter/services.dart';

import 'package:citizenapp/8964/profile/models/citizen_profile.dart';

/// 展开态资料卡：圆角方形头像 + 认证勾 + 展示名/签名/计数 + 右上三图标。
///
/// 阶段 4 用占位头像（阶段 6 换真图）；数据来自已加载的 [profile]（可空 → 占位）。
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

  bool get _isCertified => profile?.isCertified ?? false;

  String get _name =>
      profile?.resolvedDisplayName('') ?? _shortenAccount(ownerAccount);

  @override
  Widget build(BuildContext context) {
    final bio = profile?.bio.trim() ?? '';
    return Padding(
      padding: const EdgeInsets.fromLTRB(16, 0, 16, 14),
      child: Column(
        mainAxisSize: MainAxisSize.min,
        crossAxisAlignment: CrossAxisAlignment.start,
        children: [
          Row(
            crossAxisAlignment: CrossAxisAlignment.start,
            children: [
              _Avatar(isCertified: _isCertified, imageUrl: avatarUrl),
              const Spacer(),
              Padding(
                padding: const EdgeInsets.only(top: 6),
                child: actions,
              ),
            ],
          ),
          const SizedBox(height: 8),
          Row(
            children: [
              Flexible(
                child: Text(
                  _name,
                  maxLines: 1,
                  overflow: TextOverflow.ellipsis,
                  style: const TextStyle(
                    color: Colors.white,
                    fontSize: 18,
                    fontWeight: FontWeight.w600,
                  ),
                ),
              ),
              if (_isCertified) const SizedBox(width: 8),
              if (_isCertified) const _CertifiedPill(),
            ],
          ),
          const SizedBox(height: 3),
          _AddressRow(
              ownerAccount: ownerAccount, cidNumber: profile?.cidNumber),
          if (bio.isNotEmpty) ...[
            const SizedBox(height: 8),
            Text(
              bio,
              maxLines: 3,
              overflow: TextOverflow.ellipsis,
              style: const TextStyle(
                color: Colors.white,
                fontSize: 13,
                height: 1.4,
              ),
            ),
          ],
          const SizedBox(height: 10),
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
          width: 64,
          height: 64,
          decoration: BoxDecoration(
            color: Colors.white24,
            borderRadius: BorderRadius.circular(15),
            border: Border.all(color: Colors.white, width: 3),
          ),
          child: ClipRRect(
            borderRadius: BorderRadius.circular(12),
            child: url == null
                ? const _AvatarPlaceholder()
                : Image.network(
                    url,
                    fit: BoxFit.cover,
                    width: 64,
                    height: 64,
                    errorBuilder: (_, __, ___) => const _AvatarPlaceholder(),
                  ),
          ),
        ),
        if (isCertified)
          Positioned(
            right: -4,
            bottom: -4,
            child: Container(
              width: 22,
              height: 22,
              decoration: const BoxDecoration(
                color: Colors.white,
                shape: BoxShape.circle,
              ),
              child: const Icon(
                Icons.verified,
                size: 20,
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
      width: 64,
      height: 64,
      child: Icon(Icons.person, size: 30, color: Colors.white),
    );
  }
}

class _CertifiedPill extends StatelessWidget {
  const _CertifiedPill();

  @override
  Widget build(BuildContext context) {
    return Container(
      padding: const EdgeInsets.symmetric(horizontal: 8, vertical: 2),
      decoration: BoxDecoration(
        color: Colors.white24,
        borderRadius: BorderRadius.circular(20),
      ),
      child: const Text(
        '认证公民',
        style: TextStyle(color: Colors.white, fontSize: 11),
      ),
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
            style: const TextStyle(color: Colors.white70, fontSize: 12),
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
            child: Icon(Icons.copy, size: 13, color: Colors.white70),
          ),
        ),
        if (cid.isNotEmpty) ...[
          const SizedBox(width: 6),
          Flexible(
            child: Text(
              '· $cid',
              maxLines: 1,
              overflow: TextOverflow.ellipsis,
              style: const TextStyle(color: Colors.white70, fontSize: 11),
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
                color: Colors.white,
                fontSize: 13,
                fontWeight: FontWeight.w600,
              ),
            ),
            TextSpan(
              text: label,
              style: const TextStyle(color: Colors.white70, fontSize: 13),
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
