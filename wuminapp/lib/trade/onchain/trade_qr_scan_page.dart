import 'dart:async';

import 'package:flutter/material.dart';
import 'package:mobile_scanner/mobile_scanner.dart';

class TradeQrScanPage extends StatefulWidget {
  const TradeQrScanPage({super.key});

  @override
  State<TradeQrScanPage> createState() => _TradeQrScanPageState();
}

class _TradeQrScanPageState extends State<TradeQrScanPage> {
  final MobileScannerController _controller = MobileScannerController();
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
      final address = _extractAddress(raw);
      if (address == null) {
        if (!mounted) {
          return;
        }
        await showDialog<void>(
          context: context,
          builder: (context) => AlertDialog(
            title: const Text('无法识别收款码'),
            content: const Text('请扫描账户收款二维码。'),
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
      Navigator.of(context).pop(address);
    } finally {
      if (mounted) {
        _handled = false;
        unawaited(_controller.start());
      }
    }
  }

  String? _extractAddress(String raw) {
    final text = raw.trim();
    if (text.isEmpty) {
      return null;
    }
    if (text.toLowerCase().startsWith('gmb://account/')) {
      final address = text.substring('gmb://account/'.length).trim();
      return address.isEmpty ? null : address;
    }
    if (RegExp(r'^[1-9A-HJ-NP-Za-km-z]{30,80}$').hasMatch(text)) {
      return text;
    }
    return null;
  }

  @override
  Widget build(BuildContext context) {
    return Scaffold(
      appBar: AppBar(
        title: const Text('扫码收款码'),
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
                '扫描对方收款二维码',
                style: TextStyle(color: Colors.white),
              ),
            ),
          ),
        ],
      ),
    );
  }
}
