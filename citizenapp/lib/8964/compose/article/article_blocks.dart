import 'package:citizenapp/8964/models/square_models.dart';

// 阅读侧图文块模型 [ArticleContentBlock] + parseArticleContentBlocks 定义在
// square_models.dart，避免与本文件（依赖 SquareLocalMediaDraft）循环导入。
// 本文件放编辑侧常量、校验与拍平逻辑。

const int articleTitleMin = 10;
const int articleTitleMax = 50;
const int articleBodyMax = 30000;

/// 文章发布校验（纯函数，便于单测）。返回错误文案，null 表示通过。
String? articleValidationError({
  required String title,
  required bool hasCover,
  required String body,
}) {
  final trimmedTitle = title.trim();
  if (trimmedTitle.length < articleTitleMin ||
      trimmedTitle.length > articleTitleMax) {
    return '标题需 $articleTitleMin–$articleTitleMax 字';
  }
  if (!hasCover) {
    return '请选择 1 张首图';
  }
  final trimmedBody = body.trim();
  if (trimmedBody.isEmpty) {
    return '正文不能为空';
  }
  if (trimmedBody.length > articleBodyMax) {
    return '正文不能超过 $articleBodyMax 字';
  }
  return null;
}

/// 编辑侧图文块的纯值（不含 TextEditingController，便于单测拍平逻辑）。
/// 编辑器持有控制器，发布时读出 `controller.text` 转成本类型。
sealed class ArticleDraftBlock {
  const ArticleDraftBlock();
}

final class ArticleDraftText extends ArticleDraftBlock {
  const ArticleDraftText(this.text);
  final String text;
}

final class ArticleDraftImage extends ArticleDraftBlock {
  const ArticleDraftImage(this.draft);
  final SquareLocalMediaDraft draft;
}

/// 文章拍平结果：发布所需的媒体草稿顺序、正文块 JSON、以及供摘要/feed 的纯文本。
class ArticleManifestParts {
  const ArticleManifestParts({
    required this.mediaDrafts,
    required this.contentBlocks,
    required this.text,
  });

  /// `media_items` 顺序：[首图, ...按块顺序的内联图]。
  final List<SquareLocalMediaDraft> mediaDrafts;

  /// `content_blocks`：[{t:'text',text} | {t:'image',media_index}]。
  final List<Map<String, Object?>> contentBlocks;

  /// 供 feed 摘要/搜索的纯文本（各文本块用空行拼接）。
  final String text;
}

/// 把首图 + 编辑侧图文块拍平成发布参数：
/// 内联图按块顺序追加到 media_items（首图之后），块以 media_index 引用之。
ArticleManifestParts buildArticleManifest({
  required SquareLocalMediaDraft cover,
  required List<ArticleDraftBlock> body,
}) {
  final mediaDrafts = <SquareLocalMediaDraft>[cover];
  final contentBlocks = <Map<String, Object?>>[];
  final textParts = <String>[];
  for (final block in body) {
    switch (block) {
      case ArticleDraftText(:final text):
        contentBlocks.add({'t': 'text', 'text': text});
        if (text.trim().isNotEmpty) textParts.add(text.trim());
      case ArticleDraftImage(:final draft):
        mediaDrafts.add(draft);
        contentBlocks
            .add({'t': 'image', 'media_index': mediaDrafts.length - 1});
    }
  }
  return ArticleManifestParts(
    mediaDrafts: mediaDrafts,
    contentBlocks: contentBlocks,
    text: textParts.join('\n\n'),
  );
}
