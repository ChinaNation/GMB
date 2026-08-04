import 'dart:async';

import 'package:citizenapp/rpc/pallet_registry.dart';
import 'dart:convert';

import 'package:flutter/foundation.dart';
import 'package:citizenapp/log/app_log.dart';
import 'package:polkadart/polkadart.dart' show Events, Hasher;
import 'package:polkadart_keyring/polkadart_keyring.dart' show Keyring;

import 'chain_event_subscription.dart';
import 'chain_read_cache.dart';
import 'chain_rpc.dart';
import 'smoldot_client.dart';
import 'package:citizenapp/isar/app_isar.dart';
import 'package:citizenapp/transaction/shared/local_tx_store.dart';
import 'package:citizenapp/citizen/shared/account_derivation.dart';

class _DecodedTransferEvent {
  const _DecodedTransferEvent({
    required this.fromAccountId,
    required this.toAccountId,
    required this.amountFen,
    this.remark,
  });

  final String fromAccountId;
  final String toAccountId;
  final String amountFen;
  final String? remark;
}

/// 链上交易监控服务（本机增量流水模式）。
///
/// (ADR-017 全端 finalized 单一口径)：citizenapp 不查询钱包导入前
/// 历史，也不让全节点替手机维护交易索引。本服务只按 finalized 游标小步
/// 同步 System.Events 写入流水——交易状态两态(已提交→已确认)，不再扫
/// best 链、不再产生"已出块"中间态；本地页面只读 Isar 缓存。
class ChainTxMonitor {
  ChainTxMonitor._();
  static final ChainTxMonitor instance = ChainTxMonitor._();

  final ChainEventSubscription _subscription = ChainEventSubscription();
  final ChainRpc _chainRpc = ChainRpc();
  StreamSubscription<ChainEvent>? _listener;
  Future<void>? _syncInflight;
  Timer? _subscriptionRetryTimer;
  Timer? _syncRetryTimer;
  Future<void>? _subscriptionConnectFuture;
  bool _running = false;
  bool _subscriptionConnected = false;

  /// 当前监控的钱包：AccountId（小写 0x + 64 位 hex） → SS58 地址。
  final Map<String, String> _ss58AddressByAccountId = {};

  /// 本块已被 txHash 认领的转出交易 "accountId#extrinsicIndex" 集合。
  /// 每块开头由 [_confirmSubmittedByTxHash] 重置；转出侧据此跳过、绝不另建第二条。
  Set<String> _claimedThisBlock = <String>{};

  /// 余额变动回调：当检测到余额变化（写入新交易记录后）通知外部刷新。
  void Function(String ss58Address, double newBalance)? onBalanceChanged;

  /// SS58 前缀。

  /// 每次补同步最多连续处理的区块数，避免手机长时间离线后一次性压节点。
  static const int _maxBlocksPerRun = 120;

  // ──── 已知事件的 pallet_index + event_index ────

  /// Balances::Transfer (pallet=2, event=2)，仅作为底层余额事件兜底。
  static const int _balancesPallet = PalletRegistry.balancesPallet;
  static const int _transferEvent = 2;
  static const int _onchainTransactionPallet = PalletRegistry.onchainTransactionPallet;
  static const int _transferWithRemarkEvent = 2;

  /// System.Events storage key（twox128("System") + twox128("Events")）。
  static final Uint8List _eventsStorageKey = _buildEventsKey();

  static Uint8List _buildEventsKey() {
    final palletHash = Hasher.twoxx128.hashString('System');
    final storageHash = Hasher.twoxx128.hashString('Events');
    final key = Uint8List(palletHash.length + storageHash.length);
    key.setAll(0, palletHash);
    key.setAll(palletHash.length, storageHash);
    return key;
  }

  // ──── 公开 API ────

  /// 添加监控钱包。
  void watchWallet(String ss58Address, String accountId) {
    final normalizedAccountId = LocalTxStore.requireAccountId(accountId);
    _ss58AddressByAccountId[normalizedAccountId] = ss58Address;
  }

  /// 移除监控钱包。
  void unwatchWallet(String ss58Address) {
    _ss58AddressByAccountId.removeWhere((_, value) => value == ss58Address);
  }

  /// 启动监控。
  Future<void> start() async {
    if (_running) {
      _ensureSubscription();
      unawaited(_syncToLatest());
      return;
    }
    _running = true;

    // 一次性迁移：先清掉旧 `:pending:` 键的本机提交孤儿（新设计只用 `:tx:` 键，
    // 已确认的 `:blockHash:` 记录保留）；幂等，详见 purgeLegacyPendingRecords。
    final purged = await LocalTxStore.purgeLegacyPendingRecords();
    if (purged > 0) {
      AppLog.d('[TxMonitor] 清理旧 :pending: 孤儿记录 $purged 条');
    }

    _listener = _subscription.events.listen(_onEvent);
    _ensureSubscription();
    AppLog.d('[TxMonitor] 交易监控已启动，监控 ${_ss58AddressByAccountId.length} 个钱包');

    // 预热进程级 metadata/registry(polkadart 最贵的 CPU 构建):挪到启动闲时
    // 一次完成,发送/确认热路径不再现场构建(此前懒建在"发送后首个最终块解码"
    // 那一刻,是发送后 ANR 的最大头)。失败静默,首次使用时会自行再取。
    unawaited(() async {
      try {
        await _chainRpc.fetchMetadata();
      } catch (e) {
        AppLog.d('[TxMonitor] metadata 预热失败,首次使用时再取: $e');
      }
    }());

    // 启动后只补 lastSyncedBlock 之后的缺口；没有游标的钱包
    // 以当前 finalized 区块为起点，不回扫导入前历史。
    unawaited(_syncToLatest());
  }

