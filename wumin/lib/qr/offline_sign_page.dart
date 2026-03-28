import 'dart:async';

import 'package:flutter/material.dart';
import 'package:image_picker/image_picker.dart';
import 'package:mobile_scanner/mobile_scanner.dart';
import 'package:qr_flutter/qr_flutter.dart';

import '../util/amount_format.dart';
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
  QrSignRequest? _request;
  QrSignResponse? _response;
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

  int _secondsLeft(QrSignRequest request) {
    final now = DateTime.now().millisecondsSinceEpoch ~/ 1000;
    final left = request.expiresAt - now;
    return left > 0 ? left : 0;
  }

  void _startCountdown(QrSignRequest request) {
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
    // 等 MobileScanner widget 重新挂载后再启动 controller，
    // 否则 camera preview 和 widget 绑定不上会白屏。
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
            offset: const Offset(0, scanBoxOffsetY + scanBoxSize / 2 + 24),
            child: Text(
              '扫描签名请求二维码\n当前钱包：${widget.wallet.walletName}',
              textAlign: TextAlign.center,
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
    );
  }

  /// fields value 转换（如 approve: true → 赞成）。
  static String _fieldValue(String key, String value) {
    if (key == 'approve') return value == 'true' ? '赞成' : '反对';
    return value;
  }

  /// 从 display.fields（List 格式）中按 key 查找 label。
  static String? _findFieldLabel(List<dynamic> fields, String key) {
    for (final field in fields) {
      if (field is Map && field['key']?.toString() == key) {
        return field['label']?.toString();
      }
    }
    return null;
  }

  Widget _buildTransactionDetails(
      QrSignRequest request, OfflineSignVerification verification) {
    final decoded = verification.decoded;
    final match = verification.displayMatch;

    final Widget statusBanner;
    switch (match) {
      case DisplayMatchStatus.matched:
        statusBanner = _buildBanner(
          color: Colors.green,
          text: '交易内容已独立验证，与摘要一致',
        );
      case DisplayMatchStatus.mismatched:
        statusBanner = _buildBanner(
          color: Colors.red,
          text: '警告：交易内容与摘要不符，禁止签名',
        );
      case DisplayMatchStatus.decodeFailed:
        statusBanner = _buildBanner(
          color: Colors.orange,
          text: '无法独立验证交易内容，以下信息来自请求方',
        );
    }

    final display = request.display;
    final actionLabel = display['action_label']?.toString() ??
        display['action']?.toString() ??
        '未知';

    final List<Widget> detailRows;
    if (decoded != null) {
      // 解码成功：使用解码结果展示，label 从 display.fields 中获取
      final displayFields = display['fields'];
      detailRows = [
        _detailRow('交易类型', actionLabel),
        ...decoded.fields.entries.map((e) {
          final label = (displayFields is List)
              ? _findFieldLabel(displayFields, e.key) ?? e.key
              : e.key;
          return _detailRow(label, _fieldValue(e.key, e.value));
        }),
      ];
    } else {
      // 解码失败：直接使用 display.fields 渲染
      detailRows = [
        _detailRow('交易类型', actionLabel),
      ];
      final fields = display['fields'];
      if (fields is List) {
        detailRows.addAll(
          fields.whereType<Map>().map((field) {
            final key = field['key']?.toString() ?? '';
            final label = field['label']?.toString() ?? key;
            final value = field['value']?.toString() ?? '';
            final format = field['format']?.toString();
            final displayValue = _fieldValue(key, value);
            return _detailRow(
              label,
              format == 'currency' ? AmountFormat.formatString(displayValue) : displayValue,
            );
          }),
        );
      }
    }

    return Column(
      crossAxisAlignment: CrossAxisAlignment.stretch,
      children: [
        statusBanner,
        const SizedBox(height: 12),
        Card(
          child: Padding(
            padding: const EdgeInsets.all(16),
            child: Column(
              crossAxisAlignment: CrossAxisAlignment.start,
              children: detailRows,
            ),
          ),
        ),
      ],
    );
  }

  Widget _buildBanner({required MaterialColor color, required String text}) {
    return Container(
      padding: const EdgeInsets.symmetric(horizontal: 12, vertical: 10),
      decoration: BoxDecoration(
        color: color.shade50,
        borderRadius: BorderRadius.circular(12),
        border: Border.all(color: color.shade200),
      ),
      child: Row(
        children: [
          Icon(
            color == Colors.green
                ? Icons.verified
                : color == Colors.red
                    ? Icons.dangerous
                    : Icons.warning_amber,
            color: color.shade700,
            size: 20,
          ),
          const SizedBox(width: 8),
          Expanded(
            child: Text(
              text,
              style: TextStyle(
                color: color.shade700,
                fontWeight: FontWeight.w600,
              ),
            ),
          ),
        ],
      ),
    );
  }

  Widget _detailRow(String label, String value) {
    return Padding(
      padding: const EdgeInsets.symmetric(vertical: 4),
      child: Row(
        crossAxisAlignment: CrossAxisAlignment.start,
        children: [
          Text(
            '$label  ',
            style: const TextStyle(
              color: Colors.black54,
              fontWeight: FontWeight.w500,
              fontSize: 13,
            ),
          ),
          Expanded(
            child: Text(
              value,
              style: const TextStyle(fontWeight: FontWeight.w600),
            ),
          ),
        ],
      ),
    );
  }

  Widget _buildRequestSummary(QrSignRequest request) {
    final expired = _remainingSeconds <= 0;
    final verification = _verification;
    final isMismatched =
        verification?.displayMatch == DisplayMatchStatus.mismatched;
    final isDecodeFailed =
        verification?.displayMatch == DisplayMatchStatus.decodeFailed;

    return ListView(
      padding: const EdgeInsets.all(16),
      children: [
        Container(
          padding: const EdgeInsets.symmetric(horizontal: 12, vertical: 10),
          decoration: BoxDecoration(
            color: expired ? Colors.red.shade50 : Colors.green.shade50,
            borderRadius: BorderRadius.circular(12),
          ),
          child: Text(
            expired ? '签名请求已过期，请重新扫描' : '签名请求有效期剩余：${_remainingSeconds}s',
            style: TextStyle(
              color: expired ? Colors.red.shade700 : Colors.green.shade700,
              fontWeight: FontWeight.w600,
            ),
          ),
        ),
        const SizedBox(height: 12),
        Card(
          child: Padding(
            padding: const EdgeInsets.all(16),
            child: Column(
              crossAxisAlignment: CrossAxisAlignment.start,
              children: [
                _detailRow('请求 ID', request.requestId),
                _detailRow('签名账户', request.account),
              ],
            ),
          ),
        ),
        const SizedBox(height: 12),
        if (verification != null)
          _buildTransactionDetails(request, verification),
        const SizedBox(height: 16),
        if (isDecodeFailed) ...[
          Container(
            width: double.infinity,
            padding: const EdgeInsets.all(12),
            decoration: BoxDecoration(
              color: Colors.red.shade50,
              borderRadius: BorderRadius.circular(12),
              border: Border.all(color: Colors.red.shade200),
            ),
            child: Text(
              '无法独立验证交易内容，禁止签名。请升级冷钱包后重试。',
              style: TextStyle(
                color: Colors.red.shade700,
                fontWeight: FontWeight.w600,
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
                onPressed: (_signing || expired || isMismatched || isDecodeFailed)
                    ? null
                    : _signRequest,
                child: Text(_signing ? '签名中...' : '确认签名'),
              ),
            ),
          ],
        ),
      ],
    );
  }

  Widget _buildResponseView(QrSignResponse response) {
    final responseJson = _qrSigner.encodeResponse(response);
    return ListView(
      padding: const EdgeInsets.all(16),
      children: [
        Container(
          padding: const EdgeInsets.symmetric(horizontal: 12, vertical: 10),
          decoration: BoxDecoration(
            color: Colors.green.shade50,
            borderRadius: BorderRadius.circular(12),
          ),
          child: Text(
            '签名已完成，请用在线手机扫描下方回执二维码',
            style: TextStyle(
              color: Colors.green.shade700,
              fontWeight: FontWeight.w600,
            ),
          ),
        ),
        const SizedBox(height: 20),
        Center(
          child: QrImageView(
            data: responseJson,
            version: QrVersions.auto,
            size: 240,
          ),
        ),
        const SizedBox(height: 16),
        Text(
          '请求 ID：${response.requestId}',
          textAlign: TextAlign.center,
        ),
        const SizedBox(height: 8),
        Text(
          '签名公钥：${_truncate(response.pubkey)}',
          textAlign: TextAlign.center,
          style: const TextStyle(color: Colors.black54),
        ),
        const SizedBox(height: 24),
        SizedBox(
          width: double.infinity,
          child: FilledButton(
            onPressed: () => Navigator.of(context).pop(),
            child: const Text('完成'),
          ),
        ),
      ],
    );
  }

  @override
  Widget build(BuildContext context) {
    final request = _request;
    final response = _response;
    return Scaffold(
      appBar: AppBar(
        title: const Text('扫码签名'),
        centerTitle: true,
      ),
      body: response != null
          ? _buildResponseView(response)
          : (request != null ? _buildRequestSummary(request) : _buildScanner()),
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
    canvas.drawRect(rect, clearPaint);
    canvas.restore();
  }

  @override
  bool shouldRepaint(covariant _ScanOverlayPainter oldDelegate) =>
      oldDelegate.scanBoxSize != scanBoxSize || oldDelegate.offsetY != offsetY;
}

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

    canvas.drawLine(const Offset(0, 0), const Offset(cornerLen, 0), paint);
    canvas.drawLine(const Offset(0, 0), const Offset(0, cornerLen), paint);
    canvas.drawLine(Offset(w, 0), Offset(w - cornerLen, 0), paint);
    canvas.drawLine(Offset(w, 0), Offset(w, cornerLen), paint);
    canvas.drawLine(Offset(0, h), Offset(cornerLen, h), paint);
    canvas.drawLine(Offset(0, h), Offset(0, h - cornerLen), paint);
    canvas.drawLine(Offset(w, h), Offset(w - cornerLen, h), paint);
    canvas.drawLine(Offset(w, h), Offset(w, h - cornerLen), paint);
  }

  @override
  bool shouldRepaint(covariant CustomPainter oldDelegate) => false;
}
