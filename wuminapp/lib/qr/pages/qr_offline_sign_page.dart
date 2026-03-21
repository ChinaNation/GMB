import 'dart:async';

import 'package:flutter/material.dart';
import 'package:mobile_scanner/mobile_scanner.dart';
import 'package:qr_flutter/qr_flutter.dart';

import '../../signer/offline_sign_service.dart';
import '../../signer/qr_signer.dart';
import '../../wallet/core/wallet_manager.dart';

/// 离线签名页面。
///
/// 该页面用于“持有私钥的另一台设备”扫描在线手机展示的签名请求二维码，
/// 并在本机完成签名后展示回执二维码，供在线手机再扫回去。
class QrOfflineSignPage extends StatefulWidget {
  const QrOfflineSignPage({
    super.key,
    required this.wallet,
  });

  final WalletProfile wallet;

  @override
  State<QrOfflineSignPage> createState() => _QrOfflineSignPageState();
}

class _QrOfflineSignPageState extends State<QrOfflineSignPage> {
  final MobileScannerController _controller = MobileScannerController();
  final OfflineSignService _offlineSignService = OfflineSignService();
  final QrSigner _qrSigner = QrSigner();

  Timer? _timer;
  bool _handled = false;
  bool _signing = false;
  QrSignRequest? _request;
  QrSignResponse? _response;
  int _remainingSeconds = 0;

  @override
  void initState() {
    super.initState();
    if (widget.wallet.isColdWallet) {
      _remainingSeconds = 0;
    }
  }

  @override
  void dispose() {
    _timer?.cancel();
    _controller.dispose();
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
    if (_handled) {
      return;
    }
    _handled = true;
    await _controller.stop();

    try {
      final request = _offlineSignService.parseRequest(raw);
      if (!mounted) return;
      setState(() {
        _request = request;
        _response = null;
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
      _remainingSeconds = 0;
      _signing = false;
    });
    await _controller.start();
  }

  Future<void> _signRequest() async {
    final request = _request;
    if (request == null) {
      return;
    }
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
      await showDialog<void>(
        context: context,
        builder: (context) => AlertDialog(
          title: const Text('离线签名失败'),
          content: Text(e.message),
          actions: [
            TextButton(
              onPressed: () => Navigator.of(context).pop(),
              child: const Text('确定'),
            ),
          ],
        ),
      );
    } on WalletAuthException catch (e) {
      if (!mounted) return;
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
      if (!mounted) return;
      await showDialog<void>(
        context: context,
        builder: (context) => AlertDialog(
          title: const Text('离线签名失败'),
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
        setState(() {
          _signing = false;
        });
      }
    }
  }

  String _scopeLabel(QrSignScope scope) {
    switch (scope) {
      case QrSignScope.login:
        return '登录签名';
      case QrSignScope.onchainTx:
        return '链上交易签名';
    }
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
        Align(
          alignment: Alignment.topCenter,
          child: Container(
            margin: const EdgeInsets.fromLTRB(16, 24, 16, 0),
            padding: const EdgeInsets.symmetric(horizontal: 12, vertical: 10),
            decoration: BoxDecoration(
              color: Colors.black54,
              borderRadius: BorderRadius.circular(12),
            ),
            child: Text(
              '请用此设备扫描在线手机展示的签名请求二维码\n当前钱包：${widget.wallet.walletName}',
              textAlign: TextAlign.center,
              style: const TextStyle(color: Colors.white),
            ),
          ),
        ),
      ],
    );
  }

  Widget _buildRequestSummary(QrSignRequest request) {
    final expired = _remainingSeconds <= 0;
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
        const SizedBox(height: 16),
        Card(
          child: Padding(
            padding: const EdgeInsets.all(16),
            child: Column(
              crossAxisAlignment: CrossAxisAlignment.start,
              children: [
                Text(
                  _scopeLabel(request.scope),
                  style: const TextStyle(
                    fontSize: 18,
                    fontWeight: FontWeight.w700,
                  ),
                ),
                const SizedBox(height: 12),
                Text('请求 ID：${request.requestId}'),
                const SizedBox(height: 8),
                Text('签名账户：${_truncate(request.account)}'),
                const SizedBox(height: 8),
                Text('签名公钥：${_truncate(request.pubkey)}'),
                const SizedBox(height: 8),
                Text(
                  '负载长度：${(_normalizeHex(request.payloadHex).length ~/ 2)} bytes',
                ),
              ],
            ),
          ),
        ),
        const SizedBox(height: 16),
        const Text(
          '确认无误后再签名。签名完成后，本页会生成回执二维码，供在线手机扫描回收。',
          style: TextStyle(color: Colors.black54),
        ),
        const SizedBox(height: 24),
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
                onPressed: (_signing || expired) ? null : _signRequest,
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
        Row(
          children: [
            Expanded(
              child: OutlinedButton(
                onPressed: _resetToScanner,
                child: const Text('继续签名'),
              ),
            ),
            const SizedBox(width: 12),
            Expanded(
              child: FilledButton(
                onPressed: () => Navigator.of(context).pop(),
                child: const Text('完成'),
              ),
            ),
          ],
        ),
      ],
    );
  }

  @override
  Widget build(BuildContext context) {
    if (widget.wallet.isColdWallet) {
      return Scaffold(
        appBar: AppBar(
          title: const Text('离线签名'),
          centerTitle: true,
        ),
        body: const Center(
          child: Padding(
            padding: EdgeInsets.all(24),
            child: Text('冷钱包不保存私钥，无法作为离线签名执行端。'),
          ),
        ),
      );
    }

    final request = _request;
    final response = _response;
    return Scaffold(
      appBar: AppBar(
        title: const Text('离线签名'),
        centerTitle: true,
      ),
      body: response != null
          ? _buildResponseView(response)
          : (request != null ? _buildRequestSummary(request) : _buildScanner()),
    );
  }
}

String _normalizeHex(String input) {
  final trimmed = input.trim();
  if (trimmed.startsWith('0x') || trimmed.startsWith('0X')) {
    return trimmed.substring(2).toLowerCase();
  }
  return trimmed.toLowerCase();
}
