import 'dart:typed_data';

import 'package:flutter_test/flutter_test.dart';

import 'package:citizenapp/8964/chain/square_chain_service.dart';
import 'package:citizenapp/8964/models/square_models.dart';
import 'package:citizenapp/8964/services/square_api_client.dart';
import 'package:citizenapp/8964/services/square_identity_state.dart';
import 'package:citizenapp/8964/services/square_publish_service.dart';
import 'package:citizenapp/8964/services/square_upload_service.dart';
import 'package:citizenapp/8964/storage/square_draft_store.dart';
import 'package:citizenapp/rpc/chain_rpc.dart';

void main() {
  test('广场发布费为最低链上费用且余额需保留存在性保证金', () {
    expect(SquarePublishService.publishFeeFen, 10);
    expect(SquarePublishService.accountExistentialDepositFen, 111);
    expect(SquarePublishService.minimumPublishBalanceFen, 121);
  });

  test('未认证钱包不能发布竞选动态，且不会进入存储准备', () async {
    final order = <String>[];
    final upload = _FakeUploader(order);
    final chain = _FakeChainPublisher(order);
    final service = SquarePublishService(
      uploadService: upload,
      chainService: chain,
      publicationConfirmer: _FakePublicationConfirmer(),
      balanceReader: _FakeBalanceReader(order),
      draftStore: _FakeDraftStore(),
    );

    await expectLater(
      service.publish(
        identity: _identity(cidNumber: null),
        postCategory: SquarePostCategory.campaign,
        text: '竞选说明',
        mediaDrafts: [_media()],
        signLoginPayload: (_) async => '0x11',
        signChainPayload: (_) async => Uint8List(64),
      ),
      throwsA(isA<SquarePublishException>()),
    );
    expect(upload.called, isFalse);
    expect(chain.called, isFalse);
    expect(order, isEmpty);
  });

  test('普通动态按余额校验、链上扣费入块、媒体上传、feed 确认顺序发布', () async {
    final order = <String>[];
    final upload = _FakeUploader(order);
    final chain = _FakeChainPublisher(order);
    final draftStore = _FakeDraftStore();
    final stages = <SquarePublishStage>[];
    final service = SquarePublishService(
      uploadService: upload,
      chainService: chain,
      publicationConfirmer: _FakePublicationConfirmer(order),
      balanceReader: _FakeBalanceReader(order),
      draftStore: draftStore,
    );

    final result = await service.publish(
      identity: _identity(cidNumber: 'CN001-CTZN-000000001-2026'),
      postCategory: SquarePostCategory.normal,
      text: '普通动态',
      mediaDrafts: [_media()],
      signLoginPayload: (_) async => '0x11',
      signChainPayload: (_) async => Uint8List(64),
      onStage: stages.add,
    );

    expect(upload.called, isTrue);
    expect(chain.called, isTrue);
    expect(chain.postId, 'sqp_test');
    expect(chain.storageReceiptId, 'sqr_test');
    expect(result.post.contentHash, '11' * 32);
    expect(order, ['balance', 'prepare', 'chain', 'upload', 'confirm']);
    expect(draftStore.deletedOwnerAccount, 'gmb_test_owner_account');
    expect(
        stages,
        containsAllInOrder([
          SquarePublishStage.checkingBalance,
          SquarePublishStage.preparingStorage,
          SquarePublishStage.submittingChain,
          SquarePublishStage.waitingInBlock,
          SquarePublishStage.uploadingMedia,
          SquarePublishStage.completingStorage,
          SquarePublishStage.confirmingPost,
          SquarePublishStage.completed,
        ]));
  });

  test('修改动态时新发布确认成功后再删除旧动态', () async {
    final order = <String>[];
    final oldPostDeleter = _FakePostDeletionService(order);
    final service = SquarePublishService(
      uploadService: _FakeUploader(order),
      chainService: _FakeChainPublisher(order),
      publicationConfirmer: _FakePublicationConfirmer(order),
      postDeletionService: oldPostDeleter,
      balanceReader: _FakeBalanceReader(order),
      draftStore: _FakeDraftStore(),
    );

    final result = await service.publish(
      identity: _identity(cidNumber: 'CN001-CTZN-000000001-2026'),
      postCategory: SquarePostCategory.normal,
      text: '修改后的动态',
      mediaDrafts: [_media()],
      signLoginPayload: (_) async => '0x11',
      signChainPayload: (_) async => Uint8List(64),
      replacePostId: 'sqp_old',
    );

    expect(result.cleanupWarning, isNull);
    expect(oldPostDeleter.deletedPostId, 'sqp_old');
    expect(order,
        ['balance', 'prepare', 'chain', 'upload', 'confirm', 'delete_old']);
  });

  test('余额不足时不准备媒体、不提交链上，并保存本地草稿', () async {
    final order = <String>[];
    final upload = _FakeUploader(order);
    final chain = _FakeChainPublisher(order);
    final draftStore = _FakeDraftStore();
    final service = SquarePublishService(
      uploadService: upload,
      chainService: chain,
      publicationConfirmer: _FakePublicationConfirmer(order),
      balanceReader: _FakeBalanceReader(order, balanceYuan: 1.20),
      draftStore: draftStore,
    );

    await expectLater(
      service.publish(
        identity: _identity(cidNumber: 'CN001-CTZN-000000001-2026'),
        postCategory: SquarePostCategory.normal,
        text: '余额不足的动态',
        mediaDrafts: [_media()],
        signLoginPayload: (_) async => '0x11',
        signChainPayload: (_) async => Uint8List(64),
      ),
      throwsA(isA<SquarePublishException>()),
    );

    expect(order, ['balance']);
    expect(upload.called, isFalse);
    expect(chain.called, isFalse);
    expect(draftStore.savedDraft?.draftState, SquareDraftState.localOnly);
    expect(draftStore.savedDraft?.text, '余额不足的动态');
  });

  test('链上扣费未入块时不上传媒体，并保存可再次发布的草稿', () async {
    final order = <String>[];
    final upload = _FakeUploader(order);
    final chain = _FakeChainPublisher(order)..throwOnPublish = true;
    final draftStore = _FakeDraftStore();
    final service = SquarePublishService(
      uploadService: upload,
      chainService: chain,
      publicationConfirmer: _FakePublicationConfirmer(order),
      balanceReader: _FakeBalanceReader(order),
      draftStore: draftStore,
    );

    await expectLater(
      service.publish(
        identity: _identity(cidNumber: 'CN001-CTZN-000000001-2026'),
        postCategory: SquarePostCategory.normal,
        text: '链上未入块的动态',
        mediaDrafts: [_media()],
        signLoginPayload: (_) async => '0x11',
        signChainPayload: (_) async => Uint8List(64),
      ),
      throwsA(isA<SquarePublishException>()),
    );

    expect(order, ['balance', 'prepare', 'chain']);
    expect(upload.uploadCalled, isFalse);
    expect(draftStore.savedDraft?.draftState, SquareDraftState.localOnly);
    expect(draftStore.savedDraft?.postId, 'sqp_test');
    expect(draftStore.savedDraft?.storageReceiptId, 'sqr_test');
  });
}

