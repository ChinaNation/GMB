import 'dart:async';
import 'dart:convert';

import 'package:flutter/material.dart';
import 'package:image_picker/image_picker.dart';
import 'package:mobile_scanner/mobile_scanner.dart';
import 'package:qr_flutter/qr_flutter.dart';
import 'package:wuminapp_mobile/qr/login/login_models.dart';
import 'package:wuminapp_mobile/qr/login/login_service.dart';
import 'package:wuminapp_mobile/qr/contact/contact_qr_models.dart';
import 'package:wuminapp_mobile/qr/qr_router.dart';
import 'package:wuminapp_mobile/qr/transfer/transfer_qr_models.dart';
import 'package:wuminapp_mobile/user/user_service.dart';
import 'package:wuminapp_mobile/wallet/core/wallet_manager.dart';

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
  /// 扫码登录：仅识别登录码。
  login,

  /// 扫码支付：仅识别收款码 / 裸地址。
  transfer,

  /// 扫码添加好友：仅识别用户码。
  contact,
}

/// 统一扫码页。
///
/// 通过 [mode] 区分三种独立功能：
/// - [QrScanMode.login]    → 扫码登录
/// - [QrScanMode.transfer] → 扫码支付
/// - [QrScanMode.contact]  → 扫码添加好友
class QrScanPage extends StatefulWidget {
  const QrScanPage({
    super.key,
    required this.mode,
    this.walletIndex,
    this.selfAccountPubkeyHex,
  });

  /// 扫码模式。
  final QrScanMode mode;

  /// 指定钱包索引（登录签名用）。
  final int? walletIndex;

  /// 当前用户公钥（通讯录防自加用）。
  final String? selfAccountPubkeyHex;

  @override
  State<QrScanPage> createState() => _QrScanPageState();
}

