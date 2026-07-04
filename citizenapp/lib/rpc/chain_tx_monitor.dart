import 'dart:async';
import 'dart:convert';

import 'package:flutter/foundation.dart';
import 'package:polkadart/polkadart.dart' show Events, Hasher;
import 'package:polkadart_keyring/polkadart_keyring.dart' show Keyring;

import 'chain_event_subscription.dart';
import 'chain_read_cache.dart';
import 'chain_rpc.dart';
import 'smoldot_client.dart';
import 'package:citizenapp/isar/wallet_isar.dart';
import 'package:citizenapp/transaction/shared/local_tx_store.dart';

class _DecodedTransferEvent {
  const _DecodedTransferEvent({
    required this.fromHex,
    required this.toHex,
    required this.amountFen,
    this.remark,
  });

  final String fromHex;
  final String toHex;
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
  bool _running = false;
  bool _subscriptionConnected = false;

  /// 当前监控的钱包：pubkeyHex(小写，不含 0x) → SS58 地址。
  final Map<String, String> _walletAddressByPubkey = {};

  /// 余额变动回调：当检测到余额变化（写入新交易记录后）通知外部刷新。
  void Function(String walletAddress, double newBalance)? onBalanceChanged;

  /// SS58 前缀。
  static const int _ss58Prefix = 2027;

  /// 每次补同步最多连续处理的区块数，避免手机长时间离线后一次性压节点。
  static const int _maxBlocksPerRun = 120;

  // ──── 已知事件的 pallet_index + event_index ────

  /// Balances::Transfer (pallet=2, event=2)，仅作为底层余额事件兜底。
  static const int _balancesPallet = 2;
  static const int _transferEvent = 2;
  static const int _onchainTransactionPallet = 4;
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
  void watchWallet(String address, String pubkeyHex) {
    final pk = LocalTxStore.normalizePubkey(pubkeyHex);
    _walletAddressByPubkey[pk] = address;
  }

  /// 移除监控钱包。
  void unwatchWallet(String address) {
    _walletAddressByPubkey.removeWhere((_, value) => value == address);
  }

  /// 启动监控。
  Future<void> start() async {
    if (_running) {
      _ensureSubscription();
      unawaited(_syncToLatest());
      return;
    }
    _running = true;

    _listener = _subscription.events.listen(_onEvent);
    _ensureSubscription();
    debugPrint('[TxMonitor] 交易监控已启动，监控 ${_walletAddressByPubkey.length} 个钱包');

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
    debugPrint('[TxMonitor] 交易监控已停止');
  }

  /// 初始化钱包基准游标（导入钱包时可调用）。
  Future<void> initBaselineBalance(String address, String pubkeyHex) async {
    watchWallet(address, pubkeyHex);
    try {
      final finalized = await _chainRpc.fetchFinalizedBlock();
      await LocalTxStore.ensureCursor(
        walletAddress: address,
        walletPubkeyHex: pubkeyHex,
        trackingStartBlock: finalized.blockNumber,
        lastSyncedBlock: finalized.blockNumber,
      );
      debugPrint('[TxMonitor] 初始化交易记录游标: $address @${finalized.blockNumber}');
    } catch (e) {
      debugPrint('[TxMonitor] 初始化交易记录游标失败，稍后从轻节点就绪块开始: $e');
    }
  }

  // ──── 同步调度 ────

  void _ensureSubscription() {
    if (!_running) return;
    if (_subscriptionConnected) return;

    final connected = _subscription.connect();
    if (connected) {
      _subscriptionConnected = true;
      _subscriptionRetryTimer?.cancel();
      _subscriptionRetryTimer = null;
      unawaited(_syncToLatest());
      return;
    }

    _subscriptionRetryTimer ??= Timer.periodic(const Duration(seconds: 5), (_) {
      if (!_running) return;
      if (_subscriptionConnected) return;
      if (_subscription.connect()) {
        _subscriptionConnected = true;
        _subscriptionRetryTimer?.cancel();
        _subscriptionRetryTimer = null;
        unawaited(_syncToLatest());
      }
    });
  }

