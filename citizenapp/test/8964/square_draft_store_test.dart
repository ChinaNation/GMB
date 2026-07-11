import 'package:flutter_test/flutter_test.dart';

import 'package:citizenapp/8964/models/square_models.dart';
import 'package:citizenapp/8964/storage/square_draft_store.dart';
import 'package:citizenapp/isar/app_isar.dart';

void main() {
  TestWidgetsFlutterBinding.ensureInitialized();

  setUpAll(() async {
    await WalletIsar.instance.ensureTestCoreInitialized();
  });

  setUp(() async {
    await WalletIsar.instance.resetForTest();
  });

  test('广场发布草稿可按钱包账户持久化和删除', () async {
    final draft = SquarePublishDraft(
      ownerAccount: 'gmb_test_owner_account',
      postCategory: SquarePostCategory.campaign,
      text: '待重新发布的竞选动态',
      mediaDrafts: const [
        SquareLocalMediaDraft(
          mediaKind: SquareMediaKind.image,
          path: '/tmp/square-draft.jpg',
          fileName: 'square-draft.jpg',
          contentType: 'image/jpeg',
          byteSize: 2048,
        ),
      ],
      draftState: SquareDraftState.chainInBlockUploadPending,
      updatedAtMillis: 1800000000000,
      lastError: '媒体上传失败',
      uploadId: 'squ_test',
      postId: 'sqp_test',
      contentHash: '11' * 32,
      storageReceiptId: 'sqr_test',
      storageUntil: 1800000100000,
      txHash: '0xtest',
      blockHashHex: '0xblock',
    );

    await SquareDraftStore.instance.save(draft);

    final loaded = await SquareDraftStore.instance.read(
      'gmb_test_owner_account',
    );
    expect(loaded, isNotNull);
    expect(loaded!.postCategory, SquarePostCategory.campaign);
    expect(loaded.draftState, SquareDraftState.chainInBlockUploadPending);
    expect(loaded.mediaDrafts.single.fileName, 'square-draft.jpg');
    expect(loaded.storageReceiptId, 'sqr_test');

    await SquareDraftStore.instance.delete('gmb_test_owner_account');
    expect(
      await SquareDraftStore.instance.read('gmb_test_owner_account'),
      isNull,
    );
  });
}
