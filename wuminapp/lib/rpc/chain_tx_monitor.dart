import 'dart:async';
import 'dart:typed_data';

import 'package:flutter/foundation.dart';
import 'package:polkadart/polkadart.dart' show Hasher;
import 'package:polkadart_keyring/polkadart_keyring.dart' show Keyring;
import 'package:shared_preferences/shared_preferences.dart';

import 'chain_event_subscription.dart';
import 'chain_rpc.dart';
import 'smoldot_client.dart';
import 'package:wuminapp_mobile/Isar/wallet_isar.dart';
import 'package:wuminapp_mobile/trade/local_tx_store.dart';

/// 链上交易监控服务（余额变化触发模式）。
///
/// 订阅 finalized 区块，每个新区块仅查一次余额（80 字节），
/// 余额变化时才查 System.Events 获取交易明细，极大降低全节点负担。
class ChainTxMonitor {
  ChainTxMonitor._();
  static final ChainTxMonitor instance = ChainTxMonitor._();

  final ChainEventSubscription _subscription = ChainEventSubscription();
  final ChainRpc _chainRpc = ChainRpc();
  StreamSubscription<ChainEvent>? _listener;
  bool _running = false;

  /// 当前监控的钱包：address → pubkeyHex（小写，不含 0x）。
  final Map<String, String> _watchedWallets = {};

  /// 余额变动回调：当检测到余额变化（写入新交易记录后）通知外部刷新。
  void Function(String walletAddress, double newBalance)? onBalanceChanged;

  /// 本地基准余额缓存：pubkeyHex → 最近一次已知余额（yuan）。
  final Map<String, double> _knownBalances = {};

  /// SS58 前缀。
  static const int _ss58Prefix = 2027;

  /// SharedPreferences 键前缀：存储每个钱包的基准余额。
  static const String _balancePrefix = 'tx_monitor_balance_';

  // ──── 已知事件的 pallet_index + event_index ────

