import 'dart:convert';

import 'package:crypto/crypto.dart';

import 'package:citizenapp/8964/models/square_models.dart';
import 'package:citizenapp/8964/services/square_post_store.dart';

class SquareLocalPostPresenterException implements Exception {
  const SquareLocalPostPresenterException(this.message);

  final String message;

  @override
  String toString() => message;
}

/// 本人本地发布副本的展示结果。
///
/// [unavailableMediaKinds] 只表示 manifest 声明过对应媒体，但本地副本不保存媒体字节；
/// UI 必须明确提示云端媒体已清理，禁止把对象键、文件名或空字符串拼成可访问 URL。
class SquareLocalPostPresentation {
  const SquareLocalPostPresentation({
    required this.post,
    required this.unavailableMediaKinds,
  });

  final SquarePost post;
  final Set<SquareMediaKind> unavailableMediaKinds;
}

/// 把通过 [SquarePostStore] 完整性闸门的原始 manifest 转成广场只读展示模型。
///
/// 本转换仍会复核 SHA-256 和关键外层字段，防止调用方绕过 Store 直接构造损坏值。
/// 媒体只有声明，没有本地字节，因此不会生成 [SquareMediaItem] 或猜测下载地址。
class SquareLocalPostPresenter {
  const SquareLocalPostPresenter();

  SquareLocalPostPresentation present(SquareLocalPost local) {
    final actualHash = sha256.convert(local.manifestBytes).toString();
    if (actualHash != local.contentHash) {
      throw const SquareLocalPostPresenterException(
        '本地 manifest 与 content_hash 不一致',
      );
    }

    final manifest = _decodeManifest(local);
    final postCategory = _parsePostCategory(local.postCategory);
    final contentFormat = _parseContentFormat(local.contentFormat);
    final text = manifest['text'];
    final mediaItems = manifest['media_items'];
    if (text is! String || mediaItems is! List) {
      throw const SquareLocalPostPresenterException(
        '本地 manifest 正文或媒体声明不完整',
      );
    }

    final unavailableMediaKinds = <SquareMediaKind>{};
    for (final item in mediaItems) {
      if (item is! Map) {
        throw const SquareLocalPostPresenterException('本地 manifest 媒体声明损坏');
      }
      final mediaKind = item['media_kind'];
      if (mediaKind == SquareMediaKind.image.workerValue) {
        unavailableMediaKinds.add(SquareMediaKind.image);
      } else if (mediaKind == SquareMediaKind.video.workerValue) {
        unavailableMediaKinds.add(SquareMediaKind.video);
      } else {
        throw const SquareLocalPostPresenterException(
          '本地 manifest media_kind 不合法',
        );
      }
    }

    final title = manifest['title'];
    if (title != null && title is! String) {
      throw const SquareLocalPostPresenterException('本地 manifest title 不合法');
    }
    if (contentFormat == SquarePostContentFormat.article &&
        (title is! String || title.trim().isEmpty)) {
      throw const SquareLocalPostPresenterException('本地文章 title 不能为空');
    }

    return SquareLocalPostPresentation(
      post: SquarePost(
        postId: local.postId,
        author: SquareAuthor(
          accountId: local.accountId,
          cidNumber: local.cidNumber,
        ),
        postCategory: postCategory,
        contentFormat: contentFormat,
        title: title as String?,
        text: text,
        contentBlocks: parseArticleContentBlocks(manifest['content_blocks']),
        createdAt: DateTime.fromMillisecondsSinceEpoch(local.createdAt),
        contentHash: local.contentHash,
        storageReceiptId: local.storageReceiptId,
        chainBlock: local.chainBlock,
      ),
      unavailableMediaKinds: Set<SquareMediaKind>.unmodifiable(
        unavailableMediaKinds,
      ),
    );
  }

  static Map<String, dynamic> _decodeManifest(SquareLocalPost local) {
    try {
      final decoded = jsonDecode(
        utf8.decode(local.manifestBytes, allowMalformed: false),
      );
      if (decoded is! Map<String, dynamic> ||
          decoded['schema'] != SquarePostStore.manifestSchema ||
          decoded['account_id'] != local.accountId ||
          decoded['post_category'] != local.postCategory ||
          (decoded['content_format'] ?? 'normal') != local.contentFormat) {
        throw const SquareLocalPostPresenterException(
          '本地 manifest 与发布事实不一致',
        );
      }
      return decoded;
    } on SquareLocalPostPresenterException {
      rethrow;
    } on Object {
      throw const SquareLocalPostPresenterException(
        '本地 manifest 不是合法 UTF-8 JSON',
      );
    }
  }

  static SquarePostCategory _parsePostCategory(String value) {
    for (final category in SquarePostCategory.values) {
      if (category.workerValue == value) return category;
    }
    throw const SquareLocalPostPresenterException('本地 post_category 不合法');
  }

  static SquarePostContentFormat _parseContentFormat(String value) {
    for (final format in SquarePostContentFormat.values) {
      if (format.workerValue == value) return format;
    }
    throw const SquareLocalPostPresenterException('本地 content_format 不合法');
  }
}
