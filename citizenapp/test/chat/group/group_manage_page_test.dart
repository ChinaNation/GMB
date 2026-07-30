import 'package:citizenapp/chat/group/group_model.dart';
import 'package:citizenapp/chat/group/ui/group_manage_page.dart';
import 'package:citizenapp/chat/storage/chat_store.dart';
import 'package:flutter/material.dart';
import 'package:flutter_test/flutter_test.dart';

/// 只覆写渲染所需的 readGroup(避免 Isar 真异步在 widget 测 fake-async 下不 settle)。
class _FakeStore extends ChatStore {
  _FakeStore(this._group);

  final ChatGroup _group;

  @override
  Future<ChatGroup?> readGroup(
    String ownerCidNumber,
    String groupId,
  ) async =>
      _group;
}

ChatGroup _group() => const ChatGroup(
      groupId: 'grp:CN220-CTZN2-100000003-2026:n',
      name: '测试群',
      creatorCidNumber: 'CN220-CTZN2-100000003-2026',
      epoch: 1,
      roster: [
        GroupMember(
            cidNumber: 'CN220-CTZN2-100000003-2026',
            role: GroupMemberRole.admin),
        GroupMember(cidNumber: 'CN220-CTZN2-100000004-2026'),
      ],
    );

Future<void> _pump(WidgetTester tester, String cidNumber) async {
  await tester.pumpWidget(MaterialApp(
    home: GroupManagePage(
      groupId: 'grp:CN220-CTZN2-100000003-2026:n',
      store: _FakeStore(_group()),
      cidNumber: cidNumber,
    ),
  ));
  await tester.pumpAndSettle();
}

void main() {
  testWidgets('admin 可见 添加 / 移除 / 改群名', (tester) async {
    await _pump(tester, 'CN220-CTZN2-100000003-2026');
    expect(find.text('添加'), findsOneWidget);
    expect(find.byIcon(Icons.remove_circle_outline), findsWidgets); // 可移除 acctB
    expect(find.byIcon(Icons.edit_outlined), findsOneWidget); // 改群名
    expect(find.text('退出群聊'), findsOneWidget);
  });

  testWidgets('非 admin 无 添加 / 移除 / 改群名,但可退群', (tester) async {
    await _pump(tester, 'CN220-CTZN2-100000004-2026');
    expect(find.text('添加'), findsNothing);
    expect(find.byIcon(Icons.remove_circle_outline), findsNothing);
    expect(find.byIcon(Icons.edit_outlined), findsNothing);
    expect(find.text('退出群聊'), findsOneWidget); // 退群任何人可
  });
}
