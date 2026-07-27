import 'package:flutter/material.dart';
import 'package:image_picker/image_picker.dart';
import 'package:mobile_scanner/mobile_scanner.dart';

import '../qr/qr_protocols.dart';
import '../qr/envelope.dart';
import '../qr/bodies/sign_request_body.dart';
import '../wallet/wallet_manager.dart';
import 'app_theme.dart';
import 'login_sign_page.dart';
import 'offline_sign_page.dart';
import 'scan_overlay.dart';

/// 全局扫码页面（对准框 + 相册 + 手电筒）。
///
/// 签名主体是账户：扫到签名请求后，按 QR 的 signerPublicKey 在**全设备账户**中
/// 自动定位目标账户，找到则跳转对应签名页，本设备无此账户则拒绝并提示。
class ScanPage extends StatefulWidget {
  const ScanPage({super.key});

  @override
  State<ScanPage> createState() => _ScanPageState();
}

class _ScanPageState extends State<ScanPage> {
  late final MobileScannerController _controller;
  final WalletManager _walletManager = WalletManager();
  bool _handled = false;
  bool _torchOn = false;

  @override
  void initState() {
    super.initState();
    _controller = MobileScannerController(
      detectionSpeed: DetectionSpeed.normal,
      facing: CameraFacing.back,
      torchEnabled: false,
    );
  }

  @override
  void dispose() {
    _controller.dispose();
    super.dispose();
  }

  Future<void> _toggleTorch() async {
    await _controller.toggleTorch();
    setState(() => _torchOn = !_torchOn);
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

  /// 单次解析签名请求信封;非签名请求返回 null。
  SignRequestBody? _parseSignRequest(String raw) {
    try {
      final env = QrEnvelope.parse(raw);
      final body = env.body;
      if (env.kind == QrKind.signRequest && body is SignRequestBody) {
        return body;
      }
    } catch (_) {}
    return null;
  }

  Future<void> _handleCode(String raw) async {
    if (_handled) return;
    _handled = true;
    await _controller.stop();
    if (!mounted) return;

    // 只解析一次:signerPublicKey(目标账户)与 action(登录/普通)同源取自 body,
    // 避免两套解析逻辑漂移。
    final body = _parseSignRequest(raw);
    if (body == null) {
      await _showErrorAndResume('无法识别签名请求二维码');
      return;
    }

    final account =
        await _walletManager.getAccountByAccountId(body.signerPublicKeyHex);
    if (!mounted) return;
    if (account == null) {
      await _showErrorAndResume('本设备没有该签名请求指定的账户，无法签名');
      return;
    }

    final wallet = await _walletManager.getWalletByMasterId(account.masterId);
    if (!mounted) return;
    final walletName = wallet?.walletName ?? '钱包';

    final isLogin = body.action == QrActions.login;
    await Navigator.of(context).push(
      MaterialPageRoute(
        builder: (_) => isLogin
            ? LoginSignPage(account: account, walletName: walletName, raw: raw)
            : OfflineSignPage(
                account: account, walletName: walletName, raw: raw),
      ),
    );

    if (!mounted) return;
    Navigator.of(context).pop();
  }

  Future<void> _showErrorAndResume(String message) async {
    await showDialog<void>(
      context: context,
      builder: (context) => AlertDialog(
        title: const Text('无法签名'),
        content: Text(message),
        actions: [
          TextButton(
            onPressed: () => Navigator.of(context).pop(),
            child: const Text('继续扫描'),
          ),
        ],
      ),
    );
    if (!mounted) return;
    _handled = false;
    await _controller.start();
  }

  @override
  Widget build(BuildContext context) {
    return Scaffold(
      backgroundColor: Colors.black,
      appBar: AppBar(
        backgroundColor: Colors.transparent,
        title: const Text('扫码签名'),
        centerTitle: true,
      ),
      body: Stack(
        fit: StackFit.expand,
        children: [
          MobileScanner(
            controller: _controller,
            onDetect: (capture) async {
              final code = capture.barcodes.first.rawValue;
              if (code == null || code.isEmpty) return;
              await _handleCode(code);
            },
          ),
          CustomPaint(
            painter: ScanOverlayPainter(
                scanBoxSize: scanBoxSize, offsetY: scanBoxOffsetY),
            child: const SizedBox.expand(),
          ),
          Center(
            child: Transform.translate(
              offset: const Offset(0, scanBoxOffsetY),
              child: SizedBox(
                width: scanBoxSize,
                height: scanBoxSize,
                child: CustomPaint(painter: ScanCornerPainter()),
              ),
            ),
          ),
          Center(
            child: Transform.translate(
              offset: const Offset(0, scanBoxOffsetY + scanBoxSize / 2 + 28),
              child: const Text(
                '扫描签名请求二维码\n设备将自动匹配对应账户',
                textAlign: TextAlign.center,
                style: TextStyle(
                    color: Colors.white60, fontSize: 14, letterSpacing: 0.3),
              ),
            ),
          ),
          Align(
            alignment: Alignment.bottomCenter,
            child: Container(
              margin: const EdgeInsets.only(bottom: 48, left: 48, right: 48),
              padding: const EdgeInsets.symmetric(horizontal: 24, vertical: 16),
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
      ),
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
          Text(label,
              style: TextStyle(
                  color: active ? AppTheme.gold : Colors.white70,
                  fontSize: 12)),
        ],
      ),
    );
  }
}
