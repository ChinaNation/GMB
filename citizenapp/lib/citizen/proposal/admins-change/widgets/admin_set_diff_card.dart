import 'package:flutter/material.dart';

import 'package:citizenapp/citizen/proposal/admins-change/codec/account_id_codec.dart';

class AdminSetDiffCard extends StatelessWidget {
  const AdminSetDiffCard({
    super.key,
    required this.currentAdmins,
    required this.admins,
    this.balances = const {},
  });

  final List<String> currentAdmins;
  final List<String> admins;
  final Map<String, double> balances;

  @override
  Widget build(BuildContext context) {
    final current = currentAdmins.map(AdminAccountIdCodec.normalizeHex).toSet();
    final next = admins.map(AdminAccountIdCodec.normalizeHex).toSet();
    final added = next.where((item) => !current.contains(item)).toList();
    final removed = current.where((item) => !next.contains(item)).toList();
    return Card(
      elevation: 0,
      margin: EdgeInsets.zero,
      child: Padding(
        padding: const EdgeInsets.all(14),
        child: Column(
          crossAxisAlignment: CrossAxisAlignment.start,
          children: [
            const Text('变更差异',
                style: TextStyle(fontSize: 15, fontWeight: FontWeight.w700)),
            const SizedBox(height: 8),
            _buildAccountList('新增', added),
            const SizedBox(height: 8),
            _buildAccountList('移除', removed),
          ],
        ),
      ),
    );
  }

  Widget _buildAccountList(String title, List<String> accounts) {
    if (accounts.isEmpty) return Text('$title：无');
    return Column(
      crossAxisAlignment: CrossAxisAlignment.start,
      children: [
        Text('$title：'),
        const SizedBox(height: 6),
        for (final account in accounts) ...[
          ListTile(
            dense: true,
            title: Text(account),
            subtitle: Text(
                '余额：${balances[AdminAccountIdCodec.normalizeHex(account)]?.toStringAsFixed(2) ?? '-'} 元'),
          ),
          const SizedBox(height: 6),
        ],
      ],
    );
  }
}
