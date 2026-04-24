import 'dart:async';

import 'package:flutter/foundation.dart';
import 'package:isar/isar.dart';
import 'package:wuminapp_mobile/Isar/wallet_isar.dart';
import 'package:wuminapp_mobile/rpc/chain_rpc.dart';
import 'package:wuminapp_mobile/trade/local_tx_store.dart';

/// 单条 pending 记录对账后的处置结果。
enum ReconcileOutcome {
  /// 已确认：区块中找到 txHash 或 nonce 已推进。
  confirmed,

  /// 已丢失：超时且 nonce 被其他交易占用。
  lost,

  /// 仍需等待：下次再对账。
  stillPending,

  /// 对账出错（网络/RPC），下次再试。
  error,
}

/// 纯函数判定：给定已采集的链上信号，决定单条 pending 记录的处置。
///
/// 调用方负责 IO（区块搜索、nonce 查询），把结果塞进来即可。
/// 这样核心判定可独立做单元测试。
///
/// 参数：
/// - [foundBlockNumber]：按 txHash 在最近区块中搜索到的区块号，未找到传 null。
/// - [chainNonce]：链上 `accountNextIndex`，未查询传 null（例如该记录没 pubkey）。
/// - [usedNonce]：提交时的 nonce，未记录传 null。
/// - [age]：自 createdAt 起的经过时间。
/// - [lostThreshold]：超过此年龄且 nonce 已推进但区块里找不到时判定 lost。
ReconcileDecision decideReconcileOutcome({
  required int? foundBlockNumber,
  required int? chainNonce,
  required int? usedNonce,
  required Duration age,
  required Duration lostThreshold,
}) {
  // 路径 1：txHash 在区块里直接找到 —— 最权威。
  if (foundBlockNumber != null) {
    return ReconcileDecision(
      outcome: ReconcileOutcome.confirmed,
      confirmedAtBlock: foundBlockNumber,
    );
  }

  // 路径 2：nonce 已推进 → 保守判定为 confirmed。
  //
  // 重要：nonce 推进只能证明"某笔交易占用了这个 nonce 位置"，
  // 理论上可能是被同 nonce 的另一笔交易顶替（本笔其实 lost）。
  // 但我们无法在客户端区分这两种情况，而且："本笔其实执行成功了只是超出搜索窗口"
  // 是远远更常见的场景。把一笔真实成功的交易误标为 lost 比误标为 confirmed
  // 更容易误导用户（因为余额/nonce 都对得上）。
  // 所以：nonce 推进 → 一律 confirmed，不再引入 lost 分支。
  if (chainNonce != null && usedNonce != null && chainNonce > usedNonce) {
    return const ReconcileDecision(outcome: ReconcileOutcome.confirmed);
  }

  // 其余情况：继续等。
  return const ReconcileDecision(outcome: ReconcileOutcome.stillPending);
}

/// [decideReconcileOutcome] 的返回值。
class ReconcileDecision {
  const ReconcileDecision({
    required this.outcome,
    this.confirmedAtBlock,
  });

  final ReconcileOutcome outcome;

  /// 仅当 outcome == confirmed 且 txHash 在具体区块里被找到时设置。
  final int? confirmedAtBlock;
}

/// 全局 pending 交易对账服务。
///
/// 作用：
/// - 把"已上链但本地仍 pending"的 LocalTxEntity 强制推进到 confirmed。
/// - 不依赖页面生命周期，不依赖 fire-and-forget 轮询，任何时候调用都能兜底。
/// - 2026-04-23 整改:判定策略简化为**仅靠 nonce 推进**(走 `state_getStorage`
///   协议,不触发 substrate block-request 反滥用 ban)。原"按 txHash 在最近
///   区块搜索"路径已物理删除,详见 `_reconcileOne`。
class PendingTxReconciler {
  PendingTxReconciler({
    ChainRpc? chainRpc,
  }) : _chainRpc = chainRpc ?? ChainRpc();

  static final PendingTxReconciler instance = PendingTxReconciler();

  final ChainRpc _chainRpc;

