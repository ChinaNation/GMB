import 'package:flutter/material.dart';
import 'package:image_picker/image_picker.dart';
import 'package:mobile_scanner/mobile_scanner.dart';
import 'package:citizenapp/ui/app_theme.dart';
import 'package:citizenapp/qr/bodies/user_contact_body.dart';
import 'package:citizenapp/qr/bodies/user_transfer_body.dart';
import 'package:citizenapp/qr/qr_router.dart';
import 'package:citizenapp/my/user/user_service.dart';

/// 扫码结果：收款码预填数据。
class QrScanTransferResult {
  const QrScanTransferResult({
    required this.toAddress,
    this.amount,
    this.symbol,
    this.memo,
    this.bank,
  });

  final String toAddress;
  final String? amount;
  final String? symbol;
  final String? memo;
  final String? bank;
}

/// 扫码模式。
enum QrScanMode {
  /// 扫码支付：仅识别收款码 / 裸地址。
  transfer,

  /// 扫码添加好友：仅识别用户码。
  contact,

  /// 通用扫码：直接返回原始字符串，不做协议路由。
  raw,

  /// 统一「扫一扫」：按协议分类后交调用方分派——收款码/裸地址 pop
  /// [QrScanTransferResult]（走支付）；signRequest pop 原始字符串（走签名）；未来类型再加。
  dispatch,
}

/// 统一扫码页。
///
/// 通过 [mode] 区分两种独立功能：
/// - [QrScanMode.transfer] → 扫码支付
/// - [QrScanMode.contact]  → 扫码添加好友
class QrScanPage extends StatefulWidget {
  const QrScanPage({
    super.key,
    required this.mode,
    this.selfAddress,
    this.initialCode,
    this.customTitle,
  });

  /// 扫码模式。
  final QrScanMode mode;

  /// 当前用户 SS58 地址（通讯录防自加用）。
  final String? selfAddress;

  /// 如果已扫码，可直接传入原始字符串跳过扫码步骤。
  final String? initialCode;

  /// 自定义标题（为 null 时使用默认标题）。
  final String? customTitle;

  @override
  State<QrScanPage> createState() => _QrScanPageState();
}

class _QrScanPageState extends State<QrScanPage> {
  final MobileScannerController _controller = MobileScannerController();
  final QrRouter _router = QrRouter();
  final UserContactService _contactService = UserContactService();
  bool _handled = false;
  bool _torchOn = false;
  // 本页正在 pop/dispose 时置真，阻止 _handleCode 的 finally 重启相机（避免与 dispose 竞态）。
  bool _closing = false;

  @override
  void initState() {
    super.initState();
    final code = widget.initialCode;
    if (code != null) {
      WidgetsBinding.instance.addPostFrameCallback((_) => _handleCode(code));
    }
  }

  @override
  void dispose() {
    _controller.dispose();
    super.dispose();
  }

  /// 从相册选取图片识别二维码
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

  /// 切换手电筒
  Future<void> _toggleTorch() async {
    await _controller.toggleTorch();
    setState(() {
      _torchOn = !_torchOn;
    });
  }

