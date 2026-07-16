import 'package:emoji_picker_flutter/emoji_picker_flutter.dart';
import 'package:flutter/material.dart';
import 'package:flutter_chat_ui/flutter_chat_ui.dart';
import 'package:flutter_test/flutter_test.dart';

import 'package:citizenapp/chat/chat_page.dart';
import 'package:citizenapp/chat/compose/sticker_panel.dart';
import 'package:citizenapp/chat/storage/chat_store.dart';

class _EmptyStore extends ChatStore {
  @override
  Future<List<ChatStoredMessage>> readMessages(String conversationId) async =>
      const [];
}

Widget _host({ChatSendTextCallback? onSendText}) => MaterialApp(
      home: ChatPage(
        conversationId: 'conv-emoji',
        ownerAccount: 'alice-wallet',
        peerUserId: 'bob-wallet',
        title: 'Bob',
        store: _EmptyStore(),
        onSync: () async => 0,
        onSendText: onSendText,
      ),
    );

Future<void> _settleOpen(WidgetTester tester) async {
  await tester.pump(const Duration(milliseconds: 100));
  await tester.pump(const Duration(milliseconds: 100));
}

void main() {
  testWidgets('点表情开关弹 EmojiPicker,再点收起', (tester) async {
    await tester.pumpWidget(_host());
    await _settleOpen(tester);

    expect(find.byType(EmojiPicker), findsNothing);
    await tester.tap(find.byKey(const ValueKey('chat-emoji-toggle')));
    await tester.pump();
    expect(find.byType(EmojiPicker), findsOneWidget);
    await tester.tap(find.byKey(const ValueKey('chat-emoji-toggle')));
    await tester.pump();
    expect(find.byType(EmojiPicker), findsNothing);

    await tester.pumpWidget(const SizedBox.shrink());
    await tester.pump(const Duration(milliseconds: 100));
  });

  testWidgets('EmojiPicker 与 Composer 共用同一 controller,文本经 onSendText 发出',
      (tester) async {
    final sent = <String>[];
    await tester.pumpWidget(_host(onSendText: (text) async => sent.add(text)));
    await _settleOpen(tester);

    // 开表情面板,断言 EmojiPicker 与 Composer 的 controller 为同一实例——emoji 插入
    // 的文本才会进入将被发送的输入框(接错 controller 会静默丢字)。
    await tester.tap(find.byKey(const ValueKey('chat-emoji-toggle')));
    await tester.pump();
    final composer = tester.widget<Composer>(find.byType(Composer));
    final picker = tester.widget<EmojiPicker>(find.byType(EmojiPicker));
    expect(
      identical(composer.textEditingController, picker.textEditingController),
      isTrue,
    );

    // 直接向 composer 文本框输入(确定性替代点真 emoji 格:网格异步加载不可靠),
    // 走 Composer 发送按钮 → onMessageSend → _handleSend → onSendText。
    await tester.enterText(
      find.descendant(
        of: find.byType(Composer),
        matching: find.byType(EditableText),
      ),
      '你好🙂',
    );
    await tester.pump();
    await tester.tap(find.byIcon(Icons.send));
    await tester.pump();
    await tester.pump(const Duration(milliseconds: 100));

    expect(sent.single, '你好🙂');

    await tester.pumpWidget(const SizedBox.shrink());
    await tester.pump(const Duration(milliseconds: 100));
  });

  testWidgets('表情与贴纸面板互斥:切一个自动关另一个', (tester) async {
    await tester.pumpWidget(_host());
    await _settleOpen(tester);

    // 开表情
    await tester.tap(find.byKey(const ValueKey('chat-emoji-toggle')));
    await tester.pump();
    expect(find.byType(EmojiPicker), findsOneWidget);
    expect(find.byType(StickerPanel), findsNothing);

    // 切贴纸 → 表情自动关
    await tester.tap(find.byKey(const ValueKey('chat-sticker-toggle')));
    await tester.pump();
    expect(find.byType(EmojiPicker), findsNothing);
    expect(find.byType(StickerPanel), findsOneWidget);

    await tester.pumpWidget(const SizedBox.shrink());
    await tester.pump(const Duration(milliseconds: 100));
  });
}
