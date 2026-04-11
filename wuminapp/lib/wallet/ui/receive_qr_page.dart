import 'dart:typed_data';
import 'dart:ui' as ui;

import 'package:flutter/material.dart';
import 'package:flutter/rendering.dart';
import 'package:flutter/services.dart';
import 'package:flutter_svg/flutter_svg.dart';
import 'package:qr/qr.dart';
import 'package:qr_flutter/qr_flutter.dart';
import 'package:saver_gallery/saver_gallery.dart';
import 'package:wuminapp_mobile/ui/app_theme.dart';
import 'package:wuminapp_mobile/qr/qr_protocols.dart';
import 'package:wuminapp_mobile/qr/envelope.dart';
import 'package:wuminapp_mobile/qr/bodies/user_transfer_body.dart';
import 'package:wuminapp_mobile/trade/offchain/clearing_banks.dart';

/// 临时收款码页面。
///
/// 商户输入收款金额后生成带 amount 的二维码，顾客扫码后直接支付。
/// 使用统一协议 WUMIN_QR_V1 kind=user_transfer。
class ReceiveQrPage extends StatefulWidget {
  const ReceiveQrPage({
    super.key,
    required this.address,
    required this.walletName,
    this.bankShenfenId,
  });

  /// 收款钱包地址。
  final String address;

  /// 钱包名称。
  final String walletName;

  /// 已绑定的清算省储行 shenfen_id（可选）。
  final String? bankShenfenId;

  @override
  State<ReceiveQrPage> createState() => _ReceiveQrPageState();
}

class _ReceiveQrPageState extends State<ReceiveQrPage> {
  final TextEditingController _amountController = TextEditingController();
  final GlobalKey _qrKey = GlobalKey();
  bool _isSavingQr = false;

  @override
  void dispose() {
    _amountController.dispose();
    super.dispose();
  }

  String _buildQrData() {
    final amountText = _amountController.text.trim();
    final now = DateTime.now().millisecondsSinceEpoch ~/ 1000;
    final id = 'rcv_${DateTime.now().microsecondsSinceEpoch}';
    return QrEnvelope<UserTransferBody>(
      kind: QrKind.userTransfer,
      id: id,
      issuedAt: now,
      expiresAt: now + 600,
      body: UserTransferBody(
        address: widget.address,
        name: widget.walletName,
        amount: amountText,
        symbol: 'GMB',
        memo: '',
        bank: widget.bankShenfenId ?? '',
      ),
    ).toRawJson();
  }

  Future<void> _saveQrToGallery() async {
    if (_isSavingQr) return;
    setState(() => _isSavingQr = true);
    try {
      final boundary =
          _qrKey.currentContext?.findRenderObject() as RenderRepaintBoundary?;
      if (boundary == null) return;
      final image = await boundary.toImage(pixelRatio: 3.0);
      final byteData = await image.toByteData(
        format: ui.ImageByteFormat.png,
      );
      if (byteData == null) return;
      final pngBytes = byteData.buffer.asUint8List();
      final fileName =
          'receive_qr_${DateTime.now().millisecondsSinceEpoch}.png';
      final result = await SaverGallery.saveImage(
        Uint8List.fromList(pngBytes),
        fileName: fileName,
        skipIfExists: false,
      );
      if (!mounted) return;
      final success = result.isSuccess;
      ScaffoldMessenger.of(context).showSnackBar(
        SnackBar(
          content: Text(success ? '收款码已保存到相册' : '保存失败，请检查相册权限'),
        ),
      );
    } catch (e) {
      if (!mounted) return;
      ScaffoldMessenger.of(context).showSnackBar(
        SnackBar(content: Text('保存失败：$e')),
      );
    } finally {
      if (mounted) setState(() => _isSavingQr = false);
    }
  }

  String _formatAddressTwoLines(String address) {
    if (address.length <= 24) return address;
    final mid = address.length ~/ 2;
    return '${address.substring(0, mid)}\n${address.substring(mid)}';
  }

  @override
  Widget build(BuildContext context) {
    final bankName = widget.bankShenfenId != null
        ? clearingBankName(widget.bankShenfenId!)
        : null;

    return Scaffold(
      appBar: AppBar(
        title: const Text('收款'),
        centerTitle: true,
      ),
      body: ListView(
        padding: const EdgeInsets.all(16),
        children: [
          // 金额输入
          Container(
            decoration: AppTheme.cardDecoration(),
            child: Padding(
              padding: const EdgeInsets.all(16),
              child: TextField(
                controller: _amountController,
                keyboardType: TextInputType.number,
                style: const TextStyle(
                  fontSize: 24,
                  fontWeight: FontWeight.w700,
                  color: AppTheme.textPrimary,
                ),
                decoration: const InputDecoration(
                  labelText: '收款金额',
                  hintText: '不填则由付款方输入',
                  suffixText: 'GMB',
                ),
                onChanged: (_) => setState(() {}),
              ),
            ),
          ),
          const SizedBox(height: 20),

          // 二维码
          Center(
            child: Stack(
              alignment: Alignment.center,
              children: [
                RepaintBoundary(
                  key: _qrKey,
                  child: Container(
                    color: Colors.white,
                    padding: const EdgeInsets.all(8),
                    child: CustomPaint(
                      size: const Size(240, 240),
                      painter: _HollowQrPainter(
                        data: _buildQrData(),
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
                    tooltip: '保存收款码到相册',
                    constraints: const BoxConstraints(),
                    padding: EdgeInsets.zero,
                    onPressed: _isSavingQr ? null : _saveQrToGallery,
                    icon: _isSavingQr
                        ? const SizedBox(
                            width: 16,
                            height: 16,
                            child: CircularProgressIndicator(strokeWidth: 2),
                          )
                        : SvgPicture.asset(
                            'assets/icons/download.svg',
                            width: 18,
                            height: 18,
                            colorFilter: const ColorFilter.mode(
                              AppTheme.textSecondary,
                              BlendMode.srcIn,
                            ),
                          ),
                  ),
                ),
              ],
            ),
          ),
          const SizedBox(height: 12),

          // 地址
          Center(
            child: GestureDetector(
              onTap: () {
                Clipboard.setData(ClipboardData(text: widget.address));
                ScaffoldMessenger.of(context).showSnackBar(
                  const SnackBar(content: Text('收款地址已复制')),
                );
              },
              child: Text(
                _formatAddressTwoLines(widget.address),
                textAlign: TextAlign.center,
                style: const TextStyle(
                  fontSize: 13,
                  color: AppTheme.textTertiary,
                ),
              ),
            ),
          ),
          const SizedBox(height: 12),

          // 清算行信息
          if (bankName != null)
            Center(
              child: Text(
                '清算行：$bankName',
                style: const TextStyle(
                  fontSize: 13,
                  color: AppTheme.textSecondary,
                ),
              ),
            ),

          // 金额提示
          if (_amountController.text.trim().isNotEmpty)
            Padding(
              padding: const EdgeInsets.only(top: 8),
              child: Center(
                child: Text(
                  '收款金额：${_amountController.text.trim()} GMB',
                  style: const TextStyle(
                    fontSize: 15,
                    fontWeight: FontWeight.w600,
                    color: AppTheme.primary,
                  ),
                ),
              ),
            ),
        ],
      ),
    );
  }
}

/// 中央留白二维码绘制器（复用 wallet_page.dart 中的同名实现）。
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
  bool shouldRepaint(covariant _HollowQrPainter oldDelegate) =>
      oldDelegate.data != data || oldDelegate.hollowSize != hollowSize;
}
