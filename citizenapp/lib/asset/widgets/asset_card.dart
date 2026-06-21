// 资产卡片 widget(框架阶段占位)— AssetListPage 单条 cell 用。
//
// 后续任务卡 C 实装时承载:
// - 顶行:symbol 大字 + 状态徽章(Active=绿 / Closed=灰 / ForceClosed=红+倒计时)
// - 中行:持仓余额(按 decimals 友好显示) + 隐藏式 raw 数值
// - 底行:发行人 SS58 短显示(前 6 + 后 4 + 复制图标)

import 'package:flutter/material.dart';

class AssetCard extends StatelessWidget {
  const AssetCard({super.key});

  @override
  Widget build(BuildContext context) {
    // TODO: implement business logic
    return const Card(child: ListTile(title: Text('资产卡片占位')));
  }
}
