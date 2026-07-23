import 'package:flutter/material.dart';

import 'package:citizenapp/ui/app_theme.dart';
import 'topup_api.dart';
import 'topup_models.dart';
import 'topup_result_page.dart';
import 'topup_webview_page.dart';

/// 链上充值页(第 1 屏):USDC/USDT 两条入金轨 → 套餐弹窗 → WalletConnect(WebView) 支付。
///
/// 币轨/收款地址/套餐全部来自 Worker config(App 不写死合约);支付通过 WebView 内的
/// WalletConnect(AppKit JS)连自托管钱包并发 ERC-20 转账,拿 txHash 后回到 App 上报并轮询到账。
class OnchainTopupPage extends StatefulWidget {
  const OnchainTopupPage({super.key, required this.accountId, this.api});

  /// 收公民币的公民链账户(充值目标 `account_id`,0x+64 hex)。
  /// 付款用的自托管钱包由 WalletConnect 另接。
  final String accountId;

  /// 注入用于测试;生产用默认 TopupApi(连当前 Worker)。
  final TopupApi? api;

  @override
  State<OnchainTopupPage> createState() => _OnchainTopupPageState();
}

class _OnchainTopupPageState extends State<OnchainTopupPage> {
  late final TopupApi _api = widget.api ?? TopupApi();
  late Future<TopupConfig> _configFuture;

  @override
  void initState() {
    super.initState();
    _configFuture = _api.fetchConfig();
  }

  void _reload() {
    setState(() => _configFuture = _api.fetchConfig());
  }

  @override
  Widget build(BuildContext context) {
    return Scaffold(
      appBar: AppBar(title: const Text('链上充值'), centerTitle: true),
      body: FutureBuilder<TopupConfig>(
        future: _configFuture,
        builder: (context, snapshot) {
          if (snapshot.connectionState != ConnectionState.done) {
            return const Center(child: CircularProgressIndicator());
          }
          if (snapshot.hasError || snapshot.data == null) {
            return _ErrorState(onRetry: _reload);
          }
          return _buildLoaded(context, snapshot.data!);
        },
      ),
    );
  }

  Widget _buildLoaded(BuildContext context, TopupConfig config) {
    if (config.rails.isEmpty) {
      return const Center(
        child: Padding(
          padding: EdgeInsets.all(24),
          child:
              Text('充值渠道尚未开放', style: TextStyle(color: AppTheme.textTertiary)),
        ),
      );
    }
    return ListView(
      padding: const EdgeInsets.all(16),
      children: [
        const Text(
          '用稳定币购买公民币，到账后自动转入你的链上钱包。',
          style: TextStyle(
              fontSize: 13, color: AppTheme.textSecondary, height: 1.6),
        ),
        const SizedBox(height: 16),
        ...config.rails.map((rail) => _RailCard(
              rail: rail,
              onTap: () => _openPackageSheet(context, config, rail),
            )),
        const SizedBox(height: 8),
        const Row(
          children: [
            Icon(Icons.account_balance_wallet_outlined,
                size: 16, color: AppTheme.textTertiary),
            SizedBox(width: 8),
            Expanded(
              child: Text(
                '支持 MetaMask / OKX / Bitget 钱包（WalletConnect）',
                style: TextStyle(fontSize: 12, color: AppTheme.textTertiary),
              ),
            ),
          ],
        ),
      ],
    );
  }

  Future<void> _openPackageSheet(
    BuildContext context,
    TopupConfig config,
    TopupRail rail,
  ) async {
    final package = await showModalBottomSheet<TopupPackage>(
      context: context,
      isScrollControlled: true,
      backgroundColor: Colors.transparent,
      builder: (_) => _PackageSheet(rail: rail, packages: config.packages),
    );
    if (package == null || !mounted) return;
    await _startPayment(config, rail, package);
  }

  Future<void> _startPayment(
    TopupConfig config,
    TopupRail rail,
    TopupPackage package,
  ) async {
    // WebView 内 WalletConnect 连钱包并发 ERC-20 转账,返回付款交易哈希与付款地址。
    final web = await Navigator.of(context).push<TopupWebResult>(
      MaterialPageRoute(
        builder: (_) => TopupWebviewPage(
          rail: rail,
          package: package,
          recvAddress: config.recvAddress,
          accountId: widget.accountId,
        ),
      ),
    );
    if (web == null || !mounted) return;
    await Navigator.of(context).push(
      MaterialPageRoute(
        builder: (_) => TopupResultPage(
          api: _api,
          rail: rail,
          package: package,
          accountId: widget.accountId,
          evmTxHash: web.txHash,
          payerAddress: web.payerAddress,
        ),
      ),
    );
  }
}

/// 单条币轨卡片(USDC · Base / USDT · Arbitrum)。
class _RailCard extends StatelessWidget {
  const _RailCard({required this.rail, required this.onTap});

  final TopupRail rail;
  final VoidCallback onTap;

  @override
  Widget build(BuildContext context) {
    return Padding(
      padding: const EdgeInsets.only(bottom: 12),
      child: Material(
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
                  child: const Icon(Icons.paid_outlined,
                      size: 22, color: AppTheme.primary),
                ),
                const SizedBox(width: 14),
                Expanded(
                  child: Column(
                    crossAxisAlignment: CrossAxisAlignment.start,
                    mainAxisSize: MainAxisSize.min,
                    children: [
                      Text(rail.token,
                          style: const TextStyle(
                              fontSize: 16,
                              fontWeight: FontWeight.w600,
                              color: AppTheme.textPrimary)),
                      const SizedBox(height: 2),
                      Text(rail.label,
                          style: const TextStyle(
                              fontSize: 12, color: AppTheme.textTertiary)),
                    ],
                  ),
                ),
                const Text('去支付',
                    style: TextStyle(fontSize: 13, color: AppTheme.primary)),
                const Icon(Icons.chevron_right,
                    size: 18, color: AppTheme.textTertiary),
              ],
            ),
          ),
        ),
      ),
    );
  }
}

