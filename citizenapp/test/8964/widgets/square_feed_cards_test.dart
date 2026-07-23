import 'package:flutter/material.dart';
import 'package:flutter_test/flutter_test.dart';

import 'package:citizenapp/8964/models/square_models.dart';
import 'package:citizenapp/8964/widgets/square_article_card.dart';
import 'package:citizenapp/8964/widgets/square_media_grid.dart';
import 'package:citizenapp/8964/widgets/square_post_card.dart';

SquareMediaItem _img({int? w, int? h}) => SquareMediaItem(
      mediaKind: SquareMediaKind.image,
      // 空 url → tile 走占位图标，测试不触网。
      url: '',
      width: w,
      height: h,
    );

SquarePost _post({
  SquarePostCategory category = SquarePostCategory.normal,
  SquarePostContentFormat contentFormat = SquarePostContentFormat.normal,
  String? title,
  String text = '正文',
  List<SquareMediaItem> media = const [],
  String? campaignPosition,
  String? identityLevel = 'voting',
}) {
  return SquarePost(
    postId: 'p1',
    author: SquareAuthor(
      accountId:
          '0xeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeee',
      displayName: '林正华',
      identityLevel: identityLevel,
    ),
    postCategory: category,
    contentFormat: contentFormat,
    title: title,
    text: text,
    createdAt: DateTime.fromMillisecondsSinceEpoch(1000),
    mediaItems: media,
    campaignPosition: campaignPosition,
  );
}

Future<void> _pump(WidgetTester tester, Widget child) {
  return tester.pumpWidget(
    MaterialApp(
      home: Scaffold(
        body: SizedBox(width: 360, child: child),
      ),
    ),
  );
}

void main() {
  group('SquareMediaItem.isPortrait', () {
    test('高大于宽为竖屏', () {
      expect(_img(w: 1080, h: 1920).isPortrait, isTrue);
    });
    test('宽不小于高为横屏', () {
      expect(_img(w: 1920, h: 1080).isPortrait, isFalse);
      expect(_img(w: 1000, h: 1000).isPortrait, isFalse);
    });
    test('宽高缺失按横屏兜底', () {
      expect(_img().isPortrait, isFalse);
    });
  });

  group('SquareMediaGrid 数量与朝向', () {
    testWidgets('横屏单图为 16:9 单块', (tester) async {
      await _pump(
          tester, SquareMediaGrid(mediaItems: [_img(w: 1920, h: 1080)]));
      expect(find.byType(SquareMediaTile), findsOneWidget);
      final ar = tester.widget<AspectRatio>(find.byType(AspectRatio).first);
      expect(ar.aspectRatio, closeTo(16 / 9, 0.001));
    });

    testWidgets('横屏两图并排、无 +N', (tester) async {
      await _pump(
        tester,
        SquareMediaGrid(
          mediaItems: [_img(w: 1600, h: 1200), _img(w: 1600, h: 1200)],
        ),
      );
      expect(find.byType(SquareMediaTile), findsNWidgets(2));
      expect(find.textContaining('+'), findsNothing);
      final ar = tester.widget<AspectRatio>(find.byType(AspectRatio).first);
      expect(ar.aspectRatio, closeTo(2, 0.001));
    });

    testWidgets('横屏四图只出前两张、第二张右下 +2', (tester) async {
      await _pump(
        tester,
        SquareMediaGrid(
          mediaItems: List.generate(4, (_) => _img(w: 1600, h: 1200)),
        ),
      );
      expect(find.byType(SquareMediaTile), findsNWidgets(2));
      expect(find.text('+2'), findsOneWidget);
    });

    testWidgets('竖屏两图容器比例为 3:2', (tester) async {
      await _pump(
        tester,
        SquareMediaGrid(
          mediaItems: [_img(w: 1080, h: 1920), _img(w: 1080, h: 1920)],
        ),
      );
      final ar = tester.widget<AspectRatio>(find.byType(AspectRatio).first);
      expect(ar.aspectRatio, closeTo(3 / 2, 0.001));
    });
  });

  group('SquarePostCard 身份与竖屏布局', () {
    testWidgets('竞选帖显示竞选药丸和岗位', (tester) async {
      await _pump(
        tester,
        SquarePostCard(
          post: _post(
            category: SquarePostCategory.campaign,
            identityLevel: 'candidate',
            campaignPosition: '市长候选人',
          ),
        ),
      );
      expect(find.text('竞选'), findsOneWidget);
      expect(find.textContaining('市长候选人'), findsOneWidget);
    });

    testWidgets('非竞选帖不显示竞选药丸', (tester) async {
      await _pump(
        tester,
        SquarePostCard(post: _post(identityLevel: 'voting')),
      );
      expect(find.text('竞选'), findsNothing);
    });

    testWidgets('竖屏单图走左媒体右文字（正文与媒体块同存）', (tester) async {
      await _pump(
        tester,
        SquarePostCard(
          post: _post(text: '竖图说明', media: [_img(w: 1080, h: 1920)]),
        ),
      );
      expect(find.byType(SquareMediaTile), findsOneWidget);
      expect(find.text('竖图说明'), findsOneWidget);
    });
  });

  group('SquareArticleCard', () {
    testWidgets('标题、正文与强制 16:9 首图并存', (tester) async {
      await _pump(
        tester,
        SquareArticleCard(
          post: _post(
            contentFormat: SquarePostContentFormat.article,
            title: '论社区自治的三个层次',
            text: '正文摘要',
            // 竖首图也强制横屏 16:9；非空 url 使封面块渲染（加载失败走占位）。
            media: const [
              SquareMediaItem(
                mediaKind: SquareMediaKind.image,
                url: 'https://example.com/cover.jpg',
                width: 1080,
                height: 1920,
              ),
            ],
          ),
        ),
      );
      expect(find.text('论社区自治的三个层次'), findsOneWidget);
      expect(find.text('正文摘要'), findsOneWidget);
      final ar = tester.widget<AspectRatio>(find.byType(AspectRatio));
      expect(ar.aspectRatio, closeTo(16 / 9, 0.001));
    });

    testWidgets('无首图时不渲染封面块', (tester) async {
      await _pump(
        tester,
        SquareArticleCard(
          post: _post(
            contentFormat: SquarePostContentFormat.article,
            title: '纯文字文章',
            text: '正文',
          ),
        ),
      );
      expect(find.text('纯文字文章'), findsOneWidget);
    });
  });
}
