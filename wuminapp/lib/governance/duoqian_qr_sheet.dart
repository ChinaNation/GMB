import 'dart:typed_data';
import 'dart:ui' as ui;

import 'package:flutter/material.dart';
import 'package:flutter/rendering.dart';
import 'package:qr_flutter/qr_flutter.dart';
import 'package:saver_gallery/saver_gallery.dart';

import '../qr/qr_protocols.dart';
import '../qr/envelope.dart';
import '../qr/bodies/user_duoqian_body.dart';
import '../ui/app_theme.dart';

/// 多签账户二维码底部弹窗。
///
/// 展示多签账户 QR 码(WUMIN_QR_V1, kind=user_duoqian),支持保存到相册。
class DuoqianQrSheet extends StatefulWidget {
  const DuoqianQrSheet({
    super.key,
    required this.address,
    required this.name,
    this.proposalId,
  });

  /// 多签账户 SS58 地址。
  final String address;

  /// 多签账户名称。
  final String name;

  /// 提案 ID（Pending 状态时传入，用于其他管理员扫码投票）。
  final int? proposalId;

  @override
  State<DuoqianQrSheet> createState() => _DuoqianQrSheetState();
}

class _DuoqianQrSheetState extends State<DuoqianQrSheet> {
  final _qrKey = GlobalKey();
  bool _saving = false;

  String _buildQrData() {
    return QrEnvelope<UserDuoqianBody>(
      kind: QrKind.userDuoqian,
      id: null,
      issuedAt: null,
      expiresAt: null,
      body: UserDuoqianBody(
        address: widget.address,
        name: widget.name,
        proposalId: widget.proposalId ?? 0,
      ),
    ).toRawJson();
  }

  Future<void> _saveToGallery() async {
    setState(() => _saving = true);
    try {
      final boundary =
          _qrKey.currentContext?.findRenderObject() as RenderRepaintBoundary?;
      if (boundary == null) throw Exception('无法获取二维码图像');
      final image = await boundary.toImage(pixelRatio: 3.0);
      final byteData = await image.toByteData(format: ui.ImageByteFormat.png);
      if (byteData == null) throw Exception('图像转换失败');
      final pngBytes = byteData.buffer.asUint8List();
      final filename =
          'duoqian_qr_${DateTime.now().millisecondsSinceEpoch}.png';
      final result = await SaverGallery.saveImage(
        Uint8List.fromList(pngBytes),
        fileName: filename,
        skipIfExists: false,
      );
      if (!mounted) return;
      final ok = result.isSuccess;
      ScaffoldMessenger.of(context).showSnackBar(
        SnackBar(
          content: Text(ok ? '已保存到相册' : '保存失败'),
          backgroundColor: ok ? AppTheme.success : AppTheme.danger,
        ),
      );
    } catch (e) {
      if (!mounted) return;
      ScaffoldMessenger.of(context).showSnackBar(
        SnackBar(content: Text('保存失败：$e'), backgroundColor: AppTheme.danger),
      );
    } finally {
      if (mounted) setState(() => _saving = false);
    }
  }

  @override
  Widget build(BuildContext context) {
    final qrData = _buildQrData();
    return SafeArea(
      child: Padding(
        padding: const EdgeInsets.fromLTRB(24, 16, 24, 24),
        child: Column(
          mainAxisSize: MainAxisSize.min,
          children: [
            // 拖拽指示条
            Container(
              width: 36,
              height: 4,
              decoration: BoxDecoration(
                color: AppTheme.border,
                borderRadius: BorderRadius.circular(2),
              ),
            ),
            const SizedBox(height: 16),
            Text(
              widget.name,
              style: const TextStyle(
                fontSize: 17,
                fontWeight: FontWeight.w700,
                color: AppTheme.primaryDark,
              ),
            ),
            const SizedBox(height: 4),
            const Text(
              '多签账户二维码',
              style: TextStyle(fontSize: 13, color: AppTheme.textTertiary),
            ),
            const SizedBox(height: 20),
            // QR 码
            RepaintBoundary(
              key: _qrKey,
              child: Container(
                color: Colors.white,
                padding: const EdgeInsets.all(16),
                child: QrImageView(
                  data: qrData,
                  version: QrVersions.auto,
                  size: 220,
                  errorCorrectionLevel: QrErrorCorrectLevel.H,
                ),
              ),
            ),
            const SizedBox(height: 12),
            Text(
              widget.address,
              style: const TextStyle(
                fontSize: 11,
                fontFamily: 'monospace',
                color: AppTheme.textTertiary,
              ),
              textAlign: TextAlign.center,
            ),
            const SizedBox(height: 20),
            Row(
              children: [
                Expanded(
                  child: OutlinedButton(
                    onPressed: () => Navigator.pop(context),
                    style: OutlinedButton.styleFrom(
                      foregroundColor: AppTheme.textSecondary,
                      padding: const EdgeInsets.symmetric(vertical: 12),
                      shape: RoundedRectangleBorder(
                        borderRadius: BorderRadius.circular(10),
                      ),
                    ),
                    child: const Text('关闭'),
                  ),
                ),
                const SizedBox(width: 12),
                Expanded(
                  child: ElevatedButton(
                    onPressed: _saving ? null : _saveToGallery,
                    style: ElevatedButton.styleFrom(
                      backgroundColor: AppTheme.primaryDark,
                      foregroundColor: Colors.white,
                      padding: const EdgeInsets.symmetric(vertical: 12),
                      shape: RoundedRectangleBorder(
                        borderRadius: BorderRadius.circular(10),
                      ),
                    ),
                    child: _saving
                        ? const SizedBox(
                            width: 16,
                            height: 16,
                            child: CircularProgressIndicator(
                                strokeWidth: 2, color: Colors.white),
                          )
                        : const Text('保存到相册'),
                  ),
                ),
              ],
            ),
          ],
        ),
      ),
    );
  }
}
