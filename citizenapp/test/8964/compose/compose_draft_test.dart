import 'package:flutter_test/flutter_test.dart';

import 'package:citizenapp/8964/compose/drafts/compose_draft.dart';
import 'package:citizenapp/8964/models/square_models.dart';

SquareLocalMediaDraft _media(SquareMediaKind kind, String name) =>
    SquareLocalMediaDraft(
      mediaKind: kind,
      path: '/drafts/$name',
      fileName: name,
      contentType: kind == SquareMediaKind.video ? 'video/mp4' : 'image/jpeg',
      byteSize: 100,
    );

void main() {
  group('SquareComposeDraft JSON 往返', () {
    test('文章草稿含图文块和媒体完整往返', () {
      final draft = SquareComposeDraft(
        draftId: 'd1',
        accountId:
            '0x8888888888888888888888888888888888888888888888888888888888888888',
        contentFormat: SquarePostContentFormat.article,
        postCategory: SquarePostCategory.campaign,
        title: '论社区自治',
        text: '正文',
        media: [
          _media(SquareMediaKind.image, 'cover.jpg'),
          _media(SquareMediaKind.image, 'inline.jpg'),
        ],
        contentBlocks: const [
          {'t': 'text', 'text': '第一段'},
          {'t': 'image', 'media_index': 1},
        ],
        updatedAtMillis: 1800000000000,
      );
      final restored = SquareComposeDraft.fromJsonString(draft.toJsonString())!;
      expect(restored.draftId, 'd1');
      expect(restored.isArticle, isTrue);
      expect(restored.isCampaign, isTrue);
      expect(restored.title, '论社区自治');
      expect(restored.media.length, 2);
      expect(restored.media[1].fileName, 'inline.jpg');
      expect(restored.contentBlocks!.length, 2);
      expect(restored.contentBlocks![1]['media_index'], 1);
      expect(restored.updatedAtMillis, 1800000000000);
    });

    test('损坏字符串返回 null', () {
      expect(SquareComposeDraft.fromJsonString('不是json'), isNull);
      expect(SquareComposeDraft.fromJsonString(null), isNull);
    });
  });

  group('卡片派生', () {
    test('类型标签按内容形态/媒体/竞选推导', () {
      SquareComposeDraft draft({
        required SquarePostContentFormat format,
        required SquarePostCategory category,
        List<SquareLocalMediaDraft> media = const [],
      }) =>
          SquareComposeDraft(
            draftId: 'd',
            accountId:
                '0xbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbb',
            contentFormat: format,
            postCategory: category,
            text: 't',
            media: media,
            updatedAtMillis: 0,
          );

      expect(
        draft(
          format: SquarePostContentFormat.normal,
          category: SquarePostCategory.normal,
          media: [_media(SquareMediaKind.image, 'a.jpg')],
        ).typeLabel,
        '图片动态',
      );
      expect(
        draft(
          format: SquarePostContentFormat.normal,
          category: SquarePostCategory.campaign,
          media: [_media(SquareMediaKind.video, 'v.mp4')],
        ).typeLabel,
        '竞选视频动态',
      );
      expect(
        draft(
          format: SquarePostContentFormat.article,
          category: SquarePostCategory.normal,
        ).typeLabel,
        '文章',
      );
    });

    test('文章摘要优先标题；空内容 isEmpty', () {
      const article = SquareComposeDraft(
        draftId: 'd',
        accountId:
            '0xbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbb',
        contentFormat: SquarePostContentFormat.article,
        postCategory: SquarePostCategory.normal,
        title: '标题',
        text: '正文',
        media: [],
        updatedAtMillis: 0,
      );
      expect(article.summary, '标题');

      const empty = SquareComposeDraft(
        draftId: 'd',
        accountId:
            '0xbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbb',
        contentFormat: SquarePostContentFormat.normal,
        postCategory: SquarePostCategory.normal,
        text: '  ',
        media: [],
        updatedAtMillis: 0,
      );
      expect(empty.isEmpty, isTrue);
    });
  });
}
