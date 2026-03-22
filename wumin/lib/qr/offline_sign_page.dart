import 'dart:async';

import 'package:flutter/material.dart';
import 'package:mobile_scanner/mobile_scanner.dart';
import 'package:qr_flutter/qr_flutter.dart';

import '../signer/offline_sign_service.dart';
import '../signer/qr_signer.dart';
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
  final MobileScannerController _controller = MobileScannerController();
  final OfflineSignService _offlineSignService = OfflineSignService();
  final QrSigner _qrSigner = QrSigner();

  Timer? _timer;
  bool _handled = false;
  bool _signing = false;
  QrSignRequest? _request;
  QrSignResponse? _response;
  OfflineSignVerification? _verification;
  int _remainingSeconds = 0;

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
    await _controller.start();
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
              '请扫描在线手机展示的签名请求二维码\n当前钱包：${widget.wallet.walletName}',
              textAlign: TextAlign.center,
              style: const TextStyle(color: Colors.white),
            ),
          ),
        ),
      ],
    );
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

    final List<Widget> detailRows;
    if (decoded != null) {
      detailRows = [
        _detailRow('交易类型', decoded.action),
        _detailRow('摘要', decoded.summary),
        ...decoded.fields.entries.map((e) => _detailRow(e.key, e.value)),
      ];
    } else {
      final display = request.display;
      detailRows = [
        _detailRow('交易类型', display['action']?.toString() ?? '未知'),
        _detailRow('摘要', display['summary']?.toString() ?? '无'),
      ];
      final fields = display['fields'];
      if (fields is Map) {
        detailRows.addAll(
          fields.entries
              .map((e) => _detailRow(e.key.toString(), e.value.toString())),
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
          SizedBox(
            width: 80,
            child: Text(
              label,
              style: const TextStyle(
                color: Colors.black54,
                fontWeight: FontWeight.w500,
              ),
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
                Text(
                  '交易签名',
                  style: const TextStyle(
                    fontSize: 18,
                    fontWeight: FontWeight.w700,
                  ),
                ),
                const SizedBox(height: 8),
                Text('请求 ID：${request.requestId}'),
                const SizedBox(height: 4),
                Text('签名账户：${_truncate(request.account)}'),
              ],
            ),
          ),
        ),
        const SizedBox(height: 12),
        if (verification != null)
          _buildTransactionDetails(request, verification),
        const SizedBox(height: 16),
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
                onPressed: (_signing || expired || isMismatched)
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