SquareIdentityState _identity({required String? cidNumber}) {
  return SquareIdentityState(
    ownerAccount: 'gmb_test_owner_account',
    walletName: '测试钱包',
    cidNumber: cidNumber,
    walletIndex: 1,
    pubkeyHex: 'aa' * 32,
    isHotWallet: true,
  );
}

SquareLocalMediaDraft _media() {
  return const SquareLocalMediaDraft(
    mediaKind: SquareMediaKind.image,
    path: '/tmp/square-test.jpg',
    fileName: 'square-test.jpg',
    contentType: 'image/jpeg',
    byteSize: 1024,
  );
}

class _FakeUploader implements SquareContentUploader {
  _FakeUploader(this.order);

  final List<String> order;
  bool called = false;
  bool uploadCalled = false;

  @override
  Future<SquarePreparedContent> preparePostContent({
    required String ownerAccount,
    required SquarePostCategory postCategory,
    required String text,
    required List<SquareLocalMediaDraft> mediaDrafts,
    required SquareLoginSigner signLoginPayload,
    SquarePostContentFormat contentFormat = SquarePostContentFormat.normal,
    String? title,
    void Function(SquarePublishStage stage)? onStage,
  }) async {
    called = true;
    order.add('prepare');
    onStage?.call(SquarePublishStage.preparingStorage);
    return SquarePreparedContent(
      session: const SquareSession(
        sessionToken: 'sqs_test',
        ownerAccount: 'gmb_test_owner_account',
        expiresAt: 1800000000000,
      ),
      preparedUpload: const SquarePreparedUpload(
        uploadId: 'squ_test',
        postId: 'sqp_test',
        storageReceiptId: 'sqr_test',
        expiresAt: 1800000000000,
        estimatedBytes: 1024,
        manifestObjectKey: 'square/test/manifest.json',
        manifestUploadUrl: 'http://127.0.0.1/manifest',
        mediaItems: [
          SquarePreparedMediaUpload(
            mediaKind: SquareMediaKind.image,
            contentType: 'image/jpeg',
            byteSize: 1024,
            provider: 'cloudflare_images',
            providerAssetId: 'img_test',
            uploadMethod: 'worker',
            uploadUrl: 'http://127.0.0.1/media',
          ),
        ],
      ),
      postId: 'sqp_test',
      contentHash: '11' * 32,
      storageReceiptId: 'sqr_test',
      storageUntil: 1800000000000,
      manifestHash: '11' * 32,
      manifestBytes: Uint8List.fromList([1, 2, 3]),
      mediaDrafts: mediaDrafts,
    );
  }

