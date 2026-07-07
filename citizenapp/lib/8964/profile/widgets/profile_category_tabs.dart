import 'package:flutter/material.dart';

import 'package:citizenapp/ui/app_theme.dart';

/// 用户主页内容分类。照片/视频是从帖子媒体派生的视图，文章是长图文分类。
enum ProfileTab {
  posts('帖子'),
  campaign('竞选'),
  photos('照片'),
  videos('视频'),
  articles('文章');

  const ProfileTab(this.label);

  final String label;
}

/// 折叠头下方固定的分类标签栏（挂在 SliverAppBar.bottom，折叠后固定）。
class ProfileCategoryTabs extends StatelessWidget
    implements PreferredSizeWidget {
  const ProfileCategoryTabs({super.key, this.controller});

  final TabController? controller;

  static const double height = 46;

  @override
  Size get preferredSize => const Size.fromHeight(height);

  @override
  Widget build(BuildContext context) {
    return Container(
      color: AppTheme.surfaceWhite,
      alignment: Alignment.centerLeft,
      child: TabBar(
        controller: controller,
        isScrollable: true,
        labelColor: AppTheme.primary,
        unselectedLabelColor: AppTheme.textSecondary,
        indicatorColor: AppTheme.primary,
        indicatorSize: TabBarIndicatorSize.label,
        dividerColor: AppTheme.divider,
        labelStyle: const TextStyle(fontSize: 14, fontWeight: FontWeight.w600),
        unselectedLabelStyle: const TextStyle(fontSize: 14),
        tabs: [for (final tab in ProfileTab.values) Tab(text: tab.label)],
      ),
    );
  }
}
