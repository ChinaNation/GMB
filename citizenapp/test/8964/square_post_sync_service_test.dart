import 'dart:async';
import 'dart:convert';
import 'dart:typed_data';

import 'package:crypto/crypto.dart';
import 'package:flutter_test/flutter_test.dart';

import 'package:citizenapp/8964/services/square_api_client.dart';
import 'package:citizenapp/8964/services/square_post_store.dart';
import 'package:citizenapp/8964/services/square_post_sync_service.dart';

import '../support/isar_test_env.dart';

const _cid = 'R5-K3P1C1-N9-D4';
const _account =
    '0xaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa';
const _session = SquareSession(
  sessionToken: 'sqs_sync',
  cidNumber: _cid,
  accountId: _account,
  expiresAt: 1900000000000,
);

SquareLocalPost _post(String postId, int createdAt, {String cid = _cid}) {
  final bytes = Uint8List.fromList(
    utf8.encode(
      jsonEncode({
        'schema': SquarePostStore.manifestSchema,
        'account_id': _account,
        'post_category': 'normal',
        'text': '正文 $postId',
        'media_items': const <Object>[],
      }),
    ),
  );
  return SquareLocalPost(
    postId: postId,
    cidNumber: cid,
    accountId: _account,
    postCategory: 'normal',
    contentFormat: 'normal',
    manifestBytes: bytes,
    contentHash: sha256.convert(bytes).toString(),
    storageReceiptId: 'sqr_$postId',
    chainBlock: createdAt,
    createdAt: createdAt,
    postState: SquarePostStore.publishedState,
  );
}

void main() {
  useIsolatedIsar();
  TestWidgetsFlutterBinding.ensureInitialized();

  test('首次回灌完整分页，后续只扫描到旧检查点且保留本地历史', () async {
    final firstCursors = <String?>[];
    final firstSync = SquarePostSyncService(
      pageLoader: ({
        required session,
        cursor,
        required limit,
      }) async {
        expect(session.cidNumber, _cid);
        expect(limit, 5);
        firstCursors.add(cursor);
        if (cursor == null) {
          return SquareLocalPostPage(
            items: [_post('sqp_3', 3000), _post('sqp_2', 2000)],
            nextCursor: 'cursor_2',
          );
        }
        expect(cursor, 'cursor_2');
        return SquareLocalPostPage(
          items: [_post('sqp_1', 1000)],
          nextCursor: null,
        );
      },
    );

    await firstSync.sync(_session);
    expect(firstCursors, [null, 'cursor_2']);
    expect(
      (await const SquarePostStore().listByCid(_cid))
          .map((post) => post.postId),
      ['sqp_3', 'sqp_2', 'sqp_1'],
    );
    final firstCheckpoint =
        await const SquarePostStore().readSyncCheckpoint(_cid);
    expect(firstCheckpoint?.newestPostId, 'sqp_3');
    expect(firstCheckpoint?.newestCreatedAt, 3000);

    var secondPageCalls = 0;
    final incrementalSync = SquarePostSyncService(
      pageLoader: ({
        required session,
        cursor,
        required limit,
      }) async {
        secondPageCalls += 1;
        expect(cursor, isNull);
        return SquareLocalPostPage(
          items: [_post('sqp_4', 4000), _post('sqp_3', 3000)],
          nextCursor: 'must_not_be_requested',
        );
      },
    );
    await incrementalSync.sync(_session);

    expect(secondPageCalls, 1);
    expect(
      (await const SquarePostStore().listByCid(_cid))
          .map((post) => post.postId),
      ['sqp_4', 'sqp_3', 'sqp_2', 'sqp_1'],
    );
    final nextCheckpoint =
        await const SquarePostStore().readSyncCheckpoint(_cid);
    expect(nextCheckpoint?.newestPostId, 'sqp_4');
    expect(nextCheckpoint?.newestCreatedAt, 4000);
  });

  test('一页任一条归属错误时整页不落盘且不推进检查点', () async {
    final service = SquarePostSyncService(
      pageLoader: ({
        required session,
        cursor,
        required limit,
      }) async =>
          SquareLocalPostPage(
        items: [
          _post('sqp_good', 2000),
          _post('sqp_wrong_owner', 1000, cid: 'R5-K3P1C1-N8-D5'),
        ],
        nextCursor: null,
      ),
    );

    await expectLater(
      service.sync(_session),
      throwsA(isA<SquarePostStoreException>()),
    );
    expect(await const SquarePostStore().listByCid(_cid), isEmpty);
    expect(
      await const SquarePostStore().readSyncCheckpoint(_cid),
      isNull,
    );
  });

  test('远端为空只更新空检查点，不删除会员到期前已落盘的本地副本', () async {
    const store = SquarePostStore();
    await store.save(_post('sqp_local', 1000));
    final service = SquarePostSyncService(
      pageLoader: ({
        required session,
        cursor,
        required limit,
      }) async =>
          const SquareLocalPostPage(items: [], nextCursor: null),
    );

    await service.sync(_session);

    expect((await store.listByCid(_cid)).single.postId, 'sqp_local');
    final checkpoint = await store.readSyncCheckpoint(_cid);
    expect(checkpoint?.newestPostId, isNull);
    expect(checkpoint?.newestCreatedAt, 0);
  });

  test('注销清理同时删除本人副本和同步检查点', () async {
    const store = SquarePostStore();
    await store.save(_post('sqp_local', 1000));
    await store.writeSyncCheckpoint(
      cidNumber: _cid,
      checkpoint: const SquarePostSyncCheckpoint(
        newestPostId: 'sqp_local',
        newestCreatedAt: 1000,
      ),
    );

    expect(await store.deleteAllByCid(_cid), 1);
    expect(await store.listByCid(_cid), isEmpty);
    expect(await store.readSyncCheckpoint(_cid), isNull);
  });

  test('同一 CID 并发启动复用同一个回灌任务', () async {
    final pageCompleter = Completer<SquareLocalPostPage>();
    var calls = 0;
    final service = SquarePostSyncService(
      pageLoader: ({
        required session,
        cursor,
        required limit,
      }) {
        calls += 1;
        return pageCompleter.future;
      },
    );

    final first = service.sync(_session);
    final second = service.sync(_session);
    expect(identical(first, second), isTrue);
    pageCompleter.complete(
      const SquareLocalPostPage(items: [], nextCursor: null),
    );
    await Future.wait([first, second]);

    expect(calls, 1);
  });
}
