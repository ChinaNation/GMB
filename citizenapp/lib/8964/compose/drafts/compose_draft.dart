import 'dart:convert';

import 'package:citizenapp/8964/models/square_models.dart';

/// 广场草稿箱的一条草稿（全类型：图片/视频动态、文章及竞选变体）。
///
/// 与发布 manifest 同构，便于恢复到发布页并直接发布：
/// media 顺序 = [首图, ...内联图]（文章）/ 图集或单视频（动态）；path 为本地持久副本；
/// contentBlocks 仅文章（内联图按 media_index 引用 media）。
class SquareComposeDraft {
  const SquareComposeDraft({
    required this.draftId,
    required this.ownerAccount,
    required this.contentFormat,
    required this.postCategory,
    this.title,
    required this.text,
    required this.media,
    this.contentBlocks,
    required this.updatedAtMillis,
  });

  final String draftId;
  final String ownerAccount;
  final SquarePostContentFormat contentFormat;
  final SquarePostCategory postCategory;
  final String? title;
  final String text;
  final List<SquareLocalMediaDraft> media;
  final List<Map<String, Object?>>? contentBlocks;
  final int updatedAtMillis;

  bool get isArticle => contentFormat == SquarePostContentFormat.article;
  bool get isCampaign => postCategory == SquarePostCategory.campaign;

  bool get isEmpty =>
      text.trim().isEmpty &&
      media.isEmpty &&
      (title?.trim().isEmpty ?? true);

  /// 卡片类型标签：文章 / 图片动态 / 视频动态（竞选加前缀）。
  String get typeLabel {
    final base = isArticle
        ? '文章'
        : (media.isNotEmpty && media.first.mediaKind == SquareMediaKind.video
            ? '视频动态'
            : '图片动态');
    return isCampaign ? '竞选$base' : base;
  }

  /// 卡片摘要：文章优先标题，否则正文。
  String get summary {
    final trimmedTitle = title?.trim();
    if (isArticle && trimmedTitle != null && trimmedTitle.isNotEmpty) {
      return trimmedTitle;
    }
    return text.trim();
  }

  SquareComposeDraft copyWith({
    List<SquareLocalMediaDraft>? media,
    List<Map<String, Object?>>? contentBlocks,
    int? updatedAtMillis,
  }) {
    return SquareComposeDraft(
      draftId: draftId,
      ownerAccount: ownerAccount,
      contentFormat: contentFormat,
      postCategory: postCategory,
      title: title,
      text: text,
      media: media ?? this.media,
      contentBlocks: contentBlocks ?? this.contentBlocks,
      updatedAtMillis: updatedAtMillis ?? this.updatedAtMillis,
    );
  }

  Map<String, Object?> toJson() => {
        'draft_id': draftId,
        'owner_account': ownerAccount,
        'content_format': contentFormat.workerValue,
        'post_category': postCategory.workerValue,
        if (title != null) 'title': title,
        'text': text,
        'media': media.map(_mediaToJson).toList(),
        if (contentBlocks != null) 'content_blocks': contentBlocks,
        'updated_at': updatedAtMillis,
      };

  String toJsonString() => jsonEncode(toJson());

  static SquareComposeDraft? fromJsonString(String? raw) {
    if (raw == null || raw.isEmpty) return null;
    try {
      final decoded = jsonDecode(raw);
      if (decoded is! Map<String, dynamic>) return null;
      return SquareComposeDraft.fromJson(decoded);
    } on FormatException {
      return null;
    }
  }

  factory SquareComposeDraft.fromJson(Map<String, dynamic> json) {
    final rawMedia = json['media'];
    final rawBlocks = json['content_blocks'];
    return SquareComposeDraft(
      draftId: json['draft_id']?.toString() ?? '',
      ownerAccount: json['owner_account']?.toString() ?? '',
      contentFormat: json['content_format'] == 'article'
          ? SquarePostContentFormat.article
          : SquarePostContentFormat.normal,
      postCategory: json['post_category'] == 'campaign'
          ? SquarePostCategory.campaign
          : SquarePostCategory.normal,
      title: json['title']?.toString(),
      text: json['text']?.toString() ?? '',
      media: rawMedia is List
          ? rawMedia
              .whereType<Map<String, dynamic>>()
              .map(_mediaFromJson)
              .toList()
          : const <SquareLocalMediaDraft>[],
      contentBlocks: rawBlocks is List
          ? rawBlocks
              .whereType<Map>()
              .map((e) => e.map((k, v) => MapEntry(k.toString(), v)))
              .toList()
          : null,
      updatedAtMillis: json['updated_at'] is int
          ? json['updated_at'] as int
          : int.tryParse(json['updated_at']?.toString() ?? '') ?? 0,
    );
  }

  static Map<String, Object?> _mediaToJson(SquareLocalMediaDraft draft) => {
        'media_kind': draft.mediaKind.workerValue,
        'path': draft.path,
        'file_name': draft.fileName,
        'content_type': draft.contentType,
        'byte_size': draft.byteSize,
        if (draft.durationSeconds != null)
          'duration_seconds': draft.durationSeconds,
      };

  static SquareLocalMediaDraft _mediaFromJson(Map<String, dynamic> json) =>
      SquareLocalMediaDraft(
        mediaKind: json['media_kind'] == 'video'
            ? SquareMediaKind.video
            : SquareMediaKind.image,
        path: json['path']?.toString() ?? '',
        fileName: json['file_name']?.toString() ?? '',
        contentType: json['content_type']?.toString() ?? '',
        byteSize: json['byte_size'] is int
            ? json['byte_size'] as int
            : int.tryParse(json['byte_size']?.toString() ?? '') ?? 0,
        durationSeconds: json['duration_seconds'] is int
            ? json['duration_seconds'] as int
            : int.tryParse(json['duration_seconds']?.toString() ?? ''),
      );
}
