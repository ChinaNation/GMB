import 'package:flutter/material.dart';
import 'package:flutter/services.dart';

import '../wallet/wallet_manager.dart';
import 'widgets/bip39_input.dart';

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
      // 导入成功后清空剪贴板，防止助记词残留
      await Clipboard.setData(const ClipboardData(text: ''));
      _mnemonicController.clear();
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
            '逐个输入单词，从候选列表中选择匹配项',
            style: TextStyle(color: Colors.black54),
          ),
          const SizedBox(height: 16),
          Bip39InputField(controller: _mnemonicController, wordCount: 0),
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
