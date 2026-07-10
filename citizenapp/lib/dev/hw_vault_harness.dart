// 开发期真机验证入口（非生产入口）。驱动**生产** HardwareBoundSeedVault 的完整
// 路径（原生桥 + flutter_secure_storage），验证严档/宽档两档信封加解密与每次一验。
// 生产 main.dart 零改动。构建：flutter build apk --profile --target lib/dev/hw_vault_harness.dart。
// Step 3/4 e2e 打通后删除本文件。
import 'package:flutter/material.dart';

import 'package:citizenapp/wallet/core/hardware_bound_seed_vault.dart';

void main() {
  WidgetsFlutterBinding.ensureInitialized();
  runApp(const _HarnessApp());
}

class _HarnessApp extends StatelessWidget {
  const _HarnessApp();

  @override
  Widget build(BuildContext context) => const MaterialApp(
        debugShowCheckedModeBanner: false,
        home: _HarnessPage(),
      );
}

class _HarnessPage extends StatefulWidget {
  const _HarnessPage();

  @override
  State<_HarnessPage> createState() => _HarnessPageState();
}

class _HarnessPageState extends State<_HarnessPage> {
  static const int _idx = 1;
  static const String _demoSeed = 'a1b2c3d4e5f6a1b2c3d4e5f6a1b2c3d4';
  static const String _demoMnemonic =
      'legal winner thank year wave sausage worth useful legal winner thank yellow';

  final HardwareBoundSeedVault _vault = HardwareBoundSeedVault();
  final List<String> _log = <String>[];
  int _readSeedCount = 0;
  int _readMnemonicCount = 0;

  void _append(String s) => setState(() => _log.insert(0, s));

  @override
  void initState() {
    super.initState();
    _bootstrap();
  }

  Future<void> _bootstrap() async {
    try {
      _append('authStatus: ${await _vault.authStatus()}');
      await _vault.putSeed(_idx, _demoSeed);
      _append('putSeed OK (严档,静默)');
      await _vault.putMnemonic(_idx, _demoMnemonic);
      _append('putMnemonic OK (宽档,静默)');
    } catch (e) {
      _append('bootstrap ERROR: $e');
    }
  }

  Future<void> _readSeed() async {
    final n = ++_readSeedCount;
    final t0 = DateTime.now();
    try {
      final seed = await _vault.readSeed(_idx);
      final ms = DateTime.now().difference(t0).inMilliseconds;
      final ok = seed == _demoSeed ? 'MATCH' : 'MISMATCH($seed)';
      _append('readSeed #$n (${ms}ms): $ok');
    } catch (e) {
      _append('readSeed #$n ERROR: ${e.runtimeType} $e');
    }
  }

  Future<void> _readMnemonic() async {
    final n = ++_readMnemonicCount;
    final t0 = DateTime.now();
    try {
      final mnemonic = await _vault.readMnemonic(_idx);
      final ms = DateTime.now().difference(t0).inMilliseconds;
      final ok = mnemonic == _demoMnemonic ? 'MATCH' : 'MISMATCH';
      _append('readMnemonic #$n (${ms}ms): $ok');
    } catch (e) {
      _append('readMnemonic #$n ERROR: ${e.runtimeType} $e');
    }
  }

  Future<void> _reset() async {
    try {
      await _vault.deleteSeed(_idx);
      await _vault.deleteMnemonic(_idx);
      _append('reset: 两档 blob + KEK 已删');
    } catch (e) {
      _append('reset ERROR: $e');
    }
  }

  @override
  Widget build(BuildContext context) {
    return Scaffold(
      appBar: AppBar(title: const Text('硬件金库 生产路径 Harness')),
      body: Column(
        children: <Widget>[
          Padding(
            padding: const EdgeInsets.all(12),
            child: Wrap(
              spacing: 8,
              runSpacing: 8,
              children: <Widget>[
                FilledButton(
                  onPressed: _readSeed,
                  child: const Text('读 seed(严档,弹生物)'),
                ),
                FilledButton(
                  onPressed: _readMnemonic,
                  child: const Text('读助记词(宽档,生物或PIN)'),
                ),
                OutlinedButton(onPressed: _reset, child: const Text('重置')),
              ],
            ),
          ),
          const Divider(height: 1),
          Expanded(
            child: ListView.builder(
              itemCount: _log.length,
              itemBuilder: (BuildContext _, int i) => Padding(
                padding:
                    const EdgeInsets.symmetric(horizontal: 12, vertical: 6),
                child: Text(_log[i], style: const TextStyle(fontSize: 13)),
              ),
            ),
          ),
        ],
      ),
    );
  }
}
