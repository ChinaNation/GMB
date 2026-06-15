import 'package:flutter/material.dart';
import 'package:flutter_test/flutter_test.dart';
import 'package:wuminapp_mobile/im/im_session_models.dart';
import 'package:wuminapp_mobile/im/im_tab_page.dart';
import 'package:wuminapp_mobile/im/storage/im_isar_store.dart';

void main() {
  testWidgets('信息 Tab renders conversation list for communication account',
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

    expect(find.text('信息'), findsOneWidget);
    expect(find.text('Bob'), findsOneWidget);
    expect(find.text('bob-wallet'), findsNothing);
    expect(find.text('hello'), findsOneWidget);
    expect(store.lastOwnerFilter, 'alice-wallet');
    expect(find.byIcon(Icons.add_comment_outlined), findsNothing);
    expect(find.byIcon(Icons.qr_code_scanner_rounded), findsNothing);
    expect(find.byIcon(Icons.qr_code_2_rounded), findsNothing);
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

    expect(find.text('请先在用户资料中设置通信账户'), findsOneWidget);
  });
}

class _FakeImStore extends ImIsarStore {
  _FakeImStore({
    List<ImConversationPreview> conversations = const [],
  }) : _conversations = List<ImConversationPreview>.from(conversations);

  final List<ImConversationPreview> _conversations;
  String? lastOwnerFilter;

  @override
  Future<List<ImConversationPreview>> readConversationPreviews({
    String? ownerChatAccount,
  }) async {
    lastOwnerFilter = ownerChatAccount;
    return List<ImConversationPreview>.from(_conversations);
  }

  @override
  Future<int> outboundQueueCount() async {
    return 0;
  }
}
