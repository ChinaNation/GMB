import 'package:flutter/material.dart';

import 'package:wuminapp_mobile/offchain/rpc/offchain_clearing_rpc.dart';
import 'package:wuminapp_mobile/offchain/services/clearing_bank_prefs.dart';
import 'package:wuminapp_mobile/offchain/pages/deposit_page.dart';
import 'package:wuminapp_mobile/offchain/pages/withdraw_page.dart';
import 'package:wuminapp_mobile/ui/app_theme.dart';
import 'package:wuminapp_mobile/wallet/core/wallet_manager.dart';

/// 钱包详情页第 2 卡片:3 列等宽布局(充值/提现/余额)。
///
/// 中文注释:
/// - 布局:Row + 3 个 Expanded,三列等宽,spaceAround 分布。
/// - 充值列 / 提现列:已绑定清算行时进入真实充值 / 提现页;未绑定时提示先绑定。
/// - 余额列:**静态展示**,严格不加 InkWell / GestureDetector / onTap 回调。
/// - 清算行余额来自当前绑定快照中的节点端点,通过 `offchain_queryBalance`
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
      final balance = await OffchainClearingNodeRpc(
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
      // 中文注释:三列布局相对原两列更拥挤,padding 调小避免卡片臃肿。
      padding: const EdgeInsets.symmetric(vertical: 16, horizontal: 12),
      child: Row(
        mainAxisAlignment: MainAxisAlignment.spaceAround,
        children: [
          Expanded(
            child: _ClickableAction(
              icon: Icons.arrow_circle_down_outlined,
              label: '充值',
              onTap: () => _openDeposit(context),
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
            child: _StaticBalance(balanceText: _balanceText),
          ),
        ],
      ),
    );
  }

  Future<void> _openDeposit(BuildContext context) async {
    final binding = _binding;
    if (binding == null) {
      _showNeedBinding(context);
      return;
    }
    await Navigator.push(
      context,
      MaterialPageRoute(builder: (_) => DepositPage(wallet: widget.wallet)),
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
/// 中文注释:
/// - 使用 `Material + InkWell` 组合,ripple 限制在 `CircleBorder` 内,不溢出圆圈。
/// - 底部用一个非断空格 `\u00A0` 占位,保证和余额列的 `0.00 元` 行高对齐,
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
        // 中文注释:非断空格占位,保证三列底部和余额列的 0.00 元对齐。
        const Text(
          '\u00A0',
          style: TextStyle(
            fontSize: 12,
            color: AppTheme.textTertiary,
          ),
        ),
      ],
    );
  }
}

/// 余额列:纯静态展示,**禁止**包 InkWell / GestureDetector,不响应任何点击。
///
/// 中文注释:
/// - 图标用普通 Container + BoxShape.circle,没有 Material 涟漪,也没有 onTap。
/// - `0.00 元` 是占位,等清算行功能落地后接真实数据。
class _StaticBalance extends StatelessWidget {
  const _StaticBalance({required this.balanceText});

  final String balanceText;

  @override
  Widget build(BuildContext context) {
    return Column(
      mainAxisSize: MainAxisSize.min,
      children: [
        Container(
          width: 56,
          height: 56,
          decoration: BoxDecoration(
            color: AppTheme.primary.withAlpha(15),
            shape: BoxShape.circle,
          ),
          child: const Icon(
            Icons.account_balance_wallet_outlined,
            size: 28,
            color: AppTheme.primaryDark,
          ),
        ),
        const SizedBox(height: 8),
        const Text(
          '余额',
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
