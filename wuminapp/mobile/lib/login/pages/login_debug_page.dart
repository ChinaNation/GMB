import 'dart:convert';

import 'package:flutter/material.dart';
import 'package:qr_flutter/qr_flutter.dart';
import 'package:wuminapp_mobile/login/models/login_models.dart';
import 'package:wuminapp_mobile/login/services/wuminapp_login_service.dart';

class LoginDebugPage extends StatefulWidget {
  const LoginDebugPage({super.key});

  @override
  State<LoginDebugPage> createState() => _LoginDebugPageState();
}

class _LoginDebugPageState extends State<LoginDebugPage> {
  final WuminLoginService _loginService = WuminLoginService();
  final TextEditingController _challengeController = TextEditingController();
  String _selectedSystem = 'cpms';

  WuminLoginChallenge? _challenge;
  String? _signPreview;
  String? _receiptPretty;
  String? _receiptCompact;
  String? _error;

  @override
  void initState() {
    super.initState();
    _challengeController.text = _buildExampleChallenge(_selectedSystem);
  }

  @override
  void dispose() {
    _challengeController.dispose();
    super.dispose();
  }

  String _buildExampleChallenge(String system) {
    final now = DateTime.now().millisecondsSinceEpoch ~/ 1000;
    final data = <String, dynamic>{
      'proto': WuminLoginService.protocol,
      'system': system,
      'request_id': 'req-$system-$now',
      'challenge': 'base64-rand-$now',
      'nonce': 'nonce-$now',
      'issued_at': now,
      'expires_at': now + 60,
      'aud': _defaultAud(system),
      'origin': _defaultOrigin(system),
    };
    return const JsonEncoder.withIndent('  ').convert(data);
  }

  String _defaultAud(String system) {
    switch (system) {
      case 'cpms':
        return 'cpms-local-app';
      case 'sfid':
        return 'sfid-local-app';
      case 'citizenchain':
        return 'citizenchain-front';
      default:
        return 'unknown';
    }
  }

  String _defaultOrigin(String system) {
    switch (system) {
      case 'cpms':
        return 'cpms-device-id';
      case 'sfid':
        return 'sfid-device-id';
      case 'citizenchain':
        return 'citizenchain-device-id';
      default:
        return 'unknown';
    }
  }

  Future<void> _parseChallenge() async {
    try {
      final challenge = _loginService.parseChallenge(_challengeController.text);
      await _loginService.validateTrust(challenge);
      final signPreview =
          _loginService.buildSignPreview(_challengeController.text);
      if (!mounted) {
        return;
      }
      setState(() {
        _challenge = challenge;
        _signPreview = signPreview;
        _error = null;
      });
    } catch (e) {
      if (!mounted) {
        return;
      }
      setState(() {
        _challenge = null;
        _signPreview = null;
        _error = e.toString();
      });
    }
  }

  Future<void> _generateReceipt() async {
    try {
      final result =
          await _loginService.buildReceiptPayload(_challengeController.text);
      if (!mounted) {
        return;
      }
      setState(() {
        _receiptCompact = jsonEncode(result);
        _receiptPretty = const JsonEncoder.withIndent('  ').convert(result);
        _error = null;
      });
    } catch (e) {
      if (!mounted) {
        return;
      }
      setState(() {
        _receiptCompact = null;
        _receiptPretty = null;
        _error = e.toString();
      });
    }
  }

  void _fillExample() {
    setState(() {
      _challengeController.text = _buildExampleChallenge(_selectedSystem);
      _challenge = null;
      _signPreview = null;
      _receiptCompact = null;
      _receiptPretty = null;
      _error = null;
    });
  }

  @override
  Widget build(BuildContext context) {
    return Scaffold(
      appBar: AppBar(
        title: const Text('登录开发调试'),
        centerTitle: true,
      ),
      body: ListView(
        padding: const EdgeInsets.all(16),
        children: [
          Row(
            children: [
              const Text('系统: '),
              const SizedBox(width: 8),
              DropdownButton<String>(
                value: _selectedSystem,
                items: const [
                  DropdownMenuItem(value: 'cpms', child: Text('cpms')),
                  DropdownMenuItem(value: 'sfid', child: Text('sfid')),
                  DropdownMenuItem(
                    value: 'citizenchain',
                    child: Text('citizenchain'),
                  ),
                ],
                onChanged: (value) {
                  if (value == null) {
                    return;
                  }
                  setState(() {
                    _selectedSystem = value;
                  });
                },
              ),
              const Spacer(),
              FilledButton.tonal(
                onPressed: _fillExample,
                child: const Text('填充示例'),
              ),
            ],
          ),
          const SizedBox(height: 12),
          TextField(
            controller: _challengeController,
            maxLines: 12,
            decoration: const InputDecoration(
              labelText: '挑战二维码 JSON',
              border: OutlineInputBorder(),
            ),
          ),
          const SizedBox(height: 12),
          Row(
            children: [
              Expanded(
                child: OutlinedButton(
                  onPressed: _parseChallenge,
                  child: const Text('解析并校验'),
                ),
              ),
              const SizedBox(width: 8),
              Expanded(
                child: FilledButton(
                  onPressed: _generateReceipt,
                  child: const Text('生成回执'),
                ),
              ),
            ],
          ),
          if (_error != null) ...[
            const SizedBox(height: 12),
            Card(
              color: Colors.red.shade50,
              child: Padding(
                padding: const EdgeInsets.all(12),
                child: Text(
                  _error!,
                  style: TextStyle(color: Colors.red.shade800),
                ),
              ),
            ),
          ],
          if (_challenge != null) ...[
            const SizedBox(height: 12),
            Card(
              child: Padding(
                padding: const EdgeInsets.all(12),
                child: Column(
                  crossAxisAlignment: CrossAxisAlignment.start,
                  children: [
                    const Text(
                      '挑战解析结果',
                      style: TextStyle(fontWeight: FontWeight.w700),
                    ),
                    const SizedBox(height: 8),
                    Text('system: ${_challenge!.system}'),
                    Text('request_id: ${_challenge!.requestId}'),
                    Text('aud: ${_challenge!.aud}'),
                    Text('origin: ${_challenge!.origin}'),
                    Text('ttl: ${_challenge!.ttlSeconds}s'),
                  ],
                ),
              ),
            ),
          ],
          if (_signPreview != null) ...[
            const SizedBox(height: 12),
            Card(
              child: Padding(
                padding: const EdgeInsets.all(12),
                child: Column(
                  crossAxisAlignment: CrossAxisAlignment.start,
                  children: [
                    const Text(
                      '签名原文',
                      style: TextStyle(fontWeight: FontWeight.w700),
                    ),
                    const SizedBox(height: 8),
                    SelectableText(_signPreview!),
                  ],
                ),
              ),
            ),
          ],
          if (_receiptCompact != null && _receiptPretty != null) ...[
            const SizedBox(height: 12),
            Card(
              child: Padding(
                padding: const EdgeInsets.all(12),
                child: Column(
                  crossAxisAlignment: CrossAxisAlignment.start,
                  children: [
                    const Text(
                      '回执二维码',
                      style: TextStyle(fontWeight: FontWeight.w700),
                    ),
                    const SizedBox(height: 8),
                    Center(
                      child: QrImageView(
                        data: _receiptCompact!,
                        version: QrVersions.auto,
                        size: 220,
                      ),
                    ),
                    const SizedBox(height: 8),
                    SelectableText(_receiptPretty!),
                  ],
                ),
              ),
            ),
          ],
        ],
      ),
    );
  }
}
