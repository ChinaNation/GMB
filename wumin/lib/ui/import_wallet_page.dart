import 'package:flutter/material.dart';

import '../wallet/wallet_manager.dart';

/// 导入钱包页面（通过助记词）。
class ImportWalletPage extends StatefulWidget {
  const ImportWalletPage({super.key});

  @override
  State<ImportWalletPage> createState() => _ImportWalletPageState();
}

class _ImportWalletPageState extends State<ImportWalletPage> {
  final WalletManager _walletManager = WalletManager();
  final TextEditingController _mnemonicController = TextEditingController();
  bool _importing = false;

  @override
  void dispose() {
    _mnemonicController.dispose();
    super.dispose();
  }

  Future<void> _import() async {
    final mnemonic = _mnemonicController.text.trim();
    if (mnemonic.isEmpty) {
      ScaffoldMessenger.of(context).showSnackBar(
        const SnackBar(content: Text('请输入助记词')),
      );
      return;
    }

    setState(() => _importing = true);
    try {
      final profile = await _walletManager.importWallet(mnemonic);
      if (!mounted) return;
      ScaffoldMessenger.of(context).showSnackBar(
        SnackBar(content: Text('已导入「${profile.walletName}」')),
      );
      Navigator.of(context).pop(true);
    } catch (e) {
      if (!mounted) return;
      ScaffoldMessenger.of(context).showSnackBar(
        SnackBar(content: Text('导入失败：$e')),
      );
    } finally {
      if (mounted) setState(() => _importing = false);
    }
  }

  @override
  Widget build(BuildContext context) {
    return Scaffold(
      appBar: AppBar(
        title: const Text('导入钱包'),
        centerTitle: true,
      ),
      body: ListView(
        padding: const EdgeInsets.all(24),
        children: [
          const Text(
            '输入助记词',
            style: TextStyle(fontSize: 18, fontWeight: FontWeight.w700),
          ),
          const SizedBox(height: 8),
          const Text(
            '请输入助记词，用空格分隔',
            style: TextStyle(color: Colors.black54),
          ),
          const SizedBox(height: 16),
          TextField(
            controller: _mnemonicController,
            maxLines: 4,
            decoration: const InputDecoration(
              border: OutlineInputBorder(),
              hintText: 'word1 word2 word3 ...',
            ),
            textInputAction: TextInputAction.done,
            autocorrect: false,
            enableSuggestions: false,
          ),
          const SizedBox(height: 24),
          SizedBox(
            width: double.infinity,
            child: FilledButton(
              onPressed: _importing ? null : _import,
              child: Text(_importing ? '导入中...' : '导入钱包'),
            ),
          ),
        ],
      ),
    );
  }
}
