import 'dart:async';
import 'dart:convert';

import 'package:flutter/foundation.dart';
import 'package:web_socket_channel/web_socket_channel.dart';

/// 链节点 WebSocket 事件订阅。
///
/// 订阅新区块头通知，连接断开时自动重连。
class ChainEventSubscription {
  WebSocketChannel? _channel;
  final StreamController<ChainEvent> _eventController =
      StreamController<ChainEvent>.broadcast();
  String? _httpUrl;
  bool _disposed = false;
  Timer? _reconnectTimer;

  /// 新区块等事件流。
  Stream<ChainEvent> get events => _eventController.stream;

  /// 连接到链节点 WebSocket（将 http:// 转为 ws://）。
  void connect(String httpUrl) {
    if (_disposed) return;
    _httpUrl = httpUrl;
    _reconnectTimer?.cancel();
    _reconnectTimer = null;

    final wsUrl = httpUrl
        .replaceFirst('http://', 'ws://')
        .replaceFirst('https://', 'wss://');

    try {
      _channel = WebSocketChannel.connect(Uri.parse(wsUrl));
    } catch (e) {
      debugPrint('[ChainWS] connect error: $e');
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
          _handleMessage(json);
        } catch (e) {
          debugPrint('[ChainWS] message parse error: $e');
        }
      },
      onError: (Object e) {
        debugPrint('[ChainWS] stream error: $e');
        _scheduleReconnect();
      },
      onDone: () {
        debugPrint('[ChainWS] stream done, will reconnect');
        _scheduleReconnect();
      },
    );
  }

  void _handleMessage(Map<String, dynamic> json) {
    // subscription 通知的结构：{ jsonrpc, method, params: { subscription, result } }
    if (json.containsKey('params')) {
      _eventController.add(ChainEvent.newBlock);
    }
  }

  void _scheduleReconnect() {
    if (_disposed || _httpUrl == null) return;
    _reconnectTimer?.cancel();
    _reconnectTimer = Timer(const Duration(seconds: 3), () {
      if (!_disposed && _httpUrl != null) {
        debugPrint('[ChainWS] reconnecting...');
        connect(_httpUrl!);
      }
    });
  }

  /// 断开连接并释放资源。
  void disconnect() {
    _disposed = true;
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