  /// 防止重复并发调用：一次只跑一个 reconcileAll。
  Future<int>? _inflight;

  // 2026-04-23 整改:`_deepSearchDepth / _shallowSearchDepth /
  // _minDeepSearchAge` 已随 `findTxInRecentBlocks` 一并下线。

  /// 交易提交后多久仍无法在链上找到，才允许判为 lost。
  static const Duration _lostThreshold = Duration(minutes: 10);

  /// 对所有 status == 'pending' 的本地记录跑一轮对账。
  ///
  /// 返回被更新（confirmed 或 lost）的记录条数。
  Future<int> reconcileAll() {
    final existing = _inflight;
    if (existing != null) return existing;

    final future = _runReconcileAll().whenComplete(() {
      _inflight = null;
    });
    _inflight = future;
    return future;
  }

  Future<int> _runReconcileAll() async {
    final isar = await WalletIsar.instance.db();

    // 预热 walletAddress → pubkeyHex 映射，供 nonce 路径使用。
    await _preloadPubkeyCache(isar);

    final List<LocalTxEntity> pending = await isar.localTxEntitys
        .filter()
        .statusEqualTo('pending')
        .findAll();

    if (pending.isEmpty) {
      return 0;
    }

    debugPrint('[Reconciler] 开始对账，共 ${pending.length} 条 pending 记录');

    // 一次性迁移：历史数据里 blockNumber 字段可能塞的是 usedNonce，
    // 且 usedNonce 字段为空。先把它归位到 usedNonce，让后续判定逻辑统一。
    await _migrateLegacyBlockNumberIfNeeded(isar, pending);

    int updated = 0;
    for (final record in pending) {
      final outcome = await _reconcileOne(record);
      if (outcome == ReconcileOutcome.confirmed ||
          outcome == ReconcileOutcome.lost) {
        updated++;
      }
    }
    debugPrint('[Reconciler] 对账结束，更新 $updated 条');
    return updated;
  }

  /// 对单条 pending 记录执行对账。用于刚提交交易后的快速路径。
  Future<ReconcileOutcome> reconcileSingle(String txId) async {
    final isar = await WalletIsar.instance.db();
    final record =
        await isar.localTxEntitys.filter().txIdEqualTo(txId).findFirst();
    if (record == null || record.status != 'pending') {
      return ReconcileOutcome.stillPending;
    }
    if (!_pubkeyCache.containsKey(record.walletAddress)) {
      await _preloadPubkeyCache(isar);
    }
    return _reconcileOne(record, shallow: true);
  }

  Future<ReconcileOutcome> _reconcileOne(
    LocalTxEntity record, {
    // 2026-04-23 整改:`shallow` 参数保留仅为兼容调用方,内部不再区分
    // 深/浅搜索(block-body 搜索路径已整体下线)。
    bool shallow = false,
  }) async {
    final usedNonce = record.usedNonce ?? record.blockNumber;
    final ageMs =
        DateTime.now().millisecondsSinceEpoch - record.createdAtMillis;
    final age = Duration(milliseconds: ageMs);

    try {
      // 2026-04-23 整改:删除"按 txHash 在最近区块逐块拉 body 搜索"路径。
      // 原因:`findTxInRecentBlocks` 会对每个 block 发 `getBlockExtrinsics`
      // (即 smoldot `block_query(header=false, body=true)`)。substrate 端
      // `sc-network-sync::block_request_handler` 对同一 (peer+hash+BODY)
      // 请求超过 `MAX_NUMBER_OF_SAME_REQUESTS_PER_PEER=2` 次就返回 None
      // 并扣 i32::MIN 声誉分,peer 被立即 ban → 轻节点 peers 归零。
      //
      // 交易确认改为完全依赖 nonce 推进判定(`state_getStorage` 协议,
      // 与 block-request 协议无关,不受反滥用机制影响)。
      // `decideReconcileOutcome` 的路径 2 已完整覆盖该语义。
      // 代价:丢失"确认在哪块"的具体 block 号,`confirmedAtBlock` 恒为 null,
      // UI 仅显示"已确认"而不具体到块;余额/nonce 仍然正确。
      int? chainNonce;
      if (usedNonce != null) {
        final pubkeyHex = _extractPubkeyHex(record);
        if (pubkeyHex != null) {
          chainNonce = await _chainRpc.fetchConfirmedNonce(pubkeyHex);
        }
      }

      // 纯函数判定。
      final decision = decideReconcileOutcome(
        foundBlockNumber: null,
        chainNonce: chainNonce,
        usedNonce: usedNonce,
        age: age,
        lostThreshold: _lostThreshold,
      );

      switch (decision.outcome) {
        case ReconcileOutcome.confirmed:
          await _markConfirmed(record,
              realBlockNumber: decision.confirmedAtBlock);
          return ReconcileOutcome.confirmed;
        case ReconcileOutcome.lost:
          await _markLost(record);
          return ReconcileOutcome.lost;
        case ReconcileOutcome.stillPending:
        case ReconcileOutcome.error:
          return ReconcileOutcome.stillPending;
      }
    } catch (e, st) {
      debugPrint('[Reconciler] 记录 ${record.txId} 对账失败: $e\n$st');
      return ReconcileOutcome.error;
    }
  }

