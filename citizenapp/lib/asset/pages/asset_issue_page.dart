// 发行代币向导页(框架阶段占位)。
//
// 后续任务卡 C 实装时承载 7 步流程:
//   1. 选择发行人主体(CID 机构 / personal-manage 多签,从已绑定列表选)
//   2. 输入资产名 / 符号 / 描述(本地黑名单预校验,避免链端 reject 浪费提交)
//   3. 选择 decimals(0..=18,带常用预设 6 / 8 / 18)
//   4. 输入初始发行量(按 decimals 友好显示)
//   5. 提示创建费 1000 GMB 的扣款来源 + 当前 GMB 余额
//   6. 确认页:摘要 + 监管 NRC 强制提示 + 不可锚定法币提示
//   7. "发起多签提案"按钮 → 直调 VotingEngine InternalVote 业务 ACTION OAIS

import 'package:flutter/material.dart';

class AssetIssuePage extends StatelessWidget {
  const AssetIssuePage({super.key});

  @override
  Widget build(BuildContext context) {
    // TODO: implement business logic
    return Scaffold(
      appBar: AppBar(title: const Text('发行代币 (占位)')),
      body: const Center(child: Text('发行向导占位')),
    );
  }
}
