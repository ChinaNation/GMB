import 'dart:convert';
import 'dart:typed_data';

import 'package:crypto/crypto.dart';
import 'package:flutter_test/flutter_test.dart';

import 'package:citizenapp/8964/chain/square_chain_service.dart';
import 'package:citizenapp/8964/models/square_models.dart';
import 'package:citizenapp/8964/services/square_api_client.dart';
import 'package:citizenapp/8964/services/square_identity_state.dart';
import 'package:citizenapp/8964/services/square_post_deletion_coordinator.dart';
import 'package:citizenapp/8964/services/square_post_store.dart';
import 'package:citizenapp/8964/services/square_publish_service.dart';
import 'package:citizenapp/8964/services/square_upload_service.dart';
import 'package:citizenapp/rpc/chain_rpc.dart';

import '../support/isar_test_env.dart';

void main() {
  useIsolatedIsar();
  TestWidgetsFlutterBinding.ensureInitialized();

  test('广场发布余额门槛由链上动态读取且 App 不保留费用常量副本', () async {
    final reader = _FakeBalanceReader(<String>[]);
    expect(await reader.fetchMinSelfPayBalanceFen(), BigInt.from(121));
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
      localPostWriter: _FakeLocalPostWriter(),
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
    final localWriter = _FakeLocalPostWriter();
    final stages = <SquarePublishStage>[];
    final service = SquarePublishService(
      uploadService: upload,
      chainService: chain,
      publicationConfirmer: _FakePublicationConfirmer(order),
      balanceReader: _FakeBalanceReader(order),
      localPostWriter: localWriter,
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
    expect(localWriter.saved?.postId, 'sqp_test');
    expect(localWriter.saved?.cidNumber, 'CN220-CTZN2-198805200-2026');
    expect(localWriter.saved?.createdAt, 1800000000000);
    expect(order, ['balance', 'prepare', 'chain', 'upload', 'confirm']);
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
    final oldPostDeleter = _FakePostDeleteCoordinator(order);
    final service = SquarePublishService(
      uploadService: _FakeUploader(order),
      chainService: _FakeChainPublisher(order),
      publicationConfirmer: _FakePublicationConfirmer(order),
      postDeletionCoordinator: oldPostDeleter,
      balanceReader: _FakeBalanceReader(order),
      localPostWriter: _FakeLocalPostWriter(),
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

    expect(result.completionWarning, isNull);
    expect(oldPostDeleter.deletedPostId, 'sqp_old');
    expect(order,
        ['balance', 'prepare', 'chain', 'upload', 'confirm', 'delete_old']);
  });

  test('余额不足时不准备媒体、不提交链上', () async {
    final order = <String>[];
    final upload = _FakeUploader(order);
    final chain = _FakeChainPublisher(order);
    final service = SquarePublishService(
      uploadService: upload,
      chainService: chain,
      publicationConfirmer: _FakePublicationConfirmer(order),
      balanceReader: _FakeBalanceReader(order, balanceYuan: 1.20),
      localPostWriter: _FakeLocalPostWriter(),
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
  });

  test('链上扣费未入块时不上传媒体', () async {
    final order = <String>[];
    final upload = _FakeUploader(order);
    final chain = _FakeChainPublisher(order)..throwOnPublish = true;
    final service = SquarePublishService(
      uploadService: upload,
      chainService: chain,
      publicationConfirmer: _FakePublicationConfirmer(order),
      balanceReader: _FakeBalanceReader(order),
      localPostWriter: _FakeLocalPostWriter(),
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
  });

  test('远端确认后本地落盘失败仍返回发布成功并立即调度回灌', () async {
    final order = <String>[];
    final localWriter = _FakeLocalPostWriter()..throwOnSave = true;
    SquareSession? scheduledSession;
    final service = SquarePublishService(
      uploadService: _FakeUploader(order),
      chainService: _FakeChainPublisher(order),
      publicationConfirmer: _FakePublicationConfirmer(order),
      balanceReader: _FakeBalanceReader(order),
      localPostWriter: localWriter,
      recoveryScheduler: (session) => scheduledSession = session,
    );

    final result = await service.publish(
      identity: _identity(cidNumber: 'CN001-CTZN-000000001-2026'),
      postCategory: SquarePostCategory.normal,
      text: '远端已成功的动态',
      mediaDrafts: [_media()],
      signLoginPayload: (_) async => '0x11',
      signChainPayload: (_) async => Uint8List(64),
    );

    expect(result.post.postId, 'sqp_test');
    expect(result.completionWarning, contains('本地副本将在后台重新同步'));
    expect(scheduledSession?.cidNumber, 'CN220-CTZN2-198805200-2026');
  });

  test('远端确认成功后把同一份规范 manifest 原始字节写入真实 Isar', () async {
    final order = <String>[];
    final manifestBytes = Uint8List.fromList(
      utf8.encode(
        jsonEncode({
          'schema': SquarePostStore.manifestSchema,
          'account_id':
              '0xaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa',
          'post_category': 'normal',
          'text': '真实本地副本',
          'media_items': const <Object>[],
        }),
      ),
    );
    final contentHash = sha256.convert(manifestBytes).toString();
    final service = SquarePublishService(
      uploadService: _FakeUploader(order, manifestBytes: manifestBytes),
      chainService: _FakeChainPublisher(order),
      publicationConfirmer: _FakePublicationConfirmer.withHash(
        order,
        contentHash,
      ),
      balanceReader: _FakeBalanceReader(order),
    );

    await service.publish(
      identity: _identity(cidNumber: 'CN220-CTZN2-198805200-2026'),
      postCategory: SquarePostCategory.normal,
      text: '真实本地副本',
      mediaDrafts: [_media()],
      signLoginPayload: (_) async => '0x11',
      signChainPayload: (_) async => Uint8List(64),
    );

    final saved = await const SquarePostStore().read(
      cidNumber: 'CN220-CTZN2-198805200-2026',
      postId: 'sqp_test',
    );
    expect(saved?.manifestBytes, orderedEquals(manifestBytes));
    expect(saved?.contentHash, contentHash);
    expect(saved?.createdAt, 1800000000000);
  });
}

SquareIdentityState _identity({required String? cidNumber}) {
  return SquareIdentityState(
    accountId:
        '0xaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa',
    displayName: '公开昵称',
    cidNumber: cidNumber,
    walletIndex: 1,
    ss58Address: 'gmb_test_signer_ss58_address',
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
  _FakeUploader(this.order, {this.manifestBytes});

  final List<String> order;
  final Uint8List? manifestBytes;
  bool called = false;
  bool uploadCalled = false;

  @override
  Future<SquarePreparedContent> preparePostContent({
    required String accountId,
    required SquarePostCategory postCategory,
    required String text,
    required List<SquareLocalMediaDraft> mediaDrafts,
    required SquareLoginSigner signLoginPayload,
    SquarePostContentFormat contentFormat = SquarePostContentFormat.normal,
    String? title,
    List<Map<String, Object?>>? contentBlocks,
    void Function(SquarePublishStage stage)? onStage,
  }) async {
    called = true;
    order.add('prepare');
    onStage?.call(SquarePublishStage.preparingStorage);
    final bytes = manifestBytes ?? Uint8List.fromList([1, 2, 3]);
    final contentHash =
        manifestBytes == null ? '11' * 32 : sha256.convert(bytes).toString();
    return SquarePreparedContent(
      session: const SquareSession(
        sessionToken: 'sqs_test',
        cidNumber: "CN220-CTZN2-198805200-2026",
        bindingRevision: 1,
        accountId:
            '0xaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa',
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
      contentHash: contentHash,
      storageReceiptId: 'sqr_test',
      storageUntil: 1800000000000,
      manifestHash: contentHash,
      manifestBytes: bytes,
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
  _FakePublicationConfirmer([this.order]) : contentHash = '11' * 32;

  _FakePublicationConfirmer.withHash(this.order, this.contentHash);

  final List<String>? order;
  final String contentHash;

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
      author: SquareAuthor(
        accountId: session.accountId,
        cidNumber: session.cidNumber,
      ),
      postCategory: SquarePostCategory.normal,
      text: '普通动态',
      createdAt: DateTime.fromMillisecondsSinceEpoch(1800000000000),
      contentHash: contentHash,
      storageReceiptId: 'sqr_test',
      chainBlock: 88,
    );
  }
}

class _FakeLocalPostWriter implements SquareLocalPostWriter {
  SquareLocalPost? saved;
  bool throwOnSave = false;

  @override
  Future<void> save(SquareLocalPost post) async {
    if (throwOnSave) {
      throw StateError('disk unavailable');
    }
    saved = post;
  }
}

class _FakePostDeleteCoordinator implements SquarePostDeleteCoordinator {
  _FakePostDeleteCoordinator(this.order);

  final List<String> order;
  String? deletedPostId;

  @override
  Future<void> delete({
    required SquareSession session,
    required String cidNumber,
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
    required String fromSs58Address,
    required Uint8List signerPublicKey,
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
  _FakeBalanceReader(
    this.order, {
    this.balanceYuan = 1.21,
    BigInt? minimumBalanceFen,
  }) : minimumBalanceFen = minimumBalanceFen ?? BigInt.from(121);

  final List<String> order;
  final double balanceYuan;
  final BigInt minimumBalanceFen;

  @override
  Future<BigInt> fetchMinSelfPayBalanceFen() async => minimumBalanceFen;

  @override
  Future<double> fetchFreshFinalizedBalanceYuan(String publicKey) async {
    order.add('balance');
    return balanceYuan;
  }
}