  Future<void> _onEvent(ChainEvent event) async {
    if (!_running || _walletAddressByPubkey.isEmpty) return;
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
    if (_walletAddressByPubkey.isEmpty) return;
    try {
      final finalized = await _chainRpc.fetchFinalizedBlock();
      await _syncThrough(
        finalized.blockNumber,
        missingCursorStartsAt: finalized.blockNumber,
      );
    } catch (e) {
      debugPrint('[TxMonitor] 启动补同步失败: $e');
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
    if (_walletAddressByPubkey.isEmpty) return;

    final cursors = await LocalTxStore.ensureCursorsForWallets(
      walletAddressByPubkey: _walletAddressByPubkey,
      startBlock: missingCursorStartsAt,
    );
    final lastByPubkey = {
      for (final cursor in cursors)
        cursor.walletPubkeyHex: cursor.lastSyncedBlock,
    };
    final startBlock = lastByPubkey.values
            .fold<int>(targetBlock, (min, value) => value < min ? value : min) +
        1;
    if (startBlock > targetBlock) return;

    final endBlock = startBlock + _maxBlocksPerRun - 1 < targetBlock
        ? startBlock + _maxBlocksPerRun - 1
        : targetBlock;
    for (var block = startBlock; block <= endBlock; block++) {
      if (!_running || _walletAddressByPubkey.isEmpty) return;
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

      for (final pubkey in _walletAddressByPubkey.keys) {
        final last = lastByPubkey[pubkey] ?? missingCursorStartsAt;
        if (last < block) {
          await LocalTxStore.markCursorSynced(
            walletPubkeyHex: pubkey,
            blockNumber: block,
          );
          lastByPubkey[pubkey] = block;
        }
      }
      await Future<void>.delayed(const Duration(milliseconds: 20));
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

      await _decodeTransferEvents(eventsBytes, blockNumber, blockHashHex);
      return true;
    } catch (e) {
      debugPrint('[TxMonitor] 同步区块 $blockNumber 失败: $e');
      return false;
    }
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
            fromHex: transferWithRemark.fromHex,
            toHex: transferWithRemark.toHex,
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
          fromHex: transfer.fromHex,
          toHex: transfer.toHex,
          transferAmountFen: transfer.amountFen,
          blockNumber: blockNumber,
          blockHash: blockHash,
          eventRecordIndex: index,
          extrinsicIndex: extrinsicIndex,
        );
      }
      return;
    } catch (e) {
      debugPrint('[TxMonitor] metadata 事件解码失败，使用兜底解析: $e');
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

          final fromHex = _hexEncode(from);
          final toHex = _hexEncode(to);
          final transferAmountFen = _readU128LE(amountBytes, 0).toString();

          await _writeTransferForBothSides(
            fromHex: fromHex,
            toHex: toHex,
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
            fromHex: _hexEncode(from),
            toHex: _hexEncode(to),
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
    required String fromHex,
    required String toHex,
    required String transferAmountFen,
    required int blockNumber,
    required String blockHash,
    required int eventRecordIndex,
    required int? extrinsicIndex,
    String? remark,
  }) async {
    if (fromHex == toHex) return;
    final fromBytes = _hexDecode(fromHex);
    final toBytes = _hexDecode(toHex);
    await _writeWalletTransferIfMatched(
      walletPubkeyHex: toHex,
      blockNumber: blockNumber,
      blockHash: blockHash,
      eventRecordIndex: eventRecordIndex,
      extrinsicIndex: extrinsicIndex,
      amountDeltaFen: transferAmountFen,
      transferAmountFen: transferAmountFen,
      fromAddress: _pubkeyToSs58(fromBytes),
      toAddress: _walletAddressByPubkey[toHex] ?? _pubkeyToSs58(toBytes),
      counterpartyAddress: _pubkeyToSs58(fromBytes),
      remark: remark,
    );

    await _writeWalletTransferIfMatched(
      walletPubkeyHex: fromHex,
      blockNumber: blockNumber,
      blockHash: blockHash,
      eventRecordIndex: eventRecordIndex,
      extrinsicIndex: extrinsicIndex,
      amountDeltaFen: LocalTxStore.negateFen(transferAmountFen),
      transferAmountFen: transferAmountFen,
      fromAddress: _walletAddressByPubkey[fromHex] ?? _pubkeyToSs58(fromBytes),
      toAddress: _pubkeyToSs58(toBytes),
      counterpartyAddress: _pubkeyToSs58(toBytes),
      remark: remark,
    );
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

    final fromHex = _accountToPubkeyHex(from);
    final toHex = _accountToPubkeyHex(to);
    final amountFen = _eventAmountToFen(amount);
    if (fromHex == null || toHex == null || amountFen == null) return null;
    return _DecodedTransferEvent(
      fromHex: fromHex,
      toHex: toHex,
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

    final fromHex = _accountToPubkeyHex(from);
    final toHex = _accountToPubkeyHex(to);
    final amountFen = _eventAmountToFen(amount);
    if (fromHex == null || toHex == null || amountFen == null) return null;
    return _DecodedTransferEvent(
      fromHex: fromHex,
      toHex: toHex,
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

  String? _accountToPubkeyHex(dynamic raw) {
    if (raw is Uint8List) return _hexEncode(raw);
    if (raw is List) {
      final bytes = raw.whereType<int>().toList(growable: false);
      if (bytes.length == 32) return _hexEncode(Uint8List.fromList(bytes));
    }
    if (raw is String) {
      final text = raw.trim();
      final hex = text.startsWith('0x') ? text.substring(2) : text;
      final isHex = RegExp(r'^[0-9a-fA-F]{64}$').hasMatch(hex);
      if (isHex) return hex.toLowerCase();
      try {
        return _hexEncode(Uint8List.fromList(Keyring().decodeAddress(text)));
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
    required String walletPubkeyHex,
    required int blockNumber,
    required String blockHash,
    required int eventRecordIndex,
    required int? extrinsicIndex,
    required String amountDeltaFen,
    required String transferAmountFen,
    required String fromAddress,
    required String toAddress,
    required String counterpartyAddress,
    String? remark,
  }) async {
    final pubkey = LocalTxStore.normalizePubkey(walletPubkeyHex);
    final walletAddress = _walletAddressByPubkey[pubkey];
    if (walletAddress == null) return;

    await LocalTxStore.upsertBlockTransferEvent(
      walletAddress: walletAddress,
      walletPubkeyHex: pubkey,
      recordKey: LocalTxStore.blockEventRecordKey(
        pubkey,
        blockHash,
        eventRecordIndex,
      ),
      // (ADR-017)：监控只扫 finalized 链，写入状态恒为"已确认"。
      status: LocalTxStore.statusFinalized,
      amountDeltaFen: amountDeltaFen,
      transferAmountFen: transferAmountFen,
      fromAddress: fromAddress,
      toAddress: toAddress,
      counterpartyAddress: counterpartyAddress,
      blockNumber: blockNumber,
      blockHash: blockHash,
      eventIndex: eventRecordIndex,
      extrinsicIndex: extrinsicIndex,
      remark: remark,
    );

    try {
      final balance = await _chainRpc.fetchFinalizedBalance(pubkey);
      onBalanceChanged?.call(walletAddress, balance);
    } catch (_) {
      // 交易记录已经落库，余额刷新失败不能把钱包余额误写成 0。
      onBalanceChanged?.call(walletAddress, double.nan);
    }
  }

  // ──── 工具方法 ────

  String _pubkeyToSs58(Uint8List pubkey) {
    try {
      return Keyring().encodeAddress(pubkey.toList(), _ss58Prefix);
    } catch (_) {
      return '0x${_hexEncode(pubkey)}';
    }
  }

  static String _hexEncode(Uint8List bytes) {
    return bytes.map((b) => b.toRadixString(16).padLeft(2, '0')).join();
  }

  static Uint8List _hexDecode(String hex) {
    final result = Uint8List(hex.length ~/ 2);
    for (var i = 0; i < result.length; i++) {
      result[i] = int.parse(hex.substring(i * 2, i * 2 + 2), radix: 16);
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
