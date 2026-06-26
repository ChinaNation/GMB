import 'package:flutter/material.dart';

import 'package:citizenapp/citizen/proposal/proposal_placeholder.dart';

/// 发起选举提案(占位,选举机构 + 链端选举模块待接)。
class ElectionProposalPage extends StatelessWidget {
  const ElectionProposalPage({super.key});

  @override
  Widget build(BuildContext context) =>
      const ProposalPlaceholderPage(title: '发起选举', kind: '发起选举');
}
