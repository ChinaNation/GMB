import 'package:flutter/material.dart';
import 'package:flutter_svg/flutter_svg.dart';
import 'package:citizenapp/transaction/offchain-transaction/services/offchain_scan_flow.dart';
import 'package:citizenapp/transaction/onchain-transaction/onchain_payment_page.dart';
import 'package:citizenapp/transaction/personal-manage/personal_account_list_page.dart';
import 'package:citizenapp/ui/app_theme.dart';
import 'package:citizenapp/wallet/core/wallet_manager.dart';

/// 交易 Tab 页面。
///
/// 中文注释：本页只负责交易页入口编排；链上支付主体仍由 onchain 模块渲染，
/// 扫码支付内部业务仍留在链下支付功能域。
class TransactionTabPage extends StatelessWidget {
  const TransactionTabPage({super.key});

  Future<void> _openScanPayment(
    BuildContext context,
    WalletProfile? wallet,
  ) async {
    await openOffchainScanPaymentFlow(context: context, wallet: wallet);
  }

  Future<void> _openPersonalAccounts(BuildContext context) async {
    await Navigator.of(context).push(
      MaterialPageRoute(builder: (_) => const PersonalAccountListPage()),
    );
  }

  @override
  Widget build(BuildContext context) {
    return OnchainPaymentPanel(
      title: '交易',
      extraEntriesBuilder: (context, wallet) => [
        _TransactionEntryGroup(
          children: [
            _TransactionEntryTile(
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
            _TransactionEntryTile(
              icon: const Icon(
                Icons.account_tree_rounded,
                size: 20,
                color: AppTheme.primary,
              ),
              title: '多签账户',
              onTap: () => _openPersonalAccounts(context),
            ),
          ],
        ),
        const SizedBox(height: 12),
      ],
    );
  }
}

class _TransactionEntryGroup extends StatelessWidget {
  const _TransactionEntryGroup({required this.children});

  final List<_TransactionEntryTile> children;

  @override
  Widget build(BuildContext context) {
    return Container(
      decoration: AppTheme.cardDecoration(),
      child: Row(
        children: [
          for (var i = 0; i < children.length; i++) ...[
            Expanded(child: children[i]),
            if (i != children.length - 1)
              const SizedBox(
                height: 52,
                child: VerticalDivider(
                  width: 1,
                  thickness: 0.5,
                  color: AppTheme.border,
                ),
              ),
          ],
        ],
      ),
    );
  }
}

class _TransactionEntryTile extends StatelessWidget {
  const _TransactionEntryTile({
    required this.icon,
    required this.title,
    required this.onTap,
  });

  final Widget icon;
  final String title;
  final VoidCallback onTap;

  @override
  Widget build(BuildContext context) {
    return Material(
      color: Colors.transparent,
      child: InkWell(
        onTap: onTap,
        borderRadius: BorderRadius.circular(AppTheme.radiusMd),
        child: Padding(
          padding: const EdgeInsets.symmetric(horizontal: 12, vertical: 10),
          child: Row(
            mainAxisAlignment: MainAxisAlignment.center,
            children: [
              Container(
                width: 34,
                height: 34,
                decoration: BoxDecoration(
                  color: AppTheme.primary.withAlpha(20),
                  borderRadius: BorderRadius.circular(10),
                ),
                child: Center(child: icon),
              ),
              const SizedBox(width: 8),
              Flexible(
                child: Text(
                  title,
                  maxLines: 1,
                  overflow: TextOverflow.ellipsis,
                  style: const TextStyle(
                    fontSize: 15,
                    fontWeight: FontWeight.w600,
                    color: AppTheme.textPrimary,
                  ),
                ),
              ),
            ],
          ),
        ),
      ),
    );
  }
}
