import 'package:flutter/material.dart';
import 'package:flutter_test/flutter_test.dart';
import 'package:citizenapp/im/im_chat_page.dart';
import 'package:citizenapp/im/im_message_flow.dart';
import 'package:citizenapp/im/im_runtime.dart';
import 'package:citizenapp/im/im_session_models.dart';
import 'package:citizenapp/im/im_tab_page.dart';
import 'package:citizenapp/im/storage/im_isar_store.dart';

void main() {
  testWidgets('聊天 Tab renders conversation list for communication account',
      (tester) async {
    final store = _FakeImStore(
      conversations: [
        ImConversationPreview(
          conversationId: 'dm:alice-wallet:bob-wallet',
          title: 'Bob',
          walletAddress: 'bob-wallet',
          lastMessage: 'hello',
          lastUpdatedAt: DateTime.fromMillisecondsSinceEpoch(1),
          unreadCount: 1,
          deliveryState: ImMessageDeliveryState.receivedByDevice,
        ),
      ],
    );

    await tester.pumpWidget(
      MaterialApp(
        home: Scaffold(
          body: ImTabPage(
            store: store,
            currentUserId: 'alice-wallet',
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
    final store = _FakeImStore(
      conversations: [
        ImConversationPreview(
          conversationId: 'dm:alice-wallet:bob-wallet',
          title: 'Bob',
          walletAddress: 'bob-wallet',
          lastMessage: 'hello',
          lastUpdatedAt: DateTime.fromMillisecondsSinceEpoch(2),
          unreadCount: 0,
          deliveryState: ImMessageDeliveryState.sent,
        ),
        ImConversationPreview(
          conversationId: 'dm:alice-wallet:carol-wallet',
          title: 'Carol',
          walletAddress: 'carol-wallet',
          lastMessage: 'keep',
          lastUpdatedAt: DateTime.fromMillisecondsSinceEpoch(1),
          unreadCount: 0,
          deliveryState: ImMessageDeliveryState.sent,
        ),
      ],
    );

    await tester.pumpWidget(
      MaterialApp(
        home: Scaffold(
          body: ImTabPage(
            store: store,
            currentUserId: 'alice-wallet',
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

  testWidgets('信息 Tab requires a configured communication account',
      (tester) async {
    await tester.pumpWidget(
      MaterialApp(
        home: Scaffold(
          body: ImTabPage(
            store: _FakeImStore(),
            currentUserId: '',
          ),
        ),
      ),
    );
    await tester.pump(const Duration(milliseconds: 100));

    expect(find.text('请先在「我的 → 我的钱包」创建热钱包'), findsOneWidget);
  });

  testWidgets('信息 Tab opens and polls Cloudflare mailbox automatically',
      (tester) async {
    final runtime = _FakeRuntime(address: 'alice-wallet');
    final store = _FakeImStore();

    await tester.pumpWidget(
      MaterialApp(
        home: Scaffold(
          body: ImTabPage(
            store: store,
            currentUserId: 'alice-wallet',
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

  testWidgets('信息 Tab uses realtime notice before polling fallback',
      (tester) async {
    final runtime = _FakeRuntime(
      address: 'alice-wallet',
      enableRealtime: true,
    );
    final store = _FakeImStore();

    await tester.pumpWidget(
      MaterialApp(
        home: Scaffold(
          body: ImTabPage(
            store: store,
            currentUserId: 'alice-wallet',
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

  testWidgets('聊天页 opens and polls pending mailbox automatically',
      (tester) async {
    var syncCount = 0;
    final store = _FakeImStore();

    await tester.pumpWidget(
      MaterialApp(
        home: ImChatPage(
          conversationId: 'dm:alice-wallet:bob-wallet',
          currentUserId: 'alice-wallet',
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
    final store = _FakeImStore();

    await tester.pumpWidget(
      MaterialApp(
        home: ImChatPage(
          conversationId: 'dm:alice-wallet:bob-wallet',
          currentUserId: 'alice-wallet',
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

  testWidgets('聊天页 attachment button sends selected encrypted attachment',
      (tester) async {
    ImAttachmentDraft? sentAttachment;
    final store = _FakeImStore();

    await tester.pumpWidget(
      MaterialApp(
        home: ImChatPage(
          conversationId: 'dm:alice-wallet:bob-wallet',
          currentUserId: 'alice-wallet',
          peerUserId: 'bob-wallet',
          title: 'Bob',
          store: store,
          pickAttachment: () async => const ImAttachmentDraft(
            fileName: 'note.txt',
            contentType: 'text/plain',
            bytes: [1, 2, 3],
          ),
          onSendAttachment: (attachment) async {
            sentAttachment = attachment;
          },
        ),
      ),
    );
    await tester.pumpAndSettle();

    await tester.tap(find.byIcon(Icons.attachment));
    await tester.pumpAndSettle();

    expect(sentAttachment?.fileName, 'note.txt');
    expect(sentAttachment?.bytes, [1, 2, 3]);
  });

  testWidgets('聊天页 taps attachment message to download and decrypt',
      (tester) async {
    final store = _FakeImStore(
      messages: const [
        ImStoredMessage(
          envelopeId: 'env-attachment',
          conversationId: 'dm:alice-wallet:bob-wallet',
          direction: 'incoming',
          senderChatAccount: 'bob-wallet',
          recipientChatAccount: 'alice-wallet',
          messageKind: ImMessageKind.attachment,
          deliveryState: ImMessageDeliveryState.receivedByDevice,
          createdAtMillis: 3000,
          plaintext: '{"type":"gmb_im_attachment_v1","file_name":"photo.txt"}',
        ),
      ],
    );
    String? downloadedPlaintext;

    await tester.pumpWidget(
      MaterialApp(
        home: ImChatPage(
          conversationId: 'dm:alice-wallet:bob-wallet',
          currentUserId: 'alice-wallet',
          peerUserId: 'bob-wallet',
          title: 'Bob',
          store: store,
          onDownloadAttachment: (conversationId, controlPlaintext) async {
            downloadedPlaintext = controlPlaintext;
            return const ImDownloadedAttachment(
              attachmentId: 'att-1',
              fileName: 'photo.txt',
              contentType: 'text/plain',
              clearByteSize: 3,
              filePath: '/tmp/photo.txt',
              bytes: [1, 2, 3],
            );
          },
        ),
      ),
    );
    await tester.pumpAndSettle();

    await tester.tap(find.text('[附件] photo.txt'));
    await tester.pumpAndSettle();

    expect(downloadedPlaintext, contains('gmb_im_attachment_v1'));
    expect(find.text('附件已保存：photo.txt'), findsOneWidget);
  });

  testWidgets('聊天页 deletes local conversation from menu and returns',
      (tester) async {
    final store = _FakeImStore(
      messages: const [
        ImStoredMessage(
          envelopeId: 'env-delete-ui',
          conversationId: 'dm:alice-wallet:bob-wallet',
          direction: 'incoming',
          senderChatAccount: 'bob-wallet',
          recipientChatAccount: 'alice-wallet',
          messageKind: ImMessageKind.text,
          deliveryState: ImMessageDeliveryState.receivedByDevice,
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
                      builder: (_) => ImChatPage(
                        conversationId: 'dm:alice-wallet:bob-wallet',
                        currentUserId: 'alice-wallet',
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
}

class _FakeImStore extends ImIsarStore {
  _FakeImStore({
    List<ImConversationPreview> conversations = const [],
    List<ImStoredMessage> messages = const [],
  })  : _conversations = List<ImConversationPreview>.from(conversations),
        _messages = List<ImStoredMessage>.from(messages);

  final List<ImConversationPreview> _conversations;
  final List<ImStoredMessage> _messages;
  String? lastOwnerFilter;
  int readPreviewCount = 0;
  int readMessagesCount = 0;
  final List<String> deletedConversationIds = <String>[];

  @override
  Future<List<ImConversationPreview>> readConversationPreviews({
    String? ownerChatAccount,
  }) async {
    readPreviewCount += 1;
    lastOwnerFilter = ownerChatAccount;
    return List<ImConversationPreview>.from(_conversations);
  }

  @override
  Future<List<ImStoredMessage>> readMessages(String conversationId) async {
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

class _FakeRuntime extends ImRuntime {
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
  Future<String?> readCommunicationAddress() async {
    return address;
  }

  @override
  Future<int> syncPending() async {
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
