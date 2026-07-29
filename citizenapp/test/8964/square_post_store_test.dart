import 'dart:convert';
import 'dart:typed_data';

import 'package:crypto/crypto.dart';
import 'package:flutter_test/flutter_test.dart';

import 'package:citizenapp/8964/models/square_models.dart';
import 'package:citizenapp/8964/services/square_local_post_presenter.dart';
import 'package:citizenapp/8964/services/square_post_store.dart';
import 'package:citizenapp/isar/app_isar.dart';

import '../support/isar_test_env.dart';

const _accountA =
    '0xaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa';
const _accountB =
    '0xbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbb';
const _cidA = 'R5-K3P1C1-N9-D4';
const _cidB = 'R5-K3P1C1-N8-D5';

Uint8List _manifest({
  String accountId = _accountA,
  String postCategory = 'normal',
  String? contentFormat,
  String text = '本人发布的正文',
}) {
  return Uint8List.fromList(
    utf8.encode(
      jsonEncode({
        'schema': SquarePostStore.manifestSchema,
        'account_id': accountId,
        'post_category': postCategory,
        if (contentFormat != null) 'content_format': contentFormat,
        'text': text,
        'media_items': [
          {
            'media_kind': 'image',
            'file_name': 'photo.jpg',
            'content_type': 'image/jpeg',
            'byte_size': 1234,
            'sha256':
                'cccccccccccccccccccccccccccccccccccccccccccccccccccccccccccccccc',
          },
        ],
      }),
    ),
  );
}

SquareLocalPost _post({
  String postId = 'sqp_a',
  String cidNumber = _cidA,
  String accountId = _accountA,
  String postCategory = 'normal',
  String contentFormat = 'normal',
  Uint8List? manifestBytes,
  String? contentHash,
  String storageReceiptId = 'sr_a',
  int? chainBlock = 123,
  int createdAt = 1000,
  String postState = SquarePostStore.publishedState,
}) {
  final bytes = manifestBytes ??
      _manifest(
        accountId: accountId,
        postCategory: postCategory,
        contentFormat: contentFormat == 'article' ? 'article' : null,
      );
  return SquareLocalPost(
    postId: postId,
    cidNumber: cidNumber,
    accountId: accountId,
    postCategory: postCategory,
    contentFormat: contentFormat,
    manifestBytes: bytes,
    contentHash: contentHash ?? sha256.convert(bytes).toString(),
    storageReceiptId: storageReceiptId,
    chainBlock: chainBlock,
    createdAt: createdAt,
    postState: postState,
  );
}

