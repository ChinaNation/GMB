import 'package:flutter/material.dart';
import 'package:webview_flutter/webview_flutter.dart';

import 'package:citizenapp/8964/services/square_api_client.dart';

/// 首次设备绑定的 Cloudflare Turnstile 验证页；只返回单次 token，不保存浏览数据。
class SquareTurnstilePage extends StatefulWidget {
  const SquareTurnstilePage({super.key, this.baseUrl});

  final String? baseUrl;

  @override
  State<SquareTurnstilePage> createState() => _SquareTurnstilePageState();
}

class _SquareTurnstilePageState extends State<SquareTurnstilePage> {
  late final WebViewController _controller;

  @override
  void initState() {
    super.initState();
    final base = widget.baseUrl ?? SquareApiClient.defaultBaseUrl;
    _controller = WebViewController()
      ..setJavaScriptMode(JavaScriptMode.unrestricted)
      ..setBackgroundColor(Colors.white)
      ..addJavaScriptChannel(
        'Turnstile',
        onMessageReceived: (message) {
          final token = message.message.trim();
          if (token.isNotEmpty && mounted) Navigator.of(context).pop(token);
        },
      )
      ..loadRequest(Uri.parse('$base/v1/security/turnstile'));
  }

  @override
  Widget build(BuildContext context) {
    return Scaffold(
      appBar: AppBar(title: const Text('设备安全验证')),
      body: SafeArea(child: WebViewWidget(controller: _controller)),
    );
  }
}
