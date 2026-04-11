import 'dart:async';

import 'package:flutter/material.dart';
import 'package:image_picker/image_picker.dart';
import 'package:mobile_scanner/mobile_scanner.dart';
import 'package:qr_flutter/qr_flutter.dart';
import 'package:wuminapp_mobile/ui/app_theme.dart';
import 'package:wuminapp_mobile/qr/bodies/sign_request_body.dart';
import 'package:wuminapp_mobile/signer/qr_signer.dart';

/// 冷钱包扫码签名会话页面。
///
/// 两阶段交互：
/// 1. 展示签名请求二维码，等待离线设备扫描。
/// 2. 用户点击"扫描回执"，打开相机扫描离线设备生成的签名回执二维码。
///
/// 返回 [SignResponseEnvelope](成功)或 `null`(取消/超时)。
class QrSignSessionPage extends StatefulWidget {
  const QrSignSessionPage({
    super.key,
    required this.request,
    required this.requestJson,
    required this.expectedPubkey,
  });

  /// 已构建的签名请求 envelope。
  final SignRequestEnvelope request;

  /// 编码后的 JSON 字符串,直接用于二维码展示。
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
    final left = (widget.request.expiresAt ?? 0) - now;
    return left > 0 ? left : 0;
  }

  Future<void> _scanResponse() async {
    final raw = await Navigator.of(context).push<String>(
      MaterialPageRoute(builder: (_) => const _SimpleScanner()),
    );
    if (raw == null || !mounted) return;

    try {
      final expectedHash =
          QrSigner.computePayloadHash(widget.request.body.payloadHex);
      final response = QrSigner().parseResponse(
        raw,
        expectedRequestId: widget.request.id!,
        expectedPubkey: widget.expectedPubkey,
        expectedPayloadHash: expectedHash,
        expectedPayloadHex: widget.request.body.payloadHex,
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
            padding: const EdgeInsets.symmetric(horizontal: 12, vertical: 8),
            decoration: AppTheme.bannerDecoration(expired ? AppTheme.danger : AppTheme.success),
            child: Row(
              children: [
                Icon(
                  expired ? Icons.timer_off : Icons.timer_outlined,
                  size: 18,
                  color: expired ? AppTheme.danger : AppTheme.success,
                ),
                const SizedBox(width: 8),
                Expanded(
                  child: Text(
                    expired ? '签名请求已过期，请返回重新提交' : '签名请求有效期剩余：${_remainingSeconds}s',
                    style: TextStyle(
                      color: expired ? AppTheme.danger : AppTheme.success,
                      fontWeight: FontWeight.w600,
                    ),
                  ),
                ),
              ],
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
                  decoration: AppTheme.bannerDecoration(AppTheme.danger),
                  child: const Center(
                    child: Text(
                      '二维码渲染失败',
                      style: TextStyle(color: AppTheme.danger),
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
            style: TextStyle(color: AppTheme.textSecondary),
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
  static const double scanBoxSize = 260;
  static const double scanBoxOffsetY = -40;

  late final MobileScannerController _controller;
  bool _handled = false;
  bool _torchOn = false;

  @override
  void initState() {
    super.initState();
    _controller = MobileScannerController(
      detectionSpeed: DetectionSpeed.normal,
      facing: CameraFacing.back,
      torchEnabled: false,
    );
  }

  @override
  void dispose() {
    _controller.dispose();
    super.dispose();
  }

  Future<void> _toggleTorch() async {
    await _controller.toggleTorch();
    setState(() {
      _torchOn = !_torchOn;
    });
  }

  Future<void> _scanFromGallery() async {
    final picker = ImagePicker();
    final image = await picker.pickImage(source: ImageSource.gallery);
    if (image == null) return;
    final capture = await _controller.analyzeImage(image.path);
    if (capture == null || capture.barcodes.isEmpty) {
      if (!mounted) return;
      ScaffoldMessenger.of(context).showSnackBar(
        const SnackBar(content: Text('未识别到二维码')),
      );
      return;
    }
    final code = capture.barcodes.first.rawValue;
    if (code != null && code.isNotEmpty) {
      _handleCode(code);
    }
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

          CustomPaint(
            painter: _ScanOverlayPainter(
              scanBoxSize: scanBoxSize,
              offsetY: scanBoxOffsetY,
            ),
            child: const SizedBox.expand(),
          ),

          Center(
            child: Transform.translate(
              offset: const Offset(0, scanBoxOffsetY),
              child: SizedBox(
                width: scanBoxSize,
                height: scanBoxSize,
                child: CustomPaint(
                  painter: _ScanCornerPainter(),
                ),
              ),
            ),
          ),

          Center(
            child: Transform.translate(
              offset: const Offset(0, scanBoxOffsetY + scanBoxSize / 2 + 24),
              child: const Text(
                '扫描离线设备上的签名回执二维码',
                style: TextStyle(color: Colors.white70, fontSize: 14),
              ),
            ),
          ),

          Align(
            alignment: Alignment.bottomCenter,
            child: Padding(
              padding: const EdgeInsets.only(bottom: 60, left: 48, right: 48),
              child: Row(
                mainAxisAlignment: MainAxisAlignment.spaceBetween,
                children: [
                  Column(
                    mainAxisSize: MainAxisSize.min,
                    children: [
                      IconButton(
                        onPressed: _scanFromGallery,
                        icon: const Icon(Icons.photo_library_outlined),
                        iconSize: 32,
                        color: Colors.white,
                      ),
                      const Text(
                        '相册',
                        style: TextStyle(color: Colors.white, fontSize: 12),
                      ),
                    ],
                  ),
                  Column(
                    mainAxisSize: MainAxisSize.min,
                    children: [
                      IconButton(
                        onPressed: _toggleTorch,
                        icon: Icon(
                          _torchOn
                              ? Icons.flashlight_on
                              : Icons.flashlight_off_outlined,
                        ),
                        iconSize: 32,
                        color: _torchOn ? Colors.amber : Colors.white,
                      ),
                      Text(
                        _torchOn ? '关闭' : '手电筒',
                        style:
                            const TextStyle(color: Colors.white, fontSize: 12),
                      ),
                    ],
                  ),
                ],
              ),
            ),
          ),
        ],
      ),
    );
  }
}

class _ScanOverlayPainter extends CustomPainter {
  _ScanOverlayPainter({required this.scanBoxSize, this.offsetY = 0});

  final double scanBoxSize;
  final double offsetY;

  @override
  void paint(Canvas canvas, Size size) {
    final bgPaint = Paint()..color = Colors.black.withAlpha(140);
    final clearPaint = Paint()..blendMode = BlendMode.clear;

    final center = Offset(size.width / 2, size.height / 2 + offsetY);
    final rect = Rect.fromCenter(
      center: center,
      width: scanBoxSize,
      height: scanBoxSize,
    );

    canvas.saveLayer(Offset.zero & size, Paint());
    canvas.drawRect(Offset.zero & size, bgPaint);
    canvas.drawRect(rect, clearPaint);
    canvas.restore();
  }

  @override
  bool shouldRepaint(covariant _ScanOverlayPainter oldDelegate) =>
      oldDelegate.scanBoxSize != scanBoxSize || oldDelegate.offsetY != offsetY;
}

class _ScanCornerPainter extends CustomPainter {
  @override
  void paint(Canvas canvas, Size size) {
    const cornerLen = 24.0;
    const strokeWidth = 4.0;

    final paint = Paint()
      ..color = AppTheme.primary
      ..strokeWidth = strokeWidth
      ..style = PaintingStyle.stroke
      ..strokeCap = StrokeCap.round;

    final w = size.width;
    final h = size.height;

    canvas.drawLine(const Offset(0, 0), const Offset(cornerLen, 0), paint);
    canvas.drawLine(const Offset(0, 0), const Offset(0, cornerLen), paint);
    canvas.drawLine(Offset(w, 0), Offset(w - cornerLen, 0), paint);
    canvas.drawLine(Offset(w, 0), Offset(w, cornerLen), paint);
    canvas.drawLine(Offset(0, h), Offset(cornerLen, h), paint);
    canvas.drawLine(Offset(0, h), Offset(0, h - cornerLen), paint);
    canvas.drawLine(Offset(w, h), Offset(w - cornerLen, h), paint);
    canvas.drawLine(Offset(w, h), Offset(w, h - cornerLen), paint);
  }

  @override
  bool shouldRepaint(covariant CustomPainter oldDelegate) => false;
}
