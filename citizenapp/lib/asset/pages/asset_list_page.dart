// 我的资产列表页(框架阶段占位)。
//
// 后续任务卡 C 实装时承载:
// - 顶部固定显示 GMB 余额(主币)
// - 下方滚动列出当前 SS58 持有的所有用户代币(symbol / 余额 / 资产状态徽章)
// - 点击进入 AssetDetailPage
// - 顶部右上角"发行代币"按钮 → AssetIssuePage(仅 CID 机构 / personal-manage 多签可见)

import 'package:flutter/material.dart';

class AssetListPage extends StatelessWidget {
  const AssetListPage({super.key});

  @override
  Widget build(BuildContext context) {
    // TODO: implement business logic
    return const Scaffold(
      body: Center(child: Text('我的资产 (占位)')),
    );
  }
}