  /// 停止监控。
  void stop() {
    _running = false;
    _subscriptionConnected = false;
    _subscriptionRetryTimer?.cancel();
    _subscriptionRetryTimer = null;
    _syncRetryTimer?.cancel();
    _syncRetryTimer = null;
    _listener?.cancel();
    _listener = null;
    _subscription.disconnect();
    AppLog.d('[TxMonitor] 交易监控已停止');
  }

  /// 初始化钱包基准游标（导入钱包时可调用）。
  Future<void> initBaselineBalance(String address, String publicKey) async {
    watchWallet(address, publicKey);
    try {
      final finalized = await _chainRpc.fetchFinalizedBlock();
      await LocalTxStore.ensureCursor(
        ss58Address: address,
        accountId: publicKey,
        trackingStartBlock: finalized.blockNumber,
        lastSyncedBlock: finalized.blockNumber,
      );
      AppLog.d('[TxMonitor] 初始化交易记录游标: $address @${finalized.blockNumber}');
    } catch (e) {
      AppLog.d('[TxMonitor] 初始化交易记录游标失败，稍后从轻节点就绪块开始: $e');
    }
  }

  // ──── 同步调度 ────

  void _ensureSubscription() {
    if (!_running) return;
    if (_subscriptionConnected) return;
    if (_subscriptionConnectFuture != null) return;

    late final Future<void> task;
    task = _connectSubscriptionOnce().whenComplete(() {
      if (identical(_subscriptionConnectFuture, task)) {
        _subscriptionConnectFuture = null;
      }
    });
    _subscriptionConnectFuture = task;
  }

  Future<void> _connectSubscriptionOnce() async {
    final connected = await _subscription.connect();
    if (!_running) {
      _subscription.disconnect();
      return;
    }
    if (connected) {
      _subscriptionConnected = true;
      _subscriptionRetryTimer?.cancel();
      _subscriptionRetryTimer = null;
      unawaited(_syncToLatest());
      return;
    }

    _subscriptionRetryTimer ??= Timer(const Duration(seconds: 5), () {
      _subscriptionRetryTimer = null;
      _ensureSubscription();
    });
  }

  Future<void> _onEvent(ChainEvent event) async {
    if (!_running || _ss58AddressByAccountId.isEmpty) return;
    final blockNumber = event.blockNumber;
    if (blockNumber == null) return;
    switch (event.type) {
      case ChainEventType.newBlock:
        // (ADR-017)：best 头只是链尖竞争中的候选，不作为任何
        // 业务数据来源；流水统一等 finalized 头驱动。
        break;
      case ChainEventType.newFinalizedBlock:
        // (ADR-018 卡⑤)：新 finalized 块=链上状态已更新,立即失效
        // ChainReadCache,让换块后的余额/storage 读取拿到最新 finalized 状态。
        ChainReadCache.instance.invalidate();
        await _syncThrough(blockNumber, missingCursorStartsAt: blockNumber - 1);
        break;
    }
  }

  void _scheduleSyncRetry() {
    if (!_running || _syncRetryTimer != null) return;
    _syncRetryTimer = Timer(const Duration(seconds: 2), () {
      _syncRetryTimer = null;
      if (!_running) return;
      unawaited(_syncToLatest());
    });
  }

  Future<void> _syncToLatest() async {
    if (_ss58AddressByAccountId.isEmpty) return;
    try {
      final finalized = await _chainRpc.fetchFinalizedBlock();
      await _syncThrough(
        finalized.blockNumber,
        missingCursorStartsAt: finalized.blockNumber,
      );
    } catch (e) {
      AppLog.d('[TxMonitor] 启动补同步失败: $e');
      _scheduleSyncRetry();
    }
  }

  Future<void> _syncThrough(
    int targetBlock, {
    required int missingCursorStartsAt,
  }) {
    final existing = _syncInflight;
    if (existing != null) return existing;

    final task = _runSyncThrough(
      targetBlock,
      missingCursorStartsAt: missingCursorStartsAt,
    ).whenComplete(() {
      _syncInflight = null;
    });
    _syncInflight = task;
    return task;
  }