  /// Balances::Transfer (pallet=2, event=2)
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
    final pk = pubkeyHex.toLowerCase().replaceFirst('0x', '');
    _watchedWallets[address] = pk;
  }

  /// 移除监控钱包。
  void unwatchWallet(String address) {
    _watchedWallets.remove(address);
  }

  /// 启动监控。
  Future<void> start() async {
    if (_running) return;
    _running = true;

    // 加载所有监控钱包的基准余额
    await _loadKnownBalances();

    _subscription.connect();
    _listener = _subscription.events.listen(_onEvent);
    debugPrint('[TxMonitor] 交易监控已启动（余额变化模式），监控 ${_watchedWallets.length} 个钱包');
  }

  /// 停止监控。
  void stop() {
    _running = false;
    _listener?.cancel();
    _listener = null;
    _subscription.disconnect();
    debugPrint('[TxMonitor] 交易监控已停止');
  }

  /// 初始化钱包基准余额（导入钱包时调用）。
  ///
  /// 查询一次链上余额，存储为基准值。今后只有余额变化时才触发查询。
  Future<void> initBaselineBalance(String address, String pubkeyHex) async {
    final pk = pubkeyHex.toLowerCase().replaceFirst('0x', '');
    try {
      final balance = await _chainRpc.fetchBalance(pubkeyHex);
      _knownBalances[pk] = balance;
      await _saveBalance(address, balance);
      debugPrint('[TxMonitor] 初始化基准余额: $address → $balance 元');
    } catch (e) {
      debugPrint('[TxMonitor] 初始化基准余额失败: $e');
    }
  }

  // ──── 内部：余额变化检测 ────

  /// 处理新 finalized 区块事件。
  Future<void> _onEvent(ChainEvent event) async {
    if (!_running || _watchedWallets.isEmpty) return;
    if (event.blockNumber == null) return;

    try {
      await _checkBalanceChanges(event.blockNumber!);
    } catch (e) {
      debugPrint('[TxMonitor] 处理区块 ${event.blockNumber} 失败: $e');
    }
  }

  /// 批量查询所有监控钱包的余额，检测变化。
  Future<void> _checkBalanceChanges(int blockNumber) async {
    if (_watchedWallets.isEmpty) return;

    // 批量查询当前余额（单次 RPC）
    final pubkeys = _watchedWallets.values.toList();
    final Map<String, double> currentBalances;
    try {
      currentBalances = await _chainRpc.fetchBalances(pubkeys);
    } catch (e) {
      debugPrint('[TxMonitor] 批量余额查询失败: $e');
      return;
    }

    // 逐个钱包对比
    for (final entry in _watchedWallets.entries) {
      final address = entry.key;
      final pk = entry.value;
      final currentBalance = currentBalances[pk] ?? 0.0;
      final knownBalance = _knownBalances[pk];

      if (knownBalance == null) {
        // 首次：记录基准余额，不查 Events
        _knownBalances[pk] = currentBalance;
        await _saveBalance(address, currentBalance);
        continue;
      }

      // 余额未变化：什么都不做
      if ((currentBalance - knownBalance).abs() < 0.001) continue;

      debugPrint(
        '[TxMonitor] 余额变化: $address $knownBalance → $currentBalance (block $blockNumber)',
      );

      // 余额增加 → 可能有收入，查 Events 获取明细
      if (currentBalance > knownBalance) {
        await _queryBlockEvents(blockNumber, address, pk);
      }

      // 更新基准余额
      _knownBalances[pk] = currentBalance;
      await _saveBalance(address, currentBalance);

      // 通知外部余额变动
      onBalanceChanged?.call(address, currentBalance);
    }
  }

  // ──── 内部：Events 查询（仅在余额变化时触发） ────

  /// 查询指定区块的 System.Events，提取与目标钱包相关的收入交易。
  Future<void> _queryBlockEvents(
    int blockNumber,
    String walletAddress,
    String pubkeyHex,
  ) async {
    try {
      final blockHashHex =
          await SmoldotClientManager.instance.getBlockHash(blockNumber);
      if (blockHashHex == null) return;

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

      await _decodeTransferEvents(
        eventsBytes,
        blockNumber,
        walletAddress,
        pubkeyHex,
      );
    } catch (e) {
      debugPrint('[TxMonitor] 查询区块 $blockNumber Events 失败: $e');
    }
  }

  /// 解码 System.Events，仅提取 Balances::Transfer 收入事件。
  Future<void> _decodeTransferEvents(
    Uint8List data,
    int blockNumber,
    String walletAddress,
    String targetPubkey,
  ) async {
    var offset = 0;
    if (data.isEmpty) return;
    final (_, countSize) = _decodeCompactU32(data, 0);
    offset += countSize;

    while (offset + 4 < data.length) {
      final phase = data[offset];
      offset += 1;
      if (phase == 0x00) {
        offset += 4; // 跳过 ApplyExtrinsic u32
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
          final amountFen = _readU128LE(amountBytes, 0);
          final amountYuan = amountFen.toDouble() / 100.0;

          // 收入：目标钱包是收款方
          if (toHex == targetPubkey) {
            await _writeTx(
              walletAddress: walletAddress,
              txId: 'block-$blockNumber-transfer-$fromHex-$toHex',
              txType: 'transfer',
              direction: 'in',
              fromAddress: _pubkeyToSs58(from),
              toAddress: walletAddress,
              amountYuan: amountYuan,
              blockNumber: blockNumber,
              status: 'confirmed',
            );
          }

          // 支出：目标钱包是付款方（补充链上确认记录）
          if (fromHex == targetPubkey) {
            await _writeTx(
              walletAddress: walletAddress,
              txId: 'block-$blockNumber-transfer-$fromHex-$toHex',
              txType: 'transfer',
              direction: 'out',
              fromAddress: walletAddress,
              toAddress: _pubkeyToSs58(to),
              amountYuan: amountYuan,
              blockNumber: blockNumber,
              status: 'confirmed',
            );
          }

          offset = _skipTopics(data, offset);
          continue;
        }
      }

      // 未识别事件：尝试跳到下一个 EventRecord
      offset = _skipToNextEvent(data, offset);
    }
  }

  // ──── 基准余额持久化 ────

  /// 加载所有监控钱包的基准余额。
  Future<void> _loadKnownBalances() async {
    final prefs = await SharedPreferences.getInstance();
    for (final entry in _watchedWallets.entries) {
      final stored = prefs.getDouble('$_balancePrefix${entry.key}');
      if (stored != null) {
        _knownBalances[entry.value] = stored;
      }
    }
  }

  /// 保存单个钱包的基准余额。
  Future<void> _saveBalance(String address, double balance) async {
    final prefs = await SharedPreferences.getInstance();
    await prefs.setDouble('$_balancePrefix$address', balance);
  }

  // ──── 写入本地交易记录 ────

  /// 写入本地交易记录（防重复）。
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
