// 私密小群(MLS 群)的 Dart 侧边界模型与接口。
//
// 只定义可测的数据边界与注入点;真正的 OpenMLS 群加解密由 Rust native
// (chat_mls.rs 的 6 个 group FFI)实现,这里禁止自研密码学。
// 详见 memory/05-modules/citizenapp/chat/CHAT_GROUP_TECHNICAL.md。

import 'mls_boundary.dart';

/// 群成员标识 = "account_id:device_id"(MLS BasicCredential 内容)。
/// 扇出/名册以**账户**为单位(recipient_account_id),故从标识取账户段。
String accountFromMemberIdentity(String identity) {
  final index = identity.indexOf(':');
  return index < 0 ? identity : identity.substring(0, index);
}

/// 一批成员标识 → 去重账户集合(可选排除自己)。
List<String> accountsFromMemberIdentities(
  Iterable<String> identities, {
  String? excludeAccount,
}) {
  final seen = <String>{};
  final result = <String>[];
  for (final identity in identities) {
    final account = accountFromMemberIdentity(identity);
    if (account.isEmpty || account == excludeAccount) {
      continue;
    }
    if (seen.add(account)) {
      result.add(account);
    }
  }
  return result;
}

/// `group_process` 返回的 epoch 判定状态。
enum GroupProcessStatus {
  applied('applied'),
  outOfOrder('out_of_order'),
  stale('stale'),
  unknown('unknown');

  const GroupProcessStatus(this.wireName);

  final String wireName;

  static GroupProcessStatus fromWireName(String value) {
    for (final status in values) {
      if (status.wireName == value) {
        return status;
      }
    }
    return GroupProcessStatus.unknown;
  }
}

/// 入站群消息的内容类型。
enum GroupInboundKind {
  welcome('welcome'),
  commit('commit'),
  application('application'),
  unknown('unknown');

  const GroupInboundKind(this.wireName);

  final String wireName;

  static GroupInboundKind fromWireName(String value) {
    for (final kind in values) {
      if (kind.wireName == value) {
        return kind;
      }
    }
    return GroupInboundKind.unknown;
  }
}

/// 建群结果。
class GroupCreated {
  const GroupCreated({required this.groupId, required this.epoch});

  final String groupId;
  final int epoch;
}

/// 加人/删人产生的 Commit 束。
///
/// add:`commit` 发给现有成员,`welcome` 发给全部新人(单条覆盖 N 人)。
/// remove:仅 `commit`,发给剩余成员 + 被删者;`removedAccounts` 为被删账户。
class GroupCommitBundle {
  const GroupCommitBundle({
    required this.groupId,
    required this.epoch,
    required this.commit,
    this.welcome,
    this.removedAccounts = const [],
  });

  final String groupId;
  final int epoch;
  final MlsWireMessage commit;
  final MlsWireMessage? welcome;
  final List<String> removedAccounts;
}

/// `group_process` 处理入站群消息的结果。
class GroupInbound {
  const GroupInbound({
    required this.groupId,
    required this.kind,
    required this.status,
    required this.messageEpoch,
    required this.groupEpoch,
    required this.selfRemoved,
    this.plaintext,
    this.memberIdentities,
  });

  final String groupId;
  final GroupInboundKind kind;
  final GroupProcessStatus status;
  final int messageEpoch;
  final int groupEpoch;

  /// 本机是否在该 Commit 中被移除(被删/退群生效)。
  final bool selfRemoved;

  /// application 明文(仅 application applied 非空)。
  final List<int>? plaintext;

  /// 应用 Commit / 入群 Welcome 后的 MLS 权威名册(标识,含设备段)。
  final List<String>? memberIdentities;

  bool get isApplied => status == GroupProcessStatus.applied;

  bool get isOutOfOrder => status == GroupProcessStatus.outOfOrder;
}

/// 只读群状态(名册对账 + 上限守)。
class GroupState {
  const GroupState({
    required this.groupId,
    required this.epoch,
    required this.memberIdentities,
  });

  final String groupId;
  final int epoch;
  final List<String> memberIdentities;

  int get memberCount => memberIdentities.length;
}

/// OpenMLS 群 FFI 边界接口(可注入,单测用 fake)。
///
/// 实现必须调用成熟 OpenMLS native,不允许在 Dart 中自研群密码学。
abstract class MlsGroupCrypto {
  /// 建群,创建者为唯一成员。
  Future<GroupCreated> createGroup(String groupId);

  /// 批量加人:1 Commit(现有成员)+ 1 Welcome(全部新人)。
  Future<GroupCommitBundle> addMembers(
    String groupId,
    List<MlsKeyPackage> keyPackages,
  );

  /// 删人:Commit(剩余成员 + 被删者)。按**账户**移除(含其全部设备叶子)。
  Future<GroupCommitBundle> removeMembers(
    String groupId,
    List<String> memberAccountIds,
  );

  /// 群 application message(单次加密,Dart 侧扇出)。
  Future<MlsWireMessage> groupCreateMessage(
    String groupId,
    List<int> plaintext,
  );

  /// 处理入站群消息(Welcome / Commit / Application)。
  Future<GroupInbound> groupProcess(MlsWireMessage wire);

  /// 只读群状态(epoch + 名册)。
  Future<GroupState> groupState(String groupId);
}
