import 'dart:async';

import 'package:flutter/material.dart';
import 'package:mobile_scanner/mobile_scanner.dart';
import 'package:qr_flutter/qr_flutter.dart';
import 'package:wuminapp_mobile/signer/qr_signer.dart';

/// 冷钱包扫码签名会话页面。
///
/// 两阶段交互：
/// 1. 展示签名请求二维码，等待离线设备扫描。
/// 2. 用户点击"扫描回执"，打开相机扫描离线设备生成的签名回执二维码。
///
/// 返回 [QrSignResponse]（成功）或 `null`（取消/超时）。
class QrSignSessionPage extends StatefulWidget {
  const QrSignSessionPage({
    super.key,
    required this.request,
    required this.requestJson,
    required this.expectedPubkey,
  });

  /// 已构建的签名请求。
  final QrSignRequest request;

  /// 编码后的 JSON 字符串，直接用于二维码展示。
  final String requestJson;
  final String expectedPubkey;

  @override
  State<QrSignSessionPage> createState() => _QrSignSessionPageState();
}

class _QrSignSessionPageState extends State<QrSignSessionPage> {
  Timer? _timer;
  late int _remainingSeconds;

  @override
  void initState() {
    super.initState();
    _remainingSeconds = _secondsLeft();
    _timer = Timer.periodic(const Duration(seconds: 1), (_) {
      if (!mounted) return;
      setState(() {
        _remainingSeconds = _secondsLeft();
      });
    });
  }

  @override
  void dispose() {
    _timer?.cancel();
    super.dispose();
  }

  int _secondsLeft() {
    final now = DateTime.now().millisecondsSinceEpoch ~/ 1000;
    final left = widget.request.expiresAt - now;
    return left > 0 ? left : 0;
  }

  Future<void> _scanResponse() async {
    final raw = await Navigator.of(context).push<String>(
      MaterialPageRoute(builder: (_) => const _SimpleScanner()),
    );
    if (raw == null || !mounted) return;

    try {
      final expectedHash =
          QrSigner.computePayloadHash(widget.request.payloadHex);
      final response = QrSigner().parseResponse(
        raw,
        expectedRequestId: widget.request.requestId,
        expectedPubkey: widget.expectedPubkey,
        expectedPayloadHash: expectedHash,
      );
      if (!mounted) return;
      Navigator.of(context).pop(response);
    } on QrSignException catch (e) {
      if (!mounted) return;
      await showDialog<void>(
        context: context,
        builder: (context) => AlertDialog(
          title: const Text('签名回执解析失败'),
          content: Text(e.message),
          actions: [
            TextButton(
              onPressed: () => Navigator.of(context).pop(),
              child: const Text('确定'),
            ),
          ],
        ),
      );
    }
  }

  @override
  Widget build(BuildContext context) {
    final expired = _remainingSeconds <= 0;
    return Scaffold(
      appBar: AppBar(
        title: const Text('冷钱包签名'),
        centerTitle: true,
      ),
      body: ListView(
        padding: const EdgeInsets.all(16),
        children: [
          // 倒计时状态栏
          Container(
            width: double.infinity,
            padding: const EdgeInsets.symmetric(horizontal: 10, vertical: 6),
            decoration: BoxDecoration(
              color: expired ? Colors.red.shade50 : Colors.green.shade50,
              borderRadius: BorderRadius.circular(8),
            ),
            child: Text(
              expired ? '签名请求已过期，请返回重新提交' : '签名请求有效期剩余：${_remainingSeconds}s',
              style: TextStyle(
                color: expired ? Colors.red.shade700 : Colors.green.shade700,
                fontWeight: FontWeight.w600,
              ),
            ),
          ),
          const SizedBox(height: 16),

          // 请求二维码
          Center(
            child: QrImageView(
              data: widget.requestJson,
              version: QrVersions.auto,
              size: 240,
              errorStateBuilder: (cxt, err) {
                return Container(
                  width: 240,
                  height: 240,
                  padding: const EdgeInsets.all(10),
                  decoration: BoxDecoration(
                    color: Colors.red.shade50,
                    borderRadius: BorderRadius.circular(8),
                  ),
                  child: Center(
                    child: Text(
                      '二维码渲染失败：$err',
                      style: TextStyle(color: Colors.red.shade700),
                    ),
                  ),
                );
              },
            ),
          ),
          const SizedBox(height: 16),

          // 提示文字
          const Text(
            '请用离线设备扫描此二维码完成签名，\n然后点击下方按钮扫描回执二维码。',
            textAlign: TextAlign.center,
            style: TextStyle(color: Colors.black54),
          ),
          const SizedBox(height: 24),

          // 操作按钮
          Row(
            children: [
              Expanded(
                child: OutlinedButton(
                  onPressed: () => Navigator.of(context).pop(),
                  child: const Text('取消'),
                ),
              ),
              const SizedBox(width: 12),
              Expanded(
                child: FilledButton(
                  onPressed: expired ? null : _scanResponse,
                  child: const Text('扫描回执'),
                ),
              ),
            ],
          ),
        ],
      ),
    );
  }
}

// -----------------------------------------------------------------------------
// 简单扫码页：返回原始扫码字符串，不做协议路由。
// -----------------------------------------------------------------------------

class _SimpleScanner extends StatefulWidget {
  const _SimpleScanner();

  @override
  State<_SimpleScanner> createState() => _SimpleScannerState();
}

class _SimpleScannerState extends State<_SimpleScanner> {
  final MobileScannerController _controller = MobileScannerController();
  bool _handled = false;

  @override
  void dispose() {
    _controller.dispose();
    super.dispose();
  }

  void _handleCode(String raw) {
    if (_handled) return;
    _handled = true;
    Navigator.of(context).pop(raw);
  }

  @override
  Widget build(BuildContext context) {
    return Scaffold(
      appBar: AppBar(
        title: const Text('扫描签名回执'),
        centerTitle: true,
      ),
      body: Stack(
        fit: StackFit.expand,
        children: [
          MobileScanner(
            controller: _controller,
            onDetect: (capture) {
              final code = capture.barcodes.first.rawValue;
              if (code == null || code.isEmpty) return;
              _handleCode(code);
            },
          ),
          Align(
            alignment: Alignment.topCenter,
            child: Container(
              margin: const EdgeInsets.all(16),
              padding: const EdgeInsets.symmetric(horizontal: 12, vertical: 8),
              decoration: BoxDecoration(
                color: Colors.black54,
                borderRadius: BorderRadius.circular(8),
              ),
              child: const Text(
                '扫描离线设备上的签名回执二维码',
                style: TextStyle(color: Colors.white),
              ),
            ),
          ),
        ],
      ),
    );
  }
}
