// 资产余额 tile(框架阶段占位)— AssetDetailPage 顶部用。
//
// 后续任务卡 C 实装时承载:
// - 大字数字 + symbol(如 12,345.67 USDC)
// - 状态徽章:Active 绿 / Frozen 黄 / Closed 灰
// - 副行:raw 数值 + decimals 提示

import 'package:flutter/material.dart';

class AssetBalanceTile extends StatelessWidget {
  const AssetBalanceTile({super.key});

  @override
  Widget build(BuildContext context) {
    // TODO: implement business logic
    return const ListTile(title: Text('余额占位'));
  }
}
