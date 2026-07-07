import 'dart:ui' as ui;

import 'package:flutter/material.dart';
import 'package:flutter/rendering.dart';
import 'package:qr_flutter/qr_flutter.dart';
import 'package:saver_gallery/saver_gallery.dart';

import 'package:citizenapp/qr/bodies/user_contact_body.dart';
import 'package:citizenapp/qr/envelope.dart';
import 'package:citizenapp/qr/qr_protocols.dart';
import 'package:citizenapp/ui/app_theme.dart';

/// 用户名片二维码：展示某用户的钱包账户 + 昵称，扫描可加通讯录。
/// 主页 ⋮ 菜单「二维码」进入，展示当前主页用户（本人或他人）的名片码。
class UserQrPage extends StatefulWidget {
  const UserQrPage({
    super.key,
    required this.contactName,
    required this.address,
  });

  final String contactName;
  final String address;

  @override
  State<UserQrPage> createState() => _UserQrPageState();
}

class _UserQrPageState extends State<UserQrPage> {
  final GlobalKey _qrKey = GlobalKey();
  bool _saving = false;

  String get _qrData => QrEnvelope<UserContactBody>(
        kind: QrKind.userContact,
        id: null,
        issuedAt: null,
        expiresAt: null,
        body: UserContactBody(
          address: widget.address,
          contactName: widget.contactName,
        ),
      ).toRawJson();

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
        title: const Text('二维码'),
        centerTitle: true,
      ),
      body: Column(
        children: [
          const Spacer(),
          Text(
            widget.contactName,
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
                    painter: _HollowQrPainter(
                      data: _qrData,
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
          Padding(
            padding: const EdgeInsets.symmetric(horizontal: 32),
            child: Text(
              widget.address,
              textAlign: TextAlign.center,
              style: const TextStyle(
                fontSize: 13,
                color: AppTheme.textTertiary,
                height: 1.5,
              ),
            ),
          ),
          const Spacer(),
          const Padding(
            padding: EdgeInsets.only(bottom: 32),
            child: Text(
              '其他用户扫描此二维码可添加通讯录',
              style: TextStyle(color: AppTheme.textTertiary, fontSize: 12),
            ),
          ),
        ],
      ),
    );
  }
}

/// 自绘二维码，中央 [hollowSize] 像素区域不绘制任何模块（真正留白）。
class _HollowQrPainter extends CustomPainter {
  _HollowQrPainter({required this.data, required this.hollowSize});

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
  bool shouldRepaint(_HollowQrPainter oldDelegate) {
    return oldDelegate.data != data || oldDelegate.hollowSize != hollowSize;
  }
}