/// 套餐选择弹窗(第 2 屏):选套餐 → 连接钱包并支付。
class _PackageSheet extends StatefulWidget {
  const _PackageSheet({required this.rail, required this.packages});

  final TopupRail rail;
  final List<TopupPackage> packages;

  @override
  State<_PackageSheet> createState() => _PackageSheetState();
}

class _PackageSheetState extends State<_PackageSheet> {
  int _selected = 0;

  @override
  Widget build(BuildContext context) {
    final packages = widget.packages;
    return Container(
      decoration: const BoxDecoration(
        color: AppTheme.surfaceCard,
        borderRadius: BorderRadius.vertical(top: Radius.circular(20)),
      ),
      padding: const EdgeInsets.fromLTRB(16, 12, 16, 24),
      child: Column(
        mainAxisSize: MainAxisSize.min,
        crossAxisAlignment: CrossAxisAlignment.stretch,
        children: [
          Center(
            child: Container(
              width: 36,
              height: 4,
              margin: const EdgeInsets.only(bottom: 16),
              decoration: BoxDecoration(
                color: AppTheme.border,
                borderRadius: BorderRadius.circular(2),
              ),
            ),
          ),
          Text('选择充值套餐 · ${widget.rail.label}',
              style: const TextStyle(
                  fontSize: 15,
                  fontWeight: FontWeight.w600,
                  color: AppTheme.textPrimary)),
          const SizedBox(height: 14),
          ...List.generate(packages.length, (index) {
            final isLast = index == packages.length - 1;
            return Padding(
              padding: EdgeInsets.only(
                  bottom: index == packages.length - 1 ? 18 : 10),
              child: _PackageOption(
                token: widget.rail.token,
                package: packages[index],
                selected: index == _selected,
                // 末档为大额档,标「更优汇率」对应批量优惠。
                showBetterRate: isLast && packages.length > 1,
                onTap: () => setState(() => _selected = index),
              ),
            );
          }),
          FilledButton(
            style: FilledButton.styleFrom(
              backgroundColor: AppTheme.primary,
              padding: const EdgeInsets.symmetric(vertical: 14),
              shape: RoundedRectangleBorder(
                  borderRadius: BorderRadius.circular(14)),
            ),
            onPressed: packages.isEmpty
                ? null
                : () => Navigator.of(context).pop(packages[_selected]),
            child: const Text('连接钱包并支付', style: TextStyle(fontSize: 15)),
          ),
          const SizedBox(height: 10),
          const Row(
            mainAxisAlignment: MainAxisAlignment.center,
            children: [
              Icon(Icons.open_in_new, size: 14, color: AppTheme.textTertiary),
              SizedBox(width: 6),
              Text('将通过 WalletConnect 打开你的钱包确认支付',
                  style: TextStyle(fontSize: 12, color: AppTheme.textTertiary)),
            ],
          ),
        ],
      ),
    );
  }
}

class _PackageOption extends StatelessWidget {
  const _PackageOption({
    required this.token,
    required this.package,
    required this.selected,
    required this.showBetterRate,
    required this.onTap,
  });

  final String token;
  final TopupPackage package;
  final bool selected;
  final bool showBetterRate;
  final VoidCallback onTap;

  @override
  Widget build(BuildContext context) {
    return InkWell(
      borderRadius: BorderRadius.circular(14),
      onTap: onTap,
      child: Container(
        padding: const EdgeInsets.all(14),
        decoration: BoxDecoration(
          color: AppTheme.surfaceCard,
          borderRadius: BorderRadius.circular(14),
          border: Border.all(
            color: selected ? AppTheme.primary : AppTheme.border,
            width: selected ? 2 : 0.5,
          ),
        ),
        child: Row(
          children: [
            Expanded(
              child: Column(
                crossAxisAlignment: CrossAxisAlignment.start,
                mainAxisSize: MainAxisSize.min,
                children: [
                  Text('${package.coinDisplay} 公民币',
                      style: const TextStyle(
                          fontSize: 18,
                          fontWeight: FontWeight.w600,
                          color: AppTheme.textPrimary)),
                  const SizedBox(height: 2),
                  Text('支付 ${package.payDisplay} $token',
                      style: const TextStyle(
                          fontSize: 12, color: AppTheme.textSecondary)),
                ],
              ),
            ),
            if (showBetterRate)
              Container(
                padding: const EdgeInsets.symmetric(horizontal: 8, vertical: 3),
                decoration: BoxDecoration(
                  color: AppTheme.gold.withAlpha(31),
                  borderRadius: BorderRadius.circular(999),
                ),
                child: const Text('更优汇率',
                    style: TextStyle(fontSize: 11, color: AppTheme.gold)),
              )
            else if (selected)
              const Icon(Icons.check_circle, size: 22, color: AppTheme.primary),
          ],
        ),
      ),
    );
  }
}

class _ErrorState extends StatelessWidget {
  const _ErrorState({required this.onRetry});

  final VoidCallback onRetry;

  @override
  Widget build(BuildContext context) {
    return Center(
      child: Column(
        mainAxisSize: MainAxisSize.min,
        children: [
          const Text('加载充值渠道失败',
              style: TextStyle(color: AppTheme.textSecondary)),
          const SizedBox(height: 12),
          OutlinedButton(onPressed: onRetry, child: const Text('重试')),
        ],
      ),
    );
  }
}
