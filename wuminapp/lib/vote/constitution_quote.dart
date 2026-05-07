import 'package:flutter/material.dart';

import 'package:wuminapp_mobile/ui/app_theme.dart';

/// 公民宪法引言水印。
///
/// 投票 tab(vote_view) 的底层背景，由 Stack + Opacity 包装，
/// 始终若隐若现显示。空态时与之等同。
class ConstitutionQuote extends StatelessWidget {
  const ConstitutionQuote({super.key});

  @override
  Widget build(BuildContext context) {
    return const Center(
      child: Padding(
        padding: EdgeInsets.symmetric(horizontal: 32),
        child: Column(
          mainAxisSize: MainAxisSize.min,
          children: [
            Text(
              '一个国家/社会是由每个公民组成的，'
              '每个公民都应拥有投票权，'
              '"公民"App致力于让所有公权力在阳光下运行、'
              '让所有公权力接受公民的监督、'
              '让所有公权力由公民票选产生！',
              textAlign: TextAlign.center,
              style: TextStyle(
                fontSize: 15,
                height: 1.8,
                color: AppTheme.textSecondary,
                letterSpacing: 0.3,
              ),
            ),
            SizedBox(height: 20),
            SizedBox(
              width: 160,
              child: Divider(
                color: AppTheme.textTertiary,
                thickness: 0.8,
              ),
            ),
            SizedBox(height: 14),
            Text(
              '《公民宪法》撰写人 · 程伟',
              textAlign: TextAlign.center,
              style: TextStyle(
                fontSize: 13,
                color: AppTheme.textTertiary,
                letterSpacing: 0.5,
              ),
            ),
          ],
        ),
      ),
    );
  }
}
