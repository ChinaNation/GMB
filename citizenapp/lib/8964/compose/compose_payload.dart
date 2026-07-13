import 'package:citizenapp/8964/models/square_models.dart';

/// 各模式子编辑器交给发布壳的统一载荷：校验失败带 [error]，否则带发布所需字段。
class ComposePayload {
  const ComposePayload.ok({
    required this.text,
    this.title,
    required this.mediaDrafts,
    this.contentBlocks,
  }) : error = null;

  const ComposePayload.invalid(this.error)
      : text = '',
        title = null,
        mediaDrafts = const <SquareLocalMediaDraft>[],
        contentBlocks = null;

  /// 校验错误文案；null 表示通过。
  final String? error;

  /// 供 feed 摘要/搜索的正文纯文本（文章为各文本块拼接）。
  final String text;

  /// 文章标题；动态为 null。
  final String? title;

  /// media_items 顺序（文章=[首图,...内联图]；动态=图集或单视频）。
  final List<SquareLocalMediaDraft> mediaDrafts;

  /// 文章正文图文块；动态为 null。
  final List<Map<String, Object?>>? contentBlocks;

  bool get isValid => error == null;
}

/// 自动保存草稿用的内容快照（不校验，允许不完整，如文章未选首图）。
class ComposeSnapshot {
  const ComposeSnapshot({
    required this.text,
    this.title,
    required this.media,
    this.contentBlocks,
  });

  final String text;
  final String? title;
  final List<SquareLocalMediaDraft> media;
  final List<Map<String, Object?>>? contentBlocks;

  bool get isEmpty =>
      text.trim().isEmpty &&
      media.isEmpty &&
      (title?.trim().isEmpty ?? true);
}

/// 子编辑器对外暴露的接口（发布壳按当前类型调用）：发布校验 + 自动保存快照。
abstract class ComposeBodyCollector {
  ComposePayload collect();
  ComposeSnapshot snapshot();
}
