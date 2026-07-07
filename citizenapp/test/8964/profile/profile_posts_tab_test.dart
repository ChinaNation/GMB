import 'package:flutter/material.dart';
import 'package:flutter_test/flutter_test.dart';

import 'package:citizenapp/8964/models/square_models.dart';
import 'package:citizenapp/8964/profile/user_profile_page.dart';

import 'profile_test_doubles.dart';

Widget _page(FakeProfileApi api) => MaterialApp(
      home: UserProfilePage(
        ownerAccount: kOwner,
        isSelf: true,
        api: api,
        cache: FakeProfileCache(),
        sessionProvider: FakeSessionProvider(null),
      ),
    );

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
}
