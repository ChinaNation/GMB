import 'dart:convert';

import 'package:flutter/material.dart';
import 'package:flutter/services.dart';
import 'package:mobile_scanner/mobile_scanner.dart';
import 'package:qr_flutter/qr_flutter.dart';
import 'package:wuminapp_mobile/services/fcrc_login_service.dart';

class QrScanPage extends StatefulWidget {
  const QrScanPage({super.key});

  @override
  State<QrScanPage> createState() => _QrScanPageState();
}

class _QrScanPageState extends State<QrScanPage> {
  final MobileScannerController _controller = MobileScannerController();
  final FcrcLoginSignatureService _loginSignatureService =
      FcrcLoginSignatureService();
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

    final mode = _detectMode(raw);
    if (!mounted) {
      return;
    }

    switch (mode) {
      case _ScanMode.fcrcLogin:
        await _showFcrcLoginDialog(raw);
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
            content: const Text('请扫描 fcrc 登录二维码或账户收款二维码。'),
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

    if (!mounted) {
      return;
    }
    _handled = false;
    await _controller.start();
  }

  _ScanMode _detectMode(String raw) {
    final lower = raw.toLowerCase();
    if (_isFcrcLoginCode(raw)) {
      return _ScanMode.fcrcLogin;
    }
    if (lower.startsWith('gmb://account/')) {
      return _ScanMode.transfer;
    }
    if (RegExp(r'^[1-9A-HJ-NP-Za-km-z]{30,80}$').hasMatch(raw)) {
      return _ScanMode.transfer;
    }
    return _ScanMode.unknown;
  }

  bool _isFcrcLoginCode(String raw) {
    final lower = raw.toLowerCase().trim();
    if (lower.startsWith('fcrc://login') ||
        lower.startsWith('fcrc-login://') ||
        (lower.contains('fcrc') && (lower.contains('login') || lower.contains('signin')))) {
      return true;
    }
    try {
      final data = jsonDecode(raw);
      if (data is Map<String, dynamic>) {
        final type = (data['type'] ?? '').toString().toLowerCase();
        final scene = (data['scene'] ?? '').toString().toLowerCase();
        if (type.contains('fcrc') && type.contains('login')) {
          return true;
        }
        if (scene.contains('fcrc') && scene.contains('login')) {
          return true;
        }
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

  Future<void> _showFcrcLoginDialog(String payload) async {
    FcrcLoginChallenge challenge;
    try {
      challenge = _loginSignatureService.parseChallenge(payload);
    } catch (e) {
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
        title: const Text('登录 fcrc 系统'),
        content: Text('已识别登录挑战。\nnonce: ${challenge.nonce}'),
        actions: [
          TextButton(
            onPressed: () => Navigator.of(context).pop(false),
            child: const Text('取消'),
          ),
          FilledButton(
            onPressed: () => Navigator.of(context).pop(true),
            child: const Text('挑战签名'),
          ),
        ],
      ),
    );
    if (shouldSign == true && mounted) {
      await _signFcrcChallenge(payload);
    }
  }

  Future<void> _signFcrcChallenge(String payload) async {
    try {
      final result = await _loginSignatureService.buildSignaturePayload(payload);
      if (!mounted) {
        return;
      }
      final messenger = ScaffoldMessenger.of(context);
      final compact = jsonEncode(result);
      final pretty = const JsonEncoder.withIndent('  ').convert(result);
      await showDialog<void>(
        context: context,
        builder: (context) => AlertDialog(
          title: const Text('登录签名已生成'),
          content: SingleChildScrollView(
            child: Column(
              mainAxisSize: MainAxisSize.min,
              children: [
                QrImageView(
                  data: compact,
                  version: QrVersions.auto,
                  size: 220,
                ),
                const SizedBox(height: 12),
                SelectableText(pretty),
              ],
            ),
          ),
          actions: [
            TextButton(
              onPressed: () {
                Clipboard.setData(ClipboardData(text: pretty));
                messenger.showSnackBar(
                  const SnackBar(content: Text('签名 JSON 已复制')),
                );
              },
              child: const Text('复制'),
            ),
            FilledButton(
              onPressed: () => Navigator.of(context).pop(),
              child: const Text('完成'),
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
          title: const Text('签名失败'),
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

  @override
  Widget build(BuildContext context) {
    return Scaffold(
      appBar: AppBar(
        title: const Text('扫码'),
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
                '扫描 fcrc 登录码或账户收款码',
                style: TextStyle(color: Colors.white),
              ),
            ),
          ),
        ],
      ),
    );
  }
}

enum _ScanMode {
  fcrcLogin,
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
