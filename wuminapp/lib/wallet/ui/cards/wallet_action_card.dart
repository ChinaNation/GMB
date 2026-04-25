import 'package:flutter/material.dart';

import 'package:wuminapp_mobile/ui/app_theme.dart';
import 'package:wuminapp_mobile/wallet/core/wallet_manager.dart';

/// 钱包详情页第 2 卡片:3 列等宽布局(充值/提现/余额)。
///
/// 中文注释:
/// - 布局:Row + 3 个 Expanded,三列等宽,spaceAround 分布。
/// - 充值列 / 提现列:可点击,SnackBar「功能开发中」,下方用非断空格占位保持和
///   余额列底部对齐。
/// - 余额列:**静态展示**,严格不加 InkWell / GestureDetector / onTap 回调,
///   下方小字 `0.00 元` 为占位,等清算行功能落地后接真实数据。
/// - wallet 参数当前未使用,保留作后续业务对接时的入参(充值链路会要用钱包地
///   址去查清算行余额)。
class WalletActionCard extends StatelessWidget {
  const WalletActionCard({super.key, required this.wallet});

  final WalletProfile wallet;

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
              onTap: () => _showDevSnackBar(context),
            ),
          ),
          Expanded(
            child: _ClickableAction(
              icon: Icons.arrow_circle_up_outlined,
              label: '提现',
              onTap: () => _showDevSnackBar(context),
            ),
          ),
          const Expanded(
            child: _StaticBalance(),
          ),
        ],
      ),
    );
  }

  /// 统一的「功能开发中」提示。
  static void _showDevSnackBar(BuildContext context) {
    ScaffoldMessenger.of(context).showSnackBar(
      const SnackBar(
        content: Text('功能开发中'),
        duration: Duration(seconds: 2),
      ),
    );
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
  const _StaticBalance();

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
        // 中文注释:占位文本,等清算行功能落地后接真实数据。
        const Text(
          '0.00 元',
          style: TextStyle(
            fontSize: 12,
            color: AppTheme.textTertiary,
          ),
        ),
      ],
    );
  }
}
