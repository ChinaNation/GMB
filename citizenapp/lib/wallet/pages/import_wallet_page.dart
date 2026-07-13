import 'dart:async';

import 'package:flutter/material.dart';
import 'package:citizenapp/ui/widgets/bip39_input.dart';
import 'package:citizenapp/ui/app_theme.dart';
import 'package:citizenapp/wallet/core/wallet_manager.dart';
import 'package:citizenapp/wallet/pages/create_wallet_flow.dart';
import 'package:citizenapp/rpc/chain_tx_monitor.dart';

/// 导入热钱包页：输入助记词 → 验证 → 落库 + 注册设备子钥。
///
/// **二元 fail-closed**：`importWallet` 保证"导入 + 子钥注册"全部成功才返回，此时
/// `pop(true)` 交由调用方（钱包页 / 首启门禁）决定进入；任一失败即整笔回滚并抛出，
/// 弹窗提示后停留本页、助记词保留在输入框（仅成功路径 clear），用户可直接重试。
class ImportWalletPage extends StatefulWidget {
  const ImportWalletPage({super.key});

  @override
  State<ImportWalletPage> createState() => _ImportWalletPageState();
}

class _ImportWalletPageState extends State<ImportWalletPage> {
  final TextEditingController _mnemonicController = TextEditingController();
  bool _isImporting = false;
  String? _error;

  Future<void> _import() async {
    setState(() {
      _error = null;
      _isImporting = true;
    });
    try {
      final profile =
          await WalletManager().importWallet(_mnemonicController.text);
      unawaited(ChainTxMonitor.instance.initBaselineBalance(
        profile.address,
        profile.pubkeyHex,
      ));
      _mnemonicController.clear();
      if (!mounted) {
        return;
      }
      Navigator.of(context).pop(true);
    } catch (e) {
      if (!mounted) return;
      setState(() {
        _error = walletOperationErrorMessage(e);
      });
      // fail-closed：导入含子钥注册，任一失败即已回滚。弹窗提示后停留导入页，
      // 助记词保留在输入框（仅成功路径 clear），用户可直接重试。
      await showDialog<void>(
        context: context,
        builder: (context) => AlertDialog(
          title: const Text('导入失败'),
          content: Text(walletOperationErrorMessage(e)),
          actions: [
            TextButton(
              onPressed: () => Navigator.of(context).pop(),
              child: const Text('重试'),
            ),
          ],
        ),
      );
    } finally {
      if (mounted) {
        setState(() {
          _isImporting = false;
        });
      }
    }
  }

  @override
  void dispose() {
    _mnemonicController.dispose();
    super.dispose();
  }

  @override
  Widget build(BuildContext context) {
    return Scaffold(
      appBar: AppBar(title: const Text('导入热钱包')),
      body: ListView(
        padding: const EdgeInsets.all(16),
        children: [
          const Text('逐个输入单词，从候选列表中选择匹配项'),
          const SizedBox(height: 8),
          const Text('仅使用默认派生路径，不暴露自定义路径。'),
          const SizedBox(height: 12),
          Bip39InputField(controller: _mnemonicController, wordCount: 0),
          const SizedBox(height: 12),
          if (_error != null)
            Text(
              _error!,
              style: const TextStyle(color: AppTheme.danger),
            ),
          FilledButton(
            onPressed: _isImporting ? null : _import,
            child: Text(_isImporting ? '导入中...' : '确认导入'),
          ),
        ],
      ),
    );
  }
}
