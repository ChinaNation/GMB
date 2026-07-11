import 'package:flutter/material.dart';

import 'app_theme.dart';

/// 扫描框布局常量（单源，scan_page 与 offline_sign_page 共用）。
const double scanBoxSize = 260;
const double scanBoxOffsetY = -40;

/// 扫描框半透明遮罩：整屏压暗并挖空中心方框。
class ScanOverlayPainter extends CustomPainter {
  ScanOverlayPainter({required this.scanBoxSize, this.offsetY = 0});

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
  bool shouldRepaint(covariant ScanOverlayPainter oldDelegate) =>
      oldDelegate.scanBoxSize != scanBoxSize || oldDelegate.offsetY != offsetY;
}

/// 扫描框四角装饰线。
class ScanCornerPainter extends CustomPainter {
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
