import 'package:citizenapp/8964/profile/models/profile_presentation.dart';

/// 用户主页公开资料模型（对应 Worker `UserProfileResponse`）。
///
/// 头像/背景/签名/展示名是链下 R2 资料；计数与认证是 D1/链上派生。
/// App 侧只读展示，写入走 `PUT /v1/square/profile`。
class CitizenProfile {
  const CitizenProfile({
    required this.ownerAccount,
    required this.displayName,
    required this.bio,
    required this.avatarObjectKey,
    required this.bannerObjectKey,
    required this.cidNumber,
    required this.isCertified,
    required this.identityLevel,
    required this.membershipLevel,
    required this.membershipActive,
    required this.following,
    required this.followers,
    required this.posts,
    required this.isFollowing,
    required this.updatedAt,
  });

  final String ownerAccount;
  final String displayName;
  final String bio;
  final String? avatarObjectKey;
  final String? bannerObjectKey;
  final String? cidNumber;
  final bool isCertified;

  /// 链上身份档位：visitor 未认证 / voting 认证投票公民 / candidate 认证竞选公民。
  /// 认证真源=链上，徽章据此分色（访客橙/投票蓝/竞选红）。
  final String identityLevel;

  /// 已购买的会员档位（公开）：visitor/voting/candidate/null。徽章「勾」= 会员档匹配身份档。
  final String? membershipLevel;

  /// 会员是否当前有效。
  final bool membershipActive;
  final int following;
  final int followers;
  final int posts;
  final bool isFollowing;
  final int updatedAt;

  /// 本人钱包名是昵称真源，`display_name` 是公开镜像；均缺失时使用本地
  /// 稳定默认昵称，绝不把完整或截断账户当昵称。
  String resolvedDisplayName(String fallback) {
    return ProfilePresentation.forAccount(ownerAccount).resolveDisplayName(
      walletName: fallback,
      publicName: displayName,
    );
  }

  factory CitizenProfile.fromJson(Map<String, dynamic> json) {
    final counts = json['counts'];
    final countsMap = counts is Map<String, dynamic> ? counts : const {};
    return CitizenProfile(
      ownerAccount: _asString(json['owner_account']),
      displayName: _asString(json['display_name']),
      bio: _asString(json['bio']),
      avatarObjectKey: _asNullableString(json['avatar_object_key']),
      bannerObjectKey: _asNullableString(json['banner_object_key']),
      cidNumber: _asNullableString(json['cid_number']),
      isCertified: json['is_certified'] == true,
      identityLevel: _asIdentityLevel(json['identity_level']),
      membershipLevel: _asMembershipLevel(json['membership_level']),
      membershipActive: json['membership_active'] == true,
      following: _asInt(countsMap['following']),
      followers: _asInt(countsMap['followers']),
      posts: _asInt(countsMap['posts']),
      isFollowing: json['is_following'] == true,
      updatedAt: _asInt(json['updated_at']),
    );
  }

  Map<String, dynamic> toJson() => <String, dynamic>{
        'owner_account': ownerAccount,
        'display_name': displayName,
        'bio': bio,
        'avatar_object_key': avatarObjectKey,
        'banner_object_key': bannerObjectKey,
        'cid_number': cidNumber,
        'is_certified': isCertified,
        'identity_level': identityLevel,
        'membership_level': membershipLevel,
        'membership_active': membershipActive,
        'counts': <String, dynamic>{
          'following': following,
          'followers': followers,
          'posts': posts,
        },
        'is_following': isFollowing,
        'updated_at': updatedAt,
      };

  CitizenProfile copyWith({
    String? displayName,
    String? bio,
    Object? avatarObjectKey = _sentinel,
    Object? bannerObjectKey = _sentinel,
    bool? isFollowing,
    int? followers,
    int? updatedAt,
  }) {
    return CitizenProfile(
      ownerAccount: ownerAccount,
      displayName: displayName ?? this.displayName,
      bio: bio ?? this.bio,
      avatarObjectKey: identical(avatarObjectKey, _sentinel)
          ? this.avatarObjectKey
          : avatarObjectKey as String?,
      bannerObjectKey: identical(bannerObjectKey, _sentinel)
          ? this.bannerObjectKey
          : bannerObjectKey as String?,
      cidNumber: cidNumber,
      isCertified: isCertified,
      identityLevel: identityLevel,
      membershipLevel: membershipLevel,
      membershipActive: membershipActive,
      following: following,
      followers: followers ?? this.followers,
      posts: posts,
      isFollowing: isFollowing ?? this.isFollowing,
      updatedAt: updatedAt ?? this.updatedAt,
    );
  }
}

/// 关注/粉丝列表的一行（对应 Worker follows 列表项）。
class SquareFollowEntry {
  const SquareFollowEntry({
    required this.ownerAccount,
    required this.createdAt,
  });

  final String ownerAccount;
  final int createdAt;

  factory SquareFollowEntry.fromJson(Map<String, dynamic> json) {
    return SquareFollowEntry(
      ownerAccount: _asString(json['owner_account']),
      createdAt: _asInt(json['created_at']),
    );
  }
}

const Object _sentinel = Object();

String _asString(Object? value) => value?.toString() ?? '';

String? _asNullableString(Object? value) {
  final normalized = value?.toString().trim() ?? '';
  return normalized.isEmpty ? null : normalized;
}

/// 归一化链上身份档位；未知/缺失一律 visitor（fail-closed，不误判认证）。
String _asIdentityLevel(Object? value) {
  final normalized = value?.toString().trim();
  return (normalized == 'voting' || normalized == 'candidate')
      ? normalized!
      : 'visitor';
}

/// 归一化会员档位；未知/缺失/未购买 → null（不给勾）。
String? _asMembershipLevel(Object? value) {
  final normalized = value?.toString().trim();
  return (normalized == 'freedom' ||
          normalized == 'democracy' ||
          normalized == 'voting' ||
          normalized == 'candidate')
      ? normalized
      : null;
}

int _asInt(Object? value) {
  if (value is int) return value;
  if (value is num) return value.toInt();
  return int.tryParse(value?.toString() ?? '') ?? 0;
}
