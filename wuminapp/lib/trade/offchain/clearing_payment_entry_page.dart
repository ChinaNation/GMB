import 'package:flutter/material.dart';
import 'package:wuminapp_mobile/trade/offchain/clearing_bank_list_page.dart';
import 'package:wuminapp_mobile/trade/offchain/deposit_page.dart';
import 'package:wuminapp_mobile/trade/offchain/offchain_clearing_receive_page.dart';
import 'package:wuminapp_mobile/trade/offchain/withdraw_page.dart';
import 'package:wuminapp_mobile/wallet/core/wallet_manager.dart';

/// 扫码支付清算体系 Step 1 新增:清算行支付**统一入口页**。
///
/// 中文注释:
/// - 把 4 个新页面(绑定清算行 / 充值 / 提现 / 扫码付)聚合到这里,作为
///   wallet 详情页跳进来的单一入口,避免 wallet 页改动量过大。
/// - Step 2 起接入完整 UI 后,可以把这里的按钮直接平铺到 wallet 详情页,
///   并删除本页;wallet 端只需要保留对 4 个目标页的入口。
class ClearingPaymentEntryPage extends StatelessWidget {
  const ClearingPaymentEntryPage({
    super.key,
    required this.wallet,
    required this.sfidBaseUrl,
    this.clearingNodeWssUrl,
  });

  final WalletProfile wallet;

  /// SFID 后端 baseUrl(用于清算行列表搜索)。
  final String sfidBaseUrl;

  /// 当前绑定的清算行节点 WSS(用于查存款余额),未绑定时为 null。
  final String? clearingNodeWssUrl;

  @override
  Widget build(BuildContext context) {
    return Scaffold(
      appBar: AppBar(title: const Text('扫码支付(清算行)')),
      body: ListView(
        padding: const EdgeInsets.symmetric(vertical: 12),
        children: [
          _entry(
            context,
            icon: Icons.add_business_outlined,
            title: '选择/绑定清算行',
            subtitle: '从 SFID 系统拉取清算行列表;绑定即开户,无预存',
            onTap: () => Navigator.push(
              context,
              MaterialPageRoute(
                builder: (_) => ClearingBankListPage(
                  wallet: wallet,
                  sfidBaseUrl: sfidBaseUrl,
                ),
              ),
            ),
          ),
          const Divider(height: 1, indent: 16, endIndent: 16),
          _entry(
            context,
            icon: Icons.south_outlined,
            title: '充值',
            subtitle: '从自持账户转入清算行存款',
            onTap: () => Navigator.push(
              context,
              MaterialPageRoute(builder: (_) => DepositPage(wallet: wallet)),
            ),
          ),
          const Divider(height: 1, indent: 16, endIndent: 16),
          _entry(
            context,
            icon: Icons.north_outlined,
            title: '提现',
            subtitle: '从清算行存款提回自持账户',
            onTap: () => Navigator.push(
              context,
              MaterialPageRoute(
                builder: (_) => WithdrawPage(
                  wallet: wallet,
                  wssUrl: clearingNodeWssUrl,
                ),
              ),
            ),
          ),
          const Divider(height: 1, indent: 16, endIndent: 16),
          _entry(
            context,
            icon: Icons.qr_code_2_outlined,
            title: '生成收款码',
            subtitle: '带当前清算行 shenfen_id,付款方扫码即可同行支付',
            onTap: () => Navigator.push(
              context,
              MaterialPageRoute(
                builder: (_) => OffchainClearingReceivePage(
                  wallet: wallet,
                  clearingNodeWssUrl: clearingNodeWssUrl,
                ),
              ),
            ),
          ),
          const Divider(height: 1, indent: 16, endIndent: 16),
          const Padding(
            padding: EdgeInsets.fromLTRB(16, 16, 16, 8),
            child: Text(
              '提示',
              style: TextStyle(fontSize: 13, color: Colors.grey),
            ),
          ),
          const Padding(
            padding: EdgeInsets.symmetric(horizontal: 16),
            child: Text(
              '· 付款入口在主页扫码:扫带 bank 字段的商户码会跳转到新清算行付款页。\n'
              '· Step 1 仅支持同一清算行内付款;跨行与争议仲裁由后续版本提供。\n'
              '· 老"绑定清算省储行"页面绑定的是旧省储行清算模型(ADR-006 已退出)。',
              style: TextStyle(fontSize: 12, color: Colors.grey),
            ),
          ),
        ],
      ),
    );
  }

  Widget _entry(
    BuildContext context, {
    required IconData icon,
    required String title,
    required String subtitle,
    required VoidCallback onTap,
  }) {
    return ListTile(
      leading: Icon(icon),
      title: Text(title),
      subtitle: Text(subtitle, style: const TextStyle(fontSize: 12)),
      trailing: const Icon(Icons.chevron_right),
      onTap: onTap,
    );
  }
}
