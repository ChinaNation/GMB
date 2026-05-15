import 'package:flutter/material.dart';

import 'package:wuminapp_mobile/ui/app_theme.dart';

/// 公权 tab 占位页。
///
/// 后续接入：左侧省/市/省直机构垂直导航 + 公权机构列表
/// (立法院/司法院/监察院/政府/教育委员会等)。
class PublicPage extends StatelessWidget {
  const PublicPage({super.key});

  @override
  Widget build(BuildContext context) {
    return ListView(
      padding: const EdgeInsets.fromLTRB(16, 16, 16, 24),
      children: const [
        Text(
          '公权机构',
          style: TextStyle(
            fontSize: 22,
            fontWeight: FontWeight.w700,
            color: AppTheme.textPrimary,
          ),
        ),
        SizedBox(height: 80),
        Center(
          child: Padding(
            padding: EdgeInsets.symmetric(horizontal: 32),
            child: Column(
              mainAxisSize: MainAxisSize.min,
              children: [
                Icon(
                  Icons.account_balance_outlined,
                  size: 48,
                  color: AppTheme.textTertiary,
                ),
                SizedBox(height: 12),
                Text(
                  '建设中',
                  style: TextStyle(
                    fontSize: 16,
                    color: AppTheme.textSecondary,
                  ),
                ),
                SizedBox(height: 4),
                Text(
                  '公权机构与省/市导航即将上线',
                  style: TextStyle(
                    fontSize: 13,
                    color: AppTheme.textTertiary,
                  ),
                ),
              ],
            ),
          ),
        ),
      ],
    );
  }
}
