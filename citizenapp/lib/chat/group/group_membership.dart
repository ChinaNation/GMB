// 名册守卫:≤1989 上限 + 权限(仅 admin 加/删),纯函数可测。
//
// 上限双拦的 Dart 一侧;另一侧在 Rust `group_add_members`(以 MLS 实际成员数为准)。

import 'chat_group_limits.dart';

/// 群成员操作被拒。
class GroupMembershipException implements Exception {
  const GroupMembershipException(this.message);

  final String message;

  @override
  String toString() => message;
}

class GroupMembership {
  const GroupMembership._();

  /// 建群:创建者 + 邀请数不得超上限。
  static void ensureCanCreate({required int inviteeCount}) {
    if (inviteeCount < 0) {
      throw const GroupMembershipException('邀请人数不能为负');
    }
    if (1 + inviteeCount > kMaxGroupMembers) {
      throw GroupMembershipException(
        '群成员将达 ${1 + inviteeCount}，超过上限 $kMaxGroupMembers',
      );
    }
  }

  /// 加人:当前成员数 + 新增数不得超上限;至少加一人。
  static void ensureCanAdd({
    required int currentCount,
    required int addingCount,
  }) {
    if (addingCount <= 0) {
      throw const GroupMembershipException('至少添加一名成员');
    }
    if (currentCount + addingCount > kMaxGroupMembers) {
      throw GroupMembershipException(
        '群成员将达 ${currentCount + addingCount}，超过上限 $kMaxGroupMembers',
      );
    }
  }

  /// 加/删成员权限:仅 admin。
  static void ensureAdmin({
    required Set<String> adminSet,
    required String actorAccountId,
  }) {
    if (!adminSet.contains(actorAccountId)) {
      throw const GroupMembershipException('只有群管理员可以添加或移除成员');
    }
  }
}
