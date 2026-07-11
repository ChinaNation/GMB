import 'package:flutter/material.dart';

import 'package:citizenapp/8964/pages/square_home_page.dart';

/// 底部“广场”Tab 入口。
///
/// 广场首页默认进入推荐流；真实 feed 和发布闭环在后续阶段接入。
class SquareTab extends StatelessWidget {
  const SquareTab({super.key});

  @override
  Widget build(BuildContext context) => const SquareHomePage();
}