  Future<void> _handleCode(String raw) async {
    if (_handled) {
      return;
    }
    _handled = true;
    await _controller.stop();

    try {
      final result = _router.route(raw);

      switch (widget.mode) {
        case QrScanMode.transfer:
          // 扫码支付:接受 user_transfer / user_contact / 裸地址
          // (多签发现走反向索引)
          if (result.type == QrRouteType.userTransfer) {
            _handleTransfer(result);
          } else if (result.type == QrRouteType.userContact) {
            _handleContactAsRecipient(result);
          } else if (result.type == QrRouteType.legacyAddress) {
            _handleLegacyAddress(result.extractedAddress!);
          } else {
            await _showUnrecognized();
          }
        case QrScanMode.contact:
          // 扫码添加好友:接受 user_transfer(带 recipient_name)/ user_contact
          if (result.type == QrRouteType.userTransfer) {
            await _handleContactFromTransfer(result);
          } else if (result.type == QrRouteType.userContact) {
            await _handleContact(result);
          } else {
            await _showUnrecognized();
          }
        case QrScanMode.raw:
          // 通用扫码:直接返回原始字符串
          if (!mounted) return;
          _popPage(raw);
        case QrScanMode.dispatch:
          // 统一扫一扫:收款/裸地址→支付结果;signRequest→原始串交调用方签名
          if (result.type == QrRouteType.userTransfer) {
            _handleTransfer(result);
          } else if (result.type == QrRouteType.legacyAddress) {
            _handleLegacyAddress(result.extractedAddress!);
          } else if (result.type == QrRouteType.signRequest) {
            if (!mounted) return;
            _popPage(raw);
          } else {
            await _showUnrecognized();
          }
      }
    } catch (e) {
      if (!mounted) {
        return;
      }
      await showDialog<void>(
        context: context,
        builder: (context) => AlertDialog(
          title: const Text('扫码处理异常'),
          content: Text('$e'),
          actions: [
            TextButton(
              onPressed: () => Navigator.of(context).pop(),
              child: const Text('确定'),
            ),
          ],
        ),
      );
    } finally {
      if (mounted && !_closing) {
        _handled = false;
        await _controller.start();
      }
    }
  }

  /// 页级返回：先置 _closing，让 _handleCode 的 finally 不再重启相机
  /// （本页正在 pop/dispose，重启相机会与 dispose 竞争抛错）。
  void _popPage([Object? result]) {
    _closing = true;
    Navigator.of(context).pop(result);
  }

  // 收款码
  void _handleTransfer(QrRouteResult result) {
    if (!mounted) {
      return;
    }
    final body = result.envelope!.body as UserTransferBody;
    _popPage(QrScanTransferResult(
      toAddress: body.address,
      amount: body.amount.isEmpty ? null : body.amount,
      symbol: body.symbol.isEmpty ? null : body.symbol,
      memo: body.memo.isEmpty ? null : body.memo,
      bank: body.bank.isEmpty ? null : body.bank,
    ));
  }

  void _handleContactAsRecipient(QrRouteResult result) {
    if (!mounted) return;
    final body = result.envelope!.body as UserContactBody;
    _popPage(QrScanTransferResult(toAddress: body.address));
  }

  // 裸地址（向后兼容）
  void _handleLegacyAddress(String address) {
    if (!mounted) {
      return;
    }
    _popPage(QrScanTransferResult(toAddress: address));
  }

  // 收款码 → 添加通讯录
  Future<void> _handleContactFromTransfer(QrRouteResult result) async {
    if (!mounted) return;
    try {
      final body = result.envelope!.body as UserTransferBody;
      final name = body.recipientName.trim();
      if (name.isEmpty) {
        await showDialog<void>(
          context: context,
          builder: (context) => AlertDialog(
            title: const Text('无法添加'),
            content: const Text('该收款码不包含钱包名称，无法添加到通讯录'),
            actions: [
              TextButton(
                onPressed: () => Navigator.of(context).pop(),
                child: const Text('确定'),
              ),
            ],
          ),
        );
        return;
      }
      final contactResult = await _contactService.addContact(
        address: body.address,
        contactName: name,
        selfAddress: widget.selfAddress,
      );
      if (!mounted) return;
      ScaffoldMessenger.of(context).showSnackBar(
        SnackBar(
          content: Text(
            contactResult.created
                ? '已加入通讯录：${contactResult.contact.displayNickname}'
                : '已更新通讯录：${contactResult.contact.displayNickname}',
          ),
        ),
      );
      _popPage();
    } catch (e) {
      if (!mounted) return;
      await showDialog<void>(
        context: context,
        builder: (context) => AlertDialog(
          title: const Text('添加失败'),
          content: Text('$e'),
          actions: [
            TextButton(
              onPressed: () => Navigator.of(context).pop(),
              child: const Text('继续扫描'),
            ),
          ],
        ),
      );
    }
  }