  @override
  Future<SquareUploadedContent> uploadPreparedContent(
    SquarePreparedContent prepared, {
    void Function(SquarePublishStage stage)? onStage,
  }) async {
    uploadCalled = true;
    order.add('upload');
    onStage?.call(SquarePublishStage.uploadingMedia);
    onStage?.call(SquarePublishStage.completingStorage);
    return SquareUploadedContent(
      session: prepared.session,
      postId: prepared.postId,
      contentHash: prepared.contentHash,
      storageReceiptId: prepared.storageReceiptId,
      storageUntil: prepared.storageUntil,
      manifestHash: prepared.manifestHash,
    );
  }
}

class _FakePublicationConfirmer implements SquarePublicationConfirmer {
  _FakePublicationConfirmer([this.order]);

  final List<String>? order;

  @override
  Future<SquarePost> confirmPublishedPost({
    required SquareSession session,
    required String postId,
    required String blockHashHex,
    required String txHash,
  }) async {
    order?.add('confirm');
    return SquarePost(
      postId: postId,
      author: SquareAuthor(ownerAccount: session.ownerAccount),
      postCategory: SquarePostCategory.normal,
      text: '普通动态',
      createdAt: DateTime.fromMillisecondsSinceEpoch(1800000000000),
      contentHash: '11' * 32,
      storageReceiptId: 'sqr_test',
      chainBlock: 88,
    );
  }
}

class _FakePostDeletionService implements SquarePostDeletionService {
  _FakePostDeletionService(this.order);

  final List<String> order;
  String? deletedPostId;

  @override
  Future<void> deletePost({
    required SquareSession session,
    required String postId,
  }) async {
    order.add('delete_old');
    deletedPostId = postId;
  }
}

class _FakeChainPublisher implements SquarePostChainPublisher {
  _FakeChainPublisher(this.order);

  final List<String> order;
  bool called = false;
  String? postId;
  String? storageReceiptId;
  bool throwOnPublish = false;

  @override
  Future<SquareChainPublishedResult> publishPost({
    required String fromAddress,
    required Uint8List signerPubkey,
    required String postId,
    required SquarePostCategory postCategory,
    required String contentHashHex,
    required String storageReceiptId,
    required int storageUntil,
    required Future<Uint8List> Function(Uint8List payload) sign,
    TxPoolWatchCallback? onWatchEvent,
  }) async {
    called = true;
    order.add('chain');
    this.postId = postId;
    this.storageReceiptId = storageReceiptId;
    if (throwOnPublish) {
      throw StateError('交易未入块');
    }
    onWatchEvent?.call(const TxPoolWatchEvent(
      kind: TxPoolWatchKind.inBlock,
      description: 'inBlock',
      raw: 'inBlock',
      blockHashHex: '0xblock',
    ));
    return const SquareChainPublishedResult(
      txHash: '0xtest',
      usedNonce: 1,
      blockHashHex: '0xblock',
    );
  }
}

class _FakeBalanceReader implements SquarePublishBalanceReader {
  _FakeBalanceReader(this.order, {this.balanceYuan = 1.21});

  final List<String> order;
  final double balanceYuan;

  @override
  Future<double> fetchFreshFinalizedBalanceYuan(String pubkeyHex) async {
    order.add('balance');
    return balanceYuan;
  }
}

class _FakeDraftStore implements SquareDraftRepository {
  SquarePublishDraft? savedDraft;
  String? deletedOwnerAccount;

  @override
  Future<void> delete(String ownerAccount) async {
    deletedOwnerAccount = ownerAccount;
    savedDraft = null;
  }

  @override
  Future<SquarePublishDraft?> read(String ownerAccount) async {
    return savedDraft;
  }

  @override
  Future<void> save(SquarePublishDraft draft) async {
    savedDraft = draft;
  }
}
