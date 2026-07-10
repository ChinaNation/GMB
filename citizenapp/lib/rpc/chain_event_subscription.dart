import 'dart:async';
import 'dart:io' show Platform;

import 'package:flutter/foundation.dart';

import 'smoldot_client.dart';

/// 链事件订阅：监听新区块头通知。
///
/// 只通过 smoldot 轻节点订阅（无需外部 WebSocket / HTTP RPC）。
class ChainEventSubscription {
  ChainEventSubscription({SmoldotClientManager? smoldotClientManager})
      : _smoldotClientManager =
            smoldotClientManager ?? SmoldotClientManager.instance;

  final SmoldotClientManager _smoldotClientManager;
  final StreamController<ChainEvent> _eventController =
      StreamController<ChainEvent>.broadcast();

  StreamSubscription<dynamic>? _newHeadsSub;
  StreamSubscription<dynamic>? _finalizedHeadsSub;
  int _lifecycleGeneration = 0;

  /// 新区块等事件流。
  Stream<ChainEvent> get events => _eventController.stream;

  /// 开始订阅新区块头和 finalized 区块头。
  ///
  /// (ADR-017)：业务流水只由 finalizedHeads 驱动(ChainTxMonitor
  /// 只扫 finalized 链)；newHeads 不参与流水状态，仅供交易提交 watch
  /// (豁免区)做 UI 进度提示。底层异步流会统一启动并等待轻节点同步；
  /// 返回值表示轻节点已同步且两条本地订阅都已创建；初始化或同步失败时返回 false。
  Future<bool> connect() async {
    if (Platform.environment.containsKey('FLUTTER_TEST') &&
        identical(_smoldotClientManager, SmoldotClientManager.instance)) {
      // Widget/unit test 没有 APK 内的原生库；正式连接行为由注入测试 manager
      // 的生命周期测试和 Android profile 真机共同覆盖。
      return false;
    }
    final generation = _lifecycleGeneration;
    try {
      await _smoldotClientManager.ensureSynced();
    } catch (e) {
      debugPrint('[ChainSub] 轻节点尚未就绪，订阅连接失败: $e');
      return false;
    }
    if (generation != _lifecycleGeneration) return false;

    final newHeadsOk = _connectSmoldot(
      method: 'chain_subscribeNewHeads',
      type: ChainEventType.newBlock,
      logLabel: 'newHeads',
    );
    final finalizedOk = _connectSmoldot(
      method: 'chain_subscribeFinalizedHeads',
      type: ChainEventType.newFinalizedBlock,
      logLabel: 'finalizedHeads',
    );
    return newHeadsOk && finalizedOk;
  }

  bool _connectSmoldot({
    required String method,
    required ChainEventType type,
    required String logLabel,
  }) {
    if (type == ChainEventType.newBlock && _newHeadsSub != null) return true;
    if (type == ChainEventType.newFinalizedBlock &&
        _finalizedHeadsSub != null) {
      return true;
    }

    debugPrint('[ChainSub] 使用 smoldot 轻节点订阅 $logLabel');
    try {
      final stream = _smoldotClientManager.subscribe(method, []);
      final sub = stream.listen(
        (data) {
          // 解析区块头中的 number 字段（hex 编码）。
          int? blockNumber;
          if (data is Map) {
            final numHex = data['number'];
            if (numHex is String) {
              blockNumber = int.tryParse(
                numHex.startsWith('0x') ? numHex.substring(2) : numHex,
                radix: 16,
              );
            }
          }
          _eventController.add(ChainEvent(
            type: type,
            blockNumber: blockNumber,
          ));
        },
        onError: (Object e) {
          debugPrint('[ChainSub] $logLabel 订阅错误: $e');
        },
        onDone: () {
          debugPrint('[ChainSub] $logLabel 订阅结束');
          if (type == ChainEventType.newBlock) {
            _newHeadsSub = null;
          } else {
            _finalizedHeadsSub = null;
          }
        },
      );
      if (type == ChainEventType.newBlock) {
        _newHeadsSub = sub;
      } else {
        _finalizedHeadsSub = sub;
      }
      return true;
    } catch (e) {
      debugPrint('[ChainSub] $logLabel 订阅启动失败: $e');
      return false;
    }
  }

  /// 断开连接并释放资源。
  void disconnect() {
    _lifecycleGeneration += 1;
    _newHeadsSub?.cancel();
    _finalizedHeadsSub?.cancel();
    _newHeadsSub = null;
    _finalizedHeadsSub = null;
  }
}

/// 链事件类型。
enum ChainEventType {
  /// 新出块。
  newBlock,

  /// 新 finalized 区块。
  newFinalizedBlock,
}

/// 链事件。
class ChainEvent {
  const ChainEvent({required this.type, this.blockNumber});

  final ChainEventType type;
  final int? blockNumber;
}
