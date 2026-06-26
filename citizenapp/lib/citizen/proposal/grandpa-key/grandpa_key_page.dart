import 'package:flutter/material.dart';

import 'package:citizenapp/citizen/proposal/proposal_placeholder.dart';

/// 验证密钥(更换 GRANDPA 共识密钥)提案(占位,链端 grandpakey-change 客户端待接)。
class GrandpaKeyPage extends StatelessWidget {
  const GrandpaKeyPage({super.key});

  @override
  Widget build(BuildContext context) =>
      const ProposalPlaceholderPage(title: '验证密钥', kind: '验证密钥');
}
