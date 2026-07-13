import 'package:flutter/foundation.dart';

import 'package:citizenapp/8964/chain/square_chain_service.dart';
import 'package:citizenapp/8964/models/square_models.dart';
import 'package:citizenapp/8964/services/square_api_client.dart';
import 'package:citizenapp/8964/services/square_identity_state.dart';
import 'package:citizenapp/8964/services/square_upload_service.dart';
import 'package:citizenapp/8964/storage/square_draft_store.dart';
import 'package:citizenapp/rpc/chain_rpc.dart';

class SquarePublishException implements Exception {
  const SquarePublishException(this.message);

  final String message;

  @override
  String toString() => message;
}

class SquarePublishResult {
  const SquarePublishResult({
    required this.post,
    required this.txHash,
    required this.blockHashHex,
    required this.storageUntil,
    this.cleanupWarning,
  });

  final SquarePost post;
  final String txHash;
  final String blockHashHex;
  final int storageUntil;
  final String? cleanupWarning;
}

typedef SquareChainSigner = Future<Uint8List> Function(Uint8List payload);

abstract class SquarePublishBalanceReader {
  Future<double> fetchFreshFinalizedBalanceYuan(String pubkeyHex);
}

class SquareChainBalanceReader implements SquarePublishBalanceReader {
  SquareChainBalanceReader({ChainRpc? chainRpc})
      : _rpc = chainRpc ?? ChainRpc();

  final ChainRpc _rpc;

  @override
  Future<double> fetchFreshFinalizedBalanceYuan(String pubkeyHex) {
    return _rpc.fetchFinalizedBalance(pubkeyHex, forceFresh: true);
  }
}

class SquarePublishService {
  SquarePublishService({
    SquareContentUploader? uploadService,
    SquarePostChainPublisher? chainService,
    SquarePublicationConfirmer? publicationConfirmer,
    SquarePostDeletionService? postDeletionService,
    SquarePublishBalanceReader? balanceReader,
    SquareDraftRepository? draftStore,
  })  : _uploadService = uploadService ?? SquareUploadService(),
        _chainService = chainService ?? SquareChainService(),
        _publicationConfirmer = publicationConfirmer ?? SquareApiClient(),
        _postDeletionService = postDeletionService ?? SquareApiClient(),
        _balanceReader = balanceReader ?? SquareChainBalanceReader(),
        _draftStore = draftStore ?? SquareDraftStore.instance;

  final SquareContentUploader _uploadService;
  final SquarePostChainPublisher _chainService;
  final SquarePublicationConfirmer _publicationConfirmer;
  final SquarePostDeletionService _postDeletionService;
  final SquarePublishBalanceReader _balanceReader;
  final SquareDraftRepository _draftStore;

  /// 广场发布统一按链上最低费用收费：10 分 = 0.1 元。
  static const int publishFeeFen = 10;
  static const int accountExistentialDepositFen = 111;
  static const int minimumPublishBalanceFen =
      publishFeeFen + accountExistentialDepositFen;

  Future<SquarePublishResult> publish({
    required SquareIdentityState identity,
    required SquarePostCategory postCategory,
    required String text,
    required List<SquareLocalMediaDraft> mediaDrafts,
    required SquareLoginSigner signLoginPayload,
    required SquareChainSigner signChainPayload,
    SquarePostContentFormat contentFormat = SquarePostContentFormat.normal,
    String? title,
    List<Map<String, Object?>>? contentBlocks,
    String? replacePostId,
    void Function(SquarePublishStage stage)? onStage,
    TxPoolWatchCallback? onWatchEvent,
  }) async {
    final trimmedText = text.trim();
    if (!identity.hasWallet || identity.pubkeyHex == null) {
      throw const SquarePublishException('请先创建或选择钱包');
    }
    if (postCategory == SquarePostCategory.campaign && !identity.isCertified) {
      throw const SquarePublishException('只有链上认证公民才能发布竞选内容');
    }
    if (trimmedText.isEmpty && mediaDrafts.isEmpty) {
      throw const SquarePublishException('动态内容不能为空');
    }
    if (mediaDrafts.isEmpty) {
      throw const SquarePublishException('请至少选择一张图片或一个视频');
    }

    SquarePreparedContent? prepared;
    SquareChainPublishedResult? chainResult;
    try {
      onStage?.call(SquarePublishStage.checkingBalance);
      await _ensurePublishBalance(identity.pubkeyHex!);

      prepared = await _uploadService.preparePostContent(
        ownerAccount: identity.ownerAccount,
        postCategory: postCategory,
        text: trimmedText,
        mediaDrafts: mediaDrafts,
        signLoginPayload: signLoginPayload,
        contentFormat: contentFormat,
        title: title,
        contentBlocks: contentBlocks,
        onStage: onStage,
      );

      onStage?.call(SquarePublishStage.submittingChain);
      chainResult = await _chainService.publishPost(
        fromAddress: identity.ownerAccount,
        signerPubkey: SquareChainService.hexDecode(identity.pubkeyHex!),
        postId: prepared.postId,
        postCategory: postCategory,
        contentHashHex: prepared.contentHash,
        storageReceiptId: prepared.storageReceiptId,
        storageUntil: prepared.storageUntil,
        sign: signChainPayload,
        onWatchEvent: (event) {
          if (event.isIncluded) {
            onStage?.call(SquarePublishStage.waitingInBlock);
          }
          onWatchEvent?.call(event);
        },
      );

      final uploaded = await _uploadService.uploadPreparedContent(
        prepared,
        onStage: onStage,
      );

      onStage?.call(SquarePublishStage.confirmingPost);
      final confirmedPost = await _publicationConfirmer.confirmPublishedPost(
        session: uploaded.session,
        postId: uploaded.postId,
        blockHashHex: chainResult.blockHashHex,
        txHash: chainResult.txHash,
      );

      await _deleteDraftAfterSuccess(identity.ownerAccount);
      final cleanupWarning = await _deleteReplacedPostAfterSuccess(
        session: uploaded.session,
        newPostId: uploaded.postId,
        replacePostId: replacePostId,
      );
      onStage?.call(SquarePublishStage.completed);
      return SquarePublishResult(
        post: confirmedPost,
        txHash: chainResult.txHash,
        blockHashHex: chainResult.blockHashHex,
        storageUntil: uploaded.storageUntil,
        cleanupWarning: cleanupWarning,
      );
    } catch (e) {
      await _saveDraftAfterFailure(
        identity: identity,
        postCategory: postCategory,
        text: trimmedText,
        mediaDrafts: mediaDrafts,
        prepared: prepared,
        chainResult: chainResult,
        error: e,
      );
      throw SquarePublishException('${_messageOf(e)}，已保存到草稿箱');
    }
  }

