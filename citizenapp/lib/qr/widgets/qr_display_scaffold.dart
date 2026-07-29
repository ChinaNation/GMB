import 'dart:ui' as ui;

import 'package:flutter/material.dart';
import 'package:flutter/rendering.dart';
import 'package:flutter/services.dart';
import 'package:qr_flutter/qr_flutter.dart';
import 'package:saver_gallery/saver_gallery.dart';

import 'package:citizenapp/ui/app_theme.dart';

/// 展示型二维码的共用外壳:标题 + 顶部大字 + 中央留白二维码 + SS58 地址 + 底部说明。
///
/// 三种展示型码(用户码 / 钱包码 / 收款码)共用同一套外观,只有载荷与文案不同。
/// 本组件不构造任何载荷,由调用方传入已序列化好的 [qrData],避免在展示层混入
/// 「该出哪种码」的运行时判断。
class QrDisplayScaffold extends StatefulWidget {
  const QrDisplayScaffold({
    super.key,
    required this.headline,
    required this.qrData,
    required this.ss58Address,
    required this.footerText,
    this.title = '二维码',
  });

  /// AppBar 标题。
  final String title;

  /// 顶部大字。用户码为公开昵称,钱包码为本机账户标签(只在本机显示,不进载荷)。
  final String headline;

  /// 已序列化的 QR_V1 载荷。
  final String qrData;

  /// 展示态 SS58 地址(可复制);accountId 才是授权真源。
  final String ss58Address;

  /// 底部说明,必须如实覆盖该码的全部合法扫码场景。
  final String footerText;

  @override
  State<QrDisplayScaffold> createState() => _QrDisplayScaffoldState();
}

class _QrDisplayScaffoldState extends State<QrDisplayScaffold> {
  final GlobalKey _qrKey = GlobalKey();
  bool _saving = false;

  void _copyAddress() {
    Clipboard.setData(ClipboardData(text: widget.ss58Address));
    ScaffoldMessenger.of(context).showSnackBar(
      const SnackBar(content: Text('SS58 地址已复制')),
    );
  }

  Future<void> _saveQr() async {
    if (_saving) return;
    setState(() => _saving = true);
    try {
      final boundary =
          _qrKey.currentContext?.findRenderObject() as RenderRepaintBoundary?;
      if (boundary == null) return;
      final image = await boundary.toImage(pixelRatio: 3.0);
      final byteData = await image.toByteData(format: ui.ImageByteFormat.png);
      if (byteData == null || !mounted) return;
      final result = await SaverGallery.saveImage(
        byteData.buffer.asUint8List(),
        fileName: 'my_qr_${DateTime.now().millisecondsSinceEpoch}.png',
        androidRelativePath: 'Pictures/CitizenApp',
        skipIfExists: false,
      );
      if (!mounted) return;
      ScaffoldMessenger.of(context).showSnackBar(
        SnackBar(content: Text(result.isSuccess ? '已保存到相册' : '保存失败')),
      );
    } on Exception catch (e) {
      if (!mounted) return;
      ScaffoldMessenger.of(context).showSnackBar(
        SnackBar(content: Text('保存失败：$e')),
      );
    } finally {
      if (mounted) setState(() => _saving = false);
    }
  }

  @override
  Widget build(BuildContext context) {
    return Scaffold(
      appBar: AppBar(
        title: Text(widget.title),
        centerTitle: true,
      ),
      body: Column(
        children: [
          const Spacer(),
          Text(
            widget.headline,
            style: const TextStyle(
              fontSize: 20,
              fontWeight: FontWeight.w700,
            ),
          ),
          const SizedBox(height: 24),
          Stack(
            alignment: Alignment.center,
            children: [
              RepaintBoundary(
                key: _qrKey,
                child: Container(
                  color: Colors.white,
                  padding: const EdgeInsets.all(12),
                  child: CustomPaint(
                    size: const Size(240, 240),
                    painter: HollowQrPainter(
                      data: widget.qrData,
                      hollowSize: 48,
                    ),
                  ),
                ),
              ),
              Container(
                width: 36,
                height: 36,
                decoration: BoxDecoration(
                  color: Colors.white,
                  borderRadius: BorderRadius.circular(4),
                  border: Border.all(
                    color: AppTheme.border,
                    width: 1,
                  ),
                ),
                child: IconButton(
                  constraints: const BoxConstraints(),
                  padding: EdgeInsets.zero,
                  onPressed: _saving ? null : _saveQr,
                  icon: _saving
                      ? const SizedBox(
                          width: 16,
                          height: 16,
                          child: CircularProgressIndicator(strokeWidth: 2),
                        )
                      : const Icon(Icons.download,
                          size: 20, color: AppTheme.textSecondary),
                ),
              ),
            ],
          ),
          const SizedBox(height: 16),
          // 地址居中显示，复制图标浮右不抢中心。
          Stack(
            alignment: Alignment.center,
            children: [
              Padding(
                padding: const EdgeInsets.symmetric(horizontal: 48),
                child: GestureDetector(
                  onTap: _copyAddress,
                  child: Text(
                    widget.ss58Address,
                    textAlign: TextAlign.center,
                    style: const TextStyle(
                      fontSize: 13,
                      color: AppTheme.textTertiary,
                      height: 1.5,
                    ),
                  ),
                ),
              ),
              Positioned(
                right: 16,
                child: IconButton(
                  icon: const Icon(Icons.copy, size: 16),
                  color: AppTheme.textTertiary,
                  tooltip: '复制地址',
                  padding: EdgeInsets.zero,
                  constraints:
                      const BoxConstraints(minWidth: 24, minHeight: 24),
                  onPressed: _copyAddress,
                ),
              ),
            ],
          ),
          const Spacer(),
          Padding(
            padding: const EdgeInsets.only(bottom: 32),
            child: Text(
              widget.footerText,
              style: const TextStyle(
                color: AppTheme.textTertiary,
                fontSize: 12,
              ),
            ),
          ),
        ],
      ),
    );
  }
}

/// 自绘二维码，中央 [hollowSize] 像素区域不绘制任何模块（真正留白）。
class HollowQrPainter extends CustomPainter {
  HollowQrPainter({required this.data, required this.hollowSize});

  final String data;
  final double hollowSize;

  @override
  void paint(Canvas canvas, Size size) {
    final qrCode = QrCode.fromData(
      data: data,
      errorCorrectLevel: QrErrorCorrectLevel.H,
    );
    final qrImage = QrImage(qrCode);
    final moduleCount = qrImage.moduleCount;
    final moduleSize = size.width / moduleCount;
    final paint = Paint()..color = const Color(0xFF000000);

    final hollowModules = (hollowSize / moduleSize).ceil();
    final hollowStart = (moduleCount - hollowModules) ~/ 2;
    final hollowEnd = hollowStart + hollowModules;

    for (var row = 0; row < moduleCount; row++) {
      for (var col = 0; col < moduleCount; col++) {
        if (qrImage.isDark(row, col)) {
          if (row >= hollowStart &&
              row < hollowEnd &&
              col >= hollowStart &&
              col < hollowEnd) {
            continue;
          }
          canvas.drawRect(
            Rect.fromLTWH(
              col * moduleSize,
              row * moduleSize,
              moduleSize,
              moduleSize,
            ),
            paint,
          );
        }
      }
    }
  }

  @override
  bool shouldRepaint(HollowQrPainter oldDelegate) {
    return oldDelegate.data != data || oldDelegate.hollowSize != hollowSize;
  }
}
