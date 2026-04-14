import 'dart:async';

import 'package:flutter/material.dart';
import 'package:image_picker/image_picker.dart';
import 'package:mobile_scanner/mobile_scanner.dart';
import 'package:qr_flutter/qr_flutter.dart';

import '../ui/app_theme.dart';
import '../signer/action_labels.dart';
import '../signer/offline_sign_service.dart';
import '../signer/qr_signer.dart';
import '../util/screenshot_guard.dart';
import '../wallet/wallet_manager.dart';

/// 离线签名页面。
///
/// 扫描在线手机展示的签名请求二维码，
/// 在本机完成签名后展示回执二维码。
class OfflineSignPage extends StatefulWidget {
  const OfflineSignPage({
    super.key,
    required this.wallet,
    this.initialCode,
  });

  final WalletProfile wallet;
  final String? initialCode;

  @override
  State<OfflineSignPage> createState() => _OfflineSignPageState();
}

class _OfflineSignPageState extends State<OfflineSignPage> {
  static const double scanBoxSize = 260;
  static const double scanBoxOffsetY = -40;

  late final MobileScannerController _controller;
  final OfflineSignService _offlineSignService = OfflineSignService();
  final QrSigner _qrSigner = QrSigner();

  Timer? _timer;
  bool _handled = false;
  bool _signing = false;
  bool _torchOn = false;
  SignRequestEnvelope? _request;
  SignResponseEnvelope? _response;
  OfflineSignVerification? _verification;
  int _remainingSeconds = 0;