  Future<void> _runSyncThrough(
    int targetBlock, {
    required int missingCursorStartsAt,
  }) async {
    if (_ss58AddressByAccountId.isEmpty) return;

    // 确认先行,与前向扫描互不牵制:前向循环的让路/失败分支会提前 return,
    // 确认若挂在末尾会被跳过(轻节点验证态回炉时前向常年失败 → 确认永不执行,
    // 交易明明已最终却一直停在"待确认")。确认失败也只记日志,绝不挡前向。
    try {
      await _confirmOpenSubmits();
    } catch (e) {
      AppLog.d('[TxMonitor] 确认待确认记录失败,下轮再试: $e');
    }

    final cursors = await LocalTxStore.ensureCursorsForWallets(
      ss58AddressByAccountId: _ss58AddressByAccountId,
      startBlock: missingCursorStartsAt,
    );
    final lastByPublicKey = {
      for (final cursor in cursors) cursor.accountId: cursor.lastSyncedBlock,
    };
    final startBlock = lastByPublicKey.values
            .fold<int>(targetBlock, (min, value) => value < min ? value : min) +
        1;
    if (startBlock <= targetBlock) {
      final endBlock = startBlock + _maxBlocksPerRun - 1 < targetBlock
          ? startBlock + _maxBlocksPerRun - 1
          : targetBlock;
      for (var block = startBlock; block <= endBlock; block++) {
        if (!_running || _ss58AddressByAccountId.isEmpty) return;
        if (WalletIsar.instance.hasActiveOperation) {
          // 交易流水同步是低优先级后台任务；前台钱包/治理读写繁忙时让路，
          // 游标不推进，下一次新区块或启动补同步会继续补缺口。
          _scheduleSyncRetry();
          return;
        }

        final ok = await _processBlock(block);
        if (!ok) {
          _scheduleSyncRetry();
          return;
        }

        for (final normalizedAccountId in _ss58AddressByAccountId.keys) {
          final last =
              lastByPublicKey[normalizedAccountId] ?? missingCursorStartsAt;
          if (last < block) {
            await LocalTxStore.markCursorSynced(
              accountId: normalizedAccountId,
              blockNumber: block,
            );
            lastByPublicKey[normalizedAccountId] = block;
          }
        }
        await Future<void>.delayed(const Duration(milliseconds: 20));
      }
    }

  }

  /// 处理一个 finalized 区块的 System.Events。
  ///
  /// 调用方保证 [blockNumber] ≤ finalized 高度，按块哈希钉块读取，
  /// 写入的流水状态恒为 finalized(已确认)。
  Future<bool> _processBlock(int blockNumber) async {
    try {
      final blockHashHex =
          await SmoldotClientManager.instance.getBlockHash(blockNumber);
      if (blockHashHex == null || blockHashHex.isEmpty) return false;

      final keyHex = '0x${_hexEncode(_eventsStorageKey)}';
      final result = await SmoldotClientManager.instance.request(
        'state_getStorage',
        [keyHex, blockHashHex],
      );
      final eventsHex = result as String?;
      if (eventsHex == null) return true;

      final eventsBytes = _hexDecode(
        eventsHex.startsWith('0x') ? eventsHex.substring(2) : eventsHex,
      );
      if (eventsBytes.isEmpty) return true;

      // 先按 txHash 精确认本机提交的待确认交易（就地翻已确认/失败），并记下
      // 已认领的 accountId#extrinsicIndex；下面转出侧据此跳过、绝不另建第二条。
      _claimedThisBlock =
          await _confirmSubmittedByTxHash(blockNumber, blockHashHex, eventsBytes);

      await _decodeTransferEvents(eventsBytes, blockNumber, blockHashHex);
      return true;
    } catch (e) {
      AppLog.d('[TxMonitor] 同步区块 $blockNumber 失败: $e');
      return false;
    }
  }

