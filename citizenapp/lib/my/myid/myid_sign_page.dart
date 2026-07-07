// 公民身份上链热钱包扫码签名页。
//
// 公民到链上中国办理现场后:
// 1. 打开此页,摄像头扫描 OnChina 平台的 sign_request 二维码(action=2)
// 2. 独立解码投票/参选身份 SCALE 载荷并展示中文字段,解不开拒签
// 3. 验证载荷内钱包公钥 == 请求公钥 == 当前电子护照钱包
// 4. 公民确认字段后,对 `blake2_256(GMB || 0x10 || payload)` 签名
//    (经 QrSigner.signingBytesForHex,对齐 primitives::sign::OP_SIGN_CITIZEN_IDENTITY)
// 5. 构造 sign_response envelope,展示签名响应二维码,等管理端扫描

import 'dart:typed_data';

import 'package:flutter/material.dart';
import 'package:mobile_scanner/mobile_scanner.dart';
import 'package:qr_flutter/qr_flutter.dart';
import 'package:citizenapp/my/myid/voting_identity_payload.dart';
import 'package:citizenapp/qr/qr_protocols.dart';
import 'package:citizenapp/signer/qr_signer.dart';
import 'package:citizenapp/ui/app_theme.dart';
import 'package:citizenapp/wallet/core/wallet_manager.dart';

class MyIdSignPage extends StatefulWidget {
  const MyIdSignPage({super.key, required this.wallet});

  final WalletProfile wallet;

  @override
  State<MyIdSignPage> createState() => _MyIdSignPageState();
}

enum _PageStep { scanning, confirm, signing, showResponse }

class _MyIdSignPageState extends State<MyIdSignPage> {
  static const double _scanBoxSize = 260;

  _PageStep _step = _PageStep.scanning;
  SignRequestEnvelope? _request;
  VotingIdentityConsentPayload? _decoded;
  String? _responseJson;
  String? _errorMessage;
  bool _processing = false;
  final MobileScannerController _scannerController = MobileScannerController();

  @override
  void dispose() {
    _scannerController.dispose();
    super.dispose();
  }

  Future<void> _onDetect(BarcodeCapture capture) async {
    if (_processing || _step != _PageStep.scanning) return;
    final barcode = capture.barcodes.firstOrNull;
    if (barcode == null || barcode.rawValue == null) return;
    final raw = barcode.rawValue!.trim();
    if (raw.isEmpty) return;

    setState(() {
      _processing = true;
      _errorMessage = null;
    });

    try {
      final qrSigner = QrSigner();
      final request = qrSigner.parseRequest(raw);
      final decoded = _verifyCitizenIdentityRequest(request);

      if (!mounted) return;
      setState(() {
        _request = request;
        _decoded = decoded;
        _step = _PageStep.confirm;
        _processing = false;
      });
    } on QrSignException catch (e) {
      if (!mounted) return;
      setState(() {
        _errorMessage = e.message;
        _step = _PageStep.scanning;
        _processing = false;
      });
    } catch (e) {
      if (!mounted) return;
      setState(() {
        _errorMessage = e.toString();
        _step = _PageStep.scanning;
        _processing = false;
      });
    }
  }

  /// 两色识别:action、载荷解码、公钥三方一致才允许进入确认页。
  VotingIdentityConsentPayload _verifyCitizenIdentityRequest(
    SignRequestEnvelope request,
  ) {
    final body = request.body;
    if (body.action != QrActions.citizenIdentity) {
      throw Exception('只能签名公民链上身份确认请求');
    }
    final expectedPubkey = '0x${widget.wallet.pubkeyHex}'.toLowerCase();
    final requestPubkey = body.pubkeyHex.toLowerCase();
    if (requestPubkey != expectedPubkey) {
      throw Exception('签名请求中的公钥与当前钱包不一致');
    }
    final decoded = VotingIdentityConsentPayload.decode(
      Uint8List.fromList(body.payloadBytes),
    );
    if (decoded == null) {
      throw Exception('无法独立验证公民身份载荷,禁止签名');
    }
    if (decoded.walletPubkeyHex != requestPubkey) {
      throw Exception('身份载荷中的钱包公钥与签名请求不一致');
    }
    return decoded;
  }