class _QrScanPageState extends State<QrScanPage> {
  final MobileScannerController _controller = MobileScannerController();
  final QrRouter _router = QrRouter();
  final LoginService _loginService = LoginService();
  final UserContactService _contactService = UserContactService();
  bool _handled = false;
  bool _torchOn = false;

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
        case QrScanMode.login:
          // 扫码登录：仅处理登录码
          if (result.type == QrRouteType.login) {
            await _handleLogin(raw);
          } else {
            await _showUnrecognized();
          }
        case QrScanMode.transfer:
          // 扫码支付：仅处理收款码 / 裸地址
          if (result.type == QrRouteType.transfer) {
            _handleTransfer(result);
          } else if (result.type == QrRouteType.legacyAddress) {
            _handleLegacyAddress(result.extractedAddress!);
          } else {
            await _showUnrecognized();
          }
        case QrScanMode.contact:
          // 扫码添加好友：扫描收款码读取 name + to
          if (result.type == QrRouteType.transfer) {
            await _handleContactFromTransfer(result);
          } else if (result.type == QrRouteType.contact) {
            // 兼容旧版用户码
            await _handleContact(raw);
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
      if (mounted) {
        _handled = false;
        await _controller.start();
      }
    }
  }

  // ---------------------------------------------------------------------------
  // 登录码
  // ---------------------------------------------------------------------------

  Future<void> _handleLogin(String raw) async {
    LoginChallenge challenge;
    try {
      challenge = _loginService.parseChallenge(raw);
      await _loginService.validateSystemSignature(challenge);
    } catch (e) {
      if (!mounted) {
        return;
      }
      await showDialog<void>(
        context: context,
        builder: (context) => AlertDialog(
          title: const Text('无法识别登录二维码'),
          content: Text('$e'),
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

    if (!mounted) {
      return;
    }

    final shouldSign = await showDialog<bool>(
      context: context,
      builder: (context) => AlertDialog(
        title: Text('登录 ${_displaySystemName(challenge.system)}系统'),
        content: const Text('请确认后生成登录签名二维码。'),
        actions: [
          TextButton(
            onPressed: () => Navigator.of(context).pop(false),
            child: const Text('取消'),
          ),
          FilledButton(
            onPressed: () => Navigator.of(context).pop(true),
            child: const Text('签名并生成二维码'),
          ),
        ],
      ),
    );

    if (shouldSign == true && mounted) {
      await _signAndShowReceipt(challenge);
    }
  }

  Future<void> _signAndShowReceipt(LoginChallenge challenge) async {
    try {
      if (challenge.isExpired) {
        throw Exception('登录挑战已过期，请重新扫码');
      }
      final result = await _loginService.buildReceiptPayload(
        challenge,
        walletIndex: widget.walletIndex,
      );

      if (!mounted) {
        return;
      }

      final compact = jsonEncode(result);

      final goBack = await Navigator.of(context).push<bool>(
        MaterialPageRoute(
          builder: (_) => _LoginReceiptPage(
            compactPayload: compact,
            expiresAt: challenge.expiresAt,
          ),
        ),
      );
      if (goBack == true && mounted) {
        Navigator.of(context).pop();
      }
    } on WalletAuthException catch (e) {
      if (!mounted) {
        return;
      }
      await showDialog<void>(
        context: context,
        builder: (context) => AlertDialog(
          title: const Text('身份验证'),
          content: Text(e.message),
          actions: [
            TextButton(
              onPressed: () => Navigator.of(context).pop(),
              child: const Text('确定'),
            ),
          ],
        ),
      );
    } catch (e) {
      if (!mounted) {
        return;
      }
      await showDialog<void>(
        context: context,
        builder: (context) => AlertDialog(
          title: const Text('登录回执生成失败'),
          content: Text('$e'),
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

  // ---------------------------------------------------------------------------
  // 收款码
  // ---------------------------------------------------------------------------

  void _handleTransfer(QrRouteResult result) {
    if (!mounted) {
      return;
    }
    try {
      final payload = TransferQrPayload.fromJson(result.jsonData!);
      Navigator.of(context).pop(QrScanTransferResult(
        toAddress: payload.to,
        amount: payload.amount,
        symbol: payload.symbol,
        memo: payload.memo,
        bank: payload.bank,
      ));
    } catch (e) {
      showDialog<void>(
        context: context,
        builder: (context) => AlertDialog(
          title: const Text('收款码解析失败'),
          content: Text('$e'),
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

  // ---------------------------------------------------------------------------
  // 裸地址（向后兼容）
  // ---------------------------------------------------------------------------

  void _handleLegacyAddress(String address) {
    if (!mounted) {
      return;
    }
    Navigator.of(context).pop(QrScanTransferResult(toAddress: address));
  }

  // ---------------------------------------------------------------------------
  // 收款码 → 添加通讯录
  // ---------------------------------------------------------------------------

  Future<void> _handleContactFromTransfer(QrRouteResult result) async {
    if (!mounted) return;
    try {
      final payload = TransferQrPayload.fromJson(result.jsonData!);
      final name = payload.name?.trim() ?? '';
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
        address: payload.to,
        name: name,
        selfAddress: widget.selfAccountPubkeyHex,
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
      Navigator.of(context).pop();
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

  // ---------------------------------------------------------------------------
  // 用户码（兼容旧版）
  // ---------------------------------------------------------------------------

  Future<void> _handleContact(String raw) async {
    if (!mounted) return;
    try {
      final payload = ContactQrPayload.parse(raw);
      final result = await _contactService.addContact(
        address: payload.address,
        name: payload.name,
        selfAddress: widget.selfAccountPubkeyHex,
      );
      if (!mounted) return;
      ScaffoldMessenger.of(context).showSnackBar(
        SnackBar(
          content: Text(
            result.created
                ? '已加入通讯录：${result.contact.displayNickname}'
                : '已更新通讯录：${result.contact.displayNickname}',
          ),
        ),
      );
      Navigator.of(context).pop();
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

  // ---------------------------------------------------------------------------
  // 扫码签名提示
  // ---------------------------------------------------------------------------

  Future<void> _showQrSignHint() async {
    if (!mounted) return;
    await showDialog<void>(
      context: context,
      builder: (context) => AlertDialog(
        title: const Text('签名请求'),
        content: const Text('这是一个冷钱包签名请求二维码。\n请在转账页面发起交易后，通过签名会话页面扫描回执。'),
        actions: [
          TextButton(
            onPressed: () => Navigator.of(context).pop(),
            child: const Text('确定'),
          ),
        ],
      ),
    );
  }

  // ---------------------------------------------------------------------------
  // 未识别
  // ---------------------------------------------------------------------------

  String get _hintText => switch (widget.mode) {
        QrScanMode.login => '扫描登录码',
        QrScanMode.transfer => '扫描收款码',
        QrScanMode.contact => '扫描对方收款码',
      };

  String get _titleText => switch (widget.mode) {
        QrScanMode.login => '扫码登录',
        QrScanMode.transfer => '扫码支付',
        QrScanMode.contact => '扫码添加好友',
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

  String _displaySystemName(String system) {
    if (system.toLowerCase() == 'sfid') {
      return 'SFID';
    }
    return system.toUpperCase();
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
    canvas.drawRect(rect, clearPaint);
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
    const cornerLen = 24.0;
    const strokeWidth = 4.0;

    final paint = Paint()
      ..color = Colors.green
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

// -----------------------------------------------------------------------------
// 登录回执展示页（私有，仅 QrScanPage 内部使用）
// -----------------------------------------------------------------------------

class _LoginReceiptPage extends StatefulWidget {
  const _LoginReceiptPage({
    required this.compactPayload,
    required this.expiresAt,
  });

  final String compactPayload;
  final int expiresAt;

  @override
  State<_LoginReceiptPage> createState() => _LoginReceiptPageState();
}

class _LoginReceiptPageState extends State<_LoginReceiptPage> {
  Timer? _timer;
  late int _remainingSeconds;

  @override
  void initState() {
    super.initState();
    _remainingSeconds = _secondsLeft();
    _timer = Timer.periodic(const Duration(seconds: 1), (_) {
      if (!mounted) {
        return;
      }
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
    final left = widget.expiresAt - now;
    return left > 0 ? left : 0;
  }

  @override
  Widget build(BuildContext context) {
    final expired = _remainingSeconds <= 0;
    return Scaffold(
      appBar: AppBar(
        title: const Text('登录回执'),
        centerTitle: true,
      ),
      body: ListView(
        padding: const EdgeInsets.all(16),
        children: [
          Container(
            width: double.infinity,
            padding: const EdgeInsets.symmetric(horizontal: 10, vertical: 6),
            decoration: BoxDecoration(
              color: expired ? Colors.red.shade50 : Colors.green.shade50,
              borderRadius: BorderRadius.circular(8),
            ),
            child: Text(
              expired ? '该回执已过期，请重新扫码' : '回执有效期剩余：${_remainingSeconds}s',
              style: TextStyle(
                color: expired ? Colors.red.shade700 : Colors.green.shade700,
                fontWeight: FontWeight.w600,
              ),
            ),
          ),
          const SizedBox(height: 12),
          Center(
            child: QrImageView(
              data: widget.compactPayload,
              version: QrVersions.auto,
              size: 220,
              errorStateBuilder: (cxt, err) {
                return Container(
                  width: 220,
                  height: 220,
                  padding: const EdgeInsets.all(10),
                  decoration: BoxDecoration(
                    color: Colors.red.shade50,
                    borderRadius: BorderRadius.circular(8),
                  ),
                  child: Center(
                    child: Text(
                      '回执二维码渲染失败：$err',
                      style: TextStyle(color: Colors.red.shade700),
                    ),
                  ),
                );
              },
            ),
          ),
          const SizedBox(height: 12),
          const SizedBox(height: 20),
          Row(
            children: [
              Expanded(
                child: OutlinedButton(
                  onPressed: () => Navigator.of(context).pop(false),
                  child: const Text('重新扫码'),
                ),
              ),
              const SizedBox(width: 12),
              Expanded(
                child: FilledButton(
                  onPressed: () => Navigator.of(context).pop(true),
                  child: const Text('完成'),
                ),
              ),
            ],
          ),
        ],
      ),
    );
  }
}
