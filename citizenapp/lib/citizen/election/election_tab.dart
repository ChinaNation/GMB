import 'package:flutter/material.dart';

import 'package:citizenapp/ui/app_theme.dart';

/// 选举 tab 视图(ADR-028 P2 占位)。
///
/// 中文注释:选举 = 电选各机构管理员/法定代表人的活动视图(按行政层级),走 citizen-vote
/// 选举引擎(P8/P10,链端 `citizen-vote` 当前空骨架)。本期仅占位空态,不接任何链路。
class ElectionTab extends StatelessWidget {
  const ElectionTab({super.key});

  @override
  Widget build(BuildContext context) {
    return const Center(
      child: Padding(
        padding: EdgeInsets.symmetric(horizontal: 32),
        child: Column(
          mainAxisSize: MainAxisSize.min,
          children: [
            Icon(Icons.how_to_vote_outlined,
                size: 48, color: AppTheme.textTertiary),
            SizedBox(height: 14),
            Text('选举',
                style: TextStyle(
                    fontSize: 17,
                    fontWeight: FontWeight.w700,
                    color: AppTheme.textSecondary)),
            SizedBox(height: 8),
            Text('按行政层级电选机构管理员/法定代表人 功能开发中',
                textAlign: TextAlign.center,
                style: TextStyle(fontSize: 13, color: AppTheme.textTertiary)),
          ],
        ),
      ),
    );
  }
}
