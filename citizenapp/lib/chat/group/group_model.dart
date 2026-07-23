// 私密小群的 Dart 侧模型。名册以 MLS 群成员为真源,本模型为镜像视图。

/// 群成员角色。
enum GroupMemberRole {
  admin('admin'),
  member('member');

  const GroupMemberRole(this.wireName);

  final String wireName;

  static GroupMemberRole fromName(String value) {
    for (final role in values) {
      if (role.wireName == value) {
        return role;
      }
    }
    return GroupMemberRole.member;
  }
}

/// 群成员镜像(账户 + 角色)。
class GroupMember {
  const GroupMember({
    required this.accountId,
    this.role = GroupMemberRole.member,
  });

  final String accountId;
  final GroupMemberRole role;

  bool get isAdmin => role == GroupMemberRole.admin;
}

/// 群会话镜像。
class ChatGroup {
  const ChatGroup({
    required this.groupId,
    required this.name,
    required this.creatorAccountId,
    required this.epoch,
    required this.roster,
    this.leftLocally = false,
  });

  /// 群 ID = conversation_id,形如 `grp:<creator>:<nonce>`。
  final String groupId;
  final String name;
  final String creatorAccountId;

  /// MLS 当前 epoch 的本地镜像(UI/调试用)。
  final int epoch;

  /// 名册镜像(以 MLS `group_state` 对账)。
  final List<GroupMember> roster;

  /// 本机是否已退群/被移除。
  final bool leftLocally;

  /// admin 账户集合。
  Set<String> get adminSet => roster
      .where((member) => member.isAdmin)
      .map((member) => member.accountId)
      .toSet();

  /// 全体成员账户(去重)。
  List<String> get memberAccountIds =>
      roster.map((member) => member.accountId).toList(growable: false);
}
