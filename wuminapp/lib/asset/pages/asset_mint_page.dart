// 增发提案页(框架阶段占位)。
//
// 后续任务卡 C 实装时承载:
// - 选择受益人 SS58
// - 输入 mint 数量(按 decimals 友好显示)
// - 提示 GMB 计费(0.1% / ≥0.1 元)
// - "发起多签提案"按钮 → 直调 VotingEngine InternalVote 业务 ACTION OAMT

import 'package:flutter/material.dart';

class AssetMintPage extends StatelessWidget {
  const AssetMintPage({super.key, required this.assetId});

  final int assetId;

  @override
  Widget build(BuildContext context) {
    // TODO: implement business logic
    return Scaffold(
      appBar: AppBar(title: Text('增发 #$assetId (占位)')),
      body: const Center(child: Text('增发占位')),
    );
  }
}
