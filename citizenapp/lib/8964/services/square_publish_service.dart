import 'dart:async';

import 'package:flutter/foundation.dart';
import 'package:citizenapp/log/app_log.dart';

import 'package:citizenapp/8964/chain/square_chain_service.dart';
import 'package:citizenapp/8964/models/square_models.dart';
import 'package:citizenapp/8964/services/square_api_client.dart';
import 'package:citizenapp/8964/services/square_identity_state.dart';
import 'package:citizenapp/8964/services/square_post_deletion_coordinator.dart';
import 'package:citizenapp/8964/services/square_post_store.dart';
import 'package:citizenapp/8964/services/square_post_sync_service.dart';
import 'package:citizenapp/8964/services/square_upload_service.dart';
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
    this.completionWarning,
  });

  final SquarePost post;
  final String txHash;
  final String blockHashHex;
  final int storageUntil;

  /// 远端发布已经完成，但本地副本或被替换内容清理仍需后台收敛时的成功告警。
  final String? completionWarning;
}

typedef SquareChainSigner = Future<Uint8List> Function(Uint8List payload);
typedef SquarePostRecoveryScheduler = void Function(SquareSession session);

abstract class SquarePublishBalanceReader {
  Future<double> fetchFreshFinalizedBalanceYuan(String accountId);

  /// 自付一笔最低链上交易所需的余额门槛(分),取自链上常量,App 侧无副本。
  Future<BigInt> fetchMinSelfPayBalanceFen();
}

class SquareChainBalanceReader implements SquarePublishBalanceReader {
  SquareChainBalanceReader({ChainRpc? chainRpc})
      : _rpc = chainRpc ?? ChainRpc();

  final ChainRpc _rpc;

  @override
  Future<double> fetchFreshFinalizedBalanceYuan(String accountId) {
    return _rpc.fetchFinalizedBalance(accountId, forceFresh: true);
  }

  @override
  Future<BigInt> fetchMinSelfPayBalanceFen() =>
      _rpc.fetchMinSelfPayBalanceFen();
}

class SquarePublishService {
  SquarePublishService({
    SquareContentUploader? uploadService,
    SquarePostChainPublisher? chainService,
    SquarePublicationConfirmer? publicationConfirmer,
    SquarePostDeleteCoordinator? postDeletionCoordinator,
    SquarePublishBalanceReader? balanceReader,
    SquareLocalPostWriter? localPostWriter,
    SquarePostRecoveryScheduler? recoveryScheduler,
  })  : _uploadService = uploadService ?? SquareUploadService(),
        _chainService = chainService ?? SquareChainService(),
        _publicationConfirmer = publicationConfirmer ?? SquareApiClient(),
        _postDeletionCoordinator =
            postDeletionCoordinator ?? SquarePostDeletionCoordinator(),
        _balanceReader = balanceReader ?? SquareChainBalanceReader(),
        _localPostWriter = localPostWriter ?? const SquarePostStore(),
        _recoveryScheduler = recoveryScheduler ?? _scheduleDefaultLocalRecovery;

  final SquareContentUploader _uploadService;
  final SquarePostChainPublisher _chainService;
  final SquarePublicationConfirmer _publicationConfirmer;
  final SquarePostDeleteCoordinator _postDeletionCoordinator;
  final SquarePublishBalanceReader _balanceReader;
  final SquareLocalPostWriter _localPostWriter;
  final SquarePostRecoveryScheduler _recoveryScheduler;

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
    if (!identity.hasWallet || identity.ss58Address == null) {
      throw const SquarePublishException('请先创建或选择钱包');
    }
    // 发帖分类权限按身份档（用户 2026-07-16）：竞选内容只有竞选身份（candidate）可发。
    if (postCategory == SquarePostCategory.campaign && !identity.isCandidate) {
      throw const SquarePublishException('只有竞选身份的公民才能发布竞选内容');
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
      await _ensurePublishBalance(identity.accountId);

      prepared = await _uploadService.preparePostContent(
        accountId: identity.accountId,
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
        fromSs58Address: identity.ss58Address!,
        signerPublicKey: SquareChainService.hexDecode(identity.accountId),
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

      final localCopyWarning = await _saveLocalCopyAfterSuccess(
        identity: identity,
        session: uploaded.session,
        prepared: prepared,
        confirmedPost: confirmedPost,
        postCategory: postCategory,
        contentFormat: contentFormat,
      );
      final cleanupWarning = await _deleteReplacedPostAfterSuccess(
        session: uploaded.session,
        newPostId: uploaded.postId,
        replacePostId: replacePostId,
      );
      final completionWarning = <String>[
        if (localCopyWarning != null) localCopyWarning,
        if (cleanupWarning != null) cleanupWarning,
      ].join('\n');
      onStage?.call(SquarePublishStage.completed);
      return SquarePublishResult(
        post: confirmedPost,
        txHash: chainResult.txHash,
        blockHashHex: chainResult.blockHashHex,
        storageUntil: uploaded.storageUntil,
        completionWarning: completionWarning.isEmpty ? null : completionWarning,
      );
    } catch (e) {
      // 失败内容由发布页的持续自动保存兜底进草稿箱；此处只规整错误消息后上抛。
      throw SquarePublishException(_messageOf(e));
    }
  }

