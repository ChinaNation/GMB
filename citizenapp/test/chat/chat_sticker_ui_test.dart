import 'package:flutter/material.dart';
import 'package:flutter_test/flutter_test.dart';

import 'package:citizenapp/chat/chat_models.dart';
import 'package:citizenapp/chat/chat_page.dart';
import 'package:citizenapp/chat/chat_payload.dart';
import 'package:citizenapp/chat/compose/sticker_panel.dart';
import 'package:citizenapp/chat/storage/chat_store.dart';

/// 只喂固定消息列表的假 store(ChatPage 打开时只读 readMessages),不起 Isar。
class _StubStore extends ChatStore {
  _StubStore(this._messages);

  final List<ChatStoredMessage> _messages;

  @override
  Future<List<ChatStoredMessage>> readMessages(String conversationId) async =>
      _messages
          .where((message) => message.conversationId == conversationId)
          .toList(growable: false);
}

ChatStoredMessage _stickerStored(
  String stickerId, {
  String pack = 'fluent3d',
}) =>
    ChatStoredMessage(
      envelopeId: 'env-$stickerId',
      conversationId: 'conv-st',
      direction: 'incoming',
      senderAccountId:
          '0x2222222222222222222222222222222222222222222222222222222222222222',
      recipientAccountId:
          '0x1111111111111111111111111111111111111111111111111111111111111111',
      messageKind: ChatMessageKind.sticker,
      deliveryState: ChatMessageDeliveryState.receivedByDevice,
      createdAtMillis: 1000,
      plaintext: ChatPayloadCodec.encode(
        ChatContent.sticker(packId: pack, stickerId: stickerId),
      ),
    );

Widget _host({
  required ChatStore store,
  ChatSendStickerCallback? onSendSticker,
}) =>
    MaterialApp(
      home: ChatPage(
        conversationId: 'conv-st',
        accountId:
            '0x1111111111111111111111111111111111111111111111111111111111111111',
        peerUserId:
            '0x2222222222222222222222222222222222222222222222222222222222222222',
        title: 'Bob',
        store: store,
        onSync: () async => 0,
        onSendSticker: onSendSticker,
      ),
    );

Future<void> _settleOpen(WidgetTester tester) async {
  // flutter_chat_ui 空列表动画会再排一个 timer;pump 两帧稳定,不 settle(贴纸/媒体
  // 的 Image.asset 异步解码会让 pumpAndSettle 挂起)。
  await tester.pump(const Duration(milliseconds: 100));
  await tester.pump(const Duration(milliseconds: 100));
}

void main() {
  testWidgets('未知贴纸 id 渲染降级占位 [贴纸],绝不崩', (tester) async {
    await tester.pumpWidget(
      _host(store: _StubStore([_stickerStored('not_a_real_sticker')])),
    );
    await _settleOpen(tester);

    expect(find.text('[贴纸]'), findsOneWidget);
    expect(tester.takeException(), isNull);

    await tester.pumpWidget(const SizedBox.shrink());
    await tester.pump(const Duration(milliseconds: 100));
  });

  testWidgets('已知贴纸渲染不抛异常', (tester) async {
    await tester.pumpWidget(
      _host(store: _StubStore([_stickerStored('grinning_face')])),
    );
    await _settleOpen(tester);

    // Image.asset 解码在 widget 测不保证完成,errorBuilder 兜底:关键是绝不崩。
    expect(tester.takeException(), isNull);

    await tester.pumpWidget(const SizedBox.shrink());
    await tester.pump(const Duration(milliseconds: 100));
  });

  testWidgets('点开关弹面板,点选路由到 onSendSticker(fluent3d, id)', (tester) async {
    String? pickedPack;
    String? pickedSticker;
    await tester.pumpWidget(
      _host(
        store: _StubStore(const []),
        onSendSticker: (packId, stickerId) async {
          pickedPack = packId;
          pickedSticker = stickerId;
        },
      ),
    );
    await _settleOpen(tester);

    // 面板初始不在
    expect(find.byType(StickerPanel), findsNothing);
    // 点开关 → 面板出现
    await tester.tap(find.byKey(const ValueKey('chat-sticker-toggle')));
    await tester.pump();
    expect(find.byType(StickerPanel), findsOneWidget);
    // 点选 grinning_face → onSendSticker 收到 (fluent3d, grinning_face)
    await tester.tap(find.byKey(const ValueKey('sticker-grinning_face')));
    await tester.pump();
    await tester.pump(const Duration(milliseconds: 100)); // 等 _reloadMessages
    expect(pickedPack, 'fluent3d');
    expect(pickedSticker, 'grinning_face');
    // 发送后重载不得 unmount 面板(否则连发时面板闪走、分类 Tab 归零)。
    expect(find.byType(StickerPanel), findsOneWidget);

    await tester.pumpWidget(const SizedBox.shrink());
    await tester.pump(const Duration(milliseconds: 100));
  });

  testWidgets('再点开关收起面板', (tester) async {
    await tester.pumpWidget(_host(store: _StubStore(const [])));
    await _settleOpen(tester);

    await tester.tap(find.byKey(const ValueKey('chat-sticker-toggle')));
    await tester.pump();
    expect(find.byType(StickerPanel), findsOneWidget);
    await tester.tap(find.byKey(const ValueKey('chat-sticker-toggle')));
    await tester.pump();
    expect(find.byType(StickerPanel), findsNothing);

    await tester.pumpWidget(const SizedBox.shrink());
    await tester.pump(const Duration(milliseconds: 100));
  });
}