  Future<void> _markConfirmed(
    LocalTxEntity record, {
    int? realBlockNumber,
  }) async {
    if (realBlockNumber != null) {
      final isar = await WalletIsar.instance.db();
      await isar.writeTxn(() async {
        final fresh = await isar.localTxEntitys
            .where()
            .txIdEqualTo(record.txId)
            .findFirst();
        if (fresh == null) return;
        fresh.status = 'confirmed';
        fresh.blockNumber = realBlockNumber;
        fresh.confirmedAtMillis = DateTime.now().millisecondsSinceEpoch;
        await isar.localTxEntitys.put(fresh);
      });
    } else {
      await LocalTxStore.updateStatus(record.txId, 'confirmed');
    }
    debugPrint('[Reconciler] ${record.txId} → confirmed'
        '${realBlockNumber != null ? ' @block $realBlockNumber' : ''}');
  }

  Future<void> _markLost(LocalTxEntity record) async {
    await LocalTxStore.updateStatus(record.txId, 'lost');
    debugPrint('[Reconciler] ${record.txId} → lost');
  }

  /// 历史记录里 walletAddress 是 SS58，而 fetchConfirmedNonce 需要 pubkeyHex。
  /// 从预热缓存里取，未命中则返回 null（该条记录跳过 nonce 路径）。
  String? _extractPubkeyHex(LocalTxEntity record) {
    return _pubkeyCache[record.walletAddress];
  }

  /// walletAddress (SS58) → pubkeyHex 缓存。reconcileAll 开始前预热。
  final Map<String, String> _pubkeyCache = {};

  Future<void> _preloadPubkeyCache(Isar isar) async {
    final wallets = await isar.walletProfileEntitys.where().findAll();
    _pubkeyCache
      ..clear()
      ..addEntries(wallets.map((w) => MapEntry(w.address, w.pubkeyHex)));
  }

  /// 一次性数据迁移：旧版本 LocalTxEntity 把 usedNonce 存在 blockNumber 里。
  /// 当 usedNonce 为空且 status 为 pending 时，把 blockNumber 搬过去。
  Future<void> _migrateLegacyBlockNumberIfNeeded(
    Isar isar,
    List<LocalTxEntity> pending,
  ) async {
    final toMigrate = pending
        .where((r) =>
            r.usedNonce == null &&
            r.blockNumber != null &&
            r.status == 'pending')
        .toList();
    if (toMigrate.isEmpty) return;
    await isar.writeTxn(() async {
      for (final r in toMigrate) {
        r.usedNonce = r.blockNumber;
        r.blockNumber = null;
        await isar.localTxEntitys.put(r);
      }
    });
    debugPrint('[Reconciler] 迁移 ${toMigrate.length} 条历史 pending 记录 blockNumber → usedNonce');
  }
}

