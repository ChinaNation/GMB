import 'package:flutter/material.dart';
import 'package:flutter_svg/flutter_svg.dart';
import 'package:wuminapp_mobile/duoqian/institution/institution_duoqian_list_page.dart';
import 'package:wuminapp_mobile/duoqian/personal/personal_duoqian_list_page.dart';
import 'package:wuminapp_mobile/offchain/services/offchain_scan_flow.dart';
import 'package:wuminapp_mobile/onchain/onchain_payment_page.dart';
import 'package:wuminapp_mobile/ui/app_theme.dart';
import 'package:wuminapp_mobile/wallet/core/wallet_manager.dart';

/// 交易 Tab 页面。
///
/// 中文注释：本页只负责交易页入口编排；链上支付主体仍由 onchain 模块渲染，
/// 扫码支付、多签内部业务仍留在各自功能域。
class TransactionTabPage extends StatelessWidget {
  const TransactionTabPage({super.key});

  Future<void> _openScanPayment(
    BuildContext context,
    WalletProfile? wallet,
  ) async {
    await openOffchainScanPaymentFlow(context: context, wallet: wallet);
  }

  void _push(BuildContext context, Widget page) {
    Navigator.of(context).push(MaterialPageRoute(builder: (_) => page));
  }

  @override
  Widget build(BuildContext context) {
    return OnchainPaymentPanel(
      title: '交易',
      extraEntriesBuilder: (context, wallet) => [
        _TransactionEntryRow(
          icon: const Icon(
            Icons.person_outline,
            size: 18,
            color: AppTheme.primary,
          ),
          title: '个人多签',
          onTap: () => _push(context, const PersonalDuoqianListPage()),
        ),
        const SizedBox(height: 12),
        _TransactionEntryRow(
          icon: const Icon(
            Icons.account_balance_outlined,
            size: 18,
            color: AppTheme.primary,
          ),
          title: '机构多签',
          onTap: () => _push(context, const InstitutionDuoqianListPage()),
        ),
        const SizedBox(height: 12),
        _TransactionEntryRow(
          icon: SvgPicture.asset(
            'assets/icons/scan-line.svg',
            width: 18,
            height: 18,
            colorFilter: const ColorFilter.mode(
              AppTheme.primary,
              BlendMode.srcIn,
            ),
          ),
          title: '扫码支付',
          onTap: () => _openScanPayment(context, wallet),
        ),
        const SizedBox(height: 12),
      ],
    );
  }
}

class _TransactionEntryRow extends StatelessWidget {
  const _TransactionEntryRow({
    required this.icon,
    required this.title,
    required this.onTap,
  });

  final Widget icon;
  final String title;
  final VoidCallback onTap;

  @override
  Widget build(BuildContext context) {
    return Container(
      decoration: AppTheme.cardDecoration(),
      child: Material(
        color: Colors.transparent,
        child: InkWell(
          onTap: onTap,
          borderRadius: BorderRadius.circular(AppTheme.radiusMd),
          child: Padding(
            padding: const EdgeInsets.symmetric(horizontal: 14, vertical: 10),
            child: Row(
              children: [
                Container(
                  width: 36,
                  height: 36,
                  decoration: BoxDecoration(
                    color: AppTheme.primary.withAlpha(20),
                    borderRadius: BorderRadius.circular(10),
                  ),
                  child: Center(child: icon),
                ),
                const SizedBox(width: 12),
                Text(
                  title,
                  style: const TextStyle(
                    fontSize: 15,
                    fontWeight: FontWeight.w600,
                    color: AppTheme.textPrimary,
                  ),
                ),
                const Spacer(),
                const Icon(
                  Icons.chevron_right,
                  size: 20,
                  color: AppTheme.textTertiary,
                ),
              ],
            ),
          ),
        ),
      ),
    );
  }
}
