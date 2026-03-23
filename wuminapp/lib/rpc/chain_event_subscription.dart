import 'dart:async';
import 'dart:convert';

import 'package:flutter/foundation.dart';
import 'package:web_socket_channel/web_socket_channel.dart';

import 'smoldot_client.dart';

/// 链事件订阅：监听新区块头通知。
///
/// 自动选择模式：
/// - smoldot 轻节点就绪时，通过 smoldot JSON-RPC 订阅（无需外部连接）
/// - 否则回退到 WebSocket 连接远程 RPC 节点（开发调试用）
class ChainEventSubscription {
  final StreamController<ChainEvent> _eventController =
      StreamController<ChainEvent>.broadcast();
  bool _disposed = false;

  // ──── smoldot 模式 ────
  StreamSubscription<dynamic>? _smoldotSub;

  // ──── WebSocket 回退模式 ────
  WebSocketChannel? _channel;
  String? _httpUrl;
  Timer? _reconnectTimer;

  /// 新区块等事件流。
  Stream<ChainEvent> get events => _eventController.stream;

  /// 开始订阅新区块头。
  ///
  /// [httpUrl] 仅在 WebSocket 回退模式下使用（smoldot 模式忽略此参数）。
  void connect(String httpUrl) {
    if (_disposed) return;

    if (SmoldotClientManager.instance.isReady) {
      _connectSmoldot();
    } else {
      _connectWebSocket(httpUrl);
    }
  }

  // ──── smoldot 订阅 ────

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

  // ──── WebSocket 回退 ────

  void _connectWebSocket(String httpUrl) {
    _httpUrl = httpUrl;
    _reconnectTimer?.cancel();
    _reconnectTimer = null;

    final wsUrl = httpUrl
        .replaceFirst('http://', 'ws://')
        .replaceFirst('https://', 'wss://');

    debugPrint('[ChainSub] 使用 WebSocket 回退模式: $wsUrl');

    try {
      _channel = WebSocketChannel.connect(Uri.parse(wsUrl));
    } catch (e) {
      debugPrint('[ChainSub] WebSocket 连接错误: $e');
      _scheduleReconnect();
      return;
    }

    // 订阅新区块头
    _channel!.sink.add(jsonEncode({
      'jsonrpc': '2.0',
      'id': 1,
      'method': 'chain_subscribeNewHeads',
      'params': <dynamic>[],
    }));

    _channel!.stream.listen(
      (message) {
        try {
          final json = jsonDecode(message as String) as Map<String, dynamic>;
          // subscription 通知的结构：{ jsonrpc, method, params: { subscription, result } }
          if (json.containsKey('params')) {
            _eventController.add(ChainEvent.newBlock);
          }
        } catch (e) {
          debugPrint('[ChainSub] WebSocket 消息解析错误: $e');
        }
      },
      onError: (Object e) {
        debugPrint('[ChainSub] WebSocket 流错误: $e');
        _scheduleReconnect();
      },
      onDone: () {
        debugPrint('[ChainSub] WebSocket 流结束，准备重连');
        _scheduleReconnect();
      },
    );
  }

  void _scheduleReconnect() {
    if (_disposed || _httpUrl == null) return;
    _reconnectTimer?.cancel();
    _reconnectTimer = Timer(const Duration(seconds: 3), () {
      if (!_disposed && _httpUrl != null) {
        debugPrint('[ChainSub] WebSocket 重连中...');
        _connectWebSocket(_httpUrl!);
      }
    });
  }

  /// 断开连接并释放资源。
  void disconnect() {
    _disposed = true;

    // 清理 smoldot 订阅
    _smoldotSub?.cancel();
    _smoldotSub = null;

    // 清理 WebSocket 回退
    _reconnectTimer?.cancel();
    _reconnectTimer = null;
    _channel?.sink.close();
    _channel = null;
  }
}

/// 链事件类型。
enum ChainEvent {
  /// 新区块产生。
  newBlock,
}