  Future<String?> _saveLocalCopyAfterSuccess({
    required SquareIdentityState identity,
    required SquareSession session,
    required SquarePreparedContent prepared,
    required SquarePost confirmedPost,
    required SquarePostCategory postCategory,
    required SquarePostContentFormat contentFormat,
  }) async {
    try {
      if (confirmedPost.postId != prepared.postId ||
          confirmedPost.author.cidNumber != session.cidNumber ||
          confirmedPost.author.accountId != identity.accountId ||
          session.accountId != identity.accountId ||
          confirmedPost.postCategory != postCategory ||
          confirmedPost.contentFormat != contentFormat ||
          confirmedPost.contentHash != prepared.contentHash ||
          confirmedPost.storageReceiptId != prepared.storageReceiptId) {
        throw const SquarePostStoreException('发布确认字段与本地 manifest 不一致');
      }
      await _localPostWriter.save(
        SquareLocalPost(
          postId: prepared.postId,
          cidNumber: session.cidNumber,
          accountId: identity.accountId,
          postCategory: postCategory.workerValue,
          contentFormat: contentFormat.workerValue,
          manifestBytes: prepared.manifestBytes,
          contentHash: prepared.contentHash,
          storageReceiptId: prepared.storageReceiptId,
          chainBlock: confirmedPost.chainBlock,
          createdAt: confirmedPost.createdAt.millisecondsSinceEpoch,
          postState: SquarePostStore.publishedState,
        ),
      );
      return null;
    } catch (error) {
      // 链上和 Worker 已确认后绝不能把本地磁盘失败包装成“发布失败”供用户重试，
      // 否则会重复扣费、重复发帖；立即调度本人回灌，并以成功告警说明收敛状态。
      AppLog.d('[SquarePublishService] local copy pending: $error');
      _recoveryScheduler(session);
      return '内容已发布，本地副本将在后台重新同步';
    }
  }

  static void _scheduleDefaultLocalRecovery(SquareSession session) {
    unawaited(
      SquarePostSyncService().sync(session).catchError((Object error) {
        AppLog.d('[SquarePublishService] local copy recovery failed: $error');
      }),
    );
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
      await _postDeletionCoordinator.delete(
        session: session,
        cidNumber: session.cidNumber,
        postId: oldPostId,
      );
      return null;
    } catch (error) {
      final message = '新内容已发布，但旧内容清理失败：${_messageOf(error)}';
      AppLog.d('[SquarePublishService] $message');
      return message;
    }
  }

  /// 发布前余额闸。发布是自签自付的链上交易，余额不够连入池预检都过不了。
  ///
  /// 门槛 = 链上 `OnchainMinFee + ExistentialDeposit`，两个数**现取自链上 metadata**：
  /// 交易费常量的真源恒为区块链常量库（`primitives::fee_policy`，经 runtime 转发），
  /// App 侧一律不留副本。链读失败不吞，上抛由发布流程按失败处理。
  Future<void> _ensurePublishBalance(String accountId) async {
    final requiredFen = await _balanceReader.fetchMinSelfPayBalanceFen();
    final balance = await _balanceReader.fetchFreshFinalizedBalanceYuan(
      accountId,
    );
    final balanceFen = BigInt.from((balance * 100).round());
    if (balanceFen < requiredFen) {
      throw SquarePublishException(
        '钱包余额不足，发布动态需至少 ${_formatFen(requiredFen)} 元'
        '（含账户存在最低余额与链上最低交易费）',
      );
    }
  }

  static String _formatFen(BigInt fen) =>
      (fen / BigInt.from(100)).toStringAsFixed(2);

  static String _messageOf(Object error) {
    if (error is SquarePublishException) return error.message;
    if (error is SquareApiException) return error.message;
    return error.toString();
  }
}
