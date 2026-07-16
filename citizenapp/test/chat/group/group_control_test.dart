import 'package:citizenapp/chat/group/group_control.dart';
import 'package:flutter_test/flutter_test.dart';

void main() {
  test('rename 编解码保真', () {
    final decoded = GroupControlCodec.tryDecode(
      GroupControlCodec.encode(GroupControl.rename('我的群')),
    );
    expect(decoded, isNotNull);
    expect(decoded!.op, GroupControlOp.rename);
    expect(decoded.groupName, '我的群');
  });

  test('leave_request 编解码', () {
    final decoded = GroupControlCodec.tryDecode(
      GroupControlCodec.encode(const GroupControl.leaveRequest()),
    );
    expect(decoded, isNotNull);
    expect(decoded!.op, GroupControlOp.leaveRequest);
    expect(decoded.groupName, isNull);
  });

  test('普通消息 / 坏数据 / 未知 op → null(不误吞用户文本)', () {
    expect(
      GroupControlCodec.tryDecode(
          '{"t":"gmb.chat.msg","kind":"text","text":"hi"}'),
      isNull,
    );
    expect(GroupControlCodec.tryDecode('随便一句话'), isNull);
    expect(
      GroupControlCodec.tryDecode('{"t":"gmb.chat.ctrl","op":"unknown"}'),
      isNull,
    );
  });
}
