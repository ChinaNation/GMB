import 'dart:async';

import 'package:citizenapp/8964/services/square_api_client.dart';
import 'package:citizenapp/8964/services/square_post_store.dart';

typedef SquareSelfPostPageLoader = Future<SquareLocalPostPage> Function({
  required SquareSession session,
  String? cursor,
  required int limit,
});

/// 本人已发布广场内容的增量回灌协调器。
///
/// 同步只会补写 Worker 仍保留的发布事实，绝不会因为会员到期或远端删除而反向删除
/// 本地副本。每页原子落盘，全部目标页成功后才推进检查点；中途失败时下次从远端
/// 最新位置重新扫描，已落盘页依靠 post_id 不可变事实幂等重放。
class SquarePostSyncService {
  SquarePostSyncService({
    SquareSelfPostPageLoader? pageLoader,
    SquarePostStore? store,
  })  : _pageLoader =
            pageLoader ?? SquareApiClient().fetchSelfPublishedPostCopies,
        _store = store ?? const SquarePostStore();

  static const int pageSize = 5;

  final SquareSelfPostPageLoader _pageLoader;
  final SquarePostStore _store;
  final Map<String, Future<void>> _inflightByCid = <String, Future<void>>{};

  /// 同一 CID 同时只运行一个回灌任务；多个启动/刷新入口复用同一 Future。
  Future<void> sync(SquareSession session) {
    final running = _inflightByCid[session.cidNumber];
    if (running != null) return running;

    late final Future<void> task;
    task = () async {
      try {
        await _sync(session);
      } finally {
        if (identical(_inflightByCid[session.cidNumber], task)) {
          _inflightByCid.remove(session.cidNumber);
        }
      }
    }();
    _inflightByCid[session.cidNumber] = task;
    return task;
  }

  Future<void> _sync(SquareSession session) async {
    final checkpoint = await _store.readSyncCheckpoint(session.cidNumber);
    final seenCursors = <String>{};
    String? cursor;
    SquareLocalPost? remoteNewest;
    SquareLocalPost? previousItem;

    while (true) {
      final page = await _pageLoader(
        session: session,
        cursor: cursor,
        limit: pageSize,
      );
      if (page.items.length > pageSize) {
        throw const SquarePostStoreException('本人副本回灌页超过客户端上限');
      }
      if (page.items.isEmpty && page.nextCursor != null) {
        throw const SquarePostStoreException('空回灌页不得携带下一页游标');
      }

      for (final item in page.items) {
        if (item.cidNumber != session.cidNumber) {
          throw const SquarePostStoreException('回灌内容不属于当前会话 CID');
        }
        final previous = previousItem;
        if (previous != null &&
            (item.createdAt > previous.createdAt ||
                (item.createdAt == previous.createdAt &&
                    item.postId.compareTo(previous.postId) >= 0))) {
          throw const SquarePostStoreException('本人副本回灌顺序不稳定');
        }
        previousItem = item;
        remoteNewest ??= item;
      }

      // 一页内先完整校验再进入同一 Isar 事务；失败不写半页。
      await _store.saveAll(page.items);

      final reachedCheckpoint = checkpoint?.newestPostId != null &&
          page.items.any(
            (item) =>
                item.postId == checkpoint!.newestPostId &&
                item.createdAt == checkpoint.newestCreatedAt,
          );
      if (reachedCheckpoint || page.nextCursor == null) {
        break;
      }
      final nextCursor = page.nextCursor!;
      if (!seenCursors.add(nextCursor)) {
        throw const SquarePostStoreException('本人副本回灌游标发生循环');
      }
      cursor = nextCursor;
    }

    // 检查点只记录 Worker 最新发布事实，不记录本次设备同步时间。
    await _store.writeSyncCheckpoint(
      cidNumber: session.cidNumber,
      checkpoint: SquarePostSyncCheckpoint(
        newestPostId: remoteNewest?.postId,
        newestCreatedAt: remoteNewest?.createdAt ?? 0,
      ),
    );
  }
}