  Future<void> _sign() async {
    final request = _request;
    if (request == null || _processing) return;
    final now = DateTime.now().millisecondsSinceEpoch ~/ 1000;
    if ((request.expiresAt ?? 0) < now) {
      setState(() {
        _errorMessage = '签名请求已过期,请重新扫描';
        _step = _PageStep.scanning;
        _request = null;
        _decoded = null;
      });
      return;
    }

    setState(() {
      _processing = true;
      _step = _PageStep.signing;
    });

    try {
      final message = QrSigner.signingBytesForHex(
        payloadHex: request.body.payloadHex,
        action: request.body.action,
      );
      final walletManager = WalletManager();
      final signature = await walletManager.signWithWallet(
        widget.wallet.walletIndex,
        message,
      );

      // 统一通过 QrSigner 构造 sign_response,避免页面私自拼接二维码结构。
      final qrSigner = QrSigner();
      final sigHex =
          '0x${signature.map((b) => b.toRadixString(16).padLeft(2, '0')).join()}';
      final responseEnvelope =
          qrSigner.buildResponse(request: request, signatureHex: sigHex);
      final json = qrSigner.encodeResponse(responseEnvelope);

      if (!mounted) return;
      setState(() {
        _responseJson = json;
        _step = _PageStep.showResponse;
        _processing = false;
      });
    } catch (e) {
      if (!mounted) return;
      setState(() {
        _errorMessage = e.toString();
        _step = _PageStep.scanning;
        _request = null;
        _decoded = null;
        _processing = false;
      });
    }
  }

  void _cancelConfirm() {
    setState(() {
      _request = null;
      _decoded = null;
      _step = _PageStep.scanning;
    });
  }

  @override
  Widget build(BuildContext context) {
    return Scaffold(
      appBar: AppBar(
        title: const Text('扫码签名'),
        centerTitle: true,
      ),
      body: Padding(
        padding: const EdgeInsets.all(16),
        child: switch (_step) {
          _PageStep.scanning => _buildScanning(),
          _PageStep.confirm => _buildConfirm(),
          _PageStep.signing => _buildSigning(),
          _PageStep.showResponse => _buildShowResponse(),
        },
      ),
    );
  }

  Widget _buildScanning() {
    return Column(
      children: [
        // 说明
        Container(
          width: double.infinity,
          padding: const EdgeInsets.symmetric(horizontal: 12, vertical: 8),
          decoration: AppTheme.bannerDecoration(AppTheme.info),
          child: const Text(
            '请扫描链上中国平台的公民身份签名码',
            textAlign: TextAlign.center,
            style: TextStyle(color: AppTheme.info, fontWeight: FontWeight.w600),
          ),
        ),
        if (_errorMessage != null) ...[
          const SizedBox(height: 8),
          Container(
            width: double.infinity,
            padding: const EdgeInsets.symmetric(horizontal: 12, vertical: 8),
            decoration: AppTheme.bannerDecoration(AppTheme.danger),
            child: Text(
              _errorMessage!,
              style: const TextStyle(
                  color: AppTheme.danger, fontWeight: FontWeight.w600),
            ),
          ),
        ],
        const SizedBox(height: 24),
        Center(
          child: SizedBox(
            width: _scanBoxSize,
            height: _scanBoxSize,
            // 扫码签名页必须是实际正方形相机框，不能再用整块矩形相机画面。
            child: ClipRRect(
              borderRadius: BorderRadius.circular(12),
              child: Stack(
                fit: StackFit.expand,
                children: [
                  MobileScanner(
                    controller: _scannerController,
                    fit: BoxFit.cover,
                    onDetect: _onDetect,
                  ),
                  CustomPaint(
                    painter: _MyIdScanCornerPainter(),
                    child: const SizedBox.expand(),
                  ),
                ],
              ),
            ),
          ),
        ),
        const SizedBox(height: 12),
        Text(
          '钱包：${widget.wallet.address}',
          style: const TextStyle(fontSize: 12, color: AppTheme.textTertiary),
          overflow: TextOverflow.ellipsis,
        ),
        const Spacer(),
      ],
    );
  }

