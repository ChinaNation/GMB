import 'package:flutter/material.dart';

import '../wallet/wallet_manager.dart';
import 'account_detail_page.dart';
import 'app_theme.dart';

/// Lv2 钱包详情：钱包(master)名 + 账户列表 + 添加账户。
///
/// 无根设备**不存助记词/种子**;助记词只在创建时一次性展示。账户按 `//index`
/// 派生,点账户进 Lv3;「添加账户」需输入本钱包助记词临时派生下一个账户。
class WalletDetailPage extends StatefulWidget {
  const WalletDetailPage({super.key, required this.wallet});

  final Wallet wallet;

  @override
  State<WalletDetailPage> createState() => _WalletDetailPageState();
}

class _WalletDetailPageState extends State<WalletDetailPage> {
  final WalletManager _walletManager = WalletManager();

  List<Account> _accounts = [];
  bool _loading = true;
  bool _addingAccount = false;

  @override
  void initState() {
    super.initState();
    _load();
  }

  Future<void> _load() async {
    final accounts = await _walletManager.getAccounts(widget.wallet.masterId);
    if (!mounted) return;
    setState(() {
      _accounts = accounts;
      _loading = false;
    });
  }

  Future<void> _addAccount() async {
    if (_addingAccount) return;
    // 无根设备不存种子,派生新 //N 需临时用本钱包助记词重建种子。
    final mnemonic = await _promptMnemonic();
    if (mnemonic == null || mnemonic.trim().isEmpty) return;
    setState(() => _addingAccount = true);
    try {
      await _walletManager.addAccount(widget.wallet.masterId, mnemonic);
      await _load();
    } catch (e) {
      if (!mounted) return;
      ScaffoldMessenger.of(context).showSnackBar(
        SnackBar(content: Text('添加账户失败：$e')),
      );
    } finally {
      if (mounted) setState(() => _addingAccount = false);
    }
  }

  Future<String?> _promptMnemonic() async {
    final controller = TextEditingController();
    return showDialog<String>(
      context: context,
      builder: (context) => AlertDialog(
        title: const Text('添加账户'),
        content: Column(
          mainAxisSize: MainAxisSize.min,
          crossAxisAlignment: CrossAxisAlignment.start,
          children: [
            const Text(
              '本设备不保存助记词。添加新账户需输入本钱包助记词以派生下一个账户。',
              style: TextStyle(fontSize: 13, color: AppTheme.textSecondary),
            ),
            const SizedBox(height: 12),
            TextField(
              controller: controller,
              autofocus: true,
              minLines: 2,
              maxLines: 3,
              decoration: const InputDecoration(
                hintText: '输入助记词，单词以空格分隔',
                border: OutlineInputBorder(),
              ),
            ),
          ],
        ),
        actions: [
          TextButton(
            onPressed: () => Navigator.of(context).pop(),
            child: const Text('取消'),
          ),
          TextButton(
            onPressed: () => Navigator.of(context).pop(controller.text),
            child: const Text('派生账户'),
          ),
        ],
      ),
    );
  }

  Future<void> _openAccount(Account account) async {
    await Navigator.of(context).push<bool>(
      MaterialPageRoute(
        builder: (_) => AccountDetailPage(
          account: account,
          walletName: widget.wallet.walletName,
        ),
      ),
    );
    await _load();
  }

  String _shortAddress(String address) {
    if (address.length <= 16) return address;
    return '${address.substring(0, 8)}...${address.substring(address.length - 6)}';
  }

  @override
  Widget build(BuildContext context) {
    return Scaffold(
      appBar: AppBar(title: const Text('钱包详情'), centerTitle: true),
      body: _loading
          ? const Center(child: CircularProgressIndicator())
          : ListView(
              padding: const EdgeInsets.all(16),
              children: [
                _buildIdentityCard(),
                const SizedBox(height: 20),
                _buildAccountsSection(),
              ],
            ),
    );
  }

  Widget _buildIdentityCard() {
    return Container(
      decoration: AppTheme.cardDecoration(radius: AppTheme.radiusLg),
      padding: const EdgeInsets.all(16),
      child: Row(
        children: [
          const Icon(Icons.account_balance_wallet_rounded,
              size: 22, color: AppTheme.primaryLight),
          const SizedBox(width: 10),
          Expanded(
            child: Text(
              widget.wallet.walletName,
              style: const TextStyle(
                  fontSize: 18,
                  fontWeight: FontWeight.w700,
                  color: AppTheme.textPrimary),
              overflow: TextOverflow.ellipsis,
            ),
          ),
        ],
      ),
    );
  }

  Widget _buildAccountsSection() {
    return Container(
      decoration: AppTheme.cardDecoration(radius: AppTheme.radiusLg),
      child: Column(
        children: [
          Padding(
            padding: const EdgeInsets.fromLTRB(16, 14, 8, 6),
            child: Row(
              children: [
                const Text('账户',
                    style: TextStyle(
                        fontSize: 14,
                        fontWeight: FontWeight.w600,
                        color: AppTheme.textPrimary)),
                const Spacer(),
                TextButton.icon(
                  onPressed: _addingAccount ? null : _addAccount,
                  icon: _addingAccount
                      ? const SizedBox(
                          width: 14,
                          height: 14,
                          child: CircularProgressIndicator(strokeWidth: 2))
                      : const Icon(Icons.add, size: 18),
                  label: const Text('添加账户'),
                ),
              ],
            ),
          ),
          ..._accounts.map(_buildAccountRow),
          const Padding(
            padding: EdgeInsets.fromLTRB(16, 4, 16, 14),
            child: Text(
              '提示:重装或换设备后,重导助记词只自动恢复账户0,其余账户需在此凭助记词按顺序手动重新添加(离线设备无法链上探活)。',
              style: TextStyle(fontSize: 11, color: AppTheme.textTertiary),
            ),
          ),
        ],
      ),
    );
  }

  Widget _buildAccountRow(Account account) {
    return Material(
      color: Colors.transparent,
      child: InkWell(
        onTap: () => _openAccount(account),
        child: Padding(
          padding: const EdgeInsets.symmetric(horizontal: 16, vertical: 12),
          child: Row(
            children: [
              Container(
                width: 38,
                height: 38,
                decoration: BoxDecoration(
                  color: AppTheme.surfaceElevated,
                  borderRadius: BorderRadius.circular(AppTheme.radiusSm),
                ),
                alignment: Alignment.center,
                child: Text('#${account.accountIndex}',
                    style: const TextStyle(
                        fontSize: 13,
                        fontWeight: FontWeight.w600,
                        color: AppTheme.primaryLight)),
              ),
              const SizedBox(width: 12),
              Expanded(
                child: Column(
                  crossAxisAlignment: CrossAxisAlignment.start,
                  children: [
                    Text(account.accountName,
                        style: const TextStyle(
                            fontSize: 14,
                            fontWeight: FontWeight.w600,
                            color: AppTheme.textPrimary)),
                    const SizedBox(height: 2),
                    Text(_shortAddress(account.ss58Address),
                        style: const TextStyle(
                            fontSize: 12,
                            color: AppTheme.textSecondary,
                            fontFamily: 'monospace')),
                  ],
                ),
              ),
              const Icon(Icons.chevron_right,
                  size: 20, color: AppTheme.textTertiary),
            ],
          ),
        ),
      ),
    );
  }
}
