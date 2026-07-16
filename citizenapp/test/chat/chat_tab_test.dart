import 'package:flutter/material.dart';
import 'package:flutter_blurhash/flutter_blurhash.dart';
import 'package:flutter_test/flutter_test.dart';

import 'package:citizenapp/8964/profile/models/profile_presentation.dart';
import 'package:citizenapp/chat/chat_page.dart';
import 'package:citizenapp/chat/chat_flow.dart';
import 'package:citizenapp/chat/chat_payload.dart';
import 'package:citizenapp/chat/chat_runtime.dart';
import 'package:citizenapp/chat/chat_models.dart';
import 'package:citizenapp/chat/chat_tab.dart';
import 'package:citizenapp/chat/storage/chat_store.dart';

void main() {
  testWidgets('聊天标题为账户时改用稳定默认昵称', (tester) async {
    const peer = 'w5Bc7ma8qUcECfQDJmRyQM2wGmga5XSYtz7DvEengQ86xBWrT';
    final store = _FakeChatStore();
    await tester.pumpWidget(
      MaterialApp(
        home: ChatPage(
          conversationId: 'dm:alice:$peer',
          ownerAccount: 'alice',
          peerUserId: peer,
          title: peer,
          store: store,
          onSync: () async => 0,
        ),
      ),
    );
    await tester.pump(const Duration(milliseconds: 100));
    // flutter_chat_ui 的空列表动画会在首次稳定布局后再排一个 50ms timer。
    await tester.pump(const Duration(milliseconds: 100));

    expect(
      find.text(ProfilePresentation.forAccount(peer).fallbackName),
      findsOneWidget,
    );
    expect(find.text(peer), findsNothing);

    await tester.pumpWidget(const SizedBox.shrink());
    await tester.pump(const Duration(milliseconds: 100));
  });

  testWidgets('隐藏 Chat Tab 不初始化，进入后 init/resume 只同步一次', (tester) async {
    final selectedTab = ValueNotifier<int>(0);
    final runtime = _FakeRuntime(address: 'alice-wallet');

    await tester.pumpWidget(
      MaterialApp(
        home: Scaffold(
          body: ChatTab(
            store: _FakeChatStore(),
            ownerAccount: 'alice-wallet',
            runtime: runtime,
            selectedTab: selectedTab,
            tabIndex: 2,
          ),
        ),
      ),
    );
    await tester.pump(const Duration(milliseconds: 100));
    expect(runtime.syncCount, 0);

    selectedTab.value = 2;
    await tester.pump(const Duration(milliseconds: 100));
    expect(runtime.syncCount, 1);

    // 一次 pause/resume 可以同步一次；同一 resume burst 不得创建两条链。
    tester.binding.handleAppLifecycleStateChanged(
      AppLifecycleState.inactive,
    );
    tester.binding.handleAppLifecycleStateChanged(
      AppLifecycleState.resumed,
    );
    tester.binding.handleAppLifecycleStateChanged(
      AppLifecycleState.resumed,
    );
    await tester.pump(const Duration(milliseconds: 100));
    expect(runtime.syncCount, 2);

    selectedTab.dispose();
  });

  testWidgets('聊天 Tab renders conversation list for owner account',
      (tester) async {
    final store = _FakeChatStore(
      conversations: [
        ChatConversationPreview(
          conversationId: 'dm:alice-wallet:bob-wallet',
          title: 'Bob',
          peerAccount: 'bob-wallet',
          lastMessage: 'hello',
          lastUpdatedAt: DateTime.fromMillisecondsSinceEpoch(1),
          unreadCount: 1,
          deliveryState: ChatMessageDeliveryState.receivedByDevice,
        ),
      ],
    );

    await tester.pumpWidget(
      MaterialApp(
        home: Scaffold(
          body: ChatTab(
            store: store,
            ownerAccount: 'alice-wallet',
          ),
        ),
      ),
    );
    await tester.pump(const Duration(milliseconds: 100));

    expect(find.text('聊天'), findsOneWidget);
    expect(find.text('Bob'), findsOneWidget);
    expect(find.text('bob-wallet'), findsNothing);
    expect(find.text('hello'), findsOneWidget);
    expect(store.lastOwnerFilter, 'alice-wallet');
    expect(find.byIcon(Icons.add_comment_outlined), findsNothing);
    expect(find.byIcon(Icons.qr_code_scanner_rounded), findsNothing);
    expect(find.byIcon(Icons.qr_code_2_rounded), findsNothing);
  });

  testWidgets('聊天 Tab deletes one local conversation after confirmation',
      (tester) async {
    final store = _FakeChatStore(
      conversations: [
        ChatConversationPreview(
          conversationId: 'dm:alice-wallet:bob-wallet',
          title: 'Bob',
          peerAccount: 'bob-wallet',
          lastMessage: 'hello',
          lastUpdatedAt: DateTime.fromMillisecondsSinceEpoch(2),
          unreadCount: 0,
          deliveryState: ChatMessageDeliveryState.sent,
        ),
        ChatConversationPreview(
          conversationId: 'dm:alice-wallet:carol-wallet',
          title: 'Carol',
          peerAccount: 'carol-wallet',
          lastMessage: 'keep',
          lastUpdatedAt: DateTime.fromMillisecondsSinceEpoch(1),
          unreadCount: 0,
          deliveryState: ChatMessageDeliveryState.sent,
        ),
      ],
    );

    await tester.pumpWidget(
      MaterialApp(
        home: Scaffold(
          body: ChatTab(
            store: store,
            ownerAccount: 'alice-wallet',
          ),
        ),
      ),
    );
    await tester.pump(const Duration(milliseconds: 100));

    await tester.drag(find.text('Bob'), const Offset(-500, 0));
    await tester.pumpAndSettle();

    expect(find.text('删除聊天记录'), findsOneWidget);
    expect(find.text('确定删除这台设备上的聊天记录？'), findsOneWidget);

    await tester.tap(find.widgetWithText(TextButton, '删除'));
    await tester.pumpAndSettle();

    expect(store.deletedConversationIds, ['dm:alice-wallet:bob-wallet']);
    expect(find.text('Bob'), findsNothing);
    expect(find.text('Carol'), findsOneWidget);
  });

  testWidgets('聊天 Tab requires a configured owner account', (tester) async {
    await tester.pumpWidget(
      MaterialApp(
        home: Scaffold(
          body: ChatTab(
            store: _FakeChatStore(),
            ownerAccount: '',
          ),
        ),
      ),
    );
    await tester.pump(const Duration(milliseconds: 100));

    expect(find.text('请先在「我的 → 我的钱包」创建热钱包'), findsOneWidget);
  });

  testWidgets('聊天 Tab 打开后自动重试本机发送队列', (tester) async {
    final runtime = _FakeRuntime(address: 'alice-wallet');
    final store = _FakeChatStore();

    await tester.pumpWidget(
      MaterialApp(
        home: Scaffold(
          body: ChatTab(
            store: store,
            ownerAccount: 'alice-wallet',
            runtime: runtime,
          ),
        ),
      ),
    );
    await tester.pump(const Duration(milliseconds: 100));

    expect(runtime.syncCount, 1);
    expect(store.readPreviewCount, 1);

    await tester.pump(const Duration(seconds: 15));
    await tester.pump();

    expect(runtime.syncCount, 2);
    expect(store.readPreviewCount, 2);

    await tester.pumpWidget(const SizedBox.shrink());
    await tester.pump();
  });

  testWidgets('聊天 Tab uses realtime notice before polling fallback',
      (tester) async {
    final runtime = _FakeRuntime(
      address: 'alice-wallet',
      enableRealtime: true,
    );
    final store = _FakeChatStore();

    await tester.pumpWidget(
      MaterialApp(
        home: Scaffold(
          body: ChatTab(
            store: store,
            ownerAccount: 'alice-wallet',
            runtime: runtime,
          ),
        ),
      ),
    );
    await tester.pump(const Duration(milliseconds: 100));

    expect(runtime.syncCount, 1);
    expect(runtime.realtimeStartCount, 1);

    await tester.pump(const Duration(seconds: 15));
    await tester.pump();

    expect(runtime.syncCount, 1);

    await runtime.realtimeNotice?.call();
    await tester.pump();

    expect(runtime.syncCount, 2);
    expect(store.readPreviewCount, greaterThanOrEqualTo(2));

    await tester.pumpWidget(const SizedBox.shrink());
    await tester.pump();

    expect(runtime.realtimeStopCount, 1);
  });

  testWidgets('聊天页打开后自动重试本机发送队列', (tester) async {
    var syncCount = 0;
    final store = _FakeChatStore();

    await tester.pumpWidget(
      MaterialApp(
        home: ChatPage(
          conversationId: 'dm:alice-wallet:bob-wallet',
          ownerAccount: 'alice-wallet',
          peerUserId: 'bob-wallet',
          title: 'Bob',
          store: store,
          onSync: () async {
            syncCount += 1;
            return 0;
          },
        ),
      ),
    );
    await tester.pump(const Duration(milliseconds: 100));

    expect(syncCount, 1);
    expect(store.readMessagesCount, greaterThanOrEqualTo(1));

    await tester.pump(const Duration(seconds: 8));
    await tester.pump();

    expect(syncCount, 2);

    await tester.pumpWidget(const SizedBox.shrink());
    await tester.pump();
  });

  testWidgets('聊天页 uses realtime notice before polling fallback',
      (tester) async {
    var syncCount = 0;
    var realtimeStopCount = 0;
    Future<void> Function()? realtimeNotice;
    final store = _FakeChatStore();

    await tester.pumpWidget(
      MaterialApp(
        home: ChatPage(
          conversationId: 'dm:alice-wallet:bob-wallet',
          ownerAccount: 'alice-wallet',
          peerUserId: 'bob-wallet',
          title: 'Bob',
          store: store,
          onSync: () async {
            syncCount += 1;
            return 0;
          },
          onStartRealtime: ({
            required onNotice,
            onDisconnected,
          }) async {
            realtimeNotice = onNotice;
            return () async {
              realtimeStopCount += 1;
            };
          },
        ),
      ),
    );
    await tester.pump(const Duration(milliseconds: 100));

    expect(syncCount, 1);

    await tester.pump(const Duration(seconds: 8));
    await tester.pump();

    expect(syncCount, 1);

    await realtimeNotice?.call();
    await tester.pump();

    expect(syncCount, 2);
    expect(store.readMessagesCount, greaterThanOrEqualTo(2));

    await tester.pumpWidget(const SizedBox.shrink());
    await tester.pump();

    expect(realtimeStopCount, 1);
  });

  testWidgets('聊天页 attachment button sends selected encrypted media',
      (tester) async {
    ChatMediaDraft? sentMedia;
    final store = _FakeChatStore();

    await tester.pumpWidget(
      MaterialApp(
        home: ChatPage(
          conversationId: 'dm:alice-wallet:bob-wallet',
          ownerAccount: 'alice-wallet',
          peerUserId: 'bob-wallet',
          title: 'Bob',
          store: store,
          pickMedia: () async => const ChatMediaDraft(
            kind: ChatMessageKind.file,
            fileName: 'note.txt',
            contentType: 'text/plain',
            sourcePath: '/tmp/note.txt',
            byteSize: 3,
          ),
          onSendMedia: (media) async {
            sentMedia = media;
          },
        ),
      ),
    );
    await tester.pumpAndSettle();

    await tester.tap(find.byIcon(Icons.attachment));
    await tester.pumpAndSettle();

    expect(sentMedia?.kind, ChatMessageKind.file);
    expect(sentMedia?.fileName, 'note.txt');
    expect(sentMedia?.sourcePath, '/tmp/note.txt');
    expect(sentMedia?.byteSize, 3);
  });

  testWidgets('聊天页 taps a file message to save the received media',
      (tester) async {
    final store = _FakeChatStore(
      messages: [
        ChatStoredMessage(
          envelopeId: 'env-attachment',
          conversationId: 'dm:alice-wallet:bob-wallet',
          direction: 'incoming',
          senderAccount: 'bob-wallet',
          recipientAccount: 'alice-wallet',
          messageKind: ChatMessageKind.file,
          deliveryState: ChatMessageDeliveryState.receivedByDevice,
          createdAtMillis: 3000,
          plaintext: ChatPayloadCodec.encode(
            ChatContent.media(
              kind: ChatMessageKind.file,
              attachmentId: 'att-1',
              fileName: 'photo.txt',
              mime: 'text/plain',
              byteSize: 3,
            ),
          ),
        ),
      ],
    );
    String? downloadedPlaintext;

    await tester.pumpWidget(
      MaterialApp(
        home: ChatPage(
          conversationId: 'dm:alice-wallet:bob-wallet',
          ownerAccount: 'alice-wallet',
          peerUserId: 'bob-wallet',
          title: 'Bob',
          store: store,
          onDownloadAttachment: (conversationId, controlPlaintext) async {
            downloadedPlaintext = controlPlaintext;
            return const ChatDownloadedAttachment(
              attachmentId: 'att-1',
              fileName: 'photo.txt',
              contentType: 'text/plain',
              clearByteSize: 3,
              filePath: '/tmp/photo.txt',
            );
          },
        ),
      ),
    );
    await tester.pumpAndSettle();

    await tester.tap(find.text('photo.txt'));
    await tester.pumpAndSettle();

    expect(downloadedPlaintext, contains('gmb.chat.msg'));
    expect(find.text('已保存：photo.txt'), findsOneWidget);
  });

  testWidgets('聊天页 deletes local conversation from menu and returns',
      (tester) async {
    final store = _FakeChatStore(
      messages: const [
        ChatStoredMessage(
          envelopeId: 'env-delete-ui',
          conversationId: 'dm:alice-wallet:bob-wallet',
          direction: 'incoming',
          senderAccount: 'bob-wallet',
          recipientAccount: 'alice-wallet',
          messageKind: ChatMessageKind.text,
          deliveryState: ChatMessageDeliveryState.receivedByDevice,
          createdAtMillis: 1000,
          plaintext: 'hello',
        ),
      ],
    );

    await tester.pumpWidget(
      MaterialApp(
        home: Builder(
          builder: (context) => Scaffold(
            body: Center(
              child: TextButton(
                onPressed: () {
                  Navigator.of(context).push<void>(
                    MaterialPageRoute(
                      builder: (_) => ChatPage(
                        conversationId: 'dm:alice-wallet:bob-wallet',
                        ownerAccount: 'alice-wallet',
                        peerUserId: 'bob-wallet',
                        title: 'Bob',
                        store: store,
                        onDeleteConversation: () => store.deleteConversation(
                          'dm:alice-wallet:bob-wallet',
                        ),
                      ),
                    ),
                  );
                },
                child: const Text('打开聊天'),
              ),
            ),
          ),
        ),
      ),
    );

    await tester.tap(find.text('打开聊天'));
    await tester.pumpAndSettle();
    expect(find.text('hello'), findsOneWidget);

    await tester.tap(find.byIcon(Icons.more_vert_rounded));
    await tester.pumpAndSettle();
    await tester.tap(find.text('删除聊天记录'));
    await tester.pumpAndSettle();

    expect(find.text('确定删除这台设备上的聊天记录？'), findsOneWidget);

    await tester.tap(find.widgetWithText(TextButton, '删除'));
    await tester.pumpAndSettle();

    expect(store.deletedConversationIds, ['dm:alice-wallet:bob-wallet']);
    expect(find.text('打开聊天'), findsOneWidget);
  });

  testWidgets('聊天页把未到达的图片/视频消息渲染为「接收中」占位', (tester) async {
    // 无本机路径(未注入 onResolveMediaPath)→ source 为空 → 走 hasFile==false 占位分支。
    final store = _FakeChatStore(
      messages: [
        _mediaStored(
            id: 'img', kind: ChatMessageKind.image, mime: 'image/jpeg'),
        _mediaStored(id: 'vid', kind: ChatMessageKind.video, mime: 'video/mp4'),
      ],
    );
    await tester.pumpWidget(
      MaterialApp(
        home: ChatPage(
          conversationId: 'dm:alice-wallet:bob-wallet',
          ownerAccount: 'alice-wallet',
          peerUserId: 'bob-wallet',
          title: 'Bob',
          store: store,
        ),
      ),
    );
    await tester.pumpAndSettle();
    // 图片、视频两条都在"接收中"占位;误反转 hasFile 会去解码空路径而非占位。
    expect(find.text('接收中…'), findsNWidgets(2));
    // 视频占位带播放图标,与图片占位区分。
    expect(find.byIcon(Icons.play_circle_fill_rounded), findsOneWidget);
  });

  testWidgets('聊天页视频占位从 metadata 读取 blurhash 渲染封面', (tester) async {
    const hash = 'LEHV6nWB2yk8pyo0adR*.7kCMdnj';
    final store = _FakeChatStore(
      messages: [
        _mediaStored(
          id: 'vid',
          kind: ChatMessageKind.video,
          mime: 'video/mp4',
          blurhash: hash,
        ),
      ],
    );
    await tester.pumpWidget(
      MaterialApp(
        home: ChatPage(
          conversationId: 'dm:alice-wallet:bob-wallet',
          ownerAccount: 'alice-wallet',
          peerUserId: 'bob-wallet',
          title: 'Bob',
          store: store,
        ),
      ),
    );
    // 不用 pumpAndSettle:BlurHash 内部异步解码;只需确认封面 widget 已入树。
    await tester.pump();
    await tester.pump(const Duration(milliseconds: 100));
    // 视频封面用 metadata['blurhash'];若误读 message.blurhash(VideoMessage 无此字段)
    // 则渲染空 Container,BlurHash 不出现。
    expect(find.byType(BlurHash), findsOneWidget);
  });
}

