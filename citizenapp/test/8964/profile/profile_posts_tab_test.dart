import 'dart:async';
import 'dart:convert';
import 'dart:typed_data';

import 'package:crypto/crypto.dart';
import 'package:flutter/material.dart';
import 'package:flutter_test/flutter_test.dart';

import 'package:citizenapp/8964/models/square_models.dart';
import 'package:citizenapp/8964/profile/services/square_session_provider.dart';
import 'package:citizenapp/8964/profile/user_profile_page.dart';
import 'package:citizenapp/8964/services/square_api_client.dart';
import 'package:citizenapp/8964/services/square_post_store.dart';

import 'fake_profile.dart';

Widget _page(
  FakeProfileApi api, {
  bool withSession = true,
  bool isSelf = true,
  SquareSessionProvider? sessionProvider,
}) =>
    MaterialApp(
      home: UserProfilePage(
        cidNumber: fakeSession().cidNumber,
        isSelf: isSelf,
        api: api,
        cache: FakeProfileCache(),
        sessionProvider: sessionProvider ??
            FakeSessionProvider(withSession ? fakeSession() : null),
      ),
    );

class _DelayedSessionProvider extends SquareSessionProvider {
  final Completer<SquareSession?> completer = Completer<SquareSession?>();
  int calls = 0;

  @override
  Future<SquareSession?> ensureSession() {
    calls++;
    return completer.future;
  }
}

class _RefreshingSessionProvider extends SquareSessionProvider {
  int refreshCalls = 0;

  SquareSession _session(String token) => SquareSession(
        sessionToken: token,
        cidNumber: fakeSession().cidNumber,
        bindingRevision: 1,
        accountId: kOwner,
        expiresAt: DateTime.now().millisecondsSinceEpoch + 60000,
      );

  @override
  Future<SquareSession?> ensureSession() async => _session('stale-token');

  @override
  Future<SquareSession?> refreshSession() async {
    refreshCalls++;
    return _session('fresh-token');
  }
}

class _PendingAuthorPostsApi extends FakeProfileApi {
  _PendingAuthorPostsApi() : super(sampleProfile());

  final Completer<({List<SquarePost> posts, int? nextCursor})> completer =
      Completer();

  @override
  Future<({List<SquarePost> posts, int? nextCursor})> fetchAuthorPosts(
    String cidNumber, {
    SquarePostCategory? category,
    SquarePostContentFormat? contentFormat,
    int limit = 20,
    int? cursor,
    SquareSession? session,
  }) =>
      completer.future;
}

SquareLocalPost _localPost({
  String postId = 'local-1',
  String text = '本机保留的正文',
}) {
  final bytes = Uint8List.fromList(
    utf8.encode(
      jsonEncode({
        'schema': SquarePostStore.manifestSchema,
        'account_id': kOwner,
        'post_category': 'normal',
        'content_format': 'normal',
        'text': text,
        'media_items': [
          {
            'media_kind': 'image',
            'file_name': 'photo.jpg',
            'content_type': 'image/jpeg',
            'byte_size': 123,
            'sha256': 'a' * 64,
          },
        ],
      }),
    ),
  );
  return SquareLocalPost(
    postId: postId,
    cidNumber: fakeSession().cidNumber,
    accountId: kOwner,
    postCategory: 'normal',
    contentFormat: 'normal',
    manifestBytes: bytes,
    contentHash: sha256.convert(bytes).toString(),
    storageReceiptId: 'receipt-1',
    chainBlock: 88,
    createdAt: 1700000000000,
    postState: SquarePostStore.publishedState,
  );
}

