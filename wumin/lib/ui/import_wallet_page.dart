import 'package:flutter/material.dart';

import '../util/sensitive_page_mixin.dart';
import '../wallet/wallet_manager.dart';
import 'app_theme.dart';
import 'widgets/bip39_input.dart';

/// 导入钱包页面（通过助记词）。
class ImportWalletPage extends StatefulWidget {
  const ImportWalletPage({super.key});

  @override
  State<ImportWalletPage> createState() => _ImportWalletPageState();
}

class _ImportWalletPageState extends State<ImportWalletPage>
    with SensitivePageMixin {
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
    if (sensitiveContentHidden) {
      return buildHiddenPlaceholder(message: '助记词输入已隐藏');
    }
    return Scaffold(
      appBar: AppBar(
        title: const Text('导入钱包'),
        centerTitle: true,
      ),
      body: ListView(
        padding: const EdgeInsets.all(20),
        children: [
          // 图标
          Center(
            child: Container(
              width: 64,
              height: 64,
              decoration: BoxDecoration(
                color: AppTheme.primary.withAlpha(25),
                borderRadius: BorderRadius.circular(16),
              ),
              child: const Icon(
                Icons.download_rounded,
                size: 30,
                color: AppTheme.primaryLight,
              ),
            ),
          ),
          const SizedBox(height: 20),
          const Text(
            '输入助记词',
            textAlign: TextAlign.center,
            style: TextStyle(
              fontSize: 20,
              fontWeight: FontWeight.w700,
              color: AppTheme.textPrimary,
            ),
          ),
          const SizedBox(height: 8),
          const Text(
            '逐个输入单词，从候选列表中选择匹配项',
            textAlign: TextAlign.center,
            style: TextStyle(
              color: AppTheme.textSecondary,
              fontSize: 14,
            ),
          ),
          const SizedBox(height: 24),
          Bip39InputField(controller: _mnemonicController, wordCount: 0),
          const SizedBox(height: 28),
          FilledButton(
            onPressed: _importing ? null : _import,
            child: _importing
                ? const SizedBox(
                    width: 20,
                    height: 20,
                    child: CircularProgressIndicator(
                      strokeWidth: 2,
                      color: Colors.white,
                    ),
                  )
                : const Text('导入钱包'),
          ),
        ],
      ),
    );
  }
}
