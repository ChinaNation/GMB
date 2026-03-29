import 'package:flutter/material.dart';
import 'package:image_picker/image_picker.dart';
import 'package:mobile_scanner/mobile_scanner.dart';

import 'dart:convert';

import '../qr/offline_sign_page.dart';
import '../qr/qr_protocols.dart';
import '../wallet/wallet_manager.dart';
import 'app_theme.dart';
import 'login_sign_page.dart';

/// 扫码页面（对准框 + 相册 + 手电筒）。
///
/// 扫到签名请求后跳转 [OfflineSignPage]。
class ScanPage extends StatefulWidget {
  const ScanPage({super.key, required this.wallet});

  final WalletProfile wallet;

  @override
  State<ScanPage> createState() => _ScanPageState();
}

class _ScanPageState extends State<ScanPage> {
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
      await _handleCode(code);
    }
  }

  Future<void> _handleCode(String raw) async {
    if (_handled) return;
    _handled = true;
    await _controller.stop();

    if (!mounted) return;

    // 判断协议类型：登录 QR 走 LoginSignPage，其余走 OfflineSignPage。
    final isLogin = _isLoginProtocol(raw);
    await Navigator.of(context).push(
      MaterialPageRoute(
        builder: (_) => isLogin
            ? LoginSignPage(
                wallet: widget.wallet,
                challengeRaw: raw,
              )
            : OfflineSignPage(
                wallet: widget.wallet,
                initialCode: raw,
              ),
      ),
    );

    // 返回后关闭扫码页
    if (!mounted) return;
    Navigator.of(context).pop();
  }

  bool _isLoginProtocol(String raw) {
    try {
      final data = jsonDecode(raw) as Map<String, dynamic>;
      return data['proto'] == QrProtocols.login;
    } catch (_) {
      return false;
    }
  }

  @override
  Widget build(BuildContext context) {
    return Scaffold(
      backgroundColor: Colors.black,
      appBar: AppBar(
        backgroundColor: Colors.transparent,
        title: const Text('扫码签名'),
        centerTitle: true,
      ),
      body: Stack(
        fit: StackFit.expand,
        children: [
          // 摄像头画面
          MobileScanner(
            controller: _controller,
            onDetect: (capture) async {
              final code = capture.barcodes.first.rawValue;
              if (code == null || code.isEmpty) return;
              await _handleCode(code);
            },
          ),

          // 扫描框 + 半透明遮罩
          CustomPaint(
            painter: _ScanOverlayPainter(
              scanBoxSize: scanBoxSize,
              offsetY: scanBoxOffsetY,
            ),
            child: const SizedBox.expand(),
          ),

          // 扫描框四角装饰
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

          // 提示文字
          Center(
            child: Transform.translate(
              offset: const Offset(0, scanBoxOffsetY + scanBoxSize / 2 + 28),
              child: const Text(
                '将二维码放入框内即可自动扫描',
                style: TextStyle(
                  color: Colors.white60,
                  fontSize: 14,
                  letterSpacing: 0.5,
                ),
              ),
            ),
          ),

          // 底部工具栏：相册 + 手电筒
          Align(
            alignment: Alignment.bottomCenter,
            child: Container(
              margin: const EdgeInsets.only(bottom: 48, left: 48, right: 48),
              padding: const EdgeInsets.symmetric(horizontal: 24, vertical: 16),
              decoration: BoxDecoration(
                color: AppTheme.surfaceCard.withAlpha(200),
                borderRadius: BorderRadius.circular(AppTheme.radiusLg),
                border: Border.all(color: AppTheme.border.withAlpha(80)),
              ),
              child: Row(
                mainAxisAlignment: MainAxisAlignment.spaceEvenly,
                children: [
                  _buildToolButton(
                    icon: Icons.photo_library_outlined,
                    label: '相册',
                    onTap: _scanFromGallery,
                    active: false,
                  ),
                  Container(
                    width: 1,
                    height: 32,
                    color: AppTheme.border,
                  ),
                  _buildToolButton(
                    icon: _torchOn
                        ? Icons.flashlight_on_rounded
                        : Icons.flashlight_off_outlined,
                    label: _torchOn ? '关闭' : '手电筒',
                    onTap: _toggleTorch,
                    active: _torchOn,
                  ),
                ],
              ),
            ),
          ),
        ],
      ),
    );
  }

  Widget _buildToolButton({
    required IconData icon,
    required String label,
    required VoidCallback onTap,
    required bool active,
  }) {
    return GestureDetector(
      onTap: onTap,
      child: Column(
        mainAxisSize: MainAxisSize.min,
        children: [
          Icon(
            icon,
            size: 26,
            color: active ? AppTheme.gold : Colors.white,
          ),
          const SizedBox(height: 6),
          Text(
            label,
            style: TextStyle(
              color: active ? AppTheme.gold : Colors.white70,
              fontSize: 12,
            ),
          ),
        ],
      ),
    );
  }
}

// -----------------------------------------------------------------------------
// 扫描框半透明遮罩
// -----------------------------------------------------------------------------

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
    canvas.drawRRect(
      RRect.fromRectAndRadius(rect, const Radius.circular(12)),
      clearPaint,
    );
    canvas.restore();
  }

  @override
  bool shouldRepaint(covariant _ScanOverlayPainter oldDelegate) =>
      oldDelegate.scanBoxSize != scanBoxSize || oldDelegate.offsetY != offsetY;
}

// -----------------------------------------------------------------------------
// 扫描框四角装饰线
// -----------------------------------------------------------------------------

class _ScanCornerPainter extends CustomPainter {
  @override
  void paint(Canvas canvas, Size size) {
    const cornerLen = 28.0;
    const strokeWidth = 3.5;
    const cornerRadius = 12.0;

    final paint = Paint()
      ..color = AppTheme.primaryLight
      ..strokeWidth = strokeWidth
      ..style = PaintingStyle.stroke
      ..strokeCap = StrokeCap.round;

    final w = size.width;
    final h = size.height;

    // 左上
    final topLeftPath = Path()
      ..moveTo(0, cornerLen)
      ..lineTo(0, cornerRadius)
      ..quadraticBezierTo(0, 0, cornerRadius, 0)
      ..lineTo(cornerLen, 0);
    canvas.drawPath(topLeftPath, paint);

    // 右上
    final topRightPath = Path()
      ..moveTo(w - cornerLen, 0)
      ..lineTo(w - cornerRadius, 0)
      ..quadraticBezierTo(w, 0, w, cornerRadius)
      ..lineTo(w, cornerLen);
    canvas.drawPath(topRightPath, paint);

    // 左下
    final bottomLeftPath = Path()
      ..moveTo(0, h - cornerLen)
      ..lineTo(0, h - cornerRadius)
      ..quadraticBezierTo(0, h, cornerRadius, h)
      ..lineTo(cornerLen, h);
    canvas.drawPath(bottomLeftPath, paint);

    // 右下
    final bottomRightPath = Path()
      ..moveTo(w - cornerLen, h)
      ..lineTo(w - cornerRadius, h)
      ..quadraticBezierTo(w, h, w, h - cornerRadius)
      ..lineTo(w, h - cornerLen);
    canvas.drawPath(bottomRightPath, paint);
  }

  @override
  bool shouldRepaint(covariant CustomPainter oldDelegate) => false;
}
