import 'package:flutter/material.dart';
import 'package:flutter_svg/flutter_svg.dart';

import '../qr/bodies/im_node_pairing_body.dart';
import '../qr/envelope.dart';
import '../qr/pages/qr_scan_page.dart';
import '../qr/qr_protocols.dart';
import '../ui/app_theme.dart';
import 'im_runtime.dart';

typedef ImNodePairingScanner = Future<String?> Function(BuildContext context);

/// “我的 -> 设置 -> 设置通信节点”页面。
///
/// 本页面只负责把公民手机配对到用户自己的电脑通信节点。
/// 它不添加联系人，不进入聊天，不出现在信息 Tab。
class ImNodeSettingsPage extends StatefulWidget {
  ImNodeSettingsPage({
    super.key,
    ImRuntime? runtime,
    ImNodePairingScanner? scanner,
  })  : runtime = runtime ?? ImRuntime(),
        scanner = scanner ?? _defaultScanner;

  final ImRuntime runtime;
  final ImNodePairingScanner scanner;

  @override
  State<ImNodeSettingsPage> createState() => _ImNodeSettingsPageState();
}

class _ImNodeSettingsPageState extends State<ImNodeSettingsPage> {
  ImPairedNodeConfig? _config;
  bool _loading = true;
  bool _pairing = false;
  String? _error;

  @override
  void initState() {
    super.initState();
    _load();
  }

  Future<void> _load() async {
    setState(() {
      _loading = true;
      _error = null;
    });
    try {
      final config = await widget.runtime.readPairedNodeConfig();
      if (!mounted) return;
      setState(() {
        _config = config;
      });
    } catch (error) {
      if (!mounted) return;
      setState(() {
        _error = '$error';
      });
    } finally {
      if (mounted) {
        setState(() {
          _loading = false;
        });
      }
    }
  }

  Future<void> _scanAndPair() async {
    if (_pairing) return;
    setState(() {
      _pairing = true;
      _error = null;
    });
    try {
      final raw = await widget.scanner(context);
      if (raw == null || raw.trim().isEmpty) {
        return;
      }
      final envelope = QrEnvelope.parse(raw);
      if (envelope.kind != QrKind.imNodePairing ||
          envelope.body is! ImNodePairingBody) {
        throw const FormatException('请扫描区块链软件上的通信节点二维码');
      }
      final body = envelope.body as ImNodePairingBody;
      final config = await widget.runtime.pairCommunicationNode(body);
      if (!mounted) return;
      setState(() {
        _config = config;
      });
      ScaffoldMessenger.of(context).showSnackBar(
        const SnackBar(content: Text('通信节点已设置')),
      );
    } catch (error) {
      if (!mounted) return;
      setState(() {
        _error = '$error';
      });
    } finally {
      if (mounted) {
        setState(() {
          _pairing = false;
        });
      }
    }
  }

  @override
  Widget build(BuildContext context) {
    final config = _config;
    final paired = config?.isComplete == true;
    return Scaffold(
      appBar: AppBar(
        title: const Text('设置通信节点'),
        centerTitle: true,
        actions: [
          IconButton(
            tooltip: paired ? '更换通信节点' : '扫描通信节点',
            onPressed: _pairing ? null : _scanAndPair,
            icon: SvgPicture.asset(
              'assets/icons/scan-line.svg',
              width: 22,
              height: 22,
              colorFilter: const ColorFilter.mode(
                AppTheme.textPrimary,
                BlendMode.srcIn,
              ),
            ),
          ),
        ],
      ),
      body: _loading
          ? const Center(child: CircularProgressIndicator())
          : ListView(
              padding: const EdgeInsets.all(16),
              children: [
                if (_error != null) _ErrorBanner(message: _error!),
                if (paired)
                  _PairedNodeCard(config: config!)
                else
                  _EmptyNodeCard(onScan: _pairing ? null : _scanAndPair),
              ],
            ),
    );
  }
}

Future<String?> _defaultScanner(BuildContext context) {
  return Navigator.of(context).push<String>(
    MaterialPageRoute(
      builder: (_) => const QrScanPage(
        mode: QrScanMode.raw,
        customTitle: '扫描通信节点',
      ),
    ),
  );
}

