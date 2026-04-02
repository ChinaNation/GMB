import 'dart:async';
import 'dart:typed_data';

import 'package:flutter/foundation.dart';
import 'package:polkadart/polkadart.dart' show Hasher;
import 'package:polkadart_keyring/polkadart_keyring.dart' show Keyring;

import 'chain_event_subscription.dart';
import 'smoldot_client.dart';
import 'package:wuminapp_mobile/isar/wallet_isar.dart';
import 'package:wuminapp_mobile/trade/local_tx_store.dart';

/// 链上交易监控服务。
///
/// 订阅 finalized 区块，扫描 System.Events，
/// 筛选当前钱包相关事件，自动写入本地交易记录。
class ChainTxMonitor {
  ChainTxMonitor._();
  static final ChainTxMonitor instance = ChainTxMonitor._();

  final ChainEventSubscription _subscription = ChainEventSubscription();
  StreamSubscription<ChainEvent>? _listener;
  bool _running = false;

  /// 当前监控的钱包地址列表（SS58）和对应公钥（32 字节 hex）。
  final Map<String, String> _watchedWallets = {}; // address → pubkeyHex

  /// SS58 前缀。
  static const int _ss58Prefix = 2027;

  // ──── 已知事件的 pallet_index + event_index ────

  /// Balances::Transfer (pallet=2, event=2)
  static const int _balancesPallet = 2;
  static const int _transferEvent = 2;

  /// OnchainTransactionPow::FeePaid (pallet=4, event=0)
  static const int _feePallet = 4;
  static const int _feePaidEvent = 0;

  // TODO: FullnodePowReward (pallet=6) 出块奖励事件解码待后续实现。

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

  /// 中文注释：添加监控钱包。
  void watchWallet(String address, String pubkeyHex) {
    _watchedWallets[address] = pubkeyHex.toLowerCase();
  }

  /// 中文注释：移除监控钱包。
  void unwatchWallet(String address) {
    _watchedWallets.remove(address);
  }

  /// 中文注释：启动监控。
  void start() {
    if (_running) return;
    _running = true;
    _subscription.connect();
    _listener = _subscription.events.listen(_onEvent);
    debugPrint('[TxMonitor] 交易监控已启动，监控 ${_watchedWallets.length} 个钱包');
  }

  /// 中文注释：停止监控。
  void stop() {
    _running = false;
    _listener?.cancel();
    _listener = null;
    _subscription.disconnect();
    debugPrint('[TxMonitor] 交易监控已停止');
  }

  /// 中文注释：处理新 finalized 区块事件。
  Future<void> _onEvent(ChainEvent event) async {
    if (!_running || _watchedWallets.isEmpty) return;
    if (event.blockNumber == null) return;

    try {
      await _processBlock(event.blockNumber!);
    } catch (e) {
      debugPrint('[TxMonitor] 处理区块 ${event.blockNumber} 失败: $e');
    }
  }

  /// 中文注释：处理单个区块，读取事件并筛选。
  Future<void> _processBlock(int blockNumber) async {
    // 获取区块哈希
    final blockHashHex = await SmoldotClientManager.instance.getBlockHash(blockNumber);
    if (blockHashHex == null) return;

    // 中文注释：使用 state_getStorage 在指定区块哈希下读取 System.Events。
    // System.Events 每个区块被覆盖，必须指定 blockHash 才能读到该区块的事件。
    final keyHex = '0x${_hexEncode(_eventsStorageKey)}';
    final result = await SmoldotClientManager.instance.request(
      'state_getStorage',
      [keyHex, blockHashHex],
    );
    final eventsHex = result as String?;
    if (eventsHex == null) return;

    final eventsBytes = _hexDecode(
      eventsHex.startsWith('0x') ? eventsHex.substring(2) : eventsHex,
    );
    if (eventsBytes.isEmpty) return;

    // 中文注释：解码事件列表，筛选相关事件。
    await _decodeAndFilterEvents(eventsBytes, blockNumber);
  }

