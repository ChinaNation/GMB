import 'package:flutter/material.dart';

import 'package:citizenapp/my/creator/creator_money.dart';
import 'package:citizenapp/my/creator/models/creator_overview.dart';
import 'package:citizenapp/ui/app_theme.dart';

/// 创作者概览卡：订阅人数 + 预计月收入（金色锚点）+ 预计提示。
class CreatorOverviewCard extends StatelessWidget {
  const CreatorOverviewCard({super.key, required this.overview});

  final CreatorOverview overview;

  @override
  Widget build(BuildContext context) {
    return Container(
      decoration: AppTheme.cardDecoration(radius: AppTheme.radiusLg),
      padding: const EdgeInsets.all(16),
      child: Column(
        crossAxisAlignment: CrossAxisAlignment.start,
        children: [
          Row(
            children: [
              Container(
                width: 26,
                height: 26,
                alignment: Alignment.center,
                decoration: BoxDecoration(
                  color: AppTheme.primary.withAlpha(24),
                  borderRadius: BorderRadius.circular(AppTheme.radiusSm),
                ),
                child: const Icon(Icons.storefront_outlined,
                    size: 16, color: AppTheme.primary),
              ),
              const SizedBox(width: 8),
              const Text(
                '我的创作者会员',
                style: TextStyle(
                  fontSize: 14,
                  fontWeight: FontWeight.w600,
                  color: AppTheme.textPrimary,
                ),
              ),
              const Spacer(),
              Container(
                padding: const EdgeInsets.symmetric(horizontal: 8, vertical: 2),
                decoration: BoxDecoration(
                  color: AppTheme.primary.withAlpha(24),
                  borderRadius: BorderRadius.circular(20),
                ),
                child: const Text('已开通',
                    style:
                        TextStyle(fontSize: 11, color: AppTheme.primaryDark)),
              ),
            ],
          ),
          const SizedBox(height: 12),
          Row(
            children: [
              Expanded(
                child: _stat(
                  '订阅人数',
                  overview.subscriberCount.toString(),
                  AppTheme.primary,
                ),
              ),
              const SizedBox(width: 12),
              Expanded(
                child: _stat(
                  '本月已收入',
                  '${fenToYuanLabel(overview.monthIncomeFen)} 元',
                  AppTheme.gold,
                ),
              ),
            ],
          ),
          const SizedBox(height: 10),
          const Text(
            '数据来自链上扣款镜像 · 完整收入台账随税务功能上线',
            style: TextStyle(fontSize: 11, color: AppTheme.textTertiary),
          ),
        ],
      ),
    );
  }

  Widget _stat(String label, String value, Color valueColor) {
    return Container(
      padding: const EdgeInsets.symmetric(horizontal: 12, vertical: 10),
      decoration: BoxDecoration(
        color: AppTheme.surfaceMuted,
        borderRadius: BorderRadius.circular(AppTheme.radiusMd),
      ),
      child: Column(
        crossAxisAlignment: CrossAxisAlignment.start,
        children: [
          Text(label,
              style:
                  const TextStyle(fontSize: 12, color: AppTheme.textSecondary)),
          const SizedBox(height: 2),
          Text(
            value,
            style: TextStyle(
              fontSize: 24,
              fontWeight: FontWeight.w700,
              color: valueColor,
            ),
          ),
        ],
      ),
    );
  }
}
