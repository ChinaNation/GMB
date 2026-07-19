import 'dart:convert';
import 'dart:io';
import 'dart:typed_data';

import 'package:crypto/crypto.dart';

import 'package:citizenapp/8964/models/square_models.dart';
import 'package:citizenapp/8964/services/square_api_client.dart';

class SquareUploadedContent {
  const SquareUploadedContent({
    required this.session,
    required this.postId,
    required this.contentHash,
    required this.storageReceiptId,
    required this.storageUntil,
    required this.manifestHash,
  });

  final SquareSession session;
  final String postId;
  final String contentHash;
  final String storageReceiptId;
  final int storageUntil;
  final String manifestHash;
}

class SquarePreparedContent {
  SquarePreparedContent({
    required this.session,
    required this.preparedUpload,
    required this.postId,
    required this.contentHash,
    required this.storageReceiptId,
    required this.storageUntil,
    required this.manifestHash,
    required this.manifestBytes,
    required List<SquareLocalMediaDraft> mediaDrafts,
  }) : mediaDrafts = List.unmodifiable(mediaDrafts);

  final SquareSession session;
  final SquarePreparedUpload preparedUpload;
  final String postId;
  final String contentHash;
  final String storageReceiptId;
  final int storageUntil;
  final String manifestHash;
  final Uint8List manifestBytes;
  final List<SquareLocalMediaDraft> mediaDrafts;
}

abstract class SquareContentUploader {
  Future<SquarePreparedContent> preparePostContent({
    required String ownerAccount,
    required SquarePostCategory postCategory,
    required String text,
    required List<SquareLocalMediaDraft> mediaDrafts,
    required SquareLoginSigner signLoginPayload,
    SquarePostContentFormat contentFormat,
    String? title,
    List<Map<String, Object?>>? contentBlocks,
    void Function(SquarePublishStage stage)? onStage,
  });

  Future<SquareUploadedContent> uploadPreparedContent(
    SquarePreparedContent prepared, {
    void Function(SquarePublishStage stage)? onStage,
  });
}

class SquareUploadService implements SquareContentUploader {
  SquareUploadService({SquareApiClient? apiClient})
      : _api = apiClient ?? SquareApiClient();

  final SquareApiClient _api;