  /// 按 txHash 精确认本机提交的待确认交易。
  ///
  /// 遍历各监控钱包"未终态"的本机提交记录（[LocalTxStore.queryOpenLocalSubmit]），
  /// 若其 txHash 出现在本最终块 → 就地翻 finalized；若该 extrinsic 链上 ExtrinsicFailed
  /// → 翻 failed。全程只动那一条 txHash 记录，绝不另建。返回已认领的
  /// "accountId#extrinsicIndex" 集合，供转出侧跳过、避免重复建记录。
  ///
  /// 没有待确认记录时直接返回、不取块 extrinsics（省节点负担）。
  Future<Set<String>> _confirmSubmittedByTxHash(
    int blockNumber,
    String blockHashHex,
    Uint8List eventsBytes,
  ) async {
    final claimed = <String>{};
    final openRecords = <LocalTxEntity>[];
    for (final accountId in _ss58AddressByAccountId.keys) {
      openRecords.addAll(await LocalTxStore.queryOpenLocalSubmit(accountId));
    }
    if (openRecords.isEmpty) return claimed;

    final List<String> extrinsics;
    try {
      extrinsics = await SmoldotClientManager.instance
          .getFinalizedBlockExtrinsicsOnce(blockHashHex)
          .timeout(const Duration(seconds: 8));
    } catch (e) {
      // 取不到块体(丢 peer / 超时):跳过本块认领、游标照常推进,监视器绝不因此
      // 卡死或重试风暴;确认下限由 [_confirmOpenSubmits](锚比对 + nonce 兜底)
      // 保证,漏掉的记录会在那里翻状态。8s 超时防原生调用无限挂起。
      AppLog.d('[TxMonitor] 取块 $blockNumber extrinsics 失败，跳过 txHash 认领: $e');
      return claimed;
    }

    for (final record in openRecords) {
      final txHash = record.txHash;
      if (txHash == null || txHash.isEmpty) continue;
      final idx = await ChainRpc.findExtrinsicIndexInHexList(
        extrinsics,
        txHashHex: txHash,
      );
      if (idx == null) continue; // 这笔不在本块，继续等后面的最终块
      final failure = _chainRpc.findExtrinsicFailureInEvents(
        eventsBytes,
        extrinsicIndex: idx,
      );
      if (failure != null) {
        await LocalTxStore.markLocalSubmitFailed(
          accountId: record.accountId,
          txHash: txHash,
          failureReason: failure.description,
        );
      } else {
        await LocalTxStore.markLocalSubmitFinalized(
          accountId: record.accountId,
          txHash: txHash,
          blockHash: blockHashHex,
          blockNumber: blockNumber,
          extrinsicIndex: idx,
        );
      }
      claimed.add('${record.accountId}#$idx');
    }
    return claimed;
  }

  /// 确认所有"未终态"本机提交记录的唯一兜底 —— 与前向游标完全解耦,零扫块。
  ///
  /// 每轮同步末尾执行;无待确认记录时一次本地查询即返回。对每条记录按两级判据:
  ///
  /// - **判据一(锚比对)**:blockHash 是交易池 inBlock 事件写入的"本笔所在块"锚
  ///   (`dropped` 不再清它)。读该块头取块号 N;若 N ≤ finalized 高度且最终链在
  ///   N 高度的块哈希与锚相等 ⇒ 锚块已最终、本笔已上链 → 对这一个块跑一次
  ///   [_processBlock](按 txHash 认领 + ExtrinsicFailed 精查 + 事件补写;单块
  ///   一次性,与前向扫描处理一个新块同量级)。锚不等/块头取不到 → 降级判据二。
  ///
  /// - **判据二(nonce 兜底)**:账户 nonce 单调递增、只有交易上链才被消费。
  ///   finalized 状态下账户 nonce > 记录 usedNonce ⇒ 该 nonce 已被最终链消费 ⇒
  ///   本笔已上链 → 翻 finalized(不带块号,保留原字段)。私钥仅在本机、app 串行
  ///   提交,同 nonce 顶替(usurped)已在交易池 watch 单独判失败,判据严格成立。
  ///   局限:不区分"上链但执行失败"(该情形 nonce 同样被消费;概率极低 ——
  ///   提交前有余额/ED 校验,带锚记录会走判据一精查)。
  ///
  /// 资源账:每条记录至多 2 次读头 + 每账户至多 1 次 System.Account 快照读;
  /// **永不窗口扫块、永不批量下载块体**,绝不挤占链状态轮询(ChainProgressBanner)。
  Future<void> _confirmOpenSubmits() async {
    if (!_running || _ss58AddressByAccountId.isEmpty) return;
    final head = (await _chainRpc.fetchFinalizedBlock()).blockNumber;
    for (final accountId in _ss58AddressByAccountId.keys) {
      var records = await LocalTxStore.queryOpenLocalSubmit(accountId);
      if (records.isEmpty) continue;

      // 判据一:锚比对(同锚块只处理一次)。
      final processedAnchors = <String>{};
      for (final record in records) {
        if (!_running) return;
        final anchor = record.blockHash;
        if (anchor == null || anchor.isEmpty) continue;
        if (!processedAnchors.add(anchor)) continue;
        final blockNumber = await _blockNumberByHash(anchor);
        if (blockNumber == null || blockNumber > head) continue;
        final finalizedHash =
            await SmoldotClientManager.instance.getBlockHash(blockNumber);
        if (finalizedHash == null ||
            LocalTxStore.normalizeBlockHash(finalizedHash) !=
                LocalTxStore.normalizeBlockHash(anchor)) {
          // 锚块被最终链顶掉(交易可能被重排进别的块):交给判据二兜底。
          continue;
        }
        await _processBlock(blockNumber);
      }

      // 判据二:nonce 兜底(锚路径后仍未终态的记录)。
      records = await LocalTxStore.queryOpenLocalSubmit(accountId);
      if (records.isEmpty) continue;
      final int? finalizedNonce;
      try {
        finalizedNonce = (await SmoldotClientManager.instance
                .getFinalizedSystemAccountSnapshot(accountId))
            ?.nonce;
      } catch (e) {
        AppLog.d('[TxMonitor] 读取账户 nonce 失败,下轮再确认: $e');
        continue;
      }
      if (finalizedNonce == null) continue;
      for (final record in records) {
        final txHash = record.txHash;
        final usedNonce = record.usedNonce;
        if (txHash == null || txHash.isEmpty || usedNonce == null) continue;
        if (finalizedNonce > usedNonce) {
          await LocalTxStore.markLocalSubmitFinalized(
            accountId: record.accountId,
            txHash: txHash,
          );
          AppLog.d('[TxMonitor] nonce 兜底确认: tx=$txHash '
              'usedNonce=$usedNonce < 账户nonce=$finalizedNonce');
        }
      }
    }
  }