class _EmptyNodeCard extends StatelessWidget {
  const _EmptyNodeCard({required this.onScan});

  final VoidCallback? onScan;

  @override
  Widget build(BuildContext context) {
    return Container(
      decoration: AppTheme.cardDecoration(radius: AppTheme.radiusLg),
      padding: const EdgeInsets.fromLTRB(18, 22, 18, 22),
      child: Column(
        children: [
          Container(
            width: 52,
            height: 52,
            decoration: BoxDecoration(
              color: AppTheme.primary.withAlpha(22),
              borderRadius: BorderRadius.circular(8),
            ),
            child: const Icon(
              Icons.dns_outlined,
              color: AppTheme.primary,
              size: 28,
            ),
          ),
          const SizedBox(height: 14),
          const Text(
            '尚未设置通信节点',
            style: TextStyle(
              color: AppTheme.textPrimary,
              fontSize: 17,
              fontWeight: FontWeight.w700,
            ),
          ),
          const SizedBox(height: 8),
          const Text(
            '扫描区块链软件设置页上的通信节点二维码后，本手机会使用该电脑节点收发密文消息。',
            textAlign: TextAlign.center,
            style: TextStyle(
              color: AppTheme.textSecondary,
              fontSize: 13,
              height: 1.45,
            ),
          ),
          const SizedBox(height: 18),
          FilledButton.icon(
            onPressed: onScan,
            icon: SvgPicture.asset(
              'assets/icons/scan-line.svg',
              width: 18,
              height: 18,
              colorFilter: const ColorFilter.mode(
                Colors.white,
                BlendMode.srcIn,
              ),
            ),
            label: const Text('扫描通信节点'),
          ),
        ],
      ),
    );
  }
}

class _PairedNodeCard extends StatelessWidget {
  const _PairedNodeCard({required this.config});

  final ImPairedNodeConfig config;

  @override
  Widget build(BuildContext context) {
    return Container(
      decoration: AppTheme.cardDecoration(radius: AppTheme.radiusLg),
      padding: const EdgeInsets.all(16),
      child: Column(
        crossAxisAlignment: CrossAxisAlignment.start,
        children: [
          const Row(
            children: [
              Icon(Icons.check_circle_rounded, color: AppTheme.primary),
              SizedBox(width: 8),
              Text(
                '已设置通信节点',
                style: TextStyle(
                  color: AppTheme.textPrimary,
                  fontSize: 17,
                  fontWeight: FontWeight.w700,
                ),
              ),
            ],
          ),
          const SizedBox(height: 14),
          const _InfoRow(label: '节点状态', value: '已保存'),
          _InfoRow(label: 'PeerId', value: config.shortPeerId),
          _InfoRow(label: '端点', value: config.multiaddr),
          if (config.pairedAtMillis != null)
            _InfoRow(
              label: '设置时间',
              value: DateTime.fromMillisecondsSinceEpoch(
                config.pairedAtMillis!,
              ).toLocal().toString(),
            ),
        ],
      ),
    );
  }
}

class _InfoRow extends StatelessWidget {
  const _InfoRow({required this.label, required this.value});

  final String label;
  final String value;

  @override
  Widget build(BuildContext context) {
    return Padding(
      padding: const EdgeInsets.symmetric(vertical: 7),
      child: Row(
        crossAxisAlignment: CrossAxisAlignment.start,
        children: [
          SizedBox(
            width: 72,
            child: Text(
              label,
              style: const TextStyle(
                color: AppTheme.textTertiary,
                fontSize: 13,
              ),
            ),
          ),
          Expanded(
            child: Text(
              value,
              style: const TextStyle(
                color: AppTheme.textPrimary,
                fontSize: 13,
              ),
            ),
          ),
        ],
      ),
    );
  }
}

class _ErrorBanner extends StatelessWidget {
  const _ErrorBanner({required this.message});

  final String message;

  @override
  Widget build(BuildContext context) {
    return Container(
      margin: const EdgeInsets.only(bottom: 12),
      padding: const EdgeInsets.all(12),
      decoration: BoxDecoration(
        color: Colors.red.withAlpha(18),
        borderRadius: BorderRadius.circular(8),
      ),
      child: Text(
        message,
        style: const TextStyle(color: Colors.red, fontSize: 13),
      ),
    );
  }
}