  /// 中文注释：解码 System.Events 并筛选与监控钱包相关的事件。
  ///
  /// System.Events 格式：Compact<u32> count + N 个 EventRecord。
  /// EventRecord：phase(1+) + Event(pallet_index + event_index + data) + topics(Vec<Hash>)
  /// 简化处理：扫描字节流中已知的 pallet_index + event_index 模式。
  Future<void> _decodeAndFilterEvents(Uint8List data, int blockNumber) async {
    // 中文注释：简化解码——扫描字节流寻找已知事件模式。
    // 完整解码需要 metadata，这里用模式匹配识别关键事件。
    var offset = 0;
    // 跳过 Compact<u32> count
    if (data.isEmpty) return;
    final (_, countSize) = _decodeCompactU32(data, 0);
    offset += countSize;

    while (offset + 4 < data.length) {
      // 每个 EventRecord 以 phase 开头
      // ApplyExtrinsic(u32) = 0x00 + 4 bytes
      // Finalization = 0x01
      // Initialization = 0x02
      final phase = data[offset];
      offset += 1;
      if (phase == 0x00) {
        // ApplyExtrinsic: 跳过 u32 extrinsic index
        offset += 4;
      }

      if (offset + 2 > data.length) break;
      final palletIndex = data[offset];
      final eventIndex = data[offset + 1];
      offset += 2;

      // 中文注释：尝试识别已知事件。
      if (palletIndex == _balancesPallet && eventIndex == _transferEvent) {
        // Balances::Transfer { from: AccountId, to: AccountId, amount: u128 }
        if (offset + 32 + 32 + 16 <= data.length) {
          final from = data.sublist(offset, offset + 32);
          final to = data.sublist(offset + 32, offset + 64);
          final amountBytes = data.sublist(offset + 64, offset + 80);
          offset += 80;

          final fromHex = _hexEncode(from);
          final toHex = _hexEncode(to);
          final amountFen = _readU128LE(amountBytes, 0);
          final amountYuan = amountFen.toDouble() / 100.0;

          // 中文注释：检查是否与监控钱包相关。
          for (final entry in _watchedWallets.entries) {
            final pubkey = entry.value;
            if (fromHex == pubkey) {
              await _writeTx(
                walletAddress: entry.key,
                txId: 'block-$blockNumber-transfer-$fromHex-$toHex',
                txType: 'transfer',
                direction: 'out',
                fromAddress: entry.key,
                toAddress: _pubkeyToSs58(to),
                amountYuan: amountYuan,
                blockNumber: blockNumber,
                status: 'confirmed',
              );
            } else if (toHex == pubkey) {
              await _writeTx(
                walletAddress: entry.key,
                txId: 'block-$blockNumber-transfer-$fromHex-$toHex',
                txType: 'transfer',
                direction: 'in',
                fromAddress: _pubkeyToSs58(from),
                toAddress: entry.key,
                amountYuan: amountYuan,
                blockNumber: blockNumber,
                status: 'confirmed',
              );
            }
          }
          // 跳过 topics
          _skipTopics(data, offset);
          continue;
        }
      }

      if (palletIndex == _feePallet && eventIndex == _feePaidEvent) {
        // OnchainTransactionPow::FeePaid { who: AccountId, fee: u128 }
        if (offset + 32 + 16 <= data.length) {
          final who = data.sublist(offset, offset + 32);
          final feeBytes = data.sublist(offset + 32, offset + 48);
          offset += 48;

          final whoHex = _hexEncode(who);
          final feeYuan = _readU128LE(feeBytes, 0).toDouble() / 100.0;

          for (final entry in _watchedWallets.entries) {
            if (entry.value == whoHex) {
              await _writeTx(
                walletAddress: entry.key,
                txId: 'block-$blockNumber-fee-$whoHex',
                txType: 'fee_withdraw',
                direction: 'out',
                fromAddress: entry.key,
                amountYuan: feeYuan,
                blockNumber: blockNumber,
                status: 'confirmed',
              );
            }
          }
          continue;
        }
      }

      // 中文注释：未识别的事件，跳过剩余字节到下一个 EventRecord。
      // 由于不知道事件数据长度，无法精确跳过，后续事件可能解析失败。
      // 这是简化解码的局限，生产环境应使用 metadata 完整解码。
      break;
    }
  }

  /// 中文注释：写入本地交易记录（防重复）。
  Future<void> _writeTx({
    required String walletAddress,
    required String txId,
    required String txType,
    required String direction,
    String? fromAddress,
    String? toAddress,
    required double amountYuan,
    double? feeYuan,
    int? blockNumber,
    required String status,
  }) async {
    // 防重复
    final existing = await LocalTxStore.queryByTxId(txId);
    if (existing != null) return;

    final entity = LocalTxEntity()
      ..txId = txId
      ..walletAddress = walletAddress
      ..txType = txType
      ..direction = direction
      ..fromAddress = fromAddress
      ..toAddress = toAddress
      ..amountYuan = amountYuan
      ..feeYuan = feeYuan
      ..status = status
      ..blockNumber = blockNumber
      ..createdAtMillis = DateTime.now().millisecondsSinceEpoch
      ..confirmedAtMillis = DateTime.now().millisecondsSinceEpoch;
    await LocalTxStore.insert(entity);
    debugPrint('[TxMonitor] 写入交易 $txType $direction $amountYuan 元 (block $blockNumber)');
  }

  /// 中文注释：同步历史交易记录（扫描最近 N 个区块）。
  ///
  /// [walletAddress] 钱包地址。
  /// [pubkeyHex] 钱包公钥 hex。
  /// [recentBlocks] 扫描最近多少个区块（默认 100）。
  /// [onProgress] 进度回调（已扫描块数, 总块数）。
  Future<int> syncHistory({
    required String walletAddress,
    required String pubkeyHex,
    int recentBlocks = 100,
    void Function(int scanned, int total)? onProgress,
  }) async {
    final snapshot = await SmoldotClientManager.instance.getStatusSnapshot();
    if (snapshot == null || snapshot.finalizedBlockNumber == null) {
      throw Exception('节点未就绪');
    }
    final currentBlock = snapshot.finalizedBlockNumber!;
    final startBlock = (currentBlock - recentBlocks + 1).clamp(1, currentBlock);
    final total = currentBlock - startBlock + 1;

    // 中文注释：临时只监控这一个钱包。
    final savedWallets = Map<String, String>.from(_watchedWallets);
    _watchedWallets.clear();
    _watchedWallets[walletAddress] = pubkeyHex.toLowerCase();

    int found = 0;
    try {
      for (var i = currentBlock; i >= startBlock; i--) {
        await _processBlock(i);
        final scanned = currentBlock - i + 1;
        onProgress?.call(scanned, total);
        // 中文注释：每 10 个区块 yield 一下，避免阻塞 UI。
        if (scanned % 10 == 0) {
          await Future.delayed(Duration.zero);
        }
      }
    } finally {
      _watchedWallets
        ..clear()
        ..addAll(savedWallets);
    }

    return found;
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

  /// 中文注释：跳过 topics（Vec<Hash>）。
  static int _skipTopics(Uint8List data, int offset) {
    if (offset >= data.length) return offset;
    final (count, size) = _decodeCompactU32(data, offset);
    offset += size;
    offset += count * 32; // 每个 topic 是 32 字节 Hash
    return offset;
  }
}
