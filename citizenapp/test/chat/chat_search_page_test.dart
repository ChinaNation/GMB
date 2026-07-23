import 'package:flutter/material.dart';
import 'package:flutter_test/flutter_test.dart';

import 'package:citizenapp/chat/chat_models.dart';
import 'package:citizenapp/chat/chat_search_page.dart';
import 'package:citizenapp/chat/open_direct_chat.dart';
import 'package:citizenapp/chat/storage/chat_store.dart';
import 'package:citizenapp/my/user/contact_service.dart';

/// 聊天搜索页验证：一个输入框、三段结果（会话 / 联系人 / 聊天记录）。
///
/// 输入框 autofocus 会让光标动画长驻，`pumpAndSettle` 必超时，
/// 因此全程用固定次数 `pump` 推进异步链。
const _accountId =
    '0x1111111111111111111111111111111111111111111111111111111111111111';
const _peerAccountId =
    '0x2222222222222222222222222222222222222222222222222222222222222222';
const _contactAddress = 'w5Bc7ma8qUcECfQDJmRyQM2wGmga5XSYtz7DvEengQ86xBWrT';

final _dmPreview = ChatConversationPreview(
  conversationId: 'dm:me:peer',
  title: '张三',
  peerAccountId: _peerAccountId,
  lastMessage: '明天见',
  lastUpdatedAt: DateTime.fromMillisecondsSinceEpoch(20),
  unreadCount: 0,
  deliveryState: ChatMessageDeliveryState.sent,
);

final _groupPreview = ChatConversationPreview(
  conversationId: 'grp:me:1',
  title: '张家村议事群',
  peerAccountId: '',
  lastMessage: '资料已上传',
  lastUpdatedAt: DateTime.fromMillisecondsSinceEpoch(10),
  unreadCount: 0,
  deliveryState: ChatMessageDeliveryState.sent,
  conversationKind: 'group',
);

const _contact = UserContact(
  accountId: _peerAccountId,
  ss58Address: _contactAddress,
  contactName: '张三',
  createdAt: 1,
  updatedAt: 2,
);

const _message = ChatStoredMessage(
  envelopeId: 'env-1',
  conversationId: 'dm:me:peer',
  direction: 'incoming',
  senderAccountId: _peerAccountId,
  recipientAccountId: _accountId,
  messageKind: ChatMessageKind.text,
  deliveryState: ChatMessageDeliveryState.sent,
  createdAtMillis: 30,
  plaintext: '张三说的那份材料',
);

class _FakeChatStore extends ChatStore {
  _FakeChatStore({
    this.conversations = const <ChatConversationPreview>[],
    this.messages = const <ChatStoredMessage>[],
  });

  final List<ChatConversationPreview> conversations;
  final List<ChatStoredMessage> messages;

  /// 记录每次真正落到 store 的关键词，用于断言空关键词不触发检索。
  final List<String> searchedKeywords = <String>[];

  @override
  Future<List<ChatConversationPreview>> readConversationPreviews({
    String? accountId,
  }) async =>
      conversations;

  @override
  Future<List<ChatStoredMessage>> searchMessages({
    required String accountId,
    required String keyword,
    int limit = 50,
  }) async {
    searchedKeywords.add(keyword);
    final needle = keyword.toLowerCase();
    return messages
        .where((item) => (item.plaintext ?? '').toLowerCase().contains(needle))
        .toList(growable: false);
  }
}

class _FakeContacts extends UserContactService {
  _FakeContacts(this.contacts) : super(autoSync: false);

  final List<UserContact> contacts;

  @override
  Future<List<UserContact>> getContacts() async => contacts;
}

