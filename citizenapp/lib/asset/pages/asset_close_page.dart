// 关闭代币提案页(框架阶段占位)。
//
// 后续任务卡 C 实装时承载:
// - 强红色二次确认 banner:"关闭后所有持仓余额销毁"
// - 发起关闭是机构链上操作，由 actor CID 的费用账户支付 0.1 元；
//   后续管理员实际 cast 投票才由投票签名者支付 1 元。
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
      body: const Center(child: Text('链上资产关闭尚未开放')),
    );
  }
}
