import 'package:flutter/material.dart';

import 'package:citizenapp/ui/app_theme.dart';

/// 底部“广场”Tab 入口。
///
/// 当前只提供稳定入口壳，后续广场功能统一放在 `lib/8964/` 下扩展。
class SquareTabPage extends StatelessWidget {
  const SquareTabPage({super.key});

  @override
  Widget build(BuildContext context) {
    return const SafeArea(
      child: Column(
        children: [
          SizedBox(height: 18),
          Text(
            '广场',
            style: TextStyle(
              fontSize: 20,
              fontWeight: FontWeight.w700,
              color: AppTheme.primaryDark,
            ),
          ),
          Expanded(
            child: Center(
              child: Text(
                '暂未开放',
                style: TextStyle(
                  fontSize: 15,
                  color: AppTheme.textTertiary,
                ),
              ),
            ),
          ),
        ],
      ),
    );
  }
}
