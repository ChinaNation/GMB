// 用户代币转账页(框架阶段占位)。
//
// 后续任务卡 C 实装时承载:
// - 转账金额输入(自动按 decimals 显示精度)
// - 收款方 SS58 / SubjectId 输入(支持二维码扫描)
// - 提示行:GMB 计费(0.1% / ≥0.1 元)+ 当前 GMB 余额
// - "发起多签提案"按钮 → 直调 VotingEngine InternalVote(unified_voting_entry phase 4)

import 'package:flutter/material.dart';

class AssetTransferPage extends StatelessWidget {
  const AssetTransferPage({super.key, required this.assetId});

  final int assetId;

  @override
  Widget build(BuildContext context) {
    // TODO: implement business logic
    return Scaffold(
      appBar: AppBar(title: Text('转账 #$assetId (占位)')),
      body: const Center(child: Text('转账占位')),
    );
  }
}
