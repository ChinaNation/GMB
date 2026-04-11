// 热钱包投票账户现场签名页。
//
// 用户到 SFID 现场后：
// 1. 打开此页，摄像头扫描 SFID 管理端屏幕上的 sign_request 二维码
// 2. 验证 pubkey 与当前绑定钱包一致
// 3. 用热钱包私钥签名 payload
// 4. 构造 sign_response envelope
// 5. 展示回执二维码，等 SFID 管理员扫描

import 'dart:typed_data';

import 'package:flutter/material.dart';
import 'package:mobile_scanner/mobile_scanner.dart';
import 'package:qr_flutter/qr_flutter.dart';
import 'package:wuminapp_mobile/qr/bodies/sign_response_body.dart';
import 'package:wuminapp_mobile/qr/envelope.dart';
import 'package:wuminapp_mobile/qr/qr_protocols.dart';
import 'package:wuminapp_mobile/signer/qr_signer.dart';
import 'package:wuminapp_mobile/ui/app_theme.dart';
import 'package:wuminapp_mobile/wallet/core/wallet_manager.dart';

class VoteSignPage extends StatefulWidget {
  const VoteSignPage({super.key, required this.wallet});

  final WalletProfile wallet;

  @override
  State<VoteSignPage> createState() => _VoteSignPageState();
}

enum _PageStep { scanning, signing, showResponse }

class _VoteSignPageState extends State<VoteSignPage> {
  _PageStep _step = _PageStep.scanning;
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
      _step = _PageStep.signing;
      _errorMessage = null;
    });

    try {
      // 解析 sign_request
      final qrSigner = QrSigner();
      final request = qrSigner.parseRequest(raw);

      // 验证 pubkey 与当前钱包一致
      final expectedPubkey = '0x${widget.wallet.pubkeyHex}'.toLowerCase();
      final requestPubkey = request.body.pubkey.toLowerCase();
      if (requestPubkey != expectedPubkey) {
        throw Exception('签名请求中的公钥与当前钱包不一致');
      }

      // 热钱包签名
      final payloadBytes =
          Uint8List.fromList(_hexToBytes(request.body.payloadHex));
      final walletManager = WalletManager();
      final signature = await walletManager.signWithWallet(
        widget.wallet.walletIndex,
        payloadBytes,
      );

      // 构造 sign_response envelope
      final sigHex =
          '0x${signature.map((b) => b.toRadixString(16).padLeft(2, '0')).join()}';
      final payloadHash =
          QrSigner.computePayloadHash(request.body.payloadHex);
      final now = DateTime.now().millisecondsSinceEpoch ~/ 1000;
      final responseEnvelope = QrEnvelope<SignResponseBody>(
        kind: QrKind.signResponse,
        id: request.id,
        issuedAt: now,
        expiresAt: now + 120,
        body: SignResponseBody(
          pubkey: request.body.pubkey,
          sigAlg: 'sr25519',
          signature: sigHex,
          payloadHash: payloadHash,
          signedAt: now,
        ),
      );
      final json = responseEnvelope.toRawJson();

      if (!mounted) return;
      setState(() {
        _responseJson = json;
        _step = _PageStep.showResponse;
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

  @override
  Widget build(BuildContext context) {
    return Scaffold(
      appBar: AppBar(
        title: const Text('投票账户签名'),
        centerTitle: true,
      ),
      body: Padding(
        padding: const EdgeInsets.all(16),
        child: switch (_step) {
          _PageStep.scanning => _buildScanning(),
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
            '请扫描 SFID 管理端屏幕上的签名请求二维码',
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
        const SizedBox(height: 16),
        // 摄像头
        Expanded(
          child: ClipRRect(
            borderRadius: BorderRadius.circular(12),
            child: MobileScanner(
              controller: _scannerController,
              onDetect: _onDetect,
            ),
          ),
        ),
        const SizedBox(height: 12),
        Text(
          '钱包：${widget.wallet.address}',
          style: const TextStyle(fontSize: 12, color: AppTheme.textTertiary),
          overflow: TextOverflow.ellipsis,
        ),
      ],
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
            '签名成功，请让 SFID 管理员扫描下方二维码',
            style: TextStyle(
                color: AppTheme.success, fontWeight: FontWeight.w600),
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

List<int> _hexToBytes(String input) {
  final hex = input.startsWith('0x') ? input.substring(2) : input;
  final result = <int>[];
  for (var i = 0; i < hex.length; i += 2) {
    result.add(int.parse(hex.substring(i, i + 2), radix: 16));
  }
  return result;
}
