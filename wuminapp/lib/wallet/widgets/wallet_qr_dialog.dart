import 'dart:ui' as ui;

import 'package:flutter/material.dart';
import 'package:flutter/rendering.dart';
import 'package:flutter/services.dart';
import 'package:qr_flutter/qr_flutter.dart';
import 'package:saver_gallery/saver_gallery.dart';

import 'package:wuminapp_mobile/qr/bodies/user_contact_body.dart';
import 'package:wuminapp_mobile/qr/envelope.dart';
import 'package:wuminapp_mobile/qr/qr_protocols.dart';
import 'package:wuminapp_mobile/wallet/core/wallet_manager.dart';

/// 钱包收款二维码弹窗。
///
/// 中文注释:
/// - 职能:在钱包详情页第 2 张身份卡右侧 QR 小图标点击时弹出大二维码,
///   内容 = `WUMIN_QR_V1 kind=user_contact { address, name }`(固定码,
///   envelope 顶层无 id / issued_at / expires_at)。
/// - 三种扫码场景由扫码方自行处理(通讯录、扫码支付、地址栏),不生成多份 QR。
/// - 新增地址后复制图标 + 关闭右侧下载图标(RepaintBoundary + SaverGallery)。
/// - 删掉副标题 / 地址 Stack 居中,复制图标 Positioned 浮右(size 14) /
///   下载与关闭对称等宽 TextButton(下载 loading 时显示进度圈)。
class WalletQrDialog {
  /// 展示收款二维码弹窗。
  ///
  /// 参数:
  /// - [context]:用于 showDialog 的 BuildContext。
  /// - [wallet]:钱包档案,仅用于取 address 做 QR payload + 底部文字展示。
  /// - [name]:展示态钱包名(与 WalletIdentityCard 当前编辑的名字保持一致,
  ///   不直接读 `wallet.walletName` 是为了支持编辑态未落盘时的预览)。
  static Future<void> show(
    BuildContext context, {
    required WalletProfile wallet,
    required String name,
  }) {
    return showDialog<void>(
      context: context,
      builder: (_) => Dialog(
        shape: RoundedRectangleBorder(
          borderRadius: BorderRadius.circular(16),
        ),
        child: _WalletQrDialogContent(wallet: wallet, name: name),
      ),
    );
  }
}

/// 弹窗主体内容。改成 StatefulWidget 是因为需要:
/// - 持有 `GlobalKey` 引用 `RepaintBoundary` 做截图。
/// - 跟踪 `_isSaving` 状态实现保存过程中下载按钮禁用 + 进度圈。
class _WalletQrDialogContent extends StatefulWidget {
  const _WalletQrDialogContent({
    required this.wallet,
    required this.name,
  });

  final WalletProfile wallet;
  final String name;

  @override
  State<_WalletQrDialogContent> createState() => _WalletQrDialogContentState();
}

class _WalletQrDialogContentState extends State<_WalletQrDialogContent> {
  /// RepaintBoundary 的 key,保存二维码图片时通过它拿 RenderObject。
  final GlobalKey _qrKey = GlobalKey();

  /// 是否正在保存到相册。保存过程中按钮禁用 + 进度圈替换。
  bool _isSaving = false;

  /// 构造 QR payload:顶层 envelope(WUMIN_QR_V1 + kind=user_contact)
  /// + body(address + name)。
  String get _qrPayload => QrEnvelope<UserContactBody>(
        kind: QrKind.userContact,
        id: null,
        issuedAt: null,
        expiresAt: null,
        body: UserContactBody(
          address: widget.wallet.address,
          name: widget.name,
        ),
      ).toRawJson();

  /// 复制钱包地址到剪贴板并提示。
  void _copyAddress() {
    Clipboard.setData(ClipboardData(text: widget.wallet.address));
    ScaffoldMessenger.of(context).showSnackBar(
      const SnackBar(content: Text('钱包地址已复制')),
    );
  }

