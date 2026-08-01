import 'dart:async';

import 'package:flutter/material.dart';
import 'package:citizenapp/ui/widgets/bip39_input.dart';
import 'package:citizenapp/ui/app_theme.dart';
import 'package:citizenapp/wallet/core/wallet_manager.dart';
import 'package:citizenapp/wallet/pages/create_wallet_flow.dart';
import 'package:citizenapp/rpc/chain_tx_monitor.dart';

/// 导入热钱包页：输入助记词 → 验证 → 落库。
///
/// **fail-closed**：`importWallet` 保证钱包本地落库成功才返回，此时 `pop(true)`
/// 交由调用方（钱包页 / 首启门禁）决定进入；失败即整笔回滚并抛出，弹窗提示后停留
/// 本页、助记词保留在输入框（仅成功路径 clear），用户可直接重试。设备子钥不在导入时
/// 注册——改由进入需 CID 页面时由门禁按需绑定（换机导入的账户可能已有 CID）。
class ImportWalletPage extends StatefulWidget {
  const ImportWalletPage({super.key, this.dataRootRequest});

  /// 非空时，导入成功后**用同一份助记词**顺手派生该 CID 的数据根并缓存。
  ///
  /// 由需 CID 页面的门禁在本机拿不到数据根时带入（新设备、注册局代办换绑等）。
  /// 派生必须在此完成：母种子只在导入这一刻存在，本端不落盘，出了这个页面就没了。
  /// 派生失败**不回滚**已导入的钱包（钱包本身可用），错误上抛由用户重试。
  final CidDataRootRequest? dataRootRequest;

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
      final mnemonic = _mnemonicController.text;
      final profile = await WalletManager().importWallet(mnemonic);
      final request = widget.dataRootRequest;
      if (request != null) {
        await WalletManager().installCidDataRootFromMnemonic(
          mnemonic: mnemonic,
          cidNumber: request.cidNumber,
          bindingRevision: request.bindingRevision,
          accountId: request.accountId,
        );
      }
      unawaited(ChainTxMonitor.instance.initBaselineBalance(
        profile.ss58Address,
        profile.accountId,
      ));
      // 钱包名是本机标签，导入后保留本机默认值；公开昵称由 cid_number 对应的
      // display_name 独立恢复，本流程不得联网改写钱包标签。
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
      // fail-closed：钱包本地落库失败即已回滚。弹窗提示后停留导入页，
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
