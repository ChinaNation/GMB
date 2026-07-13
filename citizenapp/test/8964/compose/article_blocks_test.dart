import 'package:flutter_test/flutter_test.dart';

import 'package:citizenapp/8964/compose/article/article_blocks.dart';
import 'package:citizenapp/8964/models/square_models.dart';

SquareLocalMediaDraft _img(String name) => SquareLocalMediaDraft(
      mediaKind: SquareMediaKind.image,
      path: '/tmp/$name',
      fileName: name,
      contentType: 'image/jpeg',
      byteSize: 100,
    );

void main() {
  group('buildArticleManifest 拍平', () {
    test('内联图按块顺序追加到首图之后，块以 media_index 引用', () {
      final cover = _img('cover.jpg');
      final a = _img('a.jpg');
      final b = _img('b.jpg');
      final parts = buildArticleManifest(
        cover: cover,
        body: [
          const ArticleDraftText('第一段'),
          ArticleDraftImage(a),
          const ArticleDraftText('第二段'),
          ArticleDraftImage(b),
        ],
      );
      // media_items: [首图, a, b]
      expect(parts.mediaDrafts, [cover, a, b]);
      expect(parts.contentBlocks, [
        {'t': 'text', 'text': '第一段'},
        {'t': 'image', 'media_index': 1},
        {'t': 'text', 'text': '第二段'},
        {'t': 'image', 'media_index': 2},
      ]);
      // 纯文本供 feed 摘要：文本块空行拼接。
      expect(parts.text, '第一段\n\n第二段');
    });

    test('空文本块不进摘要，但保留块占位', () {
      final cover = _img('c.jpg');
      final parts = buildArticleManifest(
        cover: cover,
        body: [const ArticleDraftText('  '), const ArticleDraftText('正文')],
      );
      expect(parts.text, '正文');
      expect(parts.contentBlocks.length, 2);
    });
  });

  group('parseArticleContentBlocks 解析', () {
    test('解析文本/图片块，跳过未知与损坏项', () {
      final blocks = parseArticleContentBlocks([
        {'t': 'text', 'text': '甲'},
        {'t': 'image', 'media_index': 2},
        {'t': 'video'}, // 未知类型跳过
        {'t': 'image'}, // 缺 media_index 跳过
        'garbage', // 非 map 跳过
      ]);
      expect(blocks.length, 2);
      expect((blocks[0] as ArticleTextBlock).text, '甲');
      expect((blocks[1] as ArticleImageBlock).mediaIndex, 2);
    });

    test('空或非数组返回空列表', () {
      expect(parseArticleContentBlocks(null), isEmpty);
      expect(parseArticleContentBlocks('x'), isEmpty);
      expect(parseArticleContentBlocks(const []), isEmpty);
    });
  });
}