  Widget _buildConfirm() {
    final decoded = _decoded;
    if (decoded == null) return const SizedBox.shrink();
    return Column(
      crossAxisAlignment: CrossAxisAlignment.stretch,
      children: [
        Container(
          padding: const EdgeInsets.symmetric(horizontal: 12, vertical: 8),
          decoration: AppTheme.bannerDecoration(AppTheme.success),
          child: const Text(
            '身份载荷已独立验证,请核对后签名',
            textAlign: TextAlign.center,
            style:
                TextStyle(color: AppTheme.success, fontWeight: FontWeight.w600),
          ),
        ),
        const SizedBox(height: 16),
        Container(
          padding: const EdgeInsets.all(16),
          decoration: AppTheme.cardDecoration(),
          child: Column(
            children: [
              _detailRow(
                '签名类型',
                decoded.isCandidate ? '参选身份上链确认' : '投票身份上链确认',
              ),
              ...decoded.reviewEntries
                  .map((entry) => _detailRow(entry.$1, entry.$2)),
            ],
          ),
        ),
        const Spacer(),
        Row(
          children: [
            Expanded(
              child: OutlinedButton(
                onPressed: _cancelConfirm,
                child: const Text('取消'),
              ),
            ),
            const SizedBox(width: 12),
            Expanded(
              child: FilledButton(
                onPressed: _processing ? null : _sign,
                child: const Text('确认签名'),
              ),
            ),
          ],
        ),
      ],
    );
  }

  Widget _detailRow(String label, String value) {
    return Padding(
      padding: const EdgeInsets.symmetric(vertical: 6),
      child: Row(
        crossAxisAlignment: CrossAxisAlignment.start,
        children: [
          SizedBox(
            width: 92,
            child: Text(
              label,
              style: const TextStyle(
                fontSize: 13,
                color: AppTheme.textSecondary,
                fontWeight: FontWeight.w600,
              ),
            ),
          ),
          Expanded(
            child: Text(
              value,
              style: const TextStyle(
                fontSize: 13,
                color: AppTheme.textPrimary,
                height: 1.4,
              ),
            ),
          ),
        ],
      ),
    );
  }

  Widget _buildSigning() {
    return const Center(
      child: Column(
        mainAxisSize: MainAxisSize.min,
        children: [
          CircularProgressIndicator(),
          SizedBox(height: 16),
          Text('正在签名...', style: TextStyle(fontSize: 16)),
        ],
      ),
    );
  }

  Widget _buildShowResponse() {
    return Column(
      children: [
        Container(
          width: double.infinity,
          padding: const EdgeInsets.symmetric(horizontal: 12, vertical: 8),
          decoration: AppTheme.bannerDecoration(AppTheme.success),
          child: const Text(
            '签名成功，请让管理员扫描下方二维码',
            style:
                TextStyle(color: AppTheme.success, fontWeight: FontWeight.w600),
          ),
        ),
        const Spacer(),
        Center(
          child: QrImageView(
            data: _responseJson!,
            version: QrVersions.auto,
            size: 280,
            errorStateBuilder: (_, __) => const SizedBox(
              width: 280,
              height: 280,
              child: Center(child: Text('二维码渲染失败')),
            ),
          ),
        ),
        const Spacer(),
        SizedBox(
          width: double.infinity,
          child: ElevatedButton(
            onPressed: () => Navigator.of(context).pop(),
            child: const Text('完成'),
          ),
        ),
      ],
    );
  }
}

class _MyIdScanCornerPainter extends CustomPainter {
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
  bool shouldRepaint(covariant _MyIdScanCornerPainter oldDelegate) => false;
}