  @override
  Future<SquarePreparedContent> preparePostContent({
    required String ownerAccount,
    required SquarePostCategory postCategory,
    required String text,
    required List<SquareLocalMediaDraft> mediaDrafts,
    required SquareLoginSigner signLoginPayload,
    SquarePostContentFormat contentFormat = SquarePostContentFormat.normal,
    String? title,
    List<Map<String, Object?>>? contentBlocks,
    void Function(SquarePublishStage stage)? onStage,
  }) async {
    if (mediaDrafts.isEmpty) {
      throw const SquareApiException('请至少选择一张图片或一个视频');
    }

    onStage?.call(SquarePublishStage.signingIn);
    final session = await _api.ensureSession(
      ownerAccount: ownerAccount,
      signLoginPayload: signLoginPayload,
    );

    final mediaManifests = <Map<String, Object?>>[];
    for (final draft in mediaDrafts) {
      final file = File(draft.path);
      final digest = await sha256.bind(file.openRead()).first;
      mediaManifests.add({
        'media_kind': draft.mediaKind.workerValue,
        'file_name': draft.fileName,
        'content_type': draft.contentType,
        'byte_size': draft.byteSize,
        'sha256': digest.toString(),
        if (draft.durationSeconds != null)
          'duration_seconds': draft.durationSeconds,
      });
    }

    final trimmedTitle = title?.trim() ?? '';
    final manifestBytes = _canonicalJsonBytes({
      'schema': 'citizenapp.square.post.v1',
      'owner_account': ownerAccount,
      'post_category': postCategory.workerValue,
      // 普通帖不写 content_format/title，保持旧 manifest 形状；文章才带。
      if (contentFormat != SquarePostContentFormat.normal)
        'content_format': contentFormat.workerValue,
      if (trimmedTitle.isNotEmpty) 'title': trimmedTitle,
      'text': text,
      // 文章正文图文块（内联图按 media_index 引用 media_items）；动态不写。
      if (contentBlocks != null && contentBlocks.isNotEmpty)
        'content_blocks': contentBlocks,
      'media_items': mediaManifests,
    });
    final manifestHash = sha256.convert(manifestBytes).toString();

    onStage?.call(SquarePublishStage.preparingStorage);
    final membership = await _api.fetchMembership(session);
    if (!membership.active || membership.paidUntil <= 0) {
      throw const SquareApiException('需要有效会员才能使用广场内容存储');
    }
    _validateMembershipQuota(
      membership: membership,
      contentFormat: contentFormat,
      titleLength: contentFormat == SquarePostContentFormat.article
          ? trimmedTitle.length
          : 0,
      textLength: text.trim().length,
      mediaDrafts: mediaDrafts,
    );

    final prepared = await _api.prepareUpload(
      session: session,
      postCategory: postCategory,
      contentFormat: contentFormat,
      titleLength: contentFormat == SquarePostContentFormat.article
          ? trimmedTitle.length
          : 0,
      textLength: text.trim().length,
      manifestHash: manifestHash,
      mediaItems: mediaDrafts
          .map(
            (draft) => SquareUploadMediaRequest(
              mediaKind: draft.mediaKind,
              contentType: draft.contentType,
              byteSize: draft.byteSize,
              fileExt: draft.fileExt,
              durationSeconds: draft.durationSeconds,
            ),
          )
          .toList(growable: false),
    );
    if (prepared.mediaItems.length != mediaDrafts.length) {
      throw const SquareApiException('上传授权数量与本地媒体数量不一致');
    }

    return SquarePreparedContent(
      session: session,
      preparedUpload: prepared,
      postId: prepared.postId,
      contentHash: manifestHash,
      storageReceiptId: prepared.storageReceiptId,
      storageUntil: membership.paidUntil,
      manifestHash: manifestHash,
      manifestBytes: manifestBytes,
      mediaDrafts: mediaDrafts,
    );
  }

  @override
  Future<SquareUploadedContent> uploadPreparedContent(
    SquarePreparedContent prepared, {
    void Function(SquarePublishStage stage)? onStage,
  }) async {
    final mediaDrafts = prepared.mediaDrafts;
    final preparedUpload = prepared.preparedUpload;
    if (preparedUpload.mediaItems.length != mediaDrafts.length) {
      throw const SquareApiException('上传授权数量与本地媒体数量不一致');
    }

    // 媒体只允许在链上扣费入块后写入；prepare 阶段只固定链上索引所需的回执。
    onStage?.call(SquarePublishStage.uploadingMedia);
    await _api.uploadObject(
      uploadUrl: preparedUpload.manifestUploadUrl,
      contentType: 'application/json; charset=utf-8',
      body: prepared.manifestBytes,
      session: prepared.session,
    );
    for (var i = 0; i < mediaDrafts.length; i++) {
      final draft = mediaDrafts[i];
      final upload = preparedUpload.mediaItems[i];
      await _api.uploadMediaAsset(
        upload: upload,
        filePath: draft.path,
        session: prepared.session,
      );
    }

    onStage?.call(SquarePublishStage.completingStorage);
    final completed = await _api.completeUpload(
      session: prepared.session,
      uploadId: preparedUpload.uploadId,
      manifestHash: prepared.manifestHash,
      contentHash: prepared.contentHash,
    );
    if (completed.postId != prepared.postId ||
        completed.storageReceiptId != prepared.storageReceiptId) {
      throw const SquareApiException('存储完成响应与链上发布索引不一致');
    }

    return SquareUploadedContent(
      session: prepared.session,
      postId: completed.postId,
      contentHash: completed.contentHash,
      storageReceiptId: completed.storageReceiptId,
      storageUntil: prepared.storageUntil,
      manifestHash: prepared.manifestHash,
    );
  }

