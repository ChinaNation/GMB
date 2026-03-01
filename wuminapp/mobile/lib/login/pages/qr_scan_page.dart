import 'dart:async';
import 'dart:convert';

import 'package:flutter/material.dart';
import 'package:mobile_scanner/mobile_scanner.dart';
import 'package:qr_flutter/qr_flutter.dart';
import 'package:wuminapp_mobile/login/models/login_models.dart';
import 'package:wuminapp_mobile/login/services/login_sign_confirm_service.dart';
import 'package:wuminapp_mobile/login/services/wuminapp_login_service.dart';

class QrScanPage extends StatefulWidget {
  const QrScanPage({
    super.key,
    this.walletIndex,
    this.walletAddress,
  });

  final int? walletIndex;
  final String? walletAddress;

  @override
  State<QrScanPage> createState() => _QrScanPageState();
}

class _QrScanPageState extends State<QrScanPage> {
  final MobileScannerController _controller = MobileScannerController();
  final WuminLoginService _loginService = WuminLoginService();
  final LoginSignConfirmService _signConfirmService = LoginSignConfirmService();
  bool _handled = false;

  @override
  void dispose() {
    _controller.dispose();
    super.dispose();
  }

  Future<void> _handleCode(String raw) async {
    if (_handled) {
      return;
    }
    _handled = true;
    await _controller.stop();

    try {
      final mode = _detectMode(raw);
      if (!mounted) {
        return;
      }

      switch (mode) {
        case _ScanMode.login:
          await _showLoginDialog(raw);
          break;
        case _ScanMode.transfer:
          final address = _extractAddress(raw);
          await Navigator.of(context).push(
            MaterialPageRoute(
              builder: (_) => TransferDraftPage(toAddress: address),
            ),
          );
          break;
        case _ScanMode.unknown:
          await showDialog<void>(
            context: context,
            builder: (context) => AlertDialog(
              title: const Text('无法识别二维码'),
              content: const Text('请扫描 WUMINAPP_LOGIN_V1 登录二维码或账户收款二维码。'),
              actions: [
                TextButton(
                  onPressed: () => Navigator.of(context).pop(),
                  child: const Text('确定'),
                ),
              ],
            ),
          );
          break;
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

  _ScanMode _detectMode(String raw) {
    final lower = raw.toLowerCase();
    if (_isWuminLoginCode(raw)) {
      return _ScanMode.login;
    }
    if (lower.startsWith('gmb://account/')) {
      return _ScanMode.transfer;
    }
    if (RegExp(r'^[1-9A-HJ-NP-Za-km-z]{30,80}$').hasMatch(raw)) {
      return _ScanMode.transfer;
    }
    return _ScanMode.unknown;
  }

  bool _isWuminLoginCode(String raw) {
    try {
      final data = jsonDecode(raw);
      if (data is Map<String, dynamic>) {
        final proto = (data['proto'] ?? '').toString();
        return proto == WuminLoginService.protocol;
      }
    } catch (_) {
      // not json
    }
    return false;
  }

  String _extractAddress(String raw) {
    if (raw.toLowerCase().startsWith('gmb://account/')) {
      return raw.substring('gmb://account/'.length).trim();
    }
    return raw.trim();
  }

  Future<void> _showLoginDialog(String payload) async {
    WuminLoginChallenge challenge;
    try {
      challenge = _loginService.parseChallenge(payload);
      await _loginService.validateTrust(challenge);
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
      await _signLoginChallenge(challenge);
    }
  }

  Future<void> _signLoginChallenge(WuminLoginChallenge challenge) async {
    try {
      if (challenge.isExpired) {
        throw Exception('登录挑战已过期，请重新扫码');
      }
      await _signConfirmService.confirmBeforeSign();

      final result = await _loginService.buildReceiptPayloadForChallenge(
        challenge,
        walletIndex: widget.walletIndex,
      );

      if (!mounted) {
        return;
      }

      final compact = jsonEncode(result);

      final goWallet = await Navigator.of(context).push<bool>(
        MaterialPageRoute(
          builder: (_) => _LoginReceiptPage(
            compactPayload: compact,
            expiresAt: challenge.expiresAt,
          ),
        ),
      );
      if (goWallet == true && mounted) {
        Navigator.of(context).pop();
      }
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

  String _displaySystemName(String system) {
    if (system.toLowerCase() == 'sfid') {
      return 'SFID';
    }
    return system;
  }

  @override
  Widget build(BuildContext context) {
    return Scaffold(
      appBar: AppBar(
        title: Text(widget.walletIndex == null ? '扫码' : '扫码（钱包${widget.walletIndex}）'),
        centerTitle: true,
      ),
      body: Stack(
        fit: StackFit.expand,
        children: [
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
          Align(
            alignment: Alignment.topCenter,
            child: Container(
              margin: const EdgeInsets.all(16),
              padding: const EdgeInsets.symmetric(horizontal: 12, vertical: 8),
              decoration: BoxDecoration(
                color: Colors.black54,
                borderRadius: BorderRadius.circular(8),
              ),
              child: const Text(
                '扫描登录挑战码或账户收款码',
                style: TextStyle(color: Colors.white),
              ),
            ),
          ),
        ],
      ),
    );
  }
}

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
          Center(
            child: SizedBox(
              width: 180,
              child: FilledButton(
                onPressed: () => Navigator.of(context).pop(true),
                child: const Text('完成'),
              ),
            ),
          ),
        ],
      ),
    );
  }
}

enum _ScanMode {
  login,
  transfer,
  unknown,
}

class TransferDraftPage extends StatefulWidget {
  const TransferDraftPage({super.key, required this.toAddress});

  final String toAddress;

  @override
  State<TransferDraftPage> createState() => _TransferDraftPageState();
}

class _TransferDraftPageState extends State<TransferDraftPage> {
  final TextEditingController _amountController = TextEditingController();

  @override
  void dispose() {
    _amountController.dispose();
    super.dispose();
  }

  @override
  Widget build(BuildContext context) {
    return Scaffold(
      appBar: AppBar(
        title: const Text('发起转账'),
        centerTitle: true,
      ),
      body: Padding(
        padding: const EdgeInsets.all(16),
        child: Column(
          crossAxisAlignment: CrossAxisAlignment.start,
          children: [
            Text('收款地址: ${widget.toAddress}'),
            const SizedBox(height: 12),
            TextField(
              controller: _amountController,
              keyboardType: const TextInputType.numberWithOptions(decimal: true),
              decoration: const InputDecoration(
                labelText: '金额',
                hintText: '请输入转账金额',
                border: OutlineInputBorder(),
              ),
            ),
            const SizedBox(height: 12),
            FilledButton(
              onPressed: () {
                ScaffoldMessenger.of(context).showSnackBar(
                  SnackBar(
                    content: Text(
                      '已生成转账草稿：${_amountController.text}（开发中）',
                    ),
                  ),
                );
              },
              child: const Text('确认转账'),
            ),
          ],
        ),
      ),
    );
  }
}
