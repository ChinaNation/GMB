import 'dart:async';

import 'package:flutter/foundation.dart';

import 'smoldot_client.dart';

/// 链事件订阅：监听新区块头通知。
///
/// 只通过 smoldot 轻节点订阅（无需外部 WebSocket / HTTP RPC）。
class ChainEventSubscription {
  final StreamController<ChainEvent> _eventController =
      StreamController<ChainEvent>.broadcast();
  bool _disposed = false;

  StreamSubscription<dynamic>? _smoldotSub;

  /// 新区块等事件流。
  Stream<ChainEvent> get events => _eventController.stream;

  /// 开始订阅 finalized 区块头（确保事件已最终确认）。
  void connect() {
    if (_disposed) return;

    if (!SmoldotClientManager.instance.isReady) {
      debugPrint('[ChainSub] smoldot 尚未就绪，跳过区块订阅');
      return;
    }

    _connectSmoldot();
  }

  void _connectSmoldot() {
    debugPrint('[ChainSub] 使用 smoldot 轻节点订阅 finalized 区块');
    try {
      final stream = SmoldotClientManager.instance
          .subscribe('chain_subscribeFinalizedHeads', []);
      _smoldotSub = stream.listen(
        (data) {
          if (_disposed) return;
          // 中文注释：解析区块头中的 number 字段（hex 编码）。
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
            type: ChainEventType.newFinalizedBlock,
            blockNumber: blockNumber,
          ));
        },
        onError: (Object e) {
          debugPrint('[ChainSub] smoldot 订阅错误: $e');
        },
        onDone: () {
          debugPrint('[ChainSub] smoldot 订阅结束');
        },
      );
    } catch (e) {
      debugPrint('[ChainSub] smoldot 订阅启动失败: $e');
    }
  }

  /// 断开连接并释放资源。
  void disconnect() {
    _disposed = true;
    _smoldotSub?.cancel();
    _smoldotSub = null;
  }
}

/// 链事件类型。
enum ChainEventType {
  /// 新 finalized 区块。
  newFinalizedBlock,
}

/// 链事件。
class ChainEvent {
  const ChainEvent({required this.type, this.blockNumber});

  final ChainEventType type;
  final int? blockNumber;

  /// 向后兼容的静态常量。
  static const newBlock = ChainEvent(type: ChainEventType.newFinalizedBlock);
}