void main() {
  Future<_FakeChatStore> pumpPage(
    WidgetTester tester, {
    List<ChatConversationPreview> conversations =
        const <ChatConversationPreview>[],
    List<ChatStoredMessage> messages = const <ChatStoredMessage>[],
    List<UserContact> contacts = const <UserContact>[],
    DirectChatOpener? directChatOpener,
    GroupChatOpener? groupChatOpener,
  }) async {
    final store = _FakeChatStore(
      conversations: conversations,
      messages: messages,
    );
    await tester.pumpWidget(
      MaterialApp(
        home: ChatSearchPage(
          store: store,
          contactService: _FakeContacts(contacts),
          accountId: _accountId,
          directChatOpener: directChatOpener,
          groupChatOpener: groupChatOpener,
        ),
      ),
    );
    await tester.pump();
    await tester.pump();
    return store;
  }

  Future<void> search(WidgetTester tester, String keyword) async {
    await tester.enterText(
      find.byKey(const ValueKey('chat-search-input')),
      keyword,
    );
    await tester.pump();
    await tester.pump();
  }

  testWidgets('空关键词只显示提示且不触发聊天记录检索', (tester) async {
    final store = await pumpPage(
      tester,
      conversations: [_dmPreview],
      contacts: const [_contact],
      messages: const [_message],
    );

    expect(find.text('输入关键词，搜索会话、联系人与聊天记录'), findsOneWidget);
    expect(find.text('会话'), findsNothing);
    expect(store.searchedKeywords, isEmpty);
  });

  testWidgets('一个关键词分三段命中会话、联系人与聊天记录', (tester) async {
    await pumpPage(
      tester,
      conversations: [_dmPreview, _groupPreview],
      contacts: const [_contact],
      messages: const [_message],
    );

    await search(tester, '张');

    expect(find.text('会话'), findsOneWidget);
    expect(find.text('联系人'), findsOneWidget);
    expect(find.text('聊天记录'), findsOneWidget);
    expect(find.byKey(const ValueKey('search-conversation-dm:me:peer')),
        findsOneWidget);
    expect(find.byKey(const ValueKey('search-conversation-grp:me:1')),
        findsOneWidget);
    expect(find.byKey(const ValueKey('search-contact-$_peerAccountId')),
        findsOneWidget);
    expect(find.byKey(const ValueKey('search-message-env-1')), findsOneWidget);
    expect(find.text('张三说的那份材料'), findsOneWidget);
  });

  testWidgets('搜索大小写不敏感', (tester) async {
    await pumpPage(
      tester,
      conversations: [
        _dmPreview,
        ChatConversationPreview(
          conversationId: 'dm:me:bob',
          title: 'Bob',
          peerAccountId: _peerAccountId,
          lastMessage: 'hello',
          lastUpdatedAt: DateTime.fromMillisecondsSinceEpoch(5),
          unreadCount: 0,
          deliveryState: ChatMessageDeliveryState.sent,
        ),
      ],
    );

    await search(tester, 'bob');

    expect(find.byKey(const ValueKey('search-conversation-dm:me:bob')),
        findsOneWidget);
  });

  testWidgets('点单聊会话走统一私聊收口，点群聊会话走群聊收口', (tester) async {
    String? openedPeer;
    String? openedGroupId;
    await pumpPage(
      tester,
      conversations: [_dmPreview, _groupPreview],
      directChatOpener: (context,
          {required peerAccountId, required title}) async {
        openedPeer = peerAccountId;
      },
      groupChatOpener: (context, {required groupId, required title}) async {
        openedGroupId = groupId;
      },
    );

    await search(tester, '张');

    await tester
        .tap(find.byKey(const ValueKey('search-conversation-dm:me:peer')));
    await tester.pump();
    expect(openedPeer, _peerAccountId);

    await tester
        .tap(find.byKey(const ValueKey('search-conversation-grp:me:1')));
    await tester.pump();
    expect(openedGroupId, 'grp:me:1');
  });

  testWidgets('点联系人结果打开与其的一对一聊天', (tester) async {
    String? openedPeer;
    String? openedTitle;
    await pumpPage(
      tester,
      contacts: const [_contact],
      directChatOpener: (context,
          {required peerAccountId, required title}) async {
        openedPeer = peerAccountId;
        openedTitle = title;
      },
    );

    await search(tester, '张');
    await tester
        .tap(find.byKey(const ValueKey('search-contact-$_peerAccountId')));
    await tester.pump();

    expect(openedPeer, _peerAccountId);
    expect(openedTitle, '张三');
  });

  testWidgets('点聊天记录结果打开消息所在会话', (tester) async {
    String? openedPeer;
    await pumpPage(
      tester,
      conversations: [_dmPreview],
      messages: const [_message],
      directChatOpener: (context,
          {required peerAccountId, required title}) async {
        openedPeer = peerAccountId;
      },
    );

    await search(tester, '材料');
    await tester.tap(find.byKey(const ValueKey('search-message-env-1')));
    await tester.pump();

    expect(openedPeer, _peerAccountId);
  });

  testWidgets('三段都没命中时显示空态', (tester) async {
    await pumpPage(
      tester,
      conversations: [_dmPreview],
      contacts: const [_contact],
      messages: const [_message],
    );

    await search(tester, 'zzz不存在');

    expect(find.text('没有找到相关内容'), findsOneWidget);
    expect(find.text('会话'), findsNothing);
  });
}
