// 销毁提案页(框架阶段占位)。
//
// 后续任务卡 C 实装时承载:
// - 选择销毁来源 SS58(默认发行人代理账户)
// - 输入销毁数量
// - 提示 burn Free(无 GMB 计费)
// - "发起多签提案"按钮 → 直调 VotingEngine InternalVote 业务 ACTION OABN

import 'package:flutter/material.dart';

class AssetBurnPage extends StatelessWidget {
  const AssetBurnPage({super.key, required this.assetId});

  final int assetId;

  @override
  Widget build(BuildContext context) {
    // TODO: implement business logic
    return Scaffold(
      appBar: AppBar(title: Text('销毁 #$assetId (占位)')),
      body: const Center(child: Text('销毁占位')),
    );
  }
}
