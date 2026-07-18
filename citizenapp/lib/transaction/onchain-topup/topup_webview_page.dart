import 'dart:convert';

import 'package:flutter/material.dart';
import 'package:url_launcher/url_launcher.dart';
import 'package:webview_flutter/webview_flutter.dart';

import 'package:citizenapp/ui/app_theme.dart';
import 'topup_erc20.dart';
import 'topup_models.dart';

/// WebView WalletConnect 返回结果:付款交易哈希 + 付款地址。
class TopupWebResult {
  const TopupWebResult({required this.txHash, this.payerAddress});

  final String txHash;
  final String? payerAddress;
}

/// WalletConnect 支付页(方案 A):在 WebView 内加载打包的 AppKit JS 页,连自托管钱包并发
/// ERC-20 转账。App 只把「币+链+收款地址+应付额」交给页面,拿回 txHash;不引 reown Dart SDK
/// (与 flutter_secure_storage 10 / flutter_chat_core 冲突),故走 webview 里的 JS SDK。
class TopupWebviewPage extends StatefulWidget {
  const TopupWebviewPage({
    super.key,
    required this.rail,
    required this.package,
    required this.recvAddress,
    required this.gmbAddress,
  });

  final TopupRail rail;
  final TopupPackage package;
  final String recvAddress;
  final String gmbAddress;

  /// WalletConnect Project ID(公开标识,非私钥),编译期注入。
  static const projectId = String.fromEnvironment('WALLETCONNECT_PROJECT_ID');

  @override
  State<TopupWebviewPage> createState() => _TopupWebviewPageState();
}

class _TopupWebviewPageState extends State<TopupWebviewPage> {
  late final WebViewController _controller;
  String? _error;

  @override
  void initState() {
    super.initState();
    _controller = WebViewController()
      ..setJavaScriptMode(JavaScriptMode.unrestricted)
      ..addJavaScriptChannel('TopupBridge', onMessageReceived: _onBridgeMessage)
      ..setNavigationDelegate(NavigationDelegate(
        onNavigationRequest: _onNavigation,
        onPageFinished: (_) => _injectAndStart(),
      ))
      ..loadFlutterAsset('assets/topup/walletconnect.html');
  }

  /// 钱包深链(wc: / metamask: 等非 http(s))交给系统唤起对应钱包 App,阻止 WebView 内部导航。
  NavigationDecision _onNavigation(NavigationRequest request) {
    final uri = Uri.tryParse(request.url);
    if (uri != null && uri.scheme != 'http' && uri.scheme != 'https' && uri.scheme != 'about') {
      launchUrl(uri, mode: LaunchMode.externalApplication);
      return NavigationDecision.prevent;
    }
    return NavigationDecision.navigate;
  }

  Future<void> _injectAndStart() async {
    if (TopupWebviewPage.projectId.isEmpty) {
      setState(() => _error = 'WalletConnect 未配置（缺少 Project ID）');
      return;
    }
    // 收款金额与 ERC-20 calldata 在 Dart 侧构造(复用已验证的编码器),页面只负责签发。
    final data = encodeErc20Transfer(widget.recvAddress, widget.package.payAmountValue);
    final params = jsonEncode({
      'projectId': TopupWebviewPage.projectId,
      'caip2': widget.rail.caip2,
      'chainId': widget.rail.chainId,
      'to': widget.rail.tokenContract,
      'data': data,
      'token': widget.rail.token,
      'label': widget.rail.label,
      'payDisplay': widget.package.payDisplay,
    });
    await _controller.runJavaScript(
      'window.__TOPUP__=$params; if(window.startTopup){window.startTopup();}',
    );
  }

  void _onBridgeMessage(JavaScriptMessage message) {
    Map<String, dynamic> payload;
    try {
      payload = jsonDecode(message.message) as Map<String, dynamic>;
    } catch (_) {
      return;
    }
    if (payload['ok'] == true) {
      final txHash = payload['txHash']?.toString() ?? '';
      if (txHash.isEmpty) return;
      Navigator.of(context).pop(TopupWebResult(
        txHash: txHash,
        payerAddress: payload['payer']?.toString(),
      ));
      return;
    }
    final error = payload['error']?.toString() ?? '支付未完成';
    if (!mounted) return;
    ScaffoldMessenger.of(context).showSnackBar(SnackBar(content: Text(error)));
  }

  @override
  Widget build(BuildContext context) {
    return Scaffold(
      appBar: AppBar(title: const Text('连接钱包支付'), centerTitle: true),
      body: _error != null
          ? Center(
              child: Padding(
                padding: const EdgeInsets.all(24),
                child: Text(_error!,
                    textAlign: TextAlign.center,
                    style: const TextStyle(color: AppTheme.danger)),
              ),
            )
          : WebViewWidget(controller: _controller),
    );
  }
}