  // 用户码
  Future<void> _handleContact(QrRouteResult result) async {
    if (!mounted) return;
    try {
      final body = result.envelope!.body as UserContactBody;
      final addResult = await _contactService.addContact(
        address: body.address,
        contactName: body.contactName,
        selfAddress: widget.selfAddress,
      );
      if (!mounted) return;
      ScaffoldMessenger.of(context).showSnackBar(
        SnackBar(
          content: Text(
            addResult.created
                ? '已加入通讯录：${addResult.contact.displayNickname}'
                : '已更新通讯录：${addResult.contact.displayNickname}',
          ),
        ),
      );
      _popPage();
    } catch (e) {
      if (!mounted) return;
      await showDialog<void>(
        context: context,
        builder: (context) => AlertDialog(
          title: const Text('无法识别二维码'),
          content: Text('$e'),
          actions: [
            TextButton(
              onPressed: () => Navigator.of(context).pop(),
              child: const Text('继续扫描'),
            ),
          ],
        ),
      );
    }
  }

  // 未识别
  String get _hintText =>
      widget.customTitle ??
      switch (widget.mode) {
        QrScanMode.transfer => '扫描收款码',
        QrScanMode.contact => '扫描对方收款码',
        QrScanMode.raw => '扫描二维码',
        QrScanMode.dispatch => '扫描二维码',
      };

  String get _titleText =>
      widget.customTitle ??
      switch (widget.mode) {
        QrScanMode.transfer => '扫码支付',
        QrScanMode.contact => '扫码添加好友',
        QrScanMode.raw => '扫描二维码',
        QrScanMode.dispatch => '扫一扫',
      };

  Future<void> _showUnrecognized() async {
    if (!mounted) {
      return;
    }
    await showDialog<void>(
      context: context,
      builder: (context) => AlertDialog(
        title: const Text('无法识别二维码'),
        content: Text('请$_hintText。'),
        actions: [
          TextButton(
            onPressed: () => Navigator.of(context).pop(),
            child: const Text('确定'),
          ),
        ],
      ),
    );
  }

  @override
  Widget build(BuildContext context) {
    const double scanBoxSize = 240;
    // 扫描框偏移：向上移动 80 像素
    const double scanBoxOffsetY = -80;

    return Scaffold(
      appBar: AppBar(
        title: Text(_titleText),
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
              if (code == null || code.isEmpty) {
                return;
              }
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

          // 扫描框四角装饰（与遮罩使用相同像素偏移）
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

          // 提示文字（扫描框下方）
          Center(
            child: Transform.translate(
              offset: const Offset(0, scanBoxOffsetY + scanBoxSize / 2 + 24),
              child: Text(
                _hintText,
                style: const TextStyle(color: Colors.white70, fontSize: 14),
              ),
            ),
          ),

          // 底部工具栏：相册 + 手电筒
          Align(
            alignment: Alignment.bottomCenter,
            child: Padding(
              padding: const EdgeInsets.only(bottom: 60, left: 48, right: 48),
              child: Row(
                mainAxisAlignment: MainAxisAlignment.spaceBetween,
                children: [
                  // 相册图标
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
                  // 手电筒图标
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

// 扫描框半透明遮罩
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

// 扫描框四角装饰线
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

    // 左上
    canvas.drawLine(const Offset(0, 0), const Offset(cornerLen, 0), paint);
    canvas.drawLine(const Offset(0, 0), const Offset(0, cornerLen), paint);
    // 右上
    canvas.drawLine(Offset(w, 0), Offset(w - cornerLen, 0), paint);
    canvas.drawLine(Offset(w, 0), Offset(w, cornerLen), paint);
    // 左下
    canvas.drawLine(Offset(0, h), Offset(cornerLen, h), paint);
    canvas.drawLine(Offset(0, h), Offset(0, h - cornerLen), paint);
    // 右下
    canvas.drawLine(Offset(w, h), Offset(w - cornerLen, h), paint);
    canvas.drawLine(Offset(w, h), Offset(w, h - cornerLen), paint);
  }

  @override
  bool shouldRepaint(covariant CustomPainter oldDelegate) => false;
}
