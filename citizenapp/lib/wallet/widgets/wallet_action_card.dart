import 'package:flutter/material.dart';

import 'package:citizenapp/transaction/onchain-topup/onchain_topup_page.dart';
import 'package:citizenapp/transaction/offchain-transaction/rpc/offchain_clearing_rpc.dart';
import 'package:citizenapp/transaction/offchain-transaction/services/clearing_bank_prefs.dart';
import 'package:citizenapp/transaction/offchain-transaction/pages/petty_wallet_page.dart';
import 'package:citizenapp/transaction/offchain-transaction/pages/withdraw_page.dart';
import 'package:citizenapp/ui/app_theme.dart';
import 'package:citizenapp/wallet/core/wallet_manager.dart';

/// 钱包详情页第 2 卡片:3 列等宽布局(充值/提现/零钱包)。
///
///
/// - 充值:进「链上充值」页(稳定币购买公民币,与清算行无关,**不需要绑定清算行**)。
/// - 提现:零钱包 → 链上账户,需已绑定清算行,否则提示先绑定。
/// - 零钱包:**可点击**进「零钱包详情页」(链下清算行零钱包),需已绑定;页内含充值到零钱包。
/// - 零钱包余额来自当前绑定清算行快照中的节点端点,通过 `offchain_queryBalance`
///   查询;失败时展示节点不可达,不再写死 0.00 元。
class WalletActionCard extends StatefulWidget {
  const WalletActionCard({super.key, required this.wallet});

  final WalletProfile wallet;

  @override
  State<WalletActionCard> createState() => WalletActionCardState();
}

class WalletActionCardState extends State<WalletActionCard> {
  ClearingBankBindingSnapshot? _binding;
  String _balanceText = '读取中';

  @override
  void initState() {
    super.initState();
    refresh();
  }

  Future<void> refresh() async {
    final binding =
        await ClearingBankPrefs.loadSnapshot(widget.wallet.walletIndex);
    if (!mounted) return;
    setState(() {
      _binding = binding;
      _balanceText = binding == null ? '未绑定' : '查询中';
    });
    if (binding != null) {
      await _loadBalance(binding);
    }
  }

  Future<void> _loadBalance(ClearingBankBindingSnapshot binding) async {
    try {
      final balance = await OffchainClearingBankRpc(
        binding.wssUrl,
      ).queryBalance(widget.wallet.address);
      if (!mounted) return;
      setState(() => _balanceText = _fenToYuan(balance));
    } catch (_) {
      if (!mounted) return;
      setState(() => _balanceText = '节点不可达');
    }
  }

  @override
  Widget build(BuildContext context) {
    return Container(
      decoration: AppTheme.cardDecoration(radius: AppTheme.radiusLg),
      // 三列布局相对原两列更拥挤,padding 调小避免卡片臃肿。
      padding: const EdgeInsets.symmetric(vertical: 16, horizontal: 12),
      child: Row(
        mainAxisAlignment: MainAxisAlignment.spaceAround,
        children: [
          Expanded(
            child: _ClickableAction(
              icon: Icons.arrow_circle_down_outlined,
              label: '充值',
              onTap: () => _openTopup(context),
            ),
          ),
          Expanded(
            child: _ClickableAction(
              icon: Icons.arrow_circle_up_outlined,
              label: '提现',
              onTap: () => _openWithdraw(context),
            ),
          ),
          Expanded(
            child: _BalanceAction(
              balanceText: _balanceText,
              onTap: () => _openPettyWallet(context),
            ),
          ),
        ],
      ),
    );
  }

  /// 充值 = 稳定币购买公民币,进链上充值页;不依赖清算行绑定。
  Future<void> _openTopup(BuildContext context) async {
    await Navigator.push(
      context,
      MaterialPageRoute(
        builder: (_) => OnchainTopupPage(gmbAddress: widget.wallet.address),
      ),
    );
    await refresh();
  }

  Future<void> _openWithdraw(BuildContext context) async {
    final binding = _binding;
    if (binding == null) {
      _showNeedBinding(context);
      return;
    }
    await Navigator.push(
      context,
      MaterialPageRoute(
        builder: (_) => WithdrawPage(
          wallet: widget.wallet,
          wssUrl: binding.wssUrl,
        ),
      ),
    );
    await refresh();
  }

