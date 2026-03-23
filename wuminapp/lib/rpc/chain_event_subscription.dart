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

  /// 开始订阅新区块头。
  void connect() {
    if (_disposed) return;

    if (!SmoldotClientManager.instance.isReady) {
      debugPrint('[ChainSub] smoldot 尚未就绪，跳过新区块订阅');
      return;
    }

    _connectSmoldot();
  }

  void _connectSmoldot() {
    debugPrint('[ChainSub] 使用 smoldot 轻节点订阅新区块');
    try {
      final stream = SmoldotClientManager.instance
          .subscribe('chain_subscribeNewHeads', []);
      _smoldotSub = stream.listen(
        (_) {
          if (!_disposed) {
            _eventController.add(ChainEvent.newBlock);
          }
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
enum ChainEvent {
  /// 新区块产生。
  newBlock,
}
