import 'dart:ui' as ui;

import 'package:flutter/material.dart';
import 'package:flutter/rendering.dart';
import 'package:flutter/services.dart';
import 'package:qr_flutter/qr_flutter.dart';
import 'package:saver_gallery/saver_gallery.dart';

import 'package:citizenapp/qr/bodies/user_contact_body.dart';
import 'package:citizenapp/citizen/shared/account_derivation.dart';
import 'package:citizenapp/qr/envelope.dart';
import 'package:citizenapp/qr/qr_protocols.dart';
import 'package:citizenapp/ui/app_theme.dart';

/// 全 App 唯一用户二维码：同一张 `QR_V1 k=3`（userContact）名片码 = 钱包账户 + 昵称。
///
/// 扫码结果由**扫描模式**决定：contact 模式 = 加入通讯录；transfer / dispatch 模式 =
/// 按收款人进入转账。因此全 App 不再生成第二份二维码。
/// 入口：主页 ⋮ 菜单「二维码」（本人或他人）、钱包身份卡 QR 图标、聊天页「收付款」。
class UserQrPage extends StatefulWidget {
  const UserQrPage({
    super.key,
    required this.contactName,
    required this.accountId,
  });

  final String contactName;
  final String accountId;

  @override
  State<UserQrPage> createState() => _UserQrPageState();
}

class _UserQrPageState extends State<UserQrPage> {
  final GlobalKey _qrKey = GlobalKey();
  bool _saving = false;

  /// 展示态 SS58 地址（accountId 为授权真源，ss58 仅用于展示与二维码载荷）。
  String get _ss58Address => ss58FromAccountIdText(widget.accountId);

  String get _qrData => QrEnvelope<UserContactBody>(
        kind: QrKind.userContact,
        id: null,
        issuedAt: null,
        expiresAt: null,
        body: UserContactBody(
          ss58Address: _ss58Address,
          contactName: widget.contactName,
        ),
      ).toRawJson();

  /// 复制地址到剪贴板（并入原钱包收款弹窗的能力）。
  void _copyAddress() {
    Clipboard.setData(ClipboardData(text: _ss58Address));
    ScaffoldMessenger.of(context).showSnackBar(
      const SnackBar(content: Text('钱包地址已复制')),
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
          // 地址居中显示，复制图标浮右不抢中心。
          Stack(
            alignment: Alignment.center,
            children: [
              Padding(
                padding: const EdgeInsets.symmetric(horizontal: 48),
                child: GestureDetector(
                  onTap: _copyAddress,
                  child: Text(
                    _ss58Address,
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
          const Padding(
            padding: EdgeInsets.only(bottom: 32),
            child: Text(
              '扫描此二维码可加为联系人，或向其转账',
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