  /// 把当前二维码区域截图成 PNG,写入相册。
  ///
  /// 中文注释:
  /// - 通过 `_qrKey` 拿到 `RenderRepaintBoundary`,`toImage(pixelRatio: 3.0)`
  ///   生成高分图像。
  /// - `SaverGallery.saveImage` 需要 Uint8List + fileName + skipIfExists。
  /// - 异常统一 SnackBar 提示;失败 / 权限不足也返回失败态。
  Future<void> _saveQrToGallery() async {
    if (_isSaving) return;
    setState(() => _isSaving = true);
    try {
      final boundary =
          _qrKey.currentContext?.findRenderObject() as RenderRepaintBoundary?;
      if (boundary == null) {
        throw StateError('二维码尚未渲染');
      }
      final image = await boundary.toImage(pixelRatio: 3.0);
      final byteData = await image.toByteData(format: ui.ImageByteFormat.png);
      if (byteData == null) {
        throw StateError('二维码图像编码失败');
      }
      final pngBytes = byteData.buffer.asUint8List();
      final fileName = 'wallet_qr_${DateTime.now().millisecondsSinceEpoch}.png';
      final result = await SaverGallery.saveImage(
        Uint8List.fromList(pngBytes),
        fileName: fileName,
        skipIfExists: false,
      );
      if (!mounted) return;
      ScaffoldMessenger.of(context).showSnackBar(
        SnackBar(
          content: Text(
            result.isSuccess ? '二维码已保存到相册' : '保存失败,请检查相册权限',
          ),
        ),
      );
    } catch (e) {
      if (!mounted) return;
      ScaffoldMessenger.of(context).showSnackBar(
        SnackBar(content: Text('保存失败:$e')),
      );
    } finally {
      if (mounted) setState(() => _isSaving = false);
    }
  }

  @override
  Widget build(BuildContext context) {
    return Padding(
      padding: const EdgeInsets.all(24),
      child: Column(
        mainAxisSize: MainAxisSize.min,
        children: [
          // 钱包名
          Text(
            widget.name,
            style: const TextStyle(
              fontSize: 16,
              fontWeight: FontWeight.w600,
            ),
          ),
          const SizedBox(height: 16),
          // QR 本体:包在 RepaintBoundary 中,方便 _saveQrToGallery 截图。
          RepaintBoundary(
            key: _qrKey,
            child: Container(
              padding: const EdgeInsets.all(12),
              decoration: BoxDecoration(
                color: Colors.white,
                borderRadius: BorderRadius.circular(12),
                border: Border.all(color: Colors.grey[200]!),
              ),
              child: QrImageView(
                data: _qrPayload,
                version: QrVersions.auto,
                size: 240,
                eyeStyle: const QrEyeStyle(
                  eyeShape: QrEyeShape.square,
                  color: Color(0xFF134E4A),
                ),
                dataModuleStyle: const QrDataModuleStyle(
                  dataModuleShape: QrDataModuleShape.square,
                  color: Color(0xFF134E4A),
                ),
              ),
            ),
          ),
          const SizedBox(height: 12),
          // 中文注释:地址 Stack 居中显示在 QR 正下方,复制图标 Positioned 浮在右侧不抢中心。
          Stack(
            alignment: Alignment.center,
            children: [
              Padding(
                padding: const EdgeInsets.symmetric(horizontal: 32),
                child: GestureDetector(
                  onTap: _copyAddress,
                  child: Text(
                    widget.wallet.address,
                    textAlign: TextAlign.center,
                    style: TextStyle(
                      fontSize: 11,
                      color: Colors.grey[500],
                      fontFamily: 'monospace',
                    ),
                  ),
                ),
              ),
              Positioned(
                right: 0,
                child: IconButton(
                  icon: const Icon(Icons.copy, size: 14),
                  color: Colors.grey[600],
                  tooltip: '复制地址',
                  padding: EdgeInsets.zero,
                  constraints:
                      const BoxConstraints(minWidth: 24, minHeight: 24),
                  onPressed: _copyAddress,
                ),
              ),
            ],
          ),
          const SizedBox(height: 16),
          // 底部按钮行:关闭 + 下载对称等宽 TextButton,下载 loading 时显示进度圈。
          Row(
            children: [
              Expanded(
                child: TextButton(
                  onPressed: () => Navigator.of(context).pop(),
                  child: const Text('关闭'),
                ),
              ),
              const SizedBox(width: 8),
              Expanded(
                child: TextButton(
                  onPressed: _isSaving ? null : _saveQrToGallery,
                  child: _isSaving
                      ? const SizedBox(
                          width: 18,
                          height: 18,
                          child: CircularProgressIndicator(strokeWidth: 2),
                        )
                      : const Text('下载'),
                ),
              ),
            ],
          ),
        ],
      ),
    );
  }
}