  /// 按块哈希取块号（`chain_getHeader.number`）。失败返回 null。
  Future<int?> _blockNumberByHash(String blockHashHex) async {
    try {
      final header = await SmoldotClientManager.instance
          .request('chain_getHeader', [blockHashHex]);
      if (header is Map) {
        final number = header['number'];
        if (number is String && number.isNotEmpty) {
          final hex = number.startsWith('0x') ? number.substring(2) : number;
          return int.parse(hex, radix: 16);
        }
      }
    } catch (e) {
      AppLog.d('[TxMonitor] chain_getHeader($blockHashHex) 取块号失败: $e');
    }
    return null;
  }

  /// 解码 System.Events，优先提取 OnchainTransaction 转账事件。
  ///
  /// Balances::Transfer 只作为底层余额事件兜底；外部普通转账入口仍然唯一收口到
  /// OnchainTransaction::transfer_with_remark。
  Future<void> _decodeTransferEvents(
    Uint8List data,
    int blockNumber,
    String blockHash,
  ) async {
    try {
      final keyHex = '0x${_hexEncode(_eventsStorageKey)}';
      final metadata = await _chainRpc.fetchMetadata();
      final events = Events.fromJson({
        'changes': [
          [keyHex, '0x${_hexEncode(data)}']
        ],
      }, metadata.chainInfo);

      for (var index = 0; index < events.eventRecord.length; index++) {
        final record = events.eventRecord[index];
        final transferWithRemark = _readTransferWithRemark(record.event);
        if (transferWithRemark != null) {
          final extrinsicIndex = _readExtrinsicIndex(record.phase);
          await _writeTransferForBothSides(
            fromAccountId: transferWithRemark.fromAccountId,
            toAccountId: transferWithRemark.toAccountId,
            transferAmountFen: transferWithRemark.amountFen,
            blockNumber: blockNumber,
            blockHash: blockHash,
            eventRecordIndex: index,
            extrinsicIndex: extrinsicIndex,
            remark: transferWithRemark.remark,
          );
          continue;
        }
        final transfer = _readBalancesTransfer(record.event);
        if (transfer == null) continue;
        final extrinsicIndex = _readExtrinsicIndex(record.phase);
        await _writeTransferForBothSides(
          fromAccountId: transfer.fromAccountId,
          toAccountId: transfer.toAccountId,
          transferAmountFen: transfer.amountFen,
          blockNumber: blockNumber,
          blockHash: blockHash,
          eventRecordIndex: index,
          extrinsicIndex: extrinsicIndex,
        );
      }
      return;
    } catch (e) {
      AppLog.d('[TxMonitor] metadata 事件解码失败，使用兜底解析: $e');
    }

    await _decodeTransferEventsFallback(data, blockNumber, blockHash);
  }

