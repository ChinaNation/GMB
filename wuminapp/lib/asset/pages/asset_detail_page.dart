// 资产详情页(框架阶段占位)。
//
// 后续任务卡 C 实装时承载:
// - 顶部:资产 name / symbol / 状态徽章(Active / Closed / ForceClosed + 倒计时)
// - 中段:持仓余额 / 总发行量 / 持仓人数 / 发行人 SS58 / 监管者 NRC SS58
// - 历史 tab:本地 Isar 提案痕迹 + 链上事件流(双轨)
// - 操作按钮(仅发行人本身可见):增发 / 销毁 / 转账 / 关闭

import 'package:flutter/material.dart';

class AssetDetailPage extends StatelessWidget {
  const AssetDetailPage({super.key, required this.assetId});

  final int assetId;

  @override
  Widget build(BuildContext context) {
    // TODO: implement business logic
    return Scaffold(
      appBar: AppBar(title: Text('资产 #$assetId (占位)')),
      body: const Center(child: Text('详情占位')),
    );
  }
}
