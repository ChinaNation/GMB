import 'package:flutter/material.dart';

import 'package:citizenapp/ui/app_theme.dart';

/// 未开通门禁态：创作者必须先成为平台会员（链上校验）。
///
/// [onOpenMembership] 引导去「会员」页开通平台会员。
class CreatorGateView extends StatelessWidget {
  const CreatorGateView({super.key, required this.onOpenMembership});

  final VoidCallback onOpenMembership;

  @override
  Widget build(BuildContext context) {
    return Center(
      child: SingleChildScrollView(
        padding: const EdgeInsets.symmetric(horizontal: 20, vertical: 28),
        child: Column(
          mainAxisSize: MainAxisSize.min,
          children: [
            Container(
              width: 64,
              height: 64,
              alignment: Alignment.center,
              decoration: BoxDecoration(
                color: AppTheme.primary.withAlpha(24),
                borderRadius: BorderRadius.circular(AppTheme.radiusLg),
              ),
              child: const Icon(Icons.storefront_outlined,
                  size: 32, color: AppTheme.primary),
            ),
            const SizedBox(height: 14),
            const Text(
              '成为创作者',
              style: TextStyle(
                fontSize: 18,
                fontWeight: FontWeight.w700,
                color: AppTheme.textPrimary,
              ),
            ),
            const SizedBox(height: 8),
            const Text(
              '设置你的会员档位，粉丝用公民币订阅你，订阅款全额进你的钱包。',
              textAlign: TextAlign.center,
              style: TextStyle(
                fontSize: 13,
                height: 1.6,
                color: AppTheme.textSecondary,
              ),
            ),
            const SizedBox(height: 20),
            Container(
              width: double.infinity,
              padding: const EdgeInsets.all(14),
              decoration: AppTheme.bannerDecoration(AppTheme.warning),
              child: const Row(
                crossAxisAlignment: CrossAxisAlignment.start,
                children: [
                  Icon(Icons.lock_outline, size: 18, color: AppTheme.warning),
                  SizedBox(width: 10),
                  Expanded(
                    child: Column(
                      crossAxisAlignment: CrossAxisAlignment.start,
                      children: [
                        Text(
                          '需先成为平台会员',
                          style: TextStyle(
                            fontSize: 13,
                            fontWeight: FontWeight.w600,
                            color: AppTheme.textPrimary,
                          ),
                        ),
                        SizedBox(height: 2),
                        Text(
                          '开通创作者会员前，你需先订阅平台会员（链上校验）。',
                          style: TextStyle(
                            fontSize: 12,
                            height: 1.5,
                            color: AppTheme.textSecondary,
                          ),
                        ),
                      ],
                    ),
                  ),
                ],
              ),
            ),
            const SizedBox(height: 16),
            FilledButton.icon(
              onPressed: onOpenMembership,
              icon: const Icon(Icons.workspace_premium_outlined, size: 19),
              label: const Text('去开通平台会员'),
            ),
          ],
        ),
      ),
    );
  }
}