ChatStoredMessage _mediaStored({
  required String id,
  required ChatMessageKind kind,
  required String mime,
  String? blurhash,
}) {
  return ChatStoredMessage(
    envelopeId: 'env-$id',
    conversationId: 'dm:alice-wallet:bob-wallet',
    direction: 'incoming',
    senderAccount: 'bob-wallet',
    recipientAccount: 'alice-wallet',
    messageKind: kind,
    deliveryState: ChatMessageDeliveryState.receivedByDevice,
    createdAtMillis: 3000,
    plaintext: ChatPayloadCodec.encode(
      ChatContent.media(
        kind: kind,
        attachmentId: 'att-$id',
        fileName: kind == ChatMessageKind.video ? 'v.mp4' : 'p.jpg',
        mime: mime,
        byteSize: 100,
        width: 800,
        height: 600,
        blurhash: blurhash,
      ),
    ),
  );
}

class _FakeChatStore extends ChatStore {
  _FakeChatStore({
    List<ChatConversationPreview> conversations = const [],
    List<ChatStoredMessage> messages = const [],
  })  : _conversations = List<ChatConversationPreview>.from(conversations),
        _messages = List<ChatStoredMessage>.from(messages);

  final List<ChatConversationPreview> _conversations;
  final List<ChatStoredMessage> _messages;
  String? lastOwnerFilter;
  int readPreviewCount = 0;
  int readMessagesCount = 0;
  final List<String> deletedConversationIds = <String>[];

