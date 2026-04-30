import 'package:flutter/material.dart';

import 'package:wuminapp_mobile/rpc/chain_rpc.dart';
import 'package:wuminapp_mobile/ui/app_theme.dart';
import 'package:wuminapp_mobile/util/amount_format.dart';
import 'package:wuminapp_mobile/wallet/core/wallet_manager.dart';

/// 钱包链上余额卡(钱包详情页第 3 张卡)。
///
/// 中文注释:
/// - RPC 查最新块,字段 = `free + reserved`,与 polkadot.js apps 的 total 口径一致。
/// - 不再展示卡内刷新按钮,刷新由外层 [WalletDetailPage] 的 RefreshIndicator
///   下拉触发,通过 [GlobalKey<WalletOnchainBalanceCardState>] 调 [refresh()]。
/// - 卡片高度进一步收紧:padding 顶 8 / 底 12;标题与金额行间距 8。
/// - 加载态:金额位显示「— 元」占位,GMB 由外层右下角固定展示。
/// - 错误态:金额位显示「查询失败,点击刷新」,点击触发 [refresh()]。
class WalletOnchainBalanceCard extends StatefulWidget {
  const WalletOnchainBalanceCard({super.key, required this.wallet});

  final WalletProfile wallet;

  @override
  State<WalletOnchainBalanceCard> createState() =>
      WalletOnchainBalanceCardState();
}

/// 中文注释:State 类公开(去掉下划线)是为了支持外层 [GlobalKey] 引用,
/// 下拉刷新时由 [WalletDetailPage] 通过 key 调 [refresh()]。
class WalletOnchainBalanceCardState extends State<WalletOnchainBalanceCard> {
  final ChainRpc _chainRpc = ChainRpc();

  /// 查询结果(yuan),null 表示尚未查询或加载中。
  double? _balance;

  /// 最近一次查询是否失败。失败后 `_balance` 可能保留上一次成功的值,但
  /// UI 优先展示错误态并提供刷新入口。
  bool _hasError = false;

  /// 是否正在刷新。用于防止重复触发刷新。
  bool _isLoading = false;

  @override
  void initState() {
    super.initState();
    refresh();
  }

  /// 拉取链上 total 余额。
  ///
  /// 中文注释:公开方法,供外层 [WalletDetailPage] 通过 [GlobalKey] 触发下拉刷新。
  Future<void> refresh() async {
    if (_isLoading) return;
    setState(() {
      _isLoading = true;
      _hasError = false;
    });
    try {
      final total = await _chainRpc.fetchTotalBalance(widget.wallet.pubkeyHex);
      if (!mounted) return;
      setState(() {
        _balance = total;
        _isLoading = false;
      });
    } catch (e) {
      debugPrint('[WalletOnchainBalanceCard] fetchTotalBalance failed: $e');
      if (!mounted) return;
      setState(() {
        _hasError = true;
        _isLoading = false;
      });
    }
  }

  @override
  Widget build(BuildContext context) {
    return Container(
      decoration: AppTheme.cardDecoration(radius: AppTheme.radiusLg),
      padding: const EdgeInsets.fromLTRB(16, 8, 16, 12),
      child: Column(
        crossAxisAlignment: CrossAxisAlignment.start,
        children: [
          const Text(
            '链上余额',
            style: TextStyle(
              fontSize: 14,
              fontWeight: FontWeight.w600,
              color: AppTheme.textSecondary,
            ),
          ),
          const SizedBox(height: 8),
          Row(
            crossAxisAlignment: CrossAxisAlignment.end,
            children: [
              Expanded(child: _buildAmountSection()),
              const SizedBox(width: 8),
              const Padding(
                padding: EdgeInsets.only(bottom: 4),
                child: Text(
                  'GMB',
                  style: TextStyle(
                    fontSize: 15,
                    fontWeight: FontWeight.w500,
                    color: AppTheme.textTertiary,
                  ),
                ),
              ),
            ],
          ),
        ],
      ),
    );
  }

  /// 金额区:根据状态切换占位 / 错误提示 / 正常金额。
  Widget _buildAmountSection() {
    // 错误态:点击再次触发刷新。
    if (_hasError && _balance == null) {
      return GestureDetector(
        onTap: refresh,
        child: const Text(
          '查询失败,点击刷新',
          style: TextStyle(
            fontSize: 15,
            fontWeight: FontWeight.w600,
            color: AppTheme.danger,
          ),
        ),
      );
    }
    // 加载态 / 初始态:占位「— 元」,GMB 由外层右下角 Row 固定展示。
    if (_balance == null) {
      return const Text(
        '— 元',
        style: TextStyle(
          fontSize: 22,
          fontWeight: FontWeight.w700,
          color: AppTheme.textTertiary,
        ),
      );
    }
    // 正常态:金额(32 号)+ 元(22 号)。
    return Row(
      crossAxisAlignment: CrossAxisAlignment.baseline,
      textBaseline: TextBaseline.alphabetic,
      mainAxisSize: MainAxisSize.min,
      children: [
        Flexible(
          child: FittedBox(
            fit: BoxFit.scaleDown,
            alignment: Alignment.centerLeft,
            child: Text(
              AmountFormat.format(_balance!, symbol: ''),
              style: const TextStyle(
                fontSize: 32,
                fontWeight: FontWeight.w700,
                color: AppTheme.primaryDark,
              ),
            ),
          ),
        ),
        const Text(
          '元',
          style: TextStyle(
            fontSize: 22,
            fontWeight: FontWeight.w700,
            color: AppTheme.primaryDark,
          ),
        ),
      ],
    );
  }
}