  @override
  void initState() {
    super.initState();
    ScreenshotGuard.enable();
    _controller = MobileScannerController(
      detectionSpeed: DetectionSpeed.normal,
      facing: CameraFacing.back,
      torchEnabled: false,
    );
    final code = widget.initialCode;
    if (code != null) {
      WidgetsBinding.instance.addPostFrameCallback((_) => _handleCode(code));
    }
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

  @override
  void dispose() {
    _timer?.cancel();
    _controller.dispose();
    ScreenshotGuard.disable();
    super.dispose();
  }

  int _secondsLeft(SignRequestEnvelope request) {
    final now = DateTime.now().millisecondsSinceEpoch ~/ 1000;
    final left = (request.expiresAt ?? 0) - now;
    return left > 0 ? left : 0;
  }

  void _startCountdown(SignRequestEnvelope request) {
    _timer?.cancel();
    _remainingSeconds = _secondsLeft(request);
    _timer = Timer.periodic(const Duration(seconds: 1), (_) {
      if (!mounted) return;
      setState(() {
        _remainingSeconds = _secondsLeft(request);
      });
    });
  }

  Future<void> _handleCode(String raw) async {
    if (_handled) return;
    _handled = true;
    await _controller.stop();

    try {
      final request = _offlineSignService.parseRequest(raw);
      final verification = _offlineSignService.verifyPayload(request);
      if (!mounted) return;
      setState(() {
        _request = request;
        _response = null;
        _verification = verification;
      });
      _startCountdown(request);
    } on QrSignException catch (e) {
      if (!mounted) return;
      await showDialog<void>(
        context: context,
        builder: (context) => AlertDialog(
          title: const Text('签名请求解析失败'),
          content: Text(e.message),
          actions: [
            TextButton(
              onPressed: () => Navigator.of(context).pop(),
              child: const Text('继续扫描'),
            ),
          ],
        ),
      );
      if (mounted) {
        await _controller.start();
      }
    } finally {
      _handled = false;
    }
  }

  Future<void> _resetToScanner() async {
    _timer?.cancel();
    if (!mounted) return;
    setState(() {
      _request = null;
      _response = null;
      _verification = null;
      _remainingSeconds = 0;
      _signing = false;
    });
    WidgetsBinding.instance.addPostFrameCallback((_) async {
      if (mounted) {
        await _controller.start();
      }
    });
  }

  Future<void> _signRequest() async {
    final request = _request;
    if (request == null) return;
    if (_remainingSeconds <= 0) {
      ScaffoldMessenger.of(context).showSnackBar(
        const SnackBar(content: Text('签名请求已过期，请重新扫描')),
      );
      return;
    }

    setState(() {
      _signing = true;
    });
    try {
      final response = await _offlineSignService.signParsedRequest(
        walletIndex: widget.wallet.walletIndex,
        request: request,
      );
      if (!mounted) return;
      setState(() {
        _response = response;
      });
    } on OfflineSignException catch (e) {
      if (!mounted) return;
      _showError('离线签名失败', e.message);
    } on WalletAuthException catch (e) {
      if (!mounted) return;
      _showError('身份验证', e.message);
    } catch (e) {
      if (!mounted) return;
      _showError('离线签名失败', '$e');
    } finally {
      if (mounted) {
        setState(() {
          _signing = false;
        });
      }
    }
  }

  Future<void> _showError(String title, String message) async {
    await showDialog<void>(
      context: context,
      builder: (context) => AlertDialog(
        title: Text(title),
        content: Text(message),
        actions: [
          TextButton(
            onPressed: () => Navigator.of(context).pop(),
            child: const Text('确定'),
          ),
        ],
      ),
    );
  }

  String _truncate(String text, {int head = 12, int tail = 8}) {
    if (text.length <= head + tail + 3) return text;
    return '${text.substring(0, head)}...${text.substring(text.length - tail)}';
  }

  Widget _buildScanner() {
    return Stack(
      fit: StackFit.expand,
      children: [
        MobileScanner(
          controller: _controller,
          onDetect: (capture) {
            final code = capture.barcodes.first.rawValue;
            if (code == null || code.isEmpty) return;
            _handleCode(code);
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
            child: Text(
              '扫描签名请求二维码\n当前钱包：${widget.wallet.walletName}',
              textAlign: TextAlign.center,
              style: const TextStyle(
                color: Colors.white60,
                fontSize: 14,
                letterSpacing: 0.3,
              ),
            ),
          ),
        ),

        // 底部工具栏
        Align(
          alignment: Alignment.bottomCenter,
          child: Container(
            margin: const EdgeInsets.only(bottom: 48, left: 48, right: 48),
            padding:
                const EdgeInsets.symmetric(horizontal: 24, vertical: 16),
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
                Container(width: 1, height: 32, color: AppTheme.border),
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
          Icon(icon, size: 26, color: active ? AppTheme.gold : Colors.white),
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

  /// fields value 转换（如 approve: true → 赞成）。
  static String _fieldValue(String key, String value) {
    if (key == 'approve') return value == 'true' ? '赞成' : '反对';
    return value;
  }

  Widget _buildTransactionDetails(
      SignRequestEnvelope request, OfflineSignVerification verification) {
    final decoded = verification.decoded;
    final match = verification.displayMatch;

    final Widget statusBanner;
    switch (match) {
      case DisplayMatchStatus.matched:
        statusBanner = _buildBanner(
          color: AppTheme.success,
          icon: Icons.verified_rounded,
          text: '交易内容已独立验证,与摘要一致',
        );
      case DisplayMatchStatus.mismatched:
        statusBanner = _buildBanner(
          color: AppTheme.danger,
          icon: Icons.dangerous_rounded,
          text: '警告:交易内容与摘要不符,禁止签名',
        );
      case DisplayMatchStatus.decodeFailed:
        statusBanner = _buildBanner(
          color: AppTheme.warning,
          icon: Icons.warning_amber_rounded,
          text: '无法独立验证交易内容,以下信息来自请求方',
        );
    }

    final display = request.body.display;
    final actionLabel = actionLabels[display.action] ?? display.action;

    final List<Widget> detailRows;
    if (decoded != null) {
      detailRows = [
        _detailRow('交易类型', actionLabel),
        ...decoded.fields.entries.map((e) {
          // 优先从 display.fields 中找中文标签
          final displayLabel = display.fields
              .where((f) => f.key == e.key)
              .map((f) => f.label)
              .firstOrNull;
          return _detailRow(
              displayLabel ?? e.key, _fieldValue(e.key, e.value));
        }),
      ];
    } else {
      detailRows = [
        _detailRow('交易类型', actionLabel),
        ...display.fields.map(
          (f) => _detailRow(f.label, _fieldValue(f.label, f.value)),
        ),
      ];
    }

    return Column(
      crossAxisAlignment: CrossAxisAlignment.stretch,
      children: [
        statusBanner,
        const SizedBox(height: 12),
        Container(
          padding: const EdgeInsets.all(16),
          decoration: AppTheme.cardDecoration(),
          child: Column(
            crossAxisAlignment: CrossAxisAlignment.start,
            children: detailRows,
          ),
        ),
      ],
    );
  }

  Widget _buildBanner({
    required Color color,
    required IconData icon,
    required String text,
  }) {
    return Container(
      padding: const EdgeInsets.symmetric(horizontal: 14, vertical: 12),
      decoration: AppTheme.bannerDecoration(color),
      child: Row(
        children: [
          Icon(icon, color: color, size: 20),
          const SizedBox(width: 10),
          Expanded(
            child: Text(
              text,
              style: TextStyle(
                color: color,
                fontWeight: FontWeight.w600,
                fontSize: 13,
              ),
            ),
          ),
        ],
      ),
    );
  }

  Widget _detailRow(String label, String value) {
    return Padding(
      padding: const EdgeInsets.symmetric(vertical: 6),
      child: Row(
        crossAxisAlignment: CrossAxisAlignment.start,
        children: [
          SizedBox(
            width: 80,
            child: Text(
              label,
              style: const TextStyle(
                color: AppTheme.textSecondary,
                fontWeight: FontWeight.w500,
                fontSize: 13,
              ),
            ),
          ),
          const SizedBox(width: 8),
          Expanded(
            child: Text(
              value,
              style: const TextStyle(
                fontWeight: FontWeight.w600,
                color: AppTheme.textPrimary,
                fontSize: 14,
              ),
            ),
          ),
        ],
      ),
    );
  }

  Widget _buildRequestSummary(SignRequestEnvelope request) {
    final expired = _remainingSeconds <= 0;
    final verification = _verification;
    final isMismatched =
        verification?.displayMatch == DisplayMatchStatus.mismatched;
    final displayAction = request.body.display.action;
    const allowedHashedActions = {
      'developer_direct_upgrade',
      'propose_runtime_upgrade',
      'activate_admin',
      'propose_institution_rate',
      'vote_institution_rate',
      'propose_safety_fund_transfer',
      'vote_safety_fund_transfer',
      'propose_sweep_to_main',
      'vote_sweep_to_main',
      'propose_create',
      'propose_create_personal',
      'propose_transfer',
      'vote_transfer',
      'joint_vote',
    };
    final isDecodeFailed =
        verification?.displayMatch == DisplayMatchStatus.decodeFailed &&
        !allowedHashedActions.contains(displayAction);

    return ListView(
      padding: const EdgeInsets.all(16),
      children: [
        // 有效期横幅
        Container(
          padding: const EdgeInsets.symmetric(horizontal: 14, vertical: 12),
          decoration: AppTheme.bannerDecoration(
              expired ? AppTheme.danger : AppTheme.success),
          child: Row(
            children: [
              Icon(
                expired ? Icons.timer_off_rounded : Icons.timer_rounded,
                color: expired ? AppTheme.danger : AppTheme.success,
                size: 18,
              ),
              const SizedBox(width: 8),
              Text(
                expired
                    ? '签名请求已过期，请重新扫描'
                    : '签名请求有效期剩余：${_remainingSeconds}s',
                style: TextStyle(
                  color: expired ? AppTheme.danger : AppTheme.success,
                  fontWeight: FontWeight.w600,
                  fontSize: 13,
                ),
              ),
            ],
          ),
        ),
        const SizedBox(height: 12),
        // 请求基本信息
        Container(
          padding: const EdgeInsets.all(16),
          decoration: AppTheme.cardDecoration(),
          child: Column(
            crossAxisAlignment: CrossAxisAlignment.start,
            children: [
              _detailRow('请求 ID', request.id ?? ''),
              _detailRow('签名账户', request.body.address),
            ],
          ),
        ),
        const SizedBox(height: 12),
        if (verification != null)
          _buildTransactionDetails(request, verification),
        const SizedBox(height: 16),
        if (isDecodeFailed) ...[
          Container(
            width: double.infinity,
            padding: const EdgeInsets.all(14),
            decoration: AppTheme.bannerDecoration(AppTheme.danger),
            child: const Text(
              '无法独立验证交易内容，禁止签名。请升级冷钱包后重试。',
              style: TextStyle(
                color: AppTheme.danger,
                fontWeight: FontWeight.w600,
                fontSize: 13,
              ),
            ),
          ),
          const SizedBox(height: 8),
        ],
        Row(
          children: [
            Expanded(
              child: OutlinedButton(
                onPressed: _resetToScanner,
                child: const Text('重新扫描'),
              ),
            ),
            const SizedBox(width: 12),
            Expanded(
              child: FilledButton(
                onPressed:
                    (_signing || expired || isMismatched || isDecodeFailed)
                        ? null
                        : _signRequest,
                child: _signing
                    ? const SizedBox(
                        width: 20,
                        height: 20,
                        child: CircularProgressIndicator(
                          strokeWidth: 2,
                          color: Colors.white,
                        ),
                      )
                    : const Text('确认签名'),
              ),
            ),
          ],
        ),
      ],
    );
  }

  Widget _buildResponseView(SignResponseEnvelope response) {
    final responseJson = _qrSigner.encodeResponse(response);
    return ListView(
      padding: const EdgeInsets.all(16),
      children: [
        // 成功横幅
        _buildBanner(
          color: AppTheme.success,
          icon: Icons.check_circle_rounded,
          text: '签名已完成，请用在线手机扫描下方回执二维码',
        ),
        const SizedBox(height: 24),
        // QR 码容器
        Center(
          child: Container(
            padding: const EdgeInsets.all(20),
            decoration: BoxDecoration(
              color: Colors.white,
              borderRadius: BorderRadius.circular(AppTheme.radiusLg),
              boxShadow: [
                BoxShadow(
                  color: AppTheme.primary.withAlpha(20),
                  blurRadius: 20,
                  offset: const Offset(0, 8),
                ),
              ],
            ),
            child: QrImageView(
              data: responseJson,
              version: QrVersions.auto,
              size: 360,
              errorCorrectionLevel: QrErrorCorrectLevel.M,
              eyeStyle: const QrEyeStyle(
                eyeShape: QrEyeShape.square,
                color: AppTheme.primaryDark,
              ),
              dataModuleStyle: const QrDataModuleStyle(
                dataModuleShape: QrDataModuleShape.square,
                color: AppTheme.primaryDark,
              ),
            ),
          ),
        ),
        const SizedBox(height: 20),
        // 信息
        Container(
          padding: const EdgeInsets.all(16),
          decoration: AppTheme.cardDecoration(),
          child: Column(
            children: [
              _detailRow('请求 ID', response.id ?? ''),
              _detailRow('签名公钥', _truncate(response.body.pubkey)),
            ],
          ),
        ),
        const SizedBox(height: 24),
        FilledButton(
          onPressed: () => Navigator.of(context).pop(),
          child: const Text('完成'),
        ),
      ],
    );
  }

  @override
  Widget build(BuildContext context) {
    final request = _request;
    final response = _response;
    return Scaffold(
      backgroundColor:
          (response != null || request != null) ? null : Colors.black,
      appBar: AppBar(
        backgroundColor: Colors.transparent,
        title: const Text('扫码签名'),
        centerTitle: true,
      ),
      body: response != null
          ? _buildResponseView(response)
          : (request != null
              ? _buildRequestSummary(request)
              : _buildScanner()),
    );
  }
}

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