  Future<String?> _deleteReplacedPostAfterSuccess({
    required SquareSession session,
    required String newPostId,
    required String? replacePostId,
  }) async {
    final oldPostId = replacePostId?.trim();
    if (oldPostId == null || oldPostId.isEmpty || oldPostId == newPostId) {
      return null;
    }
    try {
      // 修改视为重新发布：新帖成功后再清旧帖，避免发布失败导致原内容丢失。
      await _postDeletionService.deletePost(
          session: session, postId: oldPostId);
      return null;
    } catch (error) {
      final message = '新内容已发布，但旧内容清理失败：${_messageOf(error)}';
      debugPrint('[SquarePublishService] $message');
      return message;
    }
  }

  Future<void> _ensurePublishBalance(String pubkeyHex) async {
    final balance = await _balanceReader.fetchFreshFinalizedBalanceYuan(
      pubkeyHex,
    );
    final balanceFen = (balance * 100).round();
    if (balanceFen < minimumPublishBalanceFen) {
      throw SquarePublishException(
        '钱包余额不足，发布动态需至少 ${_formatFen(minimumPublishBalanceFen)} 元'
        '（账户保留 ${_formatFen(accountExistentialDepositFen)} 元 + 发布费 '
        '${_formatFen(publishFeeFen)} 元）',
      );
    }
  }

  Future<void> _saveDraftAfterFailure({
    required SquareIdentityState identity,
    required SquarePostCategory postCategory,
    required String text,
    required List<SquareLocalMediaDraft> mediaDrafts,
    required SquarePreparedContent? prepared,
    required SquareChainPublishedResult? chainResult,
    required Object error,
  }) async {
    try {
      await _draftStore.save(
        SquarePublishDraft(
          ownerAccount: identity.ownerAccount,
          postCategory: postCategory,
          text: text,
          mediaDrafts: mediaDrafts,
          draftState: chainResult == null
              ? SquareDraftState.localOnly
              : SquareDraftState.chainInBlockUploadPending,
          updatedAtMillis: DateTime.now().millisecondsSinceEpoch,
          lastError: _messageOf(error),
          uploadId: prepared?.preparedUpload.uploadId,
          postId: prepared?.postId,
          contentHash: prepared?.contentHash,
          storageReceiptId: prepared?.storageReceiptId,
          storageUntil: prepared?.storageUntil,
          txHash: chainResult?.txHash,
          blockHashHex: chainResult?.blockHashHex,
        ),
      );
    } catch (draftError) {
      debugPrint('[SquarePublishService] 保存发布草稿失败: $draftError');
    }
  }

  Future<void> _deleteDraftAfterSuccess(String ownerAccount) async {
    try {
      await _draftStore.delete(ownerAccount);
    } catch (draftError) {
      debugPrint('[SquarePublishService] 清理已发布草稿失败: $draftError');
    }
  }

  static String _formatFen(int fen) => (fen / 100).toStringAsFixed(2);

  static String _messageOf(Object error) {
    if (error is SquarePublishException) return error.message;
    if (error is SquareApiException) return error.message;
    return error.toString();
  }
}
