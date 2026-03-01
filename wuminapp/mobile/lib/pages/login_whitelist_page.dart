import 'package:flutter/material.dart';
import 'package:wuminapp_mobile/login/services/login_whitelist_store.dart';

class LoginWhitelistPage extends StatefulWidget {
  const LoginWhitelistPage({super.key});

  @override
  State<LoginWhitelistPage> createState() => _LoginWhitelistPageState();
}

class _LoginWhitelistPageState extends State<LoginWhitelistPage> {
  final LoginWhitelistStore _store = LoginWhitelistStore();
  bool _loading = true;
  late LoginWhitelistConfig _config;

  static const List<String> _systems = ['cpms', 'sfid', 'citizenchain'];

  @override
  void initState() {
    super.initState();
    _load();
  }

  Future<void> _load() async {
    final config = await _store.load();
    if (!mounted) {
      return;
    }
    setState(() {
      _config = config;
      _loading = false;
    });
  }

  Future<void> _resetDefault() async {
    const config = LoginWhitelistConfig(
      audWhitelist: LoginWhitelistStore.defaultAudWhitelist,
      originWhitelist: LoginWhitelistStore.defaultOriginWhitelist,
    );
    await _store.save(config);
    if (!mounted) {
      return;
    }
    setState(() {
      _config = config;
    });
  }

  Future<void> _editSystem(String system) async {
    final audCtl = TextEditingController(
      text: (_config.audWhitelist[system] ?? const <String>{}).join(','),
    );
    final originCtl = TextEditingController(
      text: (_config.originWhitelist[system] ?? const <String>{}).join(','),
    );

    final ok = await showDialog<bool>(
      context: context,
      builder: (context) => AlertDialog(
        title: Text('编辑 $system 白名单'),
        content: Column(
          mainAxisSize: MainAxisSize.min,
          children: [
            TextField(
              controller: audCtl,
              decoration: const InputDecoration(
                labelText: 'aud 列表（逗号分隔）',
              ),
            ),
            const SizedBox(height: 8),
            TextField(
              controller: originCtl,
              decoration: const InputDecoration(
                labelText: 'origin 列表（逗号分隔）',
              ),
            ),
          ],
        ),
        actions: [
          TextButton(
            onPressed: () => Navigator.of(context).pop(false),
            child: const Text('取消'),
          ),
          FilledButton(
            onPressed: () => Navigator.of(context).pop(true),
            child: const Text('保存'),
          ),
        ],
      ),
    );

    if (ok != true) {
      return;
    }

    final newAud = _parseCsv(audCtl.text);
    final newOrigin = _parseCsv(originCtl.text);
    if (newAud.isEmpty || newOrigin.isEmpty) {
      if (!mounted) {
        return;
      }
      ScaffoldMessenger.of(context).showSnackBar(
        const SnackBar(content: Text('aud 和 origin 至少各保留 1 项')),
      );
      return;
    }

    final audMap = Map<String, Set<String>>.from(_config.audWhitelist);
    final originMap = Map<String, Set<String>>.from(_config.originWhitelist);
    audMap[system] = newAud;
    originMap[system] = newOrigin;

    final next = LoginWhitelistConfig(
      audWhitelist: audMap,
      originWhitelist: originMap,
    );
    await _store.save(next);
    if (!mounted) {
      return;
    }
    setState(() {
      _config = next;
    });
  }

  Set<String> _parseCsv(String input) {
    return input
        .split(',')
        .map((e) => e.trim())
        .where((e) => e.isNotEmpty)
        .toSet();
  }

  String _subtitle(String system) {
    final aud = _config.audWhitelist[system] ?? const <String>{};
    final origin = _config.originWhitelist[system] ?? const <String>{};
    return 'aud:${aud.join('|')}  origin:${origin.join('|')}';
  }

  @override
  Widget build(BuildContext context) {
    return Scaffold(
      appBar: AppBar(
        title: const Text('登录白名单'),
        centerTitle: true,
        actions: [
          TextButton(
            onPressed: _loading ? null : _resetDefault,
            child: const Text('重置默认'),
          ),
        ],
      ),
      body: _loading
          ? const Center(child: CircularProgressIndicator())
          : ListView(
              padding: const EdgeInsets.all(16),
              children: [
                for (final system in _systems)
                  Card(
                    child: ListTile(
                      title: Text(system),
                      subtitle: Text(_subtitle(system)),
                      trailing: const Icon(Icons.chevron_right),
                      onTap: () => _editSystem(system),
                    ),
                  ),
              ],
            ),
    );
  }
}