  /// 零钱包 = 进清算行零钱包详情页(余额 + 充值到零钱包 + 提现);需已绑定。
  Future<void> _openPettyWallet(BuildContext context) async {
    final binding = _binding;
    if (binding == null) {
      _showNeedBinding(context);
      return;
    }
    await Navigator.push(
      context,
      MaterialPageRoute(
        builder: (_) => PettyWalletPage(
          wallet: widget.wallet,
          wssUrl: binding.wssUrl,
          displayTitle: binding.displayTitle,
        ),
      ),
    );
    await refresh();
  }

  static void _showNeedBinding(BuildContext context) {
    ScaffoldMessenger.of(context).showSnackBar(
      const SnackBar(
        content: Text('请先在“清算行”页面绑定清算行'),
        duration: Duration(seconds: 2),
      ),
    );
  }

  static String _fenToYuan(int fen) {
    final yuan = fen ~/ 100;
    final cents = (fen % 100).abs();
    return '$yuan.${cents.toString().padLeft(2, '0')} 元';
  }
}

/// 充值 / 提现两列共用的可点击按钮:圆形图标 + 标签 + 等高占位文本。
///
///
/// - 使用 `Material + InkWell` 组合,ripple 限制在 `CircleBorder` 内,不溢出圆圈。
/// - 底部用一个非断空格 ` ` 占位,保证和零钱包列的 `0.00 元` 行高对齐,
///   避免 3 列底部不齐。
class _ClickableAction extends StatelessWidget {
  const _ClickableAction({
    required this.icon,
    required this.label,
    required this.onTap,
  });

  final IconData icon;
  final String label;
  final VoidCallback onTap;

  @override
  Widget build(BuildContext context) {
    return Column(
      mainAxisSize: MainAxisSize.min,
      children: [
        Material(
          color: AppTheme.primary.withAlpha(15),
          shape: const CircleBorder(),
          child: InkWell(
            customBorder: const CircleBorder(),
            onTap: onTap,
            child: SizedBox(
              width: 56,
              height: 56,
              child: Icon(
                icon,
                size: 28,
                color: AppTheme.primaryDark,
              ),
            ),
          ),
        ),
        const SizedBox(height: 8),
        Text(
          label,
          style: const TextStyle(
            fontSize: 14,
            fontWeight: FontWeight.w600,
            color: AppTheme.primaryDark,
          ),
        ),
        const SizedBox(height: 4),
        // 非断空格占位,保证三列底部和零钱包列的 0.00 元对齐。
        const Text(
          ' ',
          style: TextStyle(
            fontSize: 12,
            color: AppTheme.textTertiary,
          ),
        ),
      ],
    );
  }
}

/// 零钱包列:**可点击**进零钱包详情页;圆形图标 ripple + 标签 + 余额文本。
class _BalanceAction extends StatelessWidget {
  const _BalanceAction({required this.balanceText, required this.onTap});

  final String balanceText;
  final VoidCallback onTap;

  @override
  Widget build(BuildContext context) {
    return Column(
      mainAxisSize: MainAxisSize.min,
      children: [
        Material(
          color: AppTheme.primary.withAlpha(15),
          shape: const CircleBorder(),
          child: InkWell(
            customBorder: const CircleBorder(),
            onTap: onTap,
            child: const SizedBox(
              width: 56,
              height: 56,
              child: Icon(
                Icons.account_balance_wallet_outlined,
                size: 28,
                color: AppTheme.primaryDark,
              ),
            ),
          ),
        ),
        const SizedBox(height: 8),
        const Text(
          '零钱包',
          style: TextStyle(
            fontSize: 14,
            fontWeight: FontWeight.w600,
            color: AppTheme.primaryDark,
          ),
        ),
        const SizedBox(height: 4),
        Text(
          balanceText,
          style: const TextStyle(
            fontSize: 12,
            color: AppTheme.textTertiary,
          ),
        ),
      ],
    );
  }
}