  Future<void> _decodeTransferEventsFallback(
    Uint8List data,
    int blockNumber,
    String blockHash,
  ) async {
    var offset = 0;
    var eventRecordIndex = 0;
    if (data.isEmpty) return;
    final (_, countSize) = _decodeCompactU32(data, 0);
    offset += countSize;

    while (offset + 4 < data.length) {
      int? extrinsicIndex;
      final phase = data[offset];
      offset += 1;
      if (phase == 0x00) {
        if (offset + 4 > data.length) break;
        extrinsicIndex = _readU32LE(data, offset);
        offset += 4;
      }

      if (offset + 2 > data.length) break;
      final palletIndex = data[offset];
      final eventIndex = data[offset + 1];
      offset += 2;

      if (palletIndex == _balancesPallet && eventIndex == _transferEvent) {
        // Balances::Transfer { from: AccountId, to: AccountId, amount: u128 }
        if (offset + 80 <= data.length) {
          final from = data.sublist(offset, offset + 32);
          final to = data.sublist(offset + 32, offset + 64);
          final amountBytes = data.sublist(offset + 64, offset + 80);
          offset += 80;

          final fromAccountId = '0x${_hexEncode(from)}';
          final toAccountId = '0x${_hexEncode(to)}';
          final transferAmountFen = _readU128LE(amountBytes, 0).toString();

          await _writeTransferForBothSides(
            fromAccountId: fromAccountId,
            toAccountId: toAccountId,
            transferAmountFen: transferAmountFen,
            blockNumber: blockNumber,
            blockHash: blockHash,
            eventRecordIndex: eventRecordIndex,
            extrinsicIndex: extrinsicIndex,
          );

          offset = _skipTopics(data, offset);
          eventRecordIndex++;
          continue;
        }
      }
      if (palletIndex == _onchainTransactionPallet &&
          eventIndex == _transferWithRemarkEvent) {
        // OnchainTransaction::TransferWithRemark { from, beneficiary, amount, remark }
        if (offset + 81 <= data.length) {
          final from = data.sublist(offset, offset + 32);
          final to = data.sublist(offset + 32, offset + 64);
          final amountBytes = data.sublist(offset + 64, offset + 80);
          offset += 80;
          final (remarkLen, remarkLenSize) = _decodeCompactU32(data, offset);
          if (remarkLenSize == 0 ||
              offset + remarkLenSize + remarkLen > data.length) {
            break;
          }
          offset += remarkLenSize;
          final remark = remarkLen == 0
              ? null
              : utf8.decode(
                  data.sublist(offset, offset + remarkLen),
                  allowMalformed: true,
                );
          offset += remarkLen;

          await _writeTransferForBothSides(
            fromAccountId: '0x${_hexEncode(from)}',
            toAccountId: '0x${_hexEncode(to)}',
            transferAmountFen: _readU128LE(amountBytes, 0).toString(),
            blockNumber: blockNumber,
            blockHash: blockHash,
            eventRecordIndex: eventRecordIndex,
            extrinsicIndex: extrinsicIndex,
            remark: remark,
          );

          offset = _skipTopics(data, offset);
          eventRecordIndex++;
          continue;
        }
      }

      final skipped = _skipKnownEventPayload(data, offset, palletIndex,
          eventIndex: eventIndex);
      if (skipped != null) {
        offset = _skipTopics(data, skipped);
        eventRecordIndex++;
        continue;
      }

      // 未识别事件：尝试跳到下一个 EventRecord。
      offset = _skipToNextEvent(data, offset);
      eventRecordIndex++;
    }
  }

  Future<void> _writeTransferForBothSides({
    required String fromAccountId,
    required String toAccountId,
    required String transferAmountFen,
    required int blockNumber,
    required String blockHash,
    required int eventRecordIndex,
    required int? extrinsicIndex,
    String? remark,
  }) async {
    if (fromAccountId == toAccountId) return;
    final fromBytes = _hexDecode(fromAccountId);
    final toBytes = _hexDecode(toAccountId);
    await _writeWalletTransferIfMatched(
      accountId: toAccountId,
      blockNumber: blockNumber,
      blockHash: blockHash,
      eventRecordIndex: eventRecordIndex,
      extrinsicIndex: extrinsicIndex,
      amountDeltaFen: transferAmountFen,
      transferAmountFen: transferAmountFen,
      fromSs58Address: _publicKeyToSs58(fromBytes),
      toSs58Address:
          _ss58AddressByAccountId[toAccountId] ?? _publicKeyToSs58(toBytes),
      counterpartySs58Address: _publicKeyToSs58(fromBytes),
      remark: remark,
    );

    // 转出侧：若本笔已被 txHash 认领（本机提交、已在 _confirmSubmittedByTxHash
    // 里就地翻状态）→ 跳过，绝不另建 blockHash 键的第二条记录。收入侧不受影响。
    if (!_claimedThisBlock.contains('$fromAccountId#$extrinsicIndex')) {
      await _writeWalletTransferIfMatched(
        accountId: fromAccountId,
        blockNumber: blockNumber,
        blockHash: blockHash,
        eventRecordIndex: eventRecordIndex,
        extrinsicIndex: extrinsicIndex,
        amountDeltaFen: LocalTxStore.negateFen(transferAmountFen),
        transferAmountFen: transferAmountFen,
        fromSs58Address: _ss58AddressByAccountId[fromAccountId] ??
            _publicKeyToSs58(fromBytes),
        toSs58Address: _publicKeyToSs58(toBytes),
        counterpartySs58Address: _publicKeyToSs58(toBytes),
        remark: remark,
      );
    }
  }

  _DecodedTransferEvent? _readTransferWithRemark(Map<String, dynamic> event) {
    final onchain = event['OnchainTransaction'] ?? event['onchainTransaction'];
    if (onchain is! Map) return null;
    final transfer = onchain['TransferWithRemark'] ??
        onchain['transferWithRemark'] ??
        onchain['transfer_with_remark'];
    if (transfer == null) return null;

    dynamic from;
    dynamic to;
    dynamic amount;
    dynamic remark;
    if (transfer is Map) {
      from = transfer['from'] ?? transfer['0'];
      to = transfer['beneficiary'] ?? transfer['to'] ?? transfer['1'];
      amount = transfer['amount'] ?? transfer['2'];
      remark = transfer['remark'] ?? transfer['3'];
      if ((from == null || to == null || amount == null || remark == null) &&
          transfer.values.length >= 4) {
        final values = transfer.values.toList(growable: false);
        from ??= values[0];
        to ??= values[1];
        amount ??= values[2];
        remark ??= values[3];
      }
    } else if (transfer is List && transfer.length >= 4) {
      from = transfer[0];
      to = transfer[1];
      amount = transfer[2];
      remark = transfer[3];
    }

    final fromAccountId = _decodeAccountId(from);
    final toAccountId = _decodeAccountId(to);
    final amountFen = _eventAmountToFen(amount);
    if (fromAccountId == null || toAccountId == null || amountFen == null) {
      return null;
    }
    return _DecodedTransferEvent(
      fromAccountId: fromAccountId,
      toAccountId: toAccountId,
      amountFen: amountFen,
      remark: _eventRemarkToString(remark),
    );
  }