void main() {
  useIsolatedIsar();
  TestWidgetsFlutterBinding.ensureInitialized();

  const store = SquarePostStore();

  test('展示转换器只解析正文与媒体声明，不伪造本地媒体 URL', () {
    const presenter = SquareLocalPostPresenter();
    final presentation = presenter.present(
      _post(createdAt: 1700000000123),
    );

    expect(presentation.post.text, '本人发布的正文');
    expect(presentation.post.mediaItems, isEmpty);
    expect(
      presentation.unavailableMediaKinds,
      {SquareMediaKind.image},
    );
    expect(
      presentation.post.createdAt.millisecondsSinceEpoch,
      1700000000123,
    );
  });

  test('原始 manifest 字节逐字节持久化，Worker 时间和链锚不被改写', () async {
    final bytes = _manifest(text: '含中文与 emoji 🧭');
    await store.save(
      _post(
        manifestBytes: bytes,
        createdAt: 1700000000123,
        chainBlock: 456,
      ),
    );

    final saved = await store.read(cidNumber: _cidA, postId: 'sqp_a');
    expect(saved, isNotNull);
    expect(saved!.manifestBytes, orderedEquals(bytes));
    expect(saved.contentHash, sha256.convert(bytes).toString());
    expect(saved.createdAt, 1700000000123);
    expect(saved.chainBlock, 456);
    expect(saved.postState, SquarePostStore.publishedState);
  });

  test('同一发布事实可幂等重放，post_id 不得被另一内容或 CID 覆盖', () async {
    final original = _post(createdAt: 1000);
    await store.save(original);
    await store.save(original);
    final conflictingBytes = _manifest(text: '试图覆盖不可变正文');
    expect(
      () => store.save(
        _post(
          manifestBytes: conflictingBytes,
          contentHash: sha256.convert(conflictingBytes).toString(),
          createdAt: 2000,
        ),
      ),
      throwsA(isA<SquarePostStoreException>()),
    );
    final otherCidBytes = _manifest(accountId: _accountB, text: '越权覆盖');
    expect(
      () => store.save(
        _post(
          cidNumber: _cidB,
          accountId: _accountB,
          manifestBytes: otherCidBytes,
        ),
      ),
      throwsA(isA<SquarePostStoreException>()),
    );
    await store.save(
      _post(
        postId: 'sqp_b',
        cidNumber: _cidB,
        accountId: _accountB,
        manifestBytes: _manifest(accountId: _accountB, text: '另一个 CID'),
        createdAt: 9999,
      ),
    );

    final own = await store.listByCid(_cidA);
    expect(own.map((post) => post.postId), ['sqp_a']);
    expect(own.single.manifestBytes, orderedEquals(original.manifestBytes));
    expect(
      await store.read(cidNumber: _cidB, postId: 'sqp_a'),
      isNull,
    );
  });

  test('列表只用 Worker created_at 排序，同毫秒按 post_id 稳定排序', () async {
    await store.save(_post(postId: 'sqp_a', createdAt: 1000));
    await store.save(_post(postId: 'sqp_c', createdAt: 3000));
    await store.save(_post(postId: 'sqp_b', createdAt: 3000));

    final posts = await store.listByCid(_cidA);
    expect(posts.map((post) => post.postId), ['sqp_c', 'sqp_b', 'sqp_a']);
  });

  test('哈希、schema、账户、分类和内容形态不一致均拒绝且不覆盖正确副本', () async {
    final original = _post();
    await store.save(original);

    final invalidCases = <SquareLocalPost>[
      _post(
        manifestBytes: _manifest(text: '被篡改'),
        contentHash: original.contentHash,
      ),
      _post(
        manifestBytes: Uint8List.fromList(
          utf8.encode(
            jsonEncode({
              'schema': 'legacy.square.post',
              'account_id': _accountA,
              'post_category': 'normal',
              'text': '旧 schema',
              'media_items': [],
            }),
          ),
        ),
      ),
      _post(manifestBytes: _manifest(accountId: _accountB)),
      _post(manifestBytes: _manifest(postCategory: 'campaign')),
      _post(
        contentFormat: 'article',
        manifestBytes: _manifest(),
      ),
    ];

    for (final invalid in invalidCases) {
      expect(
        () => store.save(
          SquareLocalPost(
            postId: invalid.postId,
            cidNumber: invalid.cidNumber,
            accountId: invalid.accountId,
            postCategory: invalid.postCategory,
            contentFormat: invalid.contentFormat,
            manifestBytes: invalid.manifestBytes,
            contentHash: invalid.contentHash == original.contentHash
                ? invalid.contentHash
                : sha256.convert(invalid.manifestBytes).toString(),
            storageReceiptId: invalid.storageReceiptId,
            chainBlock: invalid.chainBlock,
            createdAt: invalid.createdAt,
            postState: invalid.postState,
          ),
        ),
        throwsA(isA<SquarePostStoreException>()),
      );
    }

    final saved = await store.read(cidNumber: _cidA, postId: 'sqp_a');
    expect(saved!.manifestBytes, orderedEquals(original.manifestBytes));
  });

  test('非 published、非法 UTF-8 JSON 和不完整 manifest 均不得入库', () async {
    expect(
      () => store.save(_post(postState: 'draft')),
      throwsA(isA<SquarePostStoreException>()),
    );
    expect(
      () => store.save(
        _post(
          manifestBytes: Uint8List.fromList([0xff, 0xfe]),
        ),
      ),
      throwsA(isA<SquarePostStoreException>()),
    );
    final incomplete = Uint8List.fromList(
      utf8.encode(
        jsonEncode({
          'schema': SquarePostStore.manifestSchema,
          'account_id': _accountA,
          'post_category': 'normal',
          'text': '缺少 media_items',
        }),
      ),
    );
    expect(
      () => store.save(_post(manifestBytes: incomplete)),
      throwsA(isA<SquarePostStoreException>()),
    );
  });

  test('磁盘行被篡改后读取 fail-closed，不静默返回空白内容', () async {
    await store.save(_post());
    await WalletIsar.instance.writeTxn((isar) async {
      final entity = await isar.squareLocalPostEntitys.getByPostId('sqp_a');
      entity!.manifestBytes = utf8.encode('{"broken":true}');
      await isar.squareLocalPostEntitys.put(entity);
    });

    expect(
      () => store.read(cidNumber: _cidA, postId: 'sqp_a'),
      throwsA(isA<SquarePostStoreException>()),
    );
  });

  test('删除必须匹配 CID，注销清理只删除目标 CID', () async {
    await store.save(_post(postId: 'sqp_a'));
    await store.save(_post(postId: 'sqp_b', createdAt: 2000));
    final otherBytes = _manifest(accountId: _accountB, text: '另一个 CID');
    await store.save(
      _post(
        postId: 'sqp_other',
        cidNumber: _cidB,
        accountId: _accountB,
        manifestBytes: otherBytes,
        createdAt: 3000,
      ),
    );

    expect(
      await store.delete(cidNumber: _cidB, postId: 'sqp_a'),
      isFalse,
    );
    expect(
      await store.delete(cidNumber: _cidA, postId: 'sqp_a'),
      isTrue,
    );
    expect(await store.deleteAllByCid(_cidA), 1);
    expect(await store.listByCid(_cidA), isEmpty);
    expect((await store.listByCid(_cidB)).single.postId, 'sqp_other');
  });

  test('关闭重开 Isar 后副本仍存在且实体没有媒体文件或 URL 字段', () async {
    await store.save(_post());
    final first = await WalletIsar.instance.db();
    await first.close();

    final reopened = await WalletIsar.instance.db();
    final entity = await reopened.squareLocalPostEntitys.getByPostId('sqp_a');
    expect(entity, isNotNull);
    expect(entity!.manifestBytes, isNotEmpty);
    expect(
      SquareLocalPostEntitySchema.properties.keys,
      isNot(containsAll(<String>[
        'mediaPath',
        'mediaUrl',
        'coverUrl',
        'cachedAt',
      ])),
    );
  });
}
