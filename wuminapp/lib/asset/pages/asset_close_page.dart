// 关闭代币提案页(框架阶段占位)。
//
// 后续任务卡 C 实装时承载:
// - 强红色二次确认 banner:"关闭后所有持仓余额销毁,GMB 创建费不退还"
// - 提示 close 收 VOTE_FLAT_FEE = 1 元
// - "发起多签提案"按钮 → 直调 VotingEngine InternalVote 业务 ACTION OACL

import 'package:flutter/material.dart';

class AssetClosePage extends StatelessWidget {
  const AssetClosePage({super.key, required this.assetId});

  final int assetId;

  @override
  Widget build(BuildContext context) {
    // TODO: implement business logic
    return Scaffold(
      appBar: AppBar(title: Text('关闭 #$assetId (占位)')),
      body: const Center(child: Text('关闭占位')),
    );
  }
}
