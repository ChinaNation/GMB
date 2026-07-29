import 'dart:convert';
import 'dart:typed_data';

import 'package:crypto/crypto.dart';
import 'package:flutter/material.dart';
import 'package:flutter_test/flutter_test.dart';

import 'package:citizenapp/8964/models/square_models.dart';
import 'package:citizenapp/8964/profile/user_profile_page.dart';
import 'package:citizenapp/8964/services/square_post_store.dart';

import 'fake_profile.dart';

Widget _page(FakeProfileApi api) => MaterialApp(
      home: UserProfilePage(
        cidNumber: fakeSession().cidNumber,
        isSelf: true,
        api: api,
        cache: FakeProfileCache(),
        sessionProvider: FakeSessionProvider(fakeSession()),
      ),
    );

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
