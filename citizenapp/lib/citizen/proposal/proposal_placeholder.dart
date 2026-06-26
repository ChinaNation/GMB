import 'package:flutter/material.dart';

import 'package:citizenapp/ui/app_theme.dart';

/// 提案占位页——某提案种类链端/客户端尚未对接时统一展示「开发中」。
///
/// 中文注释:proposal/ 下每种提案一个文件夹;尚未实现的提案(决议发行/决议销毁/
/// 验证密钥/发起选举)用本占位页,避免空目录,接好后替换为真实发起页。
class ProposalPlaceholderPage extends StatelessWidget {
  const ProposalPlaceholderPage({
    super.key,
    required this.title,
    required this.kind,
  });

  final String title;
  final String kind;

  @override
  Widget build(BuildContext context) {
    return Scaffold(
      backgroundColor: AppTheme.scaffoldBg,
      appBar: AppBar(
        title: Text(title),
        backgroundColor: AppTheme.surfaceWhite,
        foregroundColor: AppTheme.textPrimary,
        elevation: 0,
      ),
      body: Center(
        child: Column(
          mainAxisSize: MainAxisSize.min,
          children: [
            const Icon(Icons.construction_outlined,
                size: 44, color: AppTheme.textTertiary),
            const SizedBox(height: 12),
            Text('「$kind」提案功能开发中',
                style: const TextStyle(
                    fontSize: 14, color: AppTheme.textSecondary)),
          ],
        ),
      ),
    );
  }
}