  _DecodedTransferEvent? _readBalancesTransfer(Map<String, dynamic> event) {
    final balances = event['Balances'] ?? event['balances'];
    if (balances is! Map) return null;
    final transfer = balances['Transfer'] ?? balances['transfer'];
    if (transfer == null) return null;

    dynamic from;
    dynamic to;
    dynamic amount;
    if (transfer is Map) {
      from = transfer['from'] ?? transfer['0'];
      to = transfer['to'] ?? transfer['1'];
      amount = transfer['amount'] ?? transfer['value'] ?? transfer['2'];
      if ((from == null || to == null || amount == null) &&
          transfer.values.length >= 3) {
        final values = transfer.values.toList(growable: false);
        from ??= values[0];
        to ??= values[1];
        amount ??= values[2];
      }
    } else if (transfer is List && transfer.length >= 3) {
      from = transfer[0];
      to = transfer[1];
      amount = transfer[2];
    }

    final fromAccountId = _decodeAccountId(from);
    final toAccountId = _decodeAccountId(to);
    final amountFen = _eventAmountToFen(amount);
    if (fromAccountId == null || toAccountId == null || amountFen == null) {
      return null;
    }
    return _DecodedTransferEvent(
      fromAccountId: fromAccountId,
      toAccountId: toAccountId,
      amountFen: amountFen,
    );
  }

  int? _readExtrinsicIndex(Map<String, dynamic> phase) {
    final value = phase['ApplyExtrinsic'] ?? phase['applyExtrinsic'];
    if (value is int) return value;
    if (value is BigInt) return value.toInt();
    if (value is String) return int.tryParse(value);
    return null;
  }

