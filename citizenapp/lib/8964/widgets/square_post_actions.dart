import 'package:flutter/material.dart';

import 'package:citizenapp/ui/app_theme.dart';

/// 广场卡片底部互动栏（点赞 / 评论 / 收藏），图片/视频/文章卡共用同一版式。
class SquarePostActions extends StatelessWidget {
  const SquarePostActions({super.key});

  @override
  Widget build(BuildContext context) {
    return const Row(
      children: [
        Icon(Icons.thumb_up_alt_outlined,
            size: 18, color: AppTheme.textTertiary),
        SizedBox(width: 18),
        Icon(Icons.mode_comment_outlined,
            size: 18, color: AppTheme.textTertiary),
        SizedBox(width: 18),
        Icon(Icons.bookmark_border_rounded,
            size: 18, color: AppTheme.textTertiary),
      ],
    );
  }
}
