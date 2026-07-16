// 群控制消息(走既有 E2E application 通道,不改 proto、不进 ChatMessageKind)。
//
// 与用户消息载荷 `gmb.chat.msg` 区分:控制载荷 `t=gmb.chat.ctrl`,收端 group_flow
// 先按此判别——是控制则处理(改名/退群请求)、**绝不当聊天消息显示**;非控制退化为
// 普通消息。解码对任何非本协议/坏数据都返回 null(安全,不误吞用户文本)。

import 'dart:convert';

/// 群控制操作。
enum GroupControlOp {
  /// 创建者/管理员广播群名(补 Welcome 不带名的缺口)。
  rename('rename'),

  /// 退群者请求群 admin 代提交移除(密码学后向保密由 admin 的 removeMembers 保证)。
  leaveRequest('leave_request');

  const GroupControlOp(this.wireName);

  final String wireName;

  static GroupControlOp? fromWireName(String value) {
    for (final op in values) {
      if (op.wireName == value) {
        return op;
      }
    }
    return null;
  }
}

/// 群控制消息。
class GroupControl {
  const GroupControl._(this.op, this.groupName);

  factory GroupControl.rename(String name) =>
      GroupControl._(GroupControlOp.rename, name);

  const GroupControl.leaveRequest() : this._(GroupControlOp.leaveRequest, null);

  final GroupControlOp op;

  /// op=rename 携带的群名。
  final String? groupName;
}

class GroupControlCodec {
  GroupControlCodec._();

  static const String type = 'gmb.chat.ctrl';
  static const int version = 1;

  static String encode(GroupControl control) {
    final map = <String, Object?>{
      't': type,
      'v': version,
      'op': control.op.wireName,
    };
    if (control.op == GroupControlOp.rename) {
      map['name'] = control.groupName ?? '';
    }
    return jsonEncode(map);
  }

  /// 尝试解码为群控制消息;非控制载荷(含普通消息 / 坏数据)返回 null。
  static GroupControl? tryDecode(String raw) {
    Object? decoded;
    try {
      decoded = jsonDecode(raw);
    } catch (_) {
      return null;
    }
    if (decoded is! Map || decoded['t'] != type) {
      return null;
    }
    final op = GroupControlOp.fromWireName((decoded['op'] ?? '').toString());
    if (op == null) {
      return null;
    }
    return switch (op) {
      GroupControlOp.rename =>
        GroupControl.rename((decoded['name'] ?? '').toString()),
      GroupControlOp.leaveRequest => const GroupControl.leaveRequest(),
    };
  }
}