  String? _decodeAccountId(dynamic raw) {
    if (raw is Uint8List && raw.length == 32) {
      return '0x${_hexEncode(raw)}';
    }
    if (raw is List) {
      final bytes = raw.whereType<int>().toList(growable: false);
      if (bytes.length == 32) {
        return '0x${_hexEncode(Uint8List.fromList(bytes))}';
      }
    }
    if (raw is String) {
      final text = raw.trim();
      final hex = text.startsWith('0x') ? text.substring(2) : text;
      final isHex = RegExp(r'^[0-9a-fA-F]{64}$').hasMatch(hex);
      if (isHex) return '0x${hex.toLowerCase()}';
      try {
        return '0x${_hexEncode(
          Uint8List.fromList(Keyring().decodeAddress(text)),
        )}';
      } catch (_) {
        return null;
      }
    }
    return null;
  }

  String? _eventAmountToFen(dynamic raw) {
    if (raw is BigInt) return raw.toString();
    if (raw is int) return raw.toString();
    if (raw is String) return BigInt.tryParse(raw)?.toString();
    return null;
  }

  String? _eventRemarkToString(dynamic raw) {
    if (raw == null) return null;
    if (raw is Uint8List) {
      return raw.isEmpty ? null : utf8.decode(raw, allowMalformed: true);
    }
    if (raw is List) {
      final bytes = raw.whereType<int>().toList(growable: false);
      return bytes.isEmpty ? null : utf8.decode(bytes, allowMalformed: true);
    }
    if (raw is Map) {
      final bytes = raw.values.whereType<int>().toList(growable: false);
      if (bytes.isNotEmpty) {
        return utf8.decode(bytes, allowMalformed: true);
      }
    }
    if (raw is String) {
      final text = raw.trim();
      if (text.isEmpty) return null;
      if (RegExp(r'^0x[0-9a-fA-F]*$').hasMatch(text)) {
        final bytes = _hexDecode(text.substring(2));
        return bytes.isEmpty ? null : utf8.decode(bytes, allowMalformed: true);
      }
      return raw;
    }
    return raw.toString();
  }

  int? _skipKnownEventPayload(
    Uint8List data,
    int offset,
    int palletIndex, {
    required int eventIndex,
  }) {
    // metadata 解码正常时不会走到这里；兜底分支只显式跳过
    // 普通转账前后最常见的定长事件，避免旧版“向前扫描”误命中 payload 字节。
    final oneAccountAndAmount = offset + 48 <= data.length ? offset + 48 : null;
    if (palletIndex == _balancesPallet) {
      if (eventIndex == 7 ||
          eventIndex == 8 ||
          eventIndex == 10 ||
          eventIndex == 11) {
        return oneAccountAndAmount;
      }
    }
    // OnchainTransaction::FeePaid { who: AccountId, fee: u128 }
    if (palletIndex == 4 && eventIndex == 0) {
      return oneAccountAndAmount;
    }
    // OnchainTransaction::FeeShareBurnt { reason: BurnReason, amount: u128 }
    if (palletIndex == 4 && eventIndex == 1) {
      return offset + 17 <= data.length ? offset + 17 : null;
    }
    return null;
  }

  Future<void> _writeWalletTransferIfMatched({
    required String accountId,
    required int blockNumber,
    required String blockHash,
    required int eventRecordIndex,
    required int? extrinsicIndex,
    required String amountDeltaFen,
    required String transferAmountFen,
    required String fromSs58Address,
    required String toSs58Address,
    required String counterpartySs58Address,
    String? remark,
  }) async {
    final normalizedAccountId = LocalTxStore.requireAccountId(accountId);
    final ss58Address = _ss58AddressByAccountId[normalizedAccountId];
    if (ss58Address == null) return;

    await LocalTxStore.upsertBlockTransferEvent(
      ss58Address: ss58Address,
      accountId: normalizedAccountId,
      recordKey: LocalTxStore.blockEventRecordKey(
        normalizedAccountId,
        blockHash,
        eventRecordIndex,
      ),
      // (ADR-017)：监控只扫 finalized 链，写入状态恒为"已确认"。
      status: LocalTxStore.statusFinalized,
      amountDeltaFen: amountDeltaFen,
      transferAmountFen: transferAmountFen,
      fromSs58Address: fromSs58Address,
      toSs58Address: toSs58Address,
      counterpartySs58Address: counterpartySs58Address,
      blockNumber: blockNumber,
      blockHash: blockHash,
      eventIndex: eventRecordIndex,
      extrinsicIndex: extrinsicIndex,
      remark: remark,
    );

    try {
      final balance =
          await _chainRpc.fetchFinalizedBalance(normalizedAccountId);
      onBalanceChanged?.call(ss58Address, balance);
    } catch (_) {
      // 交易记录已经落库，余额刷新失败不能把钱包余额误写成 0。
      onBalanceChanged?.call(ss58Address, double.nan);
    }
  }

  // ──── 工具方法 ────

  String _publicKeyToSs58(Uint8List normalizedAccountId) {
    try {
      return Keyring().encodeAddress(normalizedAccountId.toList(), kGmbSs58Prefix);
    } catch (_) {
      return '0x${_hexEncode(normalizedAccountId)}';
    }
  }

  static String _hexEncode(Uint8List bytes) {
    return bytes.map((b) => b.toRadixString(16).padLeft(2, '0')).join();
  }

  static Uint8List _hexDecode(String hex) {
    final normalized = hex.startsWith('0x') ? hex.substring(2) : hex;
    final result = Uint8List(normalized.length ~/ 2);
    for (var i = 0; i < result.length; i++) {
      result[i] = int.parse(
        normalized.substring(i * 2, i * 2 + 2),
        radix: 16,
      );
    }
    return result;
  }

  static int _readU32LE(Uint8List bytes, int offset) {
    return bytes[offset] |
        (bytes[offset + 1] << 8) |
        (bytes[offset + 2] << 16) |
        (bytes[offset + 3] << 24);
  }

  static BigInt _readU128LE(Uint8List bytes, int offset) {
    var value = BigInt.zero;
    for (var i = 15; i >= 0; i--) {
      value = (value << 8) | BigInt.from(bytes[offset + i]);
    }
    return value;
  }

  static (int, int) _decodeCompactU32(Uint8List bytes, int offset) {
    if (offset >= bytes.length) return (0, 0);
    final mode = bytes[offset] & 0x03;
    switch (mode) {
      case 0:
        return (bytes[offset] >> 2, 1);
      case 1:
        if (offset + 2 > bytes.length) return (0, 0);
        return (((bytes[offset + 1] << 8) | bytes[offset]) >> 2, 2);
      case 2:
        if (offset + 4 > bytes.length) return (0, 0);
        return (
          ((bytes[offset + 3] << 24) |
                  (bytes[offset + 2] << 16) |
                  (bytes[offset + 1] << 8) |
                  bytes[offset]) >>
              2,
          4
        );
      default:
        return (0, 1);
    }
  }

  /// 跳过 topics（Vec<Hash>）。
  static int _skipTopics(Uint8List data, int offset) {
    if (offset >= data.length) return offset;
    final (count, size) = _decodeCompactU32(data, offset);
    offset += size;
    offset += count * 32;
    return offset;
  }

  /// 未识别事件时，向前扫描寻找下一个合法 EventRecord 的 phase 起点。
  static int _skipToNextEvent(Uint8List data, int offset) {
    for (var i = offset; i < data.length - 3; i++) {
      final byte = data[i];
      if (byte == 0x01 || byte == 0x02) {
        final nextPallet = data[i + 1];
        if (nextPallet < 64) return i;
      } else if (byte == 0x00 && i + 5 < data.length) {
        final possiblePallet = data[i + 5];
        if (possiblePallet < 64) return i;
      }
    }
    return data.length;
  }
}