void main() {
  testWidgets('主页帖子未返回时直接显示内容区域且不使用整页转圈', (tester) async {
    final api = _PendingAuthorPostsApi();
    await tester.pumpWidget(_page(api, isSelf: false));
    await tester.pump();
    await tester.pump();

    expect(find.text('帖子'), findsOneWidget);
    expect(find.text('正在读取内容'), findsOneWidget);
    expect(
      find.byKey(const ValueKey('profile-posts-load-progress')),
      findsOneWidget,
    );
    expect(find.byType(CircularProgressIndicator), findsNothing);

    api.completer.complete((posts: const <SquarePost>[], nextCursor: null));
    await tester.pumpAndSettle();
    expect(
      find.byKey(const ValueKey('profile-posts-load-progress')),
      findsNothing,
    );
  });
  testWidgets('posts tab renders normal author posts', (tester) async {
    final api = FakeProfileApi(
      sampleProfile(),
      authorPosts: [samplePost(id: 'n1', text: '普通帖子内容')],
    );
    await tester.pumpWidget(_page(api));
    await tester.pumpAndSettle();

    expect(find.text('普通帖子内容'), findsOneWidget);
  });

  testWidgets('campaign tab filters to campaign posts', (tester) async {
    final api = FakeProfileApi(
      sampleProfile(),
      authorPosts: [
        samplePost(id: 'n1', text: '普通内容'),
        samplePost(
          id: 'c1',
          text: '竞选宣言内容',
          category: SquarePostCategory.campaign,
        ),
      ],
    );
    await tester.pumpWidget(_page(api));
    await tester.pumpAndSettle();

    await tester.tap(find.text('竞选'));
    await tester.pumpAndSettle();

    expect(find.text('竞选宣言内容'), findsOneWidget);
    expect(find.text('普通内容'), findsNothing);
  });

  testWidgets('photos tab derives image media tiles', (tester) async {
    final api = FakeProfileApi(
      sampleProfile(),
      authorPosts: [
        samplePost(
          id: 'p1',
          media: const [
            SquareMediaItem(mediaKind: SquareMediaKind.image, url: 'a'),
          ],
        ),
      ],
    );
    await tester.pumpWidget(_page(api));
    await tester.pumpAndSettle();

    await tester.tap(find.text('照片'));
    await tester.pumpAndSettle();

    expect(find.byIcon(Icons.image_rounded), findsWidgets);
  });

  testWidgets('articles tab renders article cards with title', (tester) async {
    final api = FakeProfileApi(
      sampleProfile(),
      authorPosts: [
        samplePost(
          id: 'a1',
          contentFormat: SquarePostContentFormat.article,
          title: '我的第一篇文章',
          text: '正文内容',
        ),
      ],
    );
    await tester.pumpWidget(_page(api));
    await tester.pumpAndSettle();

    await tester.tap(find.text('文章'));
    await tester.pumpAndSettle();

    expect(find.text('我的第一篇文章'), findsOneWidget);
  });

  testWidgets('posts tab excludes articles', (tester) async {
    final api = FakeProfileApi(
      sampleProfile(),
      authorPosts: [
        samplePost(id: 'n1', text: '普通帖子正文'),
        samplePost(
          id: 'a1',
          contentFormat: SquarePostContentFormat.article,
          title: '文章标题',
          text: '文章正文',
        ),
      ],
    );
    await tester.pumpWidget(_page(api));
    await tester.pumpAndSettle();

    expect(find.text('普通帖子正文'), findsOneWidget);
    expect(find.text('文章正文'), findsNothing);
  });

  testWidgets('empty posts tab shows the empty label', (tester) async {
    await tester.pumpWidget(_page(FakeProfileApi(sampleProfile())));
    await tester.pumpAndSettle();

    expect(find.text('还没有帖子'), findsOneWidget);
  });

  testWidgets('本人主页远端失败时仍展示本地正文并明确提示媒体已清理', (tester) async {
    final api = FakeProfileApi(
      sampleProfile(),
      localPosts: [_localPost()],
      throwOnAuthorPosts: true,
    );

    await tester.pumpWidget(_page(api));
    await tester.pumpAndSettle();

    expect(find.text('本机保留的正文'), findsOneWidget);
    expect(find.text('媒体已从云端清理，本机仅保留正文'), findsOneWidget);
    expect(find.text('加载失败，下拉重试'), findsNothing);
  });

  testWidgets('本人主页无法建立会话时仍从 CID 本地副本展示正文', (tester) async {
    final api = FakeProfileApi(
      sampleProfile(),
      localPosts: [_localPost()],
      throwOnAuthorPosts: true,
    );

    await tester.pumpWidget(_page(api, withSession: false));
    await tester.pumpAndSettle();

    expect(api.localPostCalls, 1);
    expect(find.text('本机保留的正文'), findsOneWidget);
    expect(find.text('媒体已从云端清理，本机仅保留正文'), findsOneWidget);
    expect(find.text('加载失败，下拉重试'), findsNothing);
    expect(find.byType(CircularProgressIndicator), findsNothing);
  });

  testWidgets('无会话的他人主页禁止读取本机本人副本', (tester) async {
    final api = FakeProfileApi(
      sampleProfile(),
      localPosts: [_localPost()],
      throwOnAuthorPosts: true,
    );

    await tester.pumpWidget(
      _page(api, withSession: false, isSelf: false),
    );
    await tester.pumpAndSettle();

    expect(api.localPostCalls, 0);
    expect(api.authorPostCalls, 0);
    expect(find.text('本机保留的正文'), findsNothing);
    expect(find.text('加载失败，下拉重试'), findsNothing);
    expect(find.text('需要钱包账户才能浏览主页'), findsOneWidget);
  });

  testWidgets('首次会话仍在建立时不抢跑，Session 到达后当前帖子 Tab 自动加载', (tester) async {
    final provider = _DelayedSessionProvider();
    final api = FakeProfileApi(
      sampleProfile(),
      authorPosts: [samplePost(id: 'ready', text: '会话就绪后自动出现')],
    );

    await tester.pumpWidget(_page(api, sessionProvider: provider));
    await tester.pump();
    await tester.pump();

    expect(provider.calls, 1);
    expect(api.authorPostCalls, 0);
    expect(find.text('加载失败，下拉重试'), findsNothing);

    provider.completer.complete(fakeSession());
    await tester.pumpAndSettle();

    expect(api.authorPostCalls, 1);
    expect(find.text('会话就绪后自动出现'), findsOneWidget);
  });

  testWidgets('帖子请求 401 时只刷新一次 Session 并自动恢复', (tester) async {
    final provider = _RefreshingSessionProvider();
    final api = FakeProfileApi(
      sampleProfile(),
      authorPosts: [samplePost(id: 'retried', text: '刷新会话后出现')],
      unauthorizedAuthorPosts: 1,
    );

    await tester.pumpWidget(_page(api, sessionProvider: provider));
    await tester.pumpAndSettle();

    expect(provider.refreshCalls, 1);
    expect(api.authorPostCalls, 2);
    expect(api.authorPostSessions.last?.sessionToken, 'fresh-token');
    expect(find.text('刷新会话后出现'), findsOneWidget);
    expect(find.text('加载失败，下拉重试'), findsNothing);
  });

  testWidgets('同一 post_id 的 Worker 内容覆盖本地展示内容', (tester) async {
    final api = FakeProfileApi(
      sampleProfile(),
      localPosts: [_localPost(text: '本地旧展示')],
      authorPosts: [samplePost(id: 'local-1', text: 'Worker 最新展示')],
    );

    await tester.pumpWidget(_page(api));
    await tester.pumpAndSettle();

    expect(find.text('Worker 最新展示'), findsOneWidget);
    expect(find.text('本地旧展示'), findsNothing);
  });
}
