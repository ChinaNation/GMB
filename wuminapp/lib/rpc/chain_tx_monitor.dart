import 'dart:async';

import 'package:flutter/foundation.dart';
import 'package:polkadart/polkadart.dart' show Events, Hasher;
import 'package:polkadart_keyring/polkadart_keyring.dart' show Keyring;

import 'chain_event_subscription.dart';
import 'chain_rpc.dart';
import 'smoldot_client.dart';
import 'package:wuminapp_mobile/isar/wallet_isar.dart';
import 'package:wuminapp_mobile/transaction/shared/local_tx_store.dart';

enum _TransferEventStatus {
  inBlock,
  finalized;

  String get storeStatus {
    return switch (this) {
      _TransferEventStatus.inBlock => LocalTxStore.statusInBlock,
      _TransferEventStatus.finalized => LocalTxStore.statusFinalized,
    };
  }
}

class _DecodedTransferEvent {
  const _DecodedTransferEvent({
    required this.fromHex,
    required this.toHex,
    required this.amountFen,
  });

  final String fromHex;
  final String toHex;
  final String amountFen;
}

/// 链上交易监控服务（本机增量流水模式）。
///
/// 中文注释：wuminapp 不查询钱包导入前历史，也不让全节点替手机维护交易索引。
/// 本服务用 newHeads 先写入 inBlock 流水，再按 finalized 游标小步同步
/// System.Events 并升级为 finalized，本地页面只读 Isar 缓存。
class ChainTxMonitor {
  ChainTxMonitor._();
  static final ChainTxMonitor instance = ChainTxMonitor._();

  final ChainEventSubscription _subscription = ChainEventSubscription();
  final ChainRpc _chainRpc = ChainRpc();
  StreamSubscription<ChainEvent>? _listener;
  Future<void>? _syncInflight;
  Timer? _subscriptionRetryTimer;
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

  /// Balances::Transfer (pallet=2, event=2)，仅作为 metadata 解码失败后的兜底。
  static const int _balancesPallet = 2;
  static const int _transferEvent = 2;

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

    // 中文注释：启动后只补 lastSyncedBlock 之后的缺口；没有游标的钱包
    // 以当前 finalized 区块为起点，不回扫导入前历史。
    unawaited(_syncToLatest());
  }

  /// 停止监控。
  void stop() {
    _running = false;
    _subscriptionConnected = false;
    _subscriptionRetryTimer?.cancel();
    _subscriptionRetryTimer = null;
    _listener?.cancel();
    _listener = null;
    _subscription.disconnect();
    debugPrint('[TxMonitor] 交易监控已停止');
  }

  /// 初始化钱包基准游标（导入钱包时可调用）。
  Future<void> initBaselineBalance(String address, String pubkeyHex) async {
    watchWallet(address, pubkeyHex);
    try {
      final latest = await _chainRpc.fetchLatestBlock();
      await LocalTxStore.ensureCursor(
        walletAddress: address,
        walletPubkeyHex: pubkeyHex,
        trackingStartBlock: latest.blockNumber,
        lastSyncedBlock: latest.blockNumber,
      );
      debugPrint('[TxMonitor] 初始化交易记录游标: $address @${latest.blockNumber}');
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
        await _processBlock(blockNumber, status: _TransferEventStatus.inBlock);
        break;
      case ChainEventType.newFinalizedBlock:
        await _syncThrough(blockNumber, missingCursorStartsAt: blockNumber - 1);
        break;
    }
  }

  Future<void> _syncToLatest() async {
    if (_walletAddressByPubkey.isEmpty) return;
    try {
      final latest = await _chainRpc.fetchLatestBlock();
      await _syncThrough(
        latest.blockNumber,
        missingCursorStartsAt: latest.blockNumber,
      );
    } catch (e) {
      debugPrint('[TxMonitor] 启动补同步失败: $e');
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
        // 中文注释：交易流水同步是低优先级后台任务；前台钱包/治理读写繁忙时让路，
        // 游标不推进，下一次新区块或启动补同步会继续补缺口。
        return;
      }

      final ok = await _processBlock(
        block,
        status: _TransferEventStatus.finalized,
      );
      if (!ok) return;

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

  Future<bool> _processBlock(
    int blockNumber, {
    required _TransferEventStatus status,
  }) async {
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

      await _decodeTransferEvents(
        eventsBytes,
        blockNumber,
        blockHashHex,
        status: status,
      );
      return true;
    } catch (e) {
      debugPrint('[TxMonitor] 同步区块 $blockNumber 失败: $e');
      return false;
    }
  }

  /// 解码 System.Events，提取与本机钱包相关的 Balances::Transfer 余额变化。
  Future<void> _decodeTransferEvents(
    Uint8List data,
    int blockNumber,
    String blockHash, {
    required _TransferEventStatus status,
  }) async {
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
          status: status,
        );
      }
      return;
    } catch (e) {
      debugPrint('[TxMonitor] metadata 事件解码失败，使用兜底解析: $e');
    }

    await _decodeTransferEventsFallback(
      data,
      blockNumber,
      blockHash,
      status: status,
    );
  }

  Future<void> _decodeTransferEventsFallback(
    Uint8List data,
    int blockNumber,
    String blockHash, {
    required _TransferEventStatus status,
  }) async {
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
            status: status,
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
    required _TransferEventStatus status,
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
      status: status,
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
      status: status,
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

  int? _skipKnownEventPayload(
    Uint8List data,
    int offset,
    int palletIndex, {
    required int eventIndex,
  }) {
    // 中文注释：metadata 解码正常时不会走到这里；兜底分支只显式跳过
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
    required _TransferEventStatus status,
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
      status: status.storeStatus,
      amountDeltaFen: amountDeltaFen,
      transferAmountFen: transferAmountFen,
      fromAddress: fromAddress,
      toAddress: toAddress,
      counterpartyAddress: counterpartyAddress,
      blockNumber: blockNumber,
      blockHash: blockHash,
      eventIndex: eventRecordIndex,
      extrinsicIndex: extrinsicIndex,
    );

    try {
      final balance = await _chainRpc.fetchBalance(pubkey);
      onBalanceChanged?.call(walletAddress, balance);
    } catch (_) {
      // 中文注释：交易记录已经落库，余额刷新失败不能把钱包余额误写成 0。
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