  @override
  Future<List<ChatConversationPreview>> readConversationPreviews({
    String? ownerAccount,
  }) async {
    readPreviewCount += 1;
    lastOwnerFilter = ownerAccount;
    return List<ChatConversationPreview>.from(_conversations);
  }

  @override
  Future<List<ChatStoredMessage>> readMessages(String conversationId) async {
    readMessagesCount += 1;
    return _messages
        .where((message) => message.conversationId == conversationId)
        .toList(growable: false);
  }

  @override
  Future<void> deleteConversation(String conversationId) async {
    deletedConversationIds.add(conversationId);
    _conversations.removeWhere(
      (conversation) => conversation.conversationId == conversationId,
    );
    _messages
        .removeWhere((message) => message.conversationId == conversationId);
  }

  @override
  Future<int> outboundQueueCount() async {
    return 0;
  }
}

class _FakeRuntime extends ChatRuntime {
  _FakeRuntime({
    required this.address,
    this.enableRealtime = false,
  });

  final String address;
  final bool enableRealtime;
  int syncCount = 0;
  int realtimeStartCount = 0;
  int realtimeStopCount = 0;
  Future<void> Function()? realtimeNotice;
  Future<void> Function()? realtimeDisconnected;

  @override
  Future<String?> readOwnerAccount() async {
    return address;
  }

  @override
  Future<int> retryOutgoing({String? recipientAccount}) async {
    syncCount += 1;
    return 0;
  }

  @override
  Future<Future<void> Function()?> startRealtimeSync({
    required Future<void> Function() onNotice,
    Future<void> Function()? onDisconnected,
  }) async {
    realtimeStartCount += 1;
    realtimeNotice = onNotice;
    realtimeDisconnected = onDisconnected;
    if (!enableRealtime) {
      return null;
    }
    return () async {
      realtimeStopCount += 1;
    };
  }
}
