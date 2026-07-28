import 'package:flutter/material.dart';

import 'package:citizenapp/ui/app_theme.dart';
import 'package:citizenapp/wallet/core/wallet_manager.dart' show Account;

/// 弹出「更换身份账户」底部面板:从本地其他账户中挑一个作为匿名 CID 的换绑目标。
///
/// 返回选定账户的 `accountId`;取消 / 无可选账户返回 `null`。授权与提交由调用方
/// ([MyIdService.rebindCidTo])承接;目标账户已由调用方过滤掉当前身份账户。
Future<String?> showRebindAccountSheet(
  BuildContext context, {
  required List<Account> targets,
}) {
  return showModalBottomSheet<String>(
    context: context,
    isScrollControlled: true,
    builder: (_) => RebindAccountSheet(targets: targets),
  );
}

class RebindAccountSheet extends StatelessWidget {
  const RebindAccountSheet({super.key, required this.targets});

  final List<Account> targets;

  static String _shortAddress(String value) {
    if (value.length <= 18) return value;
    return '${value.substring(0, 8)}…${value.substring(value.length - 8)}';
  }

  @override
  Widget build(BuildContext context) {
    final bottomInset = MediaQuery.viewInsetsOf(context).bottom;
    return SafeArea(
      child: Padding(
        padding: EdgeInsets.fromLTRB(16, 8, 16, 16 + bottomInset),
        child: Column(
          mainAxisSize: MainAxisSize.min,
          crossAxisAlignment: CrossAxisAlignment.start,
          children: [
            Center(
              child: Container(
                width: 36,
                height: 4,
                margin: const EdgeInsets.only(bottom: 16),
                decoration: BoxDecoration(
                  color: AppTheme.textTertiary,
                  borderRadius: BorderRadius.circular(2),
                ),
              ),
            ),
            const Text(
              '更换身份账户',
              style: TextStyle(
                fontSize: 17,
                fontWeight: FontWeight.w700,
                color: AppTheme.textPrimary,
              ),
            ),
            const SizedBox(height: 6),
            const Text(
              '把当前身份 CID 号换绑到下面选中的账户。换绑需当前账户与目标账户各授权'
              '一次(各弹一次生物识别)。',
              style: TextStyle(fontSize: 12, color: AppTheme.textSecondary),
            ),
            const SizedBox(height: 16),
            if (targets.isEmpty)
              Container(
                width: double.infinity,
                padding: const EdgeInsets.fromLTRB(12, 12, 12, 12),
                decoration: AppTheme.bannerDecoration(AppTheme.warning),
                child: const Text(
                  '暂无可换绑的其他账户。请先在钱包里添加账户,再来更换。',
                  style: TextStyle(
                    fontSize: 12,
                    height: 1.5,
                    color: AppTheme.textSecondary,
                    fontWeight: FontWeight.w600,
                  ),
                ),
              )
            else
              ...targets.map(
                (account) => Padding(
                  padding: const EdgeInsets.only(bottom: 10),
                  child: _AccountOption(
                    account: account,
                    shortAddress: _shortAddress(account.ss58Address),
                    onTap: () => Navigator.of(context).pop(account.accountId),
                  ),
                ),
              ),
          ],
        ),
      ),
    );
  }
}

/// 单个可换绑账户行:账户名 + 序号 + 短地址,点击即选中并回传。
class _AccountOption extends StatelessWidget {
  const _AccountOption({
    required this.account,
    required this.shortAddress,
    required this.onTap,
  });

  final Account account;
  final String shortAddress;
  final VoidCallback onTap;

  @override
  Widget build(BuildContext context) {
    return InkWell(
      borderRadius: BorderRadius.circular(AppTheme.radiusLg),
      onTap: onTap,
      child: Container(
        width: double.infinity,
        padding: const EdgeInsets.all(14),
        decoration: BoxDecoration(
          color: AppTheme.surfaceCard,
          borderRadius: BorderRadius.circular(AppTheme.radiusLg),
          border: Border.all(color: AppTheme.border),
        ),
        child: Row(
          children: [
            Expanded(
              child: Column(
                crossAxisAlignment: CrossAxisAlignment.start,
                children: [
                  Row(
                    children: [
                      Flexible(
                        child: Text(
                          account.accountName,
                          maxLines: 1,
                          overflow: TextOverflow.ellipsis,
                          style: const TextStyle(
                            fontSize: 15,
                            fontWeight: FontWeight.w700,
                            color: AppTheme.textPrimary,
                          ),
                        ),
                      ),
                      const SizedBox(width: 8),
                      Text(
                        '#${account.accountIndex}',
                        style: const TextStyle(
                          fontSize: 12,
                          color: AppTheme.textTertiary,
                          fontWeight: FontWeight.w600,
                        ),
                      ),
                    ],
                  ),
                  const SizedBox(height: 3),
                  Text(
                    shortAddress,
                    style: const TextStyle(
                      fontSize: 12,
                      fontFamily: 'monospace',
                      color: AppTheme.textSecondary,
                    ),
                  ),
                ],
              ),
            ),
            const Icon(
              Icons.chevron_right,
              color: AppTheme.textTertiary,
              size: 20,
            ),
          ],
        ),
      ),
    );
  }
}
