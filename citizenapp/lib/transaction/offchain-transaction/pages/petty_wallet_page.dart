import 'package:flutter/material.dart';

import 'package:citizenapp/my/util/amount_format.dart';
import 'package:citizenapp/transaction/offchain-transaction/pages/deposit_page.dart';
import 'package:citizenapp/transaction/offchain-transaction/pages/withdraw_page.dart';
import 'package:citizenapp/transaction/offchain-transaction/rpc/offchain_clearing_rpc.dart';
import 'package:citizenapp/ui/app_theme.dart';
import 'package:citizenapp/wallet/core/wallet_manager.dart';

/// 零钱包详情页(链下清算行零钱包)。
///
/// 从钱包详情「零钱包」按钮进入。承接原动作卡上的「充值到清算行」入口,并提供:
/// - 零钱包余额(节点端 `offchain_queryBalance`);
/// - 充值到清算行(链上账户 → 清算行零钱包,原 `DepositPage`);
/// - 提现到链上(清算行 → 链上账户,`WithdrawPage`)。
class PettyWalletPage extends StatefulWidget {
  const PettyWalletPage({
    super.key,
    required this.wallet,
    required this.wssUrl,
    required this.displayTitle,
  });

  final WalletProfile wallet;
  final String wssUrl;
  final String displayTitle;

  @override
  State<PettyWalletPage> createState() => _PettyWalletPageState();
}

class _PettyWalletPageState extends State<PettyWalletPage> {
  String _balanceText = '查询中';

  @override
  void initState() {
    super.initState();
    _loadBalance();
  }

  Future<void> _loadBalance() async {
    try {
      final fen =
          await OffchainClearingBankRpc(widget.wssUrl).queryBalance(widget.wallet.address);
      if (!mounted) return;
      setState(() => _balanceText = '¥${AmountFormat.formatThousands(fen / 100.0)}');
    } catch (_) {
      if (!mounted) return;
      setState(() => _balanceText = '节点不可达');
    }
  }

  @override
  Widget build(BuildContext context) {
    return Scaffold(
      appBar: AppBar(title: const Text('零钱包'), centerTitle: true),
      body: RefreshIndicator(
        onRefresh: _loadBalance,
        child: ListView(
          padding: const EdgeInsets.all(16),
          physics: const AlwaysScrollableScrollPhysics(),
          children: [
            Container(
              padding: const EdgeInsets.all(20),
              decoration: AppTheme.cardDecoration(radius: AppTheme.radiusLg),
              child: Column(
                crossAxisAlignment: CrossAxisAlignment.start,
                children: [
                  Text(widget.displayTitle,
                      style: const TextStyle(
                          fontSize: 13, color: AppTheme.textSecondary)),
                  const SizedBox(height: 10),
                  Text(_balanceText,
                      style: const TextStyle(
                          fontSize: 28,
                          fontWeight: FontWeight.w700,
                          color: AppTheme.primaryDark)),
                  const SizedBox(height: 4),
                  const Text('零钱包余额（链下清算行）',
                      style:
                          TextStyle(fontSize: 12, color: AppTheme.textTertiary)),
                ],
              ),
            ),
            const SizedBox(height: 16),
            _ActionTile(
              icon: Icons.arrow_circle_down_outlined,
              title: '充值到清算行',
              subtitle: '从链上账户转入清算行零钱包',
              onTap: _openDeposit,
            ),
            const SizedBox(height: 12),
            _ActionTile(
              icon: Icons.arrow_circle_up_outlined,
              title: '提现到链上',
              subtitle: '从零钱包提现回链上账户',
              onTap: _openWithdraw,
            ),
          ],
        ),
      ),
    );
  }

  Future<void> _openDeposit() async {
    await Navigator.push(
      context,
      MaterialPageRoute(builder: (_) => DepositPage(wallet: widget.wallet)),
    );
    await _loadBalance();
  }

  Future<void> _openWithdraw() async {
    await Navigator.push(
      context,
      MaterialPageRoute(
        builder: (_) => WithdrawPage(wallet: widget.wallet, wssUrl: widget.wssUrl),
      ),
    );
    await _loadBalance();
  }
}

class _ActionTile extends StatelessWidget {
  const _ActionTile({
    required this.icon,
    required this.title,
    required this.subtitle,
    required this.onTap,
  });

  final IconData icon;
  final String title;
  final String subtitle;
  final VoidCallback onTap;

  @override
  Widget build(BuildContext context) {
    return Material(
      color: Colors.transparent,
      child: InkWell(
        borderRadius: BorderRadius.circular(AppTheme.radiusLg),
        onTap: onTap,
        child: Container(
          padding: const EdgeInsets.all(16),
          decoration: AppTheme.cardDecoration(radius: AppTheme.radiusLg),
          child: Row(
            children: [
              Container(
                width: 44,
                height: 44,
                decoration: BoxDecoration(
                  color: AppTheme.primary.withAlpha(26),
                  shape: BoxShape.circle,
                ),
                child: Icon(icon, size: 22, color: AppTheme.primaryDark),
              ),
              const SizedBox(width: 14),
              Expanded(
                child: Column(
                  crossAxisAlignment: CrossAxisAlignment.start,
                  mainAxisSize: MainAxisSize.min,
                  children: [
                    Text(title,
                        style: const TextStyle(
                            fontSize: 16,
                            fontWeight: FontWeight.w600,
                            color: AppTheme.textPrimary)),
                    const SizedBox(height: 2),
                    Text(subtitle,
                        style: const TextStyle(
                            fontSize: 12, color: AppTheme.textTertiary)),
                  ],
                ),
              ),
              const Icon(Icons.chevron_right,
                  size: 18, color: AppTheme.textTertiary),
            ],
          ),
        ),
      ),
    );
  }
}