  Uint8List _canonicalJsonBytes(Map<String, Object?> value) {
    return Uint8List.fromList(utf8.encode(jsonEncode(value)));
  }

  // 只按会员档校验用量额度（发帖分类权限须竞选身份，按身份档在 compose 层校验；
  // 会员与身份解耦，用户 2026-07-16）。
  void _validateMembershipQuota({
    required SquareMembershipState membership,
    required SquarePostContentFormat contentFormat,
    required int titleLength,
    required int textLength,
    required List<SquareLocalMediaDraft> mediaDrafts,
  }) {
    final plan = membership.activePlan;
    if (plan == null) {
      throw const SquareApiException('会员套餐信息不完整，请刷新会员状态后重试');
    }
    if (contentFormat == SquarePostContentFormat.article) {
      _validateArticleQuota(
        plan: plan,
        titleLength: titleLength,
        textLength: textLength,
        mediaDrafts: mediaDrafts,
      );
      return;
    }
    _validateDynamicQuota(
      plan: plan,
      textLength: textLength,
      mediaDrafts: mediaDrafts,
    );
  }

  void _validateDynamicQuota({
    required SquareMembershipPlan plan,
    required int textLength,
    required List<SquareLocalMediaDraft> mediaDrafts,
  }) {
    if (textLength > plan.dynamicTextMaxChars) {
      throw SquareApiException('动态文字不能超过 ${plan.dynamicTextMaxChars} 字');
    }
    final imageCount = mediaDrafts
        .where((draft) => draft.mediaKind == SquareMediaKind.image)
        .length;
    final videoCount = mediaDrafts
        .where((draft) => draft.mediaKind == SquareMediaKind.video)
        .length;
    if (imageCount > plan.dynamicMaxImages) {
      throw SquareApiException('动态图片不能超过 ${plan.dynamicMaxImages} 张');
    }
    if (videoCount > plan.dynamicMaxVideos) {
      throw SquareApiException('动态视频不能超过 ${plan.dynamicMaxVideos} 个');
    }
    for (final draft in mediaDrafts) {
      if (draft.mediaKind == SquareMediaKind.video &&
          (draft.durationSeconds ?? 0) > plan.dynamicMaxVideoSeconds) {
        throw SquareApiException(
          '单个视频不能超过 ${plan.dynamicMaxVideoSeconds} 秒',
        );
      }
    }
  }

  void _validateArticleQuota({
    required SquareMembershipPlan plan,
    required int titleLength,
    required int textLength,
    required List<SquareLocalMediaDraft> mediaDrafts,
  }) {
    if (titleLength < plan.articleTitleMinChars ||
        titleLength > plan.articleTitleMaxChars) {
      throw SquareApiException(
        '文章标题必须是 ${plan.articleTitleMinChars}-${plan.articleTitleMaxChars} 字',
      );
    }
    if (textLength == 0) {
      throw const SquareApiException('文章正文不能为空');
    }
    if (textLength > plan.articleBodyMaxChars) {
      throw SquareApiException('文章正文不能超过 ${plan.articleBodyMaxChars} 字');
    }
    final hasVideo =
        mediaDrafts.any((draft) => draft.mediaKind == SquareMediaKind.video);
    if (hasVideo) {
      throw const SquareApiException('文章不能上传视频');
    }
    if (mediaDrafts.isEmpty ||
        mediaDrafts.first.mediaKind != SquareMediaKind.image) {
      throw const SquareApiException('文章必须上传 1 张首图');
    }
    final bodyImageCount = mediaDrafts.length - 1;
    if (bodyImageCount > plan.articleMaxImages) {
      throw SquareApiException('文章正文图片不能超过 ${plan.articleMaxImages} 张');
    }
  }
}
